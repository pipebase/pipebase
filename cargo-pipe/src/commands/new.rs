use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use crate::ops::do_new::do_exec;
use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("new")
        .about("Create a new pipe app")
        .args(vec![Arg::new("name")
            .short('n')
            .about("Specify the app name")
            .takes_value(true)])
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult {
    let opts = match args.value_of("name") {
        Some(name) => NewOptions::new(Some(name.to_owned())),
        None => NewOptions::new(None),
    };
    do_exec(config, &opts)?;
    Ok(())
}

pub struct NewOptions {
    app_name: Option<String>,
}

impl NewOptions {
    pub fn new(app_name: Option<String>) -> Self {
        NewOptions { app_name }
    }

    pub fn get_app_name(&self) -> Option<&String> {
        self.app_name.as_ref()
    }
}
