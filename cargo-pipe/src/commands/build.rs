use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use crate::ops::do_build::do_exec;
use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("build").about("Cargo build pipe app").args(vec![
        Arg::new("name")
            .short('n')
            .help("Specify the app name")
            .takes_value(true),
        Arg::new("out")
            .short('o')
            .help("Specify output binary path")
            .takes_value(true),
        Arg::new("release")
            .short('r')
            .help("Specify build in release mode with optimizations"),
        Arg::new("debug").short('d').help("Enable debug"),
        Arg::new("verbose").short('v').help("Enable verbose"),
    ])
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult<()> {
    let app_name = args.value_of("name").map(|app_name| app_name.to_owned());
    let out = args.value_of("out").map(|out| out.to_owned());
    let release = args.is_present("release");
    let debug = args.is_present("debug");
    let verbose = args.is_present("verbose");
    let opts = BuildOptions::new(app_name, out, release, debug, verbose);
    do_exec(config, &opts)?;
    Ok(())
}

pub struct BuildOptions {
    app_name: Option<String>,
    out: Option<String>,
    release: bool,
    debug: bool,
    verbose: bool,
}

impl BuildOptions {
    pub fn new(
        app_name: Option<String>,
        out: Option<String>,
        release: bool,
        debug: bool,
        verbose: bool,
    ) -> Self {
        BuildOptions {
            app_name,
            out,
            release,
            debug,
            verbose,
        }
    }

    pub fn get_app_name(&self) -> Option<&String> {
        self.app_name.as_ref()
    }

    pub fn out(&self) -> Option<&String> {
        self.out.as_ref()
    }

    pub fn release(&self) -> bool {
        self.release
    }

    pub fn debug(&self) -> bool {
        self.debug
    }

    pub fn verbose(&self) -> bool {
        self.verbose
    }
}
