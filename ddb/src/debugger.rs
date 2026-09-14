use core::{fmt, ptr::null_mut};

use libc::pid_t;

use crate::{Command, Error, Fatal, NonFatal, Process, Signal, syscall};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Running,
    Stopped,
    Exited,
    Terminated,
}

impl fmt::Display for State {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Running => write!(f, "running")?,
            Self::Stopped => write!(f, "stopped")?,
            Self::Exited => write!(f, "exited")?,
            Self::Terminated => write!(f, "terminated")?,
        }

        Ok(())
    }
}

#[derive(Debug)]
#[must_use]
pub struct StateChange {
    pub state: State,
    pub signal: Signal,
}

#[derive(Debug)]
pub struct Debugger {
    pid: pid_t,
    state: State,
}

impl Drop for Debugger {
    fn drop(&mut self) {
        if let Err(e) = self.detach() {
            eprintln!("failed to detach pid {} in drop: {e}", self.pid);
        }
    }
}

impl Debugger {
    pub fn pid(&self) -> pid_t {
        self.pid
    }

    fn bind(pid: pid_t) -> Result<Self, Fatal> {
        let mut attachment = Self {
            pid,
            state: State::Stopped,
        };
        // TODO: check the state change for unexpected behavior?
        let _ = attachment.wait_for_state_change()?;
        Ok(attachment)
    }

    pub fn spawn_attached(command: Command) -> Result<(Process, Self), Fatal> {
        let process = command
            .pre_exec(|| {
                unsafe {
                    syscall::ptrace(
                        libc::PTRACE_TRACEME,
                        0,
                        null_mut(),
                        null_mut(),
                    )?;
                }
                Ok(())
            })
            .spawn()?;
        let attachment = Self::bind(process.pid())?;

        Ok((process, attachment))
    }

    pub fn attach(pid: pid_t) -> Result<Self, Fatal> {
        unsafe {
            syscall::ptrace(libc::PTRACE_ATTACH, pid, null_mut(), null_mut())?;
        }
        Self::bind(pid)
    }

    pub fn state(&self) -> State {
        self.state
    }

    pub fn resume(&mut self) -> Result<(), Error> {
        match self.state {
            State::Running => return Ok(()),
            State::Stopped => (),
            State::Exited | State::Terminated => {
                Err(NonFatal::CannotResume(self.state))?
            }
        }

        unsafe {
            syscall::ptrace(
                libc::PTRACE_CONT,
                self.pid,
                null_mut(),
                null_mut(),
            )?;
        }

        self.state = State::Running;
        Ok(())
    }

    pub fn stop(&mut self) -> Result<(), Error> {
        match self.state {
            State::Running => (),
            State::Stopped => return Ok(()),
            State::Exited | State::Terminated => {
                Err(NonFatal::CannotStop(self.state))?
            }
        }

        unsafe {
            syscall::kill(self.pid, libc::SIGSTOP)?;
        }

        // TODO: check the state change for unexpected behavior?
        let _ = self.wait_for_state_change()?;

        Ok(())
    }

    pub fn wait_for_state_change(&mut self) -> Result<StateChange, Fatal> {
        let mut status = 0;
        unsafe {
            syscall::waitpid(self.pid, &mut status, 0)?;
        }

        let (state, signal) = if libc::WIFEXITED(status) {
            (State::Exited, libc::WEXITSTATUS(status))
        } else if libc::WIFSIGNALED(status) {
            (State::Terminated, libc::WTERMSIG(status))
        } else {
            (State::Stopped, libc::WSTOPSIG(status))
        };

        self.state = state;

        Ok(StateChange {
            state,
            signal: Signal::new(signal),
        })
    }

    fn detach(&mut self) -> Result<(), Error> {
        self.stop()?;

        unsafe {
            syscall::ptrace(
                libc::PTRACE_DETACH,
                self.pid,
                null_mut(),
                null_mut(),
            )?;
        }

        unsafe {
            syscall::kill(self.pid, libc::SIGCONT)?;
        }

        Ok(())
    }
}
