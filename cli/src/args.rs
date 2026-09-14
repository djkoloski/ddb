use core::{fmt, num::ParseIntError, str::Utf8Error};
use std::{env::args_os, path::PathBuf};

use libc::pid_t;

#[derive(Debug)]
pub enum ParseArgsError {
    MissingArgs,
    ExpectedPid,
    PidNotUtf8(Utf8Error),
    PidNotInteger(ParseIntError),
    TrailingArgs,
}

impl fmt::Display for ParseArgsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingArgs => write!(f, "no attach target was provided")?,
            Self::ExpectedPid => write!(f, "expected pid after -p")?,
            Self::PidNotUtf8(e) => write!(f, "pid was not valid UTF-8: {e}")?,
            Self::PidNotInteger(e) => {
                write!(f, "pid was not a valid integer: {e}")?
            }
            Self::TrailingArgs => write!(f, "unrecognized trailing arguments")?,
        }

        Ok(())
    }
}

pub struct Args {
    pub command: Subcommand,
}

pub enum Subcommand {
    Attach { pid: pid_t },
    Launch { path: PathBuf },
}

impl Args {
    pub fn parse() -> Result<Args, ParseArgsError> {
        let mut command = None;

        let mut args = args_os();
        let _ = args.next();

        while let Some(next) = args.next() {
            if command.is_some() {
                return Err(ParseArgsError::TrailingArgs);
            }

            if next == "-p" {
                let second = args.next().ok_or(ParseArgsError::ExpectedPid)?;
                let pid = <&str>::try_from(&*second)
                    .map_err(ParseArgsError::PidNotUtf8)?
                    .parse()
                    .map_err(ParseArgsError::PidNotInteger)?;
                command = Some(Subcommand::Attach { pid });
            } else {
                command = Some(Subcommand::Launch {
                    path: PathBuf::from(next),
                });
            }
        }

        Ok(Args {
            command: command.ok_or(ParseArgsError::MissingArgs)?,
        })
    }
}
