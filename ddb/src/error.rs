use std::ffi::NulError;

use crate::{Errno, State};

macro_rules! define_errors {
    (
        pub enum Fatal {
            $(
                #[error($fatal_fmt:literal)]
                $fatal_variant:ident
                // Tuple variant
                $(($fatal_tty:ty $(,)?))?
                // Struct variant
                $({ $($fatal_sname:ident: $fatal_sty:ty),* $(,)? })?
            ),* $(,)?
        }

        pub enum NonFatal {
            $(
                #[error($nonfatal_fmt:literal)]
                $nonfatal_variant:ident
                // Tuple variant
                $(($nonfatal_tty:ty $(,)?))?
                // Struct variant
                $({ $($nonfatal_sname:ident: $nonfatal_sty:ty),* $(,)? })?
            ),* $(,)?
        }
    ) => {
        #[derive(::core::fmt::Debug)]
        pub struct Error {
            fatal: Fatal,
        }

        impl Error {
            pub fn recover(self) -> ::core::result::Result<NonFatal, Fatal> {
                match self.fatal {
                    $(
                        Fatal::$fatal_variant
                        $(
                            (_) if false => {
                                let _: $fatal_tty; ::core::unreachable!()
                            }
                            Fatal::$fatal_variant (_0__)
                        )?
                        $({ $($fatal_sname),* })?
                        => Err(
                            Fatal::$fatal_variant
                            $(({ let _: $fatal_tty; _0__ }))?
                            $({ $($fatal_sname),* })?
                        ),
                    )*
                    $(
                        Fatal::$nonfatal_variant
                        $(
                            (_) if false => {
                                let _: $nonfatal_tty; ::core::unreachable!()
                            }
                            Fatal::$nonfatal_variant (_0__)
                        )?
                        $({ $($nonfatal_sname),* })?
                        => Ok(
                            NonFatal::$nonfatal_variant
                            $(({ let _: $nonfatal_tty; _0__ }))?
                            $({ $($nonfatal_sname),* })?
                        ),
                    )*
                }
            }
        }

        impl ::core::fmt::Display for Error {
            fn fmt(
                &self,
                f__: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                self.fatal.fmt(f__)
            }
        }

        impl From<Fatal> for Error {
            fn from(fatal: Fatal) -> Self {
                Self { fatal }
            }
        }

        impl From<NonFatal> for Error {
            fn from(non_fatal: NonFatal) -> Self {
                Self {
                    fatal: non_fatal.worsen(),
                }
            }
        }

        #[derive(::core::fmt::Debug)]
        pub enum Fatal {
            $(
                $fatal_variant
                $(($fatal_tty))?
                $({ $($fatal_sname: $fatal_sty,)* })?
                ,
            )*
            $(
                $nonfatal_variant
                $(($nonfatal_tty))?
                $({ $($nonfatal_sname: $nonfatal_sty,)* })?
                ,
            )*
        }

        impl ::core::fmt::Display for Fatal {
            fn fmt(
                &self,
                f__: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                match self {
                    $(
                        Self::$fatal_variant
                        $(
                            (_) if false => {
                                let _: $fatal_tty; ::core::unreachable!()
                            }
                            Self::$fatal_variant (_0__)
                        )?
                        $({ $($fatal_sname),* })?
                        => {
                            ::core::write!(
                                f__,
                                $fatal_fmt,
                                $({ let _: $fatal_tty; _0__ },)?
                            )
                        }
                    )*
                    $(
                        Self::$nonfatal_variant
                        $(
                            (_) if false => {
                                let _: $nonfatal_tty; ::core::unreachable!()
                            }
                            Self::$nonfatal_variant (_0__)
                        )?
                        $({ $($nonfatal_sname),* })?
                        => {
                            ::core::write!(
                                f__,
                                $nonfatal_fmt,
                                $({ let _: $nonfatal_tty; _0__ },)?
                            )
                        }
                    )*
                }
            }
        }

        #[derive(Debug)]
        pub enum NonFatal {
            $(
                $nonfatal_variant
                $(($nonfatal_tty))?
                $({ $($nonfatal_sname: $nonfatal_sty,)* })?
                ,
            )*
        }

        impl NonFatal {
            pub fn worsen(self) -> Fatal {
                match self {
                    $(
                        Self::$nonfatal_variant
                        $(
                            (_) if false => {
                                let _: $nonfatal_tty; ::core::unreachable!()
                            }
                            Self::$nonfatal_variant (_0__)
                        )?
                        $({ $($nonfatal_sname),* })?
                        => Fatal::$nonfatal_variant
                        $(({ let _: $nonfatal_tty; _0__ }))?
                        $({ $($nonfatal_sname),* })?,
                    )*
                }
            }
        }

        impl ::core::fmt::Display for NonFatal {
            fn fmt(
                &self,
                f__: &mut ::core::fmt::Formatter<'_>,
            ) -> ::core::fmt::Result {
                match self {
                    $(
                        Self::$nonfatal_variant
                        $(
                            (_) if false => {
                                let _: $nonfatal_tty; ::core::unreachable!()
                            }
                            Self::$nonfatal_variant (_0__)
                        )?
                        $({ $($nonfatal_sname: $nonfatal_sty),* })?
                        => {
                            ::core::write!(
                                f__,
                                $nonfatal_fmt,
                                $({ let _: $nonfatal_tty; _0__ },)?
                            )
                        }
                    )*
                }
            }
        }
    }
}

define_errors! {
    pub enum Fatal {
        #[error("`{name}` failed: {errno}")]
        SyscallFailed { name: &'static str, errno: Errno },
        #[error("{0}")]
        InvalidPath(NulError),
        #[error("child message too short (expected 9 bytes, got {0} bytes)")]
        MessageTooShort(usize),
        #[error("invalid child messaage (id={id}, status={status}])")]
        InvalidMessage { id: u8, status: i64 },
        #[error("ptrace failed in launched process with status {0}")]
        FailedToPtraceChild(i64),
        #[error("exec failed in launched process with status {0}")]
        FailedToExecChild(i64),
    }

    pub enum NonFatal {
        #[error("cannot resume from '{0}' state")]
        CannotResume(State),
        #[error("cannot stop from '{0}' state")]
        CannotStop(State),
    }
}

impl Fatal {
    pub fn syscall(name: &'static str) -> Self {
        Self::SyscallFailed {
            name,
            errno: Errno::read(),
        }
    }
}

pub trait Recoverable: Sized {
    type Value;

    fn worsen(self) -> Result<Self::Value, Fatal> {
        self.recover()?.map_err(NonFatal::worsen)
    }

    fn recover(self) -> Result<Result<Self::Value, NonFatal>, Fatal>;
}

impl<T> Recoverable for Result<T, Error> {
    type Value = T;

    fn recover(self) -> Result<Result<Self::Value, NonFatal>, Fatal> {
        match self {
            Ok(value) => Ok(Ok(value)),
            Err(e) => e.recover().map(|c| Err(c)),
        }
    }
}
