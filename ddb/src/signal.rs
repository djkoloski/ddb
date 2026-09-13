use core::{ffi::c_int, fmt};

#[derive(Debug)]
pub struct Signal {
    value: c_int,
}

impl Signal {
    pub fn new(value: c_int) -> Self {
        Self { value }
    }
}

impl fmt::Display for Signal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        macro_rules! match_signal {
            ($($name:ident => $desc:literal),* $(,)?) => {
                match self.value {
                    $(
                        #[allow(unreachable_patterns)]
                        libc::$name => write!(
                            f,
                            "{}: {}",
                            ::core::stringify!($name),
                            $desc,
                        )?,
                    )*
                    _ => write!(f, "unknown signal: {}", self.value)?,
                }
            }
        }

        match_signal! {
            SIGABRT => "Abort signal from abort(3)",
            SIGALRM => "Timer signal from alarm(2)",
            SIGBUS => "Bus error (bad memory access)",
            SIGCHLD => "Child stopped, terminated, or continued",
            SIGCONT => "Continue if stopped",
            SIGFPE => "Erroneous arithmetic operation",
            SIGHUP => "Hangup detected on controlling terminal or death of \
            controlling process",
            SIGILL => "Illegal Instruction",
            SIGINT => "Interrupt from keyboard",
            SIGIO => "I/O now possible (4.2BSD)",
            SIGIOT => "IOT trap.  A synonym for SIGABRT",
            SIGKILL => "Kill signal",
            SIGPIPE => "Broken pipe: write to pipe with no readers; see \
            pipe(7)",
            SIGPOLL => "Pollable event (Sys V); synonym for SIGIO",
            SIGPROF => "Profiling timer expired",
            SIGPWR => "Power failure (System V)",
            SIGQUIT => "Quit from keyboard",
            SIGSEGV => "Invalid memory reference",
            SIGSTKFLT => "Stack fault on coprocessor (unused)",
            SIGSTOP => "Stop process",
            SIGTSTP => "Stop typed at terminal",
            SIGSYS => "Bad system call (SVr4); see also seccomp(2)",
            SIGTERM => "Termination signal",
            SIGTRAP => "Trace/breakpoint trap",
            SIGTTIN => "Terminal input for background process",
            SIGTTOU => "Terminal output for background process",
            SIGURG => "Urgent condition on socket (4.2BSD)",
            SIGUSR1 => "User-defined signal 1",
            SIGUSR2 => "User-defined signal 2",
            SIGVTALRM => "Virtual alarm clock (4.2BSD)",
            SIGXCPU => "CPU time limit exceeded (4.2BSD); see setrlimit(2)",
            SIGXFSZ => "File size limit exceeded (4.2BSD); see setrlimit(2)",
            SIGWINCH => "Window resize signal (4.3BSD, Sun)",
        }

        Ok(())
    }
}
