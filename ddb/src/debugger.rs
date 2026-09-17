use core::{
    ffi::c_void,
    fmt,
    mem::MaybeUninit,
    ptr::{addr_of_mut, null, null_mut},
};

use libc::pid_t;

use crate::{
    Command, Error, Fatal, NonFatal, Process, Register, RegisterInfo,
    RegisterKind, RegisterTypeError, RegisterValue, Signal, syscall,
};

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
    user: MaybeUninit<libc::user>,
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
        let mut debugger = Self {
            pid,
            state: State::Stopped,
            user: MaybeUninit::zeroed(),
        };
        // TODO: check the state change for unexpected behavior?
        let _ = debugger.wait_for_state_change()?;
        Ok(debugger)
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
        let debugger = Self::bind(process.pid())?;

        Ok((process, debugger))
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

        if state == State::Stopped {
            self.read_all_registers()?;
        }

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

    unsafe fn register_ptr(
        data: *mut libc::user,
        info: &RegisterInfo,
    ) -> *mut () {
        unsafe { data.cast::<u8>().add(info.offset).cast() }
    }

    fn read_all_registers(&mut self) -> Result<(), Fatal> {
        unsafe {
            syscall::ptrace(
                libc::PTRACE_GETREGS,
                self.pid,
                null(),
                addr_of_mut!((*self.user.as_mut_ptr()).regs).cast(),
            )?;
        }
        unsafe {
            syscall::ptrace(
                libc::PTRACE_GETFPREGS,
                self.pid,
                null(),
                addr_of_mut!((*self.user.as_mut_ptr()).i387).cast(),
            )?;
        }

        for (i, reg) in Register::DEBUG_REGS.iter().enumerate() {
            let info = reg.info();
            let mut value = 0;
            unsafe {
                syscall::ptrace(
                    libc::PTRACE_PEEKUSER,
                    self.pid,
                    info.offset as *const c_void,
                    (&mut value as *mut u64).cast(),
                )?;
            }
            unsafe {
                addr_of_mut!((*self.user.as_mut_ptr()).u_debugreg)
                    .cast::<u64>()
                    .add(i)
                    .write(value);
            }
        }

        Ok(())
    }

    pub fn read_reg_value(&self, register: Register) -> RegisterValue {
        let info = register.info();

        let data_ptr = self.user.as_ptr().cast_mut();
        let ptr = unsafe { Self::register_ptr(data_ptr, info) };

        unsafe { info.ty.read(ptr) }
    }

    pub fn read_reg<T>(&self, register: Register) -> Result<T, Fatal>
    where
        T: TryFrom<RegisterValue, Error = RegisterTypeError>,
    {
        T::try_from(self.read_reg_value(register))
            .map_err(Fatal::RegisterTypeError)
    }

    fn write_user_area(
        &mut self,
        offset: usize,
        data: u64,
    ) -> Result<(), Fatal> {
        unsafe {
            syscall::ptrace(
                libc::PTRACE_POKEUSER,
                self.pid,
                offset as *mut c_void,
                data as *mut c_void,
            )?;
        }

        Ok(())
    }

    #[expect(unused)]
    fn write_gp_regs(&mut self) -> Result<(), Fatal> {
        unsafe {
            syscall::ptrace(
                libc::PTRACE_SETREGS,
                self.pid,
                null(),
                addr_of_mut!((*self.user.as_mut_ptr()).regs).cast(),
            )?;
        }
        Ok(())
    }

    fn write_fp_regs(&mut self) -> Result<(), Fatal> {
        unsafe {
            syscall::ptrace(
                libc::PTRACE_SETFPREGS,
                self.pid,
                null(),
                addr_of_mut!((*self.user.as_mut_ptr()).i387).cast(),
            )?;
        }
        Ok(())
    }

    pub fn write_reg_value(
        &mut self,
        register: Register,
        value: RegisterValue,
    ) -> Result<(), Fatal> {
        let info = register.info();
        if info.ty != value.ty() {
            return Err(Fatal::RegisterTypeError(RegisterTypeError {
                expected: info.ty,
                actual: value.ty(),
            }));
        }

        let ptr = unsafe { Self::register_ptr(self.user.as_mut_ptr(), info) };

        unsafe {
            value.write(ptr);
        }

        match info.kind {
            RegisterKind::Gp | RegisterKind::SubGp | RegisterKind::Debug => {
                let word_offset = ptr as usize % align_of::<u64>();
                let word = unsafe {
                    ptr.cast::<u8>().sub(word_offset).cast::<u64>().read()
                };

                self.write_user_area(info.offset - word_offset, word)?;
            }
            RegisterKind::Fp => self.write_fp_regs()?,
        }

        Ok(())
    }

    pub fn write_reg(
        &mut self,
        register: Register,
        value: impl Into<RegisterValue>,
    ) -> Result<(), Fatal> {
        self.write_reg_value(register, value.into())
    }
}
