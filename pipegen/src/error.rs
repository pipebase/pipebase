use std::fmt::{self, Debug, Display};
use std::{error, result};

/// An error that happened when generate the pipe
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
    Api(String),
    IO(std::io::Error),
    Yaml(serde_yaml::Error),
}

impl ErrorImpl {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ErrorImpl::IO(err) => Some(err),
            ErrorImpl::Yaml(err) => Some(err),
            _ => None,
        }
    }

    fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorImpl::Api(msg) => Display::fmt(msg, f),
            ErrorImpl::IO(err) => Display::fmt(err, f),
            ErrorImpl::Yaml(err) => Display::fmt(err, f),
        }
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorImpl::Api(msg) => f.debug_tuple("Api").field(msg).finish(),
            ErrorImpl::IO(err) => f.debug_tuple("Io").field(err).finish(),
            ErrorImpl::Yaml(err) => f.debug_tuple("Yaml").field(err).finish(),
        }
    }
}

pub fn api_error<E: Display>(detail: E) -> Error {
    Error(Box::new(ErrorImpl::Api(format!("{}", detail))))
}

pub fn io_error(err: std::io::Error) -> Error {
    Error(Box::new(ErrorImpl::IO(err)))
}

pub fn yaml_error(err: serde_yaml::Error) -> Error {
    Error(Box::new(ErrorImpl::Yaml(err)))
}
