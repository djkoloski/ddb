use core::ffi::{c_char, c_int, c_long, c_uint, c_void};

use libc::{pid_t, size_t, ssize_t};

macro_rules! wrap_syscalls {
    ($(
        pub unsafe fn $syscall:ident(
            $($arg_name:ident: $arg_ty:ty),* $(,)?
        ) -> $ret_ty:ty;
    )*) => {
        $(
            pub unsafe fn $syscall(
                $($arg_name: $arg_ty),*
            ) -> Result<$ret_ty, $crate::Fatal> {
                let status = unsafe {
                    ::libc::$syscall($($arg_name),*)
                };
                if status < 0 {
                    return Err($crate::Fatal::syscall(
                        ::core::stringify!($syscall)
                    ));
                }

                Ok(status)
            }
        )*
    }
}

wrap_syscalls! {
    pub unsafe fn ptrace(
        request: c_uint,
        pid: pid_t,
        address: *const c_void,
        data: *mut c_void,
    ) -> c_long;

    pub unsafe fn kill(
        pid: pid_t,
        sig: c_int,
    ) -> c_int;

    pub unsafe fn fork() -> pid_t;

    pub unsafe fn waitpid(
        pid: pid_t,
        status: *mut c_int,
        options: c_int,
    ) -> pid_t;

    pub unsafe fn pipe2(fds: *mut c_int, flags: c_int) -> c_int;

    pub unsafe fn close(fd: c_int) -> c_int;

    pub unsafe fn read(
        fd: c_int,
        buf: *mut c_void,
        count: size_t,
    ) -> ssize_t;
    pub unsafe fn write(
        fd: c_int,
        buf: *const c_void,
        count: size_t,
    ) -> ssize_t;

    pub unsafe fn execvp(
        file: *const c_char,
        arg0: *const *const c_char,
    ) -> c_int;

    pub unsafe fn open(path: *const c_char, flags: c_int) -> c_int;

    pub unsafe fn dup2(oldfd: c_int, newfd: c_int) -> c_int;
}

pub use libc::exit;
