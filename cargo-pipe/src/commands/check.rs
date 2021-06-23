use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use crate::ops::do_check::do_exec;
use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("check").args(vec![
        Arg::new("pipe")
            .short('p')
            .about("validate pipes in manifest"),
        Arg::new("object")
            .short('o')
            .about("validate objects in manifest"),
    ])
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult {
    let opts = match (args.is_present("pipe"), args.is_present("object")) {
        (true, false) => CheckOptions {
            pipe: true,
            object: false,
        },
        (false, true) => CheckOptions {
            pipe: false,
            object: true,
        },
        (_, _) => CheckOptions {
            pipe: true,
            object: true,
        },
    };
    do_exec(config, &opts)?;
    Ok(())
}

pub struct CheckOptions {
    pipe: bool,
    object: bool,
}

impl CheckOptions {
    pub fn check_pipe(&self) -> bool {
        self.pipe
    }
    pub fn check_object(&self) -> bool {
        self.object
    }
}
