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
    let mut args = std::env::args_os().peekable();
    let mut cmd_and_args = vec![args.next().unwrap()]; // cargo
    if args.peek().map_or(false, |arg| arg == "pipe") {
        args.next().unwrap(); // pipe
    }
    cmd_and_args.extend(args);
    // setup args and subcommands (including subcommand args)
    let matches = clap::App::new("cargo-pipe")
        .arg(
            clap::Arg::new("directory")
                .short('d')
                .takes_value(true)
                .about("Absolute path to working directory"),
        )
        .subcommands(commands::cmds())
        .get_matches_from(cmd_and_args);

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
