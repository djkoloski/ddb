mod errno;
mod error;
mod pipe;
mod signal;

use core::{
    ffi::{c_char, c_void},
    fmt,
    ptr::null_mut,
};
use std::{ffi::CString, path::Path};

use libc::{
    PTRACE_ATTACH, PTRACE_CONT, PTRACE_DETACH, PTRACE_TRACEME, SIGCONT,
    SIGKILL, SIGSTOP, WEXITSTATUS, WIFEXITED, WIFSIGNALED, WSTOPSIG, WTERMSIG,
    execlp, exit, fork, kill, pid_t, ptrace, waitpid,
};

pub use self::{errno::*, error::*, signal::*};
use crate::pipe::pipe;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Stopped,
    Exited,
    Terminated,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Stopped => write!(f, "stopped")?,
            Self::Exited => write!(f, "exited")?,
            Self::Terminated => write!(f, "terminated")?,
        }

        Ok(())
    }
}

pub struct StateChange {
    pub state: State,
    pub signal: Signal,
}

#[derive(Debug, PartialEq, Eq)]
enum DropAction {
    Detach,
    DetachAndTerminate,
}

#[derive(Debug)]
pub struct Process {
    pid: pid_t,
    state: Option<State>,
    drop_action: DropAction,
}

impl Drop for Process {
    fn drop(&mut self) {
        if let Err(e) = self.detach() {
            eprintln!("failed to detach from pid {} in drop: {e}", self.pid);
        }

        if self.drop_action == DropAction::DetachAndTerminate
            && let Err(e) = self.terminate()
        {
            eprintln!("failed to terminate pid {} in drop: {e}", self.pid);
        }
    }
}

impl Process {
    pub fn pid(&self) -> pid_t {
        self.pid
    }

    pub fn state(&self) -> Option<State> {
        self.state
    }

    fn bind(pid: pid_t, drop_action: DropAction) -> Result<Self, Fatal> {
        let mut result = Self {
            pid,
            state: None,
            drop_action,
        };
        result.wait_for_state_change()?;
        result.state = Some(State::Stopped);

        Ok(result)
    }

    pub fn attach(pid: pid_t) -> Result<Self, Fatal> {
        let status = unsafe {
            ptrace(
                PTRACE_ATTACH,
                pid,
                null_mut::<c_void>(),
                null_mut::<c_void>(),
            )
        };
        if status < 0 {
            return Err(Fatal::syscall("ptrace"));
        }

        Self::bind(pid, DropAction::Detach)
    }

    pub fn launch_attached(path: &Path) -> Result<Self, Fatal> {
        const PTRACE_FAILED: u8 = 0;
        const EXEC_FAILED: u8 = 1;

        fn serialize(id: u8, status: i64) -> [u8; 9] {
            let mut result = [0; 9];
            result[0] = id;
            result[1..].copy_from_slice(&status.to_le_bytes());
            result
        }

        fn deserialize(buf: &[u8]) -> Result<(u8, i64), Fatal> {
            if buf.len() != 9 {
                return Err(Fatal::MessageTooShort(buf.len()));
            }
            let status = i64::from_le_bytes([
                buf[1], buf[2], buf[3], buf[4], buf[5], buf[6], buf[7], buf[8],
            ]);
            Ok((buf[0], status))
        }

        let path = CString::new(path.as_os_str().as_encoded_bytes())
            .map_err(Fatal::InvalidPath)?;

        let (mut reader, mut writer) = pipe(true)?;

        let status = unsafe { fork() };
        if status < 0 {
            return Err(Fatal::syscall("fork"));
        } else if status != 0 {
            drop(writer);
            let data = reader.read()?;
            drop(reader);

            if !data.is_empty() {
                let (id, status) = deserialize(&data)?;
                match id {
                    PTRACE_FAILED => {
                        return Err(Fatal::FailedToPtraceChild(status));
                    }
                    EXEC_FAILED => {
                        return Err(Fatal::FailedToExecChild(status));
                    }
                    _ => return Err(Fatal::InvalidMessage { id, status }),
                }
            }

            return Self::bind(status, DropAction::DetachAndTerminate);
        }

        drop(reader);

        let status = unsafe {
            ptrace(
                PTRACE_TRACEME,
                0,
                null_mut::<c_void>(),
                null_mut::<c_void>(),
            )
        };
        if status < 0 {
            writer.write(&serialize(PTRACE_FAILED, status))?;
            unsafe {
                exit(-1);
            }
        }

        let status = unsafe {
            execlp(path.as_ptr(), path.as_ptr(), null_mut::<c_char>())
        };
        if status < 0 {
            writer.write(&serialize(EXEC_FAILED, status as i64))?;
            unsafe {
                exit(-1);
            }
        }

        unreachable!();
    }

    pub fn wait_for_state_change(&mut self) -> Result<StateChange, Fatal> {
        let mut status = 0;
        let pid = unsafe { waitpid(self.pid, &mut status, 0) };
        if pid < 0 {
            return Err(Fatal::syscall("waitpid"));
        }

        let (state, signal) = if WIFEXITED(status) {
            (State::Exited, WEXITSTATUS(status))
        } else if WIFSIGNALED(status) {
            (State::Terminated, WTERMSIG(status))
        } else {
            (State::Stopped, WSTOPSIG(status))
        };
        self.state = Some(state);

        Ok(StateChange {
            state,
            signal: Signal::new(signal),
        })
    }

    pub fn resume(&mut self) -> Result<(), Error> {
        match self.state {
            None => return Ok(()),
            Some(State::Stopped) => (),
            Some(state @ (State::Exited | State::Terminated)) => {
                Err(NonFatal::CannotResume(state))?
            }
        }

        let status = unsafe {
            ptrace(
                PTRACE_CONT,
                self.pid,
                null_mut::<c_void>(),
                null_mut::<c_void>(),
            )
        };
        if status < 0 {
            Err(Fatal::syscall("ptrace"))?;
        }

        self.state = None;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        match self.state {
            None => (),
            Some(State::Stopped) => return Ok(()),
            Some(state @ (State::Exited | State::Terminated)) => {
                Err(NonFatal::CannotStop(state))?
            }
        }

        let status = unsafe { kill(self.pid, SIGSTOP) };
        if status < 0 {
            Err(Fatal::syscall("kill"))?;
        }

        self.wait_for_state_change()?;

        Ok(())
    }

    fn detach(&mut self) -> Result<(), Error> {
        self.stop()?;

        let status = unsafe {
            ptrace(
                PTRACE_DETACH,
                self.pid,
                null_mut::<c_void>(),
                null_mut::<c_void>(),
            )
        };
        if status < 0 {
            Err(Fatal::syscall("ptrace"))?;
        }

        let status = unsafe { kill(self.pid, SIGCONT) };
        if status < 0 {
            Err(Fatal::syscall("kill"))?;
        }

        Ok(())
    }

    fn terminate(&mut self) -> Result<(), Fatal> {
        let status = unsafe { kill(self.pid, SIGKILL) };
        if status < 0 {
            Err(Fatal::syscall("kill"))?;
        }

        self.wait_for_state_change()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
