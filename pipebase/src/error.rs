use std::fmt::{self, Debug, Display};
use std::{error, result};

/// An error that happened when generate / run the pipe
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
    Csv(csv::Error),
    IO(std::io::Error),
    Join(tokio::task::JoinError),
    ParseEnum(strum::ParseError),
    Receive(std::sync::mpsc::RecvError),
    ReceiveTimeout(std::sync::mpsc::RecvTimeoutError),
    Send(String),
    Yaml(serde_yaml::Error),
}

impl ErrorImpl {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ErrorImpl::IO(err) => Some(err),
            ErrorImpl::Join(err) => Some(err),
            ErrorImpl::Yaml(err) => Some(err),
            ErrorImpl::ParseEnum(err) => Some(err),
            ErrorImpl::Csv(err) => Some(err),
            ErrorImpl::Receive(err) => Some(err),
            ErrorImpl::ReceiveTimeout(err) => Some(err),
            _ => None,
        }
    }

    fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorImpl::Api(msg) => Display::fmt(msg, f),
            ErrorImpl::IO(err) => Display::fmt(err, f),
            ErrorImpl::Join(err) => Display::fmt(err, f),
            ErrorImpl::Yaml(err) => Display::fmt(err, f),
            ErrorImpl::ParseEnum(err) => Display::fmt(err, f),
            ErrorImpl::Csv(err) => Display::fmt(err, f),
            ErrorImpl::Receive(err) => Display::fmt(err, f),
            ErrorImpl::ReceiveTimeout(err) => Display::fmt(err, f),
            ErrorImpl::Send(msg) => Display::fmt(msg, f),
        }
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorImpl::Api(msg) => f.debug_tuple("Api").field(msg).finish(),
            ErrorImpl::IO(err) => f.debug_tuple("Io").field(err).finish(),
            ErrorImpl::Join(err) => f.debug_tuple("Join").field(err).finish(),
            ErrorImpl::Yaml(err) => f.debug_tuple("Yaml").field(err).finish(),
            ErrorImpl::ParseEnum(err) => f.debug_tuple("ParseEnum").field(err).finish(),
            ErrorImpl::Csv(err) => f.debug_tuple("Csv").field(err).finish(),
            ErrorImpl::Receive(err) => f.debug_tuple("Recv").field(err).finish(),
            ErrorImpl::ReceiveTimeout(err) => f.debug_tuple("RecvTimeout").field(err).finish(),
            ErrorImpl::Send(msg) => f.debug_tuple("Send").field(msg).finish(),
        }
    }
}

pub fn api_error(msg: &str) -> Error {
    Error(Box::new(ErrorImpl::Api(format!("[Api Error] {}", msg))))
}

pub fn io_error(err: std::io::Error) -> Error {
    Error(Box::new(ErrorImpl::IO(err)))
}

pub fn join_error(err: tokio::task::JoinError) -> Error {
    Error(Box::new(ErrorImpl::Join(err)))
}

pub fn yaml_error(err: serde_yaml::Error) -> Error {
    Error(Box::new(ErrorImpl::Yaml(err)))
}

pub fn parse_enum_error(err: strum::ParseError) -> Error {
    Error(Box::new(ErrorImpl::ParseEnum(err)))
}

pub fn csv_error(err: csv::Error) -> Error {
    Error(Box::new(ErrorImpl::Csv(err)))
}

pub fn recv_error(err: std::sync::mpsc::RecvError) -> Error {
    Error(Box::new(ErrorImpl::Receive(err)))
}

pub fn recv_timeout_error(err: std::sync::mpsc::RecvTimeoutError) -> Error {
    Error(Box::new(ErrorImpl::ReceiveTimeout(err)))
}

pub fn send_error(msg: &str) -> Error {
    Error(Box::new(ErrorImpl::Send(format!("[Send Error] {}", msg))))
}
