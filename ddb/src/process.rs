use core::ptr::null;
use std::ffi::{CString, OsStr, OsString};

use libc::pid_t;

use crate::{Fatal, pipe::pipe, syscall};

pub struct Command {
    program: OsString,
    pre_exec: Option<Box<dyn FnOnce() -> Result<(), Fatal>>>,
    args: Vec<OsString>,
}

impl Command {
    pub fn new(program: impl AsRef<OsStr>) -> Self {
        Self {
            program: program.as_ref().to_owned(),
            pre_exec: None,
            args: Vec::new(),
        }
    }

    pub fn arg(mut self, arg: impl AsRef<OsStr>) -> Self {
        self.args.push(arg.as_ref().to_owned());
        self
    }

    pub fn args<I>(mut self, args: I) -> Self
    where
        I: Iterator,
        I::Item: AsRef<OsStr>,
    {
        for arg in args {
            self = self.arg(arg);
        }
        self
    }

    pub fn pre_exec(
        mut self,
        pre_exec: impl FnOnce() -> Result<(), Fatal> + 'static,
    ) -> Self {
        if let Some(existing_pre_exec) = self.pre_exec.take() {
            self.pre_exec = Some(Box::new(|| {
                (existing_pre_exec)()?;
                (pre_exec)()
            }))
        } else {
            self.pre_exec = Some(Box::new(pre_exec));
        }

        self
    }

    pub fn spawn(self) -> Result<Process, Fatal> {
        let path = CString::new(self.program.as_encoded_bytes())
            .map_err(Fatal::InvalidArg)?;

        let (mut reader, mut writer) = pipe(true)?;

        let status = unsafe { syscall::fork()? };
        if status == 0 {
            drop(reader);

            // Replace stdin/stdout/stderr with /dev/null
            let dev_null =
                unsafe { syscall::open(c"/dev/null".as_ptr(), libc::O_RDWR)? };
            unsafe {
                syscall::dup2(dev_null, libc::STDIN_FILENO)?;
                syscall::dup2(dev_null, libc::STDOUT_FILENO)?;
                syscall::dup2(dev_null, libc::STDERR_FILENO)?;
                syscall::close(dev_null)?;
            }

            let child_main = || -> Result<(), Fatal> {
                if let Some(f) = self.pre_exec {
                    (f)()?;
                }

                let args = self
                    .args
                    .iter()
                    .map(|a| {
                        CString::new(a.as_encoded_bytes())
                            .map_err(Fatal::InvalidArg)
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let mut argv = Vec::with_capacity(args.len() + 2);
                argv.push(path.as_ptr());
                argv.extend(args.iter().map(|a| a.as_ptr()));
                argv.push(null());

                unsafe {
                    syscall::execvp(path.as_ptr(), argv.as_ptr())?;
                }
                unreachable!()
            };
            if let Err(e) = child_main()
                && let Err(write_err) = writer.write(format!("{e}").as_bytes())
            {
                eprintln!("failed to send error '{e}' to parent: {write_err}");
            }

            unsafe {
                syscall::exit(-1);
            }
        }

        drop(writer);
        let data = reader.read()?;
        drop(reader);

        if !data.is_empty() {
            let e =
                String::from_utf8(data).map_err(Fatal::InvalidChildMessage)?;
            return Err(Fatal::ChildFailed(e));
        }
        Ok(Process { pid: status })
    }
}

#[derive(Debug)]
pub struct Process {
    pid: pid_t,
}

impl Drop for Process {
    fn drop(&mut self) {
        if let Err(e) = self.terminate() {
            eprintln!("failed to terminate pid {} in drop: {e}", self.pid);
        }
    }
}

impl Process {
    pub fn pid(&self) -> pid_t {
        self.pid
    }

    fn terminate(&mut self) -> Result<(), Fatal> {
        unsafe {
            syscall::kill(self.pid, libc::SIGKILL)?;
        }

        let _ = unsafe { syscall::waitpid(self.pid, &mut 0, 0)? };

        Ok(())
    }
}
