use core::ptr::null;
use std::{ffi::CString, path::Path};

use libc::{SIGKILL, pid_t};

use crate::{Fatal, pipe::pipe, state::StateChange, syscall};

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

    pub fn launch_with_pre_exec(
        path: impl AsRef<Path>,
        pre_exec: impl FnOnce() -> Result<(), Fatal>,
    ) -> Result<Self, Fatal> {
        let path = CString::new(path.as_ref().as_os_str().as_encoded_bytes())
            .map_err(Fatal::InvalidPath)?;

        let (mut reader, mut writer) = pipe(true)?;

        let status = unsafe { syscall::fork()? };
        if status == 0 {
            drop(reader);

            let child_main = || -> Result<(), Fatal> {
                pre_exec()?;
                unsafe {
                    syscall::execvp(
                        path.as_ptr(),
                        [path.as_ptr(), null()].as_ptr(),
                    )?;
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
        Ok(Self { pid: status })
    }

    pub fn launch(path: impl AsRef<Path>) -> Result<Self, Fatal> {
        Self::launch_with_pre_exec(path.as_ref(), || Ok(()))
    }

    fn terminate(&mut self) -> Result<(), Fatal> {
        unsafe {
            syscall::kill(self.pid, SIGKILL)?;
        }

        // TODO: check the state change for unexpected behavior
        let _ = StateChange::wait_for_pid(self.pid)?;

        Ok(())
    }
}
