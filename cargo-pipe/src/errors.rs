use std::{io, string::FromUtf8Error};
use thiserror::Error;

pub type CmdResult<T> = Result<T, CmdError>;

#[derive(Debug)]
pub struct CmdError {
    pub error: Box<ErrorImpl>,
    pub exit_code: i32,
}

#[derive(Debug, Error)]
pub enum ErrorImpl {
    #[error("clap exception")]
    Clap(#[from] clap::Error),
    #[error("io exception")]
    Io(#[from] io::Error),
    #[error("pipegen exception")]
    Pipegen(#[from] pipegen::error::Error),
    #[error("toml deserialize exception")]
    TomlDe(#[from] toml::de::Error),
    #[error("toml Serialize exception")]
    TomlSer(#[from] toml::ser::Error),
    #[error("utf8 exception")]
    Utf8(#[from] FromUtf8Error),
    #[error("cargo {cmd:?} error")]
    Cargo { cmd: String },
}

impl CmdError {
    pub fn new(error: Box<ErrorImpl>, exit_code: i32) -> CmdError {
        CmdError { error, exit_code }
    }
}

impl From<pipegen::error::Error> for CmdError {
    fn from(err: pipegen::error::Error) -> CmdError {
        CmdError::new(Box::new(ErrorImpl::Pipegen(err)), 101)
    }
}

impl From<io::Error> for CmdError {
    fn from(err: io::Error) -> Self {
        CmdError::new(Box::new(ErrorImpl::Io(err)), 102)
    }
}

impl From<toml::de::Error> for CmdError {
    fn from(err: toml::de::Error) -> Self {
        CmdError::new(Box::new(ErrorImpl::TomlDe(err)), 103)
    }
}

impl From<toml::ser::Error> for CmdError {
    fn from(err: toml::ser::Error) -> Self {
        CmdError::new(Box::new(ErrorImpl::TomlSer(err)), 104)
    }
}

impl From<FromUtf8Error> for CmdError {
    fn from(err: FromUtf8Error) -> Self {
        CmdError::new(Box::new(ErrorImpl::Utf8(err)), 105)
    }
}

impl From<clap::Error> for CmdError {
    fn from(err: clap::Error) -> CmdError {
        let code = if err.use_stderr() { 1 } else { 0 };
        CmdError::new(Box::new(ErrorImpl::Clap(err)), code)
    }
}

pub fn cargo_error(cmd: &str, status_code: i32) -> CmdError {
    CmdError::new(
        Box::new(ErrorImpl::Cargo {
            cmd: String::from(cmd),
        }),
        status_code,
    )
}
