use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use crate::ops::do_build::do_exec;
use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("build").about("Cargo build pipe app").args(vec![
        Arg::new("name")
            .short('n')
            .about("Specify app name")
            .takes_value(true),
        Arg::new("out")
            .short('o')
            .about("Specify output binary path")
            .takes_value(true),
    ])
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult {
    let app_name = match args.value_of("name") {
        Some(app_name) => Some(app_name.to_owned()),
        None => None,
    };
    let out = match args.value_of("out") {
        Some(out) => Some(out.to_owned()),
        None => None,
    };
    let opts = BuildOptions::new(app_name, out);
    do_exec(config, &opts)?;
    Ok(())
}

pub struct BuildOptions {
    app_name: Option<String>,
    out: Option<String>,
}

impl BuildOptions {
    pub fn new(app_name: Option<String>, out: Option<String>) -> Self {
        BuildOptions { app_name, out }
    }

    pub fn get_app_name(&self) -> Option<&String> {
        self.app_name.as_ref()
    }

    pub fn out(&self) -> Option<&String> {
        self.out.as_ref()
    }
}