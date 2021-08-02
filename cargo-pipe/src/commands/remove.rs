use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use crate::ops::do_remove::do_exec;
use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("remove")
        .about("Remove pipe app")
        .args(vec![Arg::new("name")
            .short('n')
            .about("app name")
            .takes_value(true)])
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult {
    let opts = match args.value_of("name") {
        Some(name) => RemoveOptions::new(Some(name.to_owned())),
        None => RemoveOptions::new(None),
    };
    do_exec(config, &opts)?;
    Ok(())
}

pub struct RemoveOptions {
    app_name: Option<String>,
}

impl RemoveOptions {
    pub fn new(app_name: Option<String>) -> Self {
        RemoveOptions { app_name }
    }

    pub fn get_app_name(&self) -> Option<&String> {
        self.app_name.as_ref()
    }
}
