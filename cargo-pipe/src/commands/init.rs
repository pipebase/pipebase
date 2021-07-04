use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use crate::ops::do_init::do_exec;

pub fn cmd() -> Cmd {
    Cmd::new("init")
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult {
    let ops = InitOptions::new();
    do_exec(config, &ops)?;
    Ok(())
}

pub struct InitOptions {}

impl InitOptions {
    pub fn new() -> Self {
        InitOptions {}
    }
}
