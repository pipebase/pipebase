pub type CmdResult = Result<(), CmdError>;

#[derive(Debug)]
pub struct CmdError {
    pub error: anyhow::Error,
    pub exit_code: i32,
}

impl CmdError {
    pub fn new(error: anyhow::Error, code: i32) -> CmdError {
        CmdError {
            error: error,
            exit_code: code,
        }
    }
}

impl From<anyhow::Error> for CmdError {
    fn from(err: anyhow::Error) -> CmdError {
        CmdError::new(err, 101)
    }
}

impl From<clap::Error> for CmdError {
    fn from(err: clap::Error) -> CmdError {
        let code = if err.use_stderr() { 1 } else { 0 };
        CmdError::new(err.into(), code)
    }
}
