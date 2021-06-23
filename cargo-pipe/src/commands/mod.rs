use clap::ArgMatches;

use crate::config::Config;
use crate::errors::CmdResult;

mod check;
mod describe;

pub type Cmd = clap::App<'static>;

pub fn cmds() -> Vec<Cmd> {
    vec![check::cmd()]
}

pub fn exec(cmd: &str) -> Option<fn(&Config, &clap::ArgMatches) -> CmdResult> {
    let f = match cmd {
        "check" => check::exec,
        _ => return None,
    };
    Some(f)
}
