use std::fmt::{self, Debug, Display};
use std::{error, result};
use tokio::sync::mpsc::Sender;
use tracing::error;

/// Runtime error
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
    Any(anyhow::Error),
    IO(std::io::Error),
    Join(tokio::task::JoinError),
}

impl ErrorImpl {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            ErrorImpl::Any(err) => err.source(),
            ErrorImpl::IO(err) => Some(err),
            ErrorImpl::Join(err) => Some(err),
        }
    }

    fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorImpl::Any(err) => Display::fmt(err, f),
            ErrorImpl::IO(err) => Display::fmt(err, f),
            ErrorImpl::Join(err) => Display::fmt(err, f),
        }
    }

    fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorImpl::Any(err) => f.debug_tuple("Anyhow").field(err).finish(),
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

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error(Box::new(ErrorImpl::IO(err)))
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(err: tokio::task::JoinError) -> Self {
        Error(Box::new(ErrorImpl::Join(err)))
    }
}

impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        Error(Box::new(ErrorImpl::Any(err)))
    }
}

#[derive(Debug)]
pub struct PipeError {
    pub pipe_name: String,
    pub error: anyhow::Error,
}

impl PipeError {
    pub fn new(pipe_name: String, error: anyhow::Error) -> Self {
        PipeError { pipe_name, error }
    }
}

pub trait SubscribeError {
    fn subscribe_error(&mut self, tx: Sender<PipeError>);
}

pub(crate) async fn send_pipe_error(tx: Option<&Sender<PipeError>>, pipe_error: PipeError) {
    let tx = match tx {
        Some(tx) => tx,
        None => return,
    };
    match tx.send(pipe_error).await {
        Ok(_) => (),
        Err(e) => {
            error!("send pipe error failed '{}'", e)
        }
    }
}
