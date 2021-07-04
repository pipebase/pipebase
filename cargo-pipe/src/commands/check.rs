use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use crate::ops::do_check::do_exec;
use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("check")
        .about("Cargo check pipe app code")
        .args(vec![
            Arg::new("name")
                .short('n')
                .about("app name")
                .takes_value(true),
            Arg::new("warning").short('w').about("enable warning"),
            Arg::new("verbose").short('v').about("enable verbose"),
        ])
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult {
    let app_name = match args.value_of("name") {
        Some(app_name) => Some(app_name.to_owned()),
        None => None,
    };
    let warning = args.is_present("warning");
    let verbose = args.is_present("verbose");
    let opts = CheckOptions::new(app_name, warning, verbose);
    do_exec(config, &opts)?;
    Ok(())
}

pub struct CheckOptions {
    app_name: Option<String>,
    warning: bool,
    verbose: bool,
}

impl CheckOptions {
    pub fn new(app_name: Option<String>, warning: bool, verbose: bool) -> Self {
        CheckOptions {
            app_name,
            warning,
            verbose,
        }
    }

    pub fn get_app_name(&self) -> Option<&String> {
        self.app_name.as_ref()
    }

    pub fn warning(&self) -> bool {
        self.warning
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }
}
