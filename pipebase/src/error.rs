use std::fmt::{self, Debug, Display};
use std::{error, result};

/// An error that happened when run the pipe
pub struct Error(Box<ErrorImpl>);

pub type Result<T> = result::Result<T, Error>;

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.0.source()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.display(f)
    }
}

impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.debug(f)
    }
}
#[derive(Debug)]
pub enum ErrorImpl {
    IO(std::io::Error),
    Join(tokio::task::JoinError),
}

impl ErrorImpl {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ErrorImpl::IO(err) => Some(err),
            ErrorImpl::Join(err) => Some(err),
        }
    }

    fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorImpl::IO(err) => Display::fmt(err, f),
            ErrorImpl::Join(err) => Display::fmt(err, f),
        }
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorImpl::IO(err) => f.debug_tuple("Io").field(err).finish(),
            ErrorImpl::Join(err) => f.debug_tuple("Join").field(err).finish(),
        }
    }
}

pub fn io_error(err: std::io::Error) -> Error {
    Error(Box::new(ErrorImpl::IO(err)))
}

pub fn join_error(err: tokio::task::JoinError) -> Error {
    Error(Box::new(ErrorImpl::Join(err)))
}
