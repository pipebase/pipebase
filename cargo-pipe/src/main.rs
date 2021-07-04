use config::Config;
use errors::CmdResult;
use std::io::{self, Write};
use std::process;

mod commands;
mod config;
mod errors;
mod ops;
mod print;

fn main() {
    let result = run();
    process::exit(match result {
        Ok(_) => 0,
        Err(err) => {
            let _ = writeln!(io::stderr(), "{}", err.error);
            err.exit_code
        }
    })
}

fn run() -> CmdResult {
    // setup args and subcommands (including subcommand args)
    let matches = clap::App::new("cargo-pipe")
        .arg(clap::Arg::new("pipe").index(1))
        .arg(
            clap::Arg::new("directory")
                .short('d')
                .takes_value(true)
                .about("Absolute path to working directory"),
        )
        .subcommands(commands::cmds())
        .get_matches();

    // setup CLI configuration
    let config = match matches.value_of("directory") {
        Some(directory) => Config::new(Some(directory)),
        None => Config::new(None),
    };
    let config = match config {
        Ok(config) => config,
        Err(err) => return Err(err.into()),
    };
    // find subcommand and args for subcommand
    let (cmd, args) = match matches.subcommand() {
        Some((cmd, args)) => (cmd, args),
        None => return Ok(()),
    };
    // execute subcommand
    match commands::exec(cmd) {
        Some(f) => f(&config, args)?,
        None => (),
    };
    Ok(())
}
