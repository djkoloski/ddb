use core::fmt;

use libc::{WEXITSTATUS, WIFEXITED, WIFSIGNALED, WSTOPSIG, WTERMSIG, pid_t};

use crate::{Fatal, Signal, syscall};

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

#[must_use]
pub struct StateChange {
    pub state: State,
    pub signal: Signal,
}

impl StateChange {
    pub fn wait_for_pid(pid: pid_t) -> Result<Self, Fatal> {
        let mut status = 0;
        unsafe {
            syscall::waitpid(pid, &mut status, 0)?;
        }

        let (state, signal) = if WIFEXITED(status) {
            (State::Exited, WEXITSTATUS(status))
        } else if WIFSIGNALED(status) {
            (State::Terminated, WTERMSIG(status))
        } else {
            (State::Stopped, WSTOPSIG(status))
        };

        Ok(StateChange {
            state,
            signal: Signal::new(signal),
        })
    }
}
