use crate::config::Config;
use crate::errors::CmdResult;

pub(crate) mod check;
pub(crate) mod describe;
pub(crate) mod init;

pub type Cmd = clap::App<'static>;

pub fn cmds() -> Vec<Cmd> {
    vec![check::cmd(), describe::cmd(), init::cmd()]
}

pub fn exec(cmd: &str) -> Option<fn(&Config, &clap::ArgMatches) -> CmdResult> {
    let f = match cmd {
        "check" => check::exec,
        "describe" => describe::exec,
        "init" => init::exec,
        _ => return None,
    };
    Some(f)
}
