use core::ffi::c_int;

use crate::{Fatal, syscall};

pub fn pipe(close_on_exec: bool) -> Result<(ReadEnd, WriteEnd), Fatal> {
    let flags = if close_on_exec { libc::O_CLOEXEC } else { 0 };
    let mut fds = [0; 2];
    unsafe {
        syscall::pipe2(fds.as_mut_ptr(), flags)?;
    }

    Ok((ReadEnd { fd: fds[0] }, WriteEnd { fd: fds[1] }))
}

pub struct ReadEnd {
    fd: c_int,
}

impl Drop for ReadEnd {
    fn drop(&mut self) {
        unsafe {
            let _ = syscall::close(self.fd);
        }
    }
}

impl ReadEnd {
    pub fn read(&mut self) -> Result<Vec<u8>, Fatal> {
        const BUF_SIZE: usize = 1024;

        let mut buf = [0; BUF_SIZE];
        let len = unsafe {
            syscall::read(self.fd, buf.as_mut_ptr().cast(), BUF_SIZE)?
        };

        Ok(buf[..len as usize].to_owned())
    }
}

pub struct WriteEnd {
    fd: c_int,
}

impl Drop for WriteEnd {
    fn drop(&mut self) {
        unsafe {
            let _ = syscall::close(self.fd);
        }
    }
}

impl WriteEnd {
    pub fn write(&mut self, bytes: &[u8]) -> Result<(), Fatal> {
        unsafe {
            syscall::write(self.fd, bytes.as_ptr().cast(), bytes.len())?;
        }

        Ok(())
    }
}
