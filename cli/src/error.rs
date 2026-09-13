use core::fmt;
use std::io;

use crate::args::ParseArgsError;

#[derive(Debug)]
pub enum Error {
    ParseArgs(ParseArgsError),
    Debugger(ddb::Fatal),
    Io(io::Error),
}

impl From<ParseArgsError> for Error {
    fn from(e: ParseArgsError) -> Self {
        Self::ParseArgs(e)
    }
}

impl From<ddb::Fatal> for Error {
    fn from(e: ddb::Fatal) -> Self {
        Self::Debugger(e)
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

const USAGE: &str = "\
USAGE: ddb (-p <pid> | <path>)
  Debugs a process. If -p is provided, ddb will attach to the process with the
  given pid. Otherwise, ddb will launch the process at the given path and \
                     attach
  to it.";

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ParseArgs(e) => {
                write!(f, "unable to parse arguments: {e}\n\n{USAGE}")?;
            }
            Self::Debugger(e) => {
                write!(f, "{e}")?;
            }
            Self::Io(e) => {
                write!(f, "{e}")?;
            }
        }

        Ok(())
    }
}
