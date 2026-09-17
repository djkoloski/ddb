use std::{ffi::NulError, string::FromUtf8Error};

use crate::{Errno, RegisterTypeError, State};

macro_rules! define_errors {
    (
        #[fatal]
        $(#[$fatal_metas:meta])*
        pub enum $fatal:ident {
            $(
                #[error($fatal_fmt:literal)]
                $fatal_variant:ident
                // Tuple variant
                $(($fatal_tty:ty $(,)?))?
                // Struct variant
                $({ $($fatal_sname:ident: $fatal_sty:ty),* $(,)? })?
            ),* $(,)?
        }

        #[non_fatal]
        $(#[$non_fatal_metas:meta])*
        pub enum $non_fatal:ident {
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
            fatal: $fatal,
        }

        impl Error {
            pub fn recover(self) -> ::core::result::Result<$non_fatal, $fatal> {
                match self.fatal {
                    $(
                        $fatal::$fatal_variant
                        $(
                            (_) if false => {
                                let _: $fatal_tty; ::core::unreachable!()
                            }
                            $fatal::$fatal_variant (_0__)
                        )?
                        $({ $($fatal_sname),* })?
                        => Err(
                            $fatal::$fatal_variant
                            $(({ let _: $fatal_tty; _0__ }))?
                            $({ $($fatal_sname),* })?
                        ),
                    )*
                    $(
                        $fatal::$nonfatal_variant
                        $(
                            (_) if false => {
                                let _: $nonfatal_tty; ::core::unreachable!()
                            }
                            $fatal::$nonfatal_variant (_0__)
                        )?
                        $({ $($nonfatal_sname),* })?
                        => Ok(
                            $non_fatal::$nonfatal_variant
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

        impl From<$fatal> for Error {
            fn from(fatal: $fatal) -> Self {
                Self { fatal }
            }
        }

        impl From<$non_fatal> for Error {
            fn from(non_fatal: $non_fatal) -> Self {
                Self {
                    fatal: non_fatal.worsen(),
                }
            }
        }

        #[derive(::core::fmt::Debug)]
        pub enum $fatal {
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

        impl ::core::fmt::Display for $fatal {
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
        pub enum $non_fatal {
            $(
                $nonfatal_variant
                $(($nonfatal_tty))?
                $({ $($nonfatal_sname: $nonfatal_sty,)* })?
                ,
            )*
        }

        impl $non_fatal {
            pub fn worsen(self) -> $fatal {
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
                        => $fatal::$nonfatal_variant
                        $(({ let _: $nonfatal_tty; _0__ }))?
                        $({ $($nonfatal_sname),* })?,
                    )*
                }
            }
        }

        impl ::core::fmt::Display for $non_fatal {
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
    #[fatal]
    pub enum Fatal {
        #[error("`{name}` failed: {errno}")]
        SyscallFailed { name: &'static str, errno: Errno },
        #[error("{0}")]
        InvalidArg(NulError),
        #[error("invalid child messaage: {0}")]
        InvalidChildMessage(FromUtf8Error),
        #[error("child process failed: {0}")]
        ChildFailed(String),
        #[error("{0}")]
        RegisterTypeError(RegisterTypeError),
    }

    #[non_fatal]
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
