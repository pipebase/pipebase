use config::Config;
use errors::CmdResult;
use std::io::{self, Write};
use std::process;

use crate::print::Printer;

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

fn run() -> CmdResult<()> {
    let mut args = std::env::args_os().peekable();
    let mut cmd_and_args = vec![args.next().unwrap()]; // cargo
    if args.peek().map_or(false, |arg| arg == "pipe") {
        args.next().unwrap(); // pipe
    }
    cmd_and_args.extend(args);
    // setup args and subcommands (including subcommand args)
    let matches = clap::App::new("cargo-pipe")
        .args(vec![
            clap::Arg::new("directory")
                .short('d')
                .takes_value(true)
                .help("Absolute path to working directory"),
            clap::Arg::new("manifest")
                .short('m')
                .takes_value(true)
                .help("Manifest file name in working directory"),
        ])
        .subcommands(commands::cmds())
        .get_matches_from(cmd_and_args);

    let mut printer = Printer::new();
    // setup CLI configuration
    let directory = matches.value_of("directory");
    let manifest = matches.value_of("manifest");
    let config = match Config::new(directory, manifest) {
        Ok(config) => config,
        Err(err) => {
            printer.error(format!("new config error: {:#?}", err))?;
            return Err(err);
        }
    };
    // find subcommand and args for subcommand
    let (cmd, args) = match matches.subcommand() {
        Some((cmd, args)) => (cmd, args),
        None => {
            // if no subcomand match, clap should throw exception
            printer.error("no matched subcommand")?;
            unreachable!();
        }
    };
    // execute subcommand
    match commands::exec(cmd) {
        Some(f) => f(&config, args)?,
        None => {
            // subcommand exec method not available yet
            printer.error(format!("subcommand::exec {} not found", cmd))?;
            unreachable!()
        }
    };
    Ok(())
}
