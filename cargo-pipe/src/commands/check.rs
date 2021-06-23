use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use clap::Arg;
use pipegen::api::App;

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
    match (args.is_present("pipe"), args.is_present("object")) {
        (true, false) => println!("validate pipes"),
        (false, true) => println!("validate objects"),
        (_, _) => println!("check all"),
    }
    Ok(())
}
