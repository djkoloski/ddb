use core::ptr::null_mut;
use std::path::Path;

use libc::{
    PTRACE_ATTACH, PTRACE_CONT, PTRACE_DETACH, PTRACE_TRACEME, SIGCONT,
    SIGSTOP, pid_t,
};

use crate::{
    Error, Fatal, NonFatal, Process,
    state::{State, StateChange},
    syscall,
};

#[derive(Debug)]
pub struct Attachment {
    pid: pid_t,
    state: State,
}

impl Drop for Attachment {
    fn drop(&mut self) {
        if let Err(e) = self.detach() {
            eprintln!("failed to detach pid {} in drop: {e}", self.pid);
        }
    }
}

impl Attachment {
    pub fn pid(&self) -> pid_t {
        self.pid
    }

    fn bind(pid: pid_t) -> Result<Self, Fatal> {
        // TODO: check the state change for unexpected behavior?
        let _ = StateChange::wait_for_pid(pid)?;
        Ok(Self {
            pid,
            state: State::Stopped,
        })
    }

    pub fn launch(path: impl AsRef<Path>) -> Result<(Process, Self), Fatal> {
        let process = Process::launch_with_pre_exec(path, || {
            unsafe {
                syscall::ptrace(PTRACE_TRACEME, 0, null_mut(), null_mut())?;
            }
            Ok(())
        })?;
        let attachment = Self::bind(process.pid())?;

        Ok((process, attachment))
    }

    pub fn attach(pid: pid_t) -> Result<Self, Fatal> {
        unsafe {
            syscall::ptrace(PTRACE_ATTACH, pid, null_mut(), null_mut())?;
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
            syscall::ptrace(PTRACE_CONT, self.pid, null_mut(), null_mut())?;
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
            syscall::kill(self.pid, SIGSTOP)?;
        }

        // TODO: check the state change for unexpected behavior?
        let _ = StateChange::wait_for_pid(self.pid)?;

        Ok(())
    }

    fn detach(&mut self) -> Result<(), Error> {
        self.stop()?;

        unsafe {
            syscall::ptrace(PTRACE_DETACH, self.pid, null_mut(), null_mut())?;
        }

        unsafe {
            syscall::kill(self.pid, SIGCONT)?;
        }

        Ok(())
    }
}
