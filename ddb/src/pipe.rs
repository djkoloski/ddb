use core::ffi::c_int;

use libc::{O_CLOEXEC, close, pipe2, read, write};

use crate::Fatal;

pub fn pipe(close_on_exec: bool) -> Result<(ReadEnd, WriteEnd), Fatal> {
    let flags = if close_on_exec { O_CLOEXEC } else { 0 };
    let mut fds = [0; 2];
    let status = unsafe { pipe2(fds.as_mut_ptr(), flags) };
    if status < 0 {
        return Err(Fatal::syscall("pipe2"));
    }

    Ok((ReadEnd { fd: fds[0] }, WriteEnd { fd: fds[1] }))
}

pub struct ReadEnd {
    fd: c_int,
}

impl Drop for ReadEnd {
    fn drop(&mut self) {
        unsafe {
            close(self.fd);
        }
    }
}

impl ReadEnd {
    pub fn read(&mut self) -> Result<Vec<u8>, Fatal> {
        const BUF_SIZE: usize = 1024;

        let mut buf = [0; BUF_SIZE];
        let status =
            unsafe { read(self.fd, buf.as_mut_ptr().cast(), BUF_SIZE) };
        if status < 0 {
            return Err(Fatal::syscall("read"));
        }

        Ok(buf[..status as usize].to_owned())
    }
}

pub struct WriteEnd {
    fd: c_int,
}

impl Drop for WriteEnd {
    fn drop(&mut self) {
        unsafe {
            close(self.fd);
        }
    }
}

impl WriteEnd {
    pub fn write(&mut self, bytes: &[u8]) -> Result<(), Fatal> {
        let status =
            unsafe { write(self.fd, bytes.as_ptr().cast(), bytes.len()) };
        if status < 0 {
            return Err(Fatal::syscall("write"));
        }

        Ok(())
    }
}
