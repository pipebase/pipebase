use config::Config;
use errors::CmdResult;

mod commands;
mod config;
mod errors;

fn main() -> CmdResult {
    let matches = clap::App::new("cargo-pipe")
        .arg(clap::Arg::new("pipe").index(1))
        .arg(
            clap::Arg::new("manifest")
                .short('m')
                .takes_value(true)
                .about("Absolute path to manifest"),
        )
        .subcommands(commands::cmds())
        .get_matches();

    let config = match matches.value_of("manifest") {
        Some(manifest) => Config::new(Some(manifest)),
        None => Config::new(None),
    };
    let config = match config {
        Ok(config) => config,
        Err(err) => return Err(err.into()),
    };
    let (cmd, args) = match matches.subcommand() {
        Some((cmd, args)) => (cmd, args),
        None => return Ok(()),
    };
    match commands::exec(cmd) {
        Some(f) => f(&config, args)?,
        None => return Ok(()),
    };
    Ok(())
}
