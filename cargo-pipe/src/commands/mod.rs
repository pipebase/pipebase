use crate::config::Config;
use crate::errors::CmdResult;

pub(crate) mod check;
pub(crate) mod describe;
pub(crate) mod generate;
pub(crate) mod new;
pub(crate) mod remove;

pub type Cmd = clap::App<'static>;

pub fn cmds() -> Vec<Cmd> {
    vec![
        check::cmd(),
        describe::cmd(),
        new::cmd(),
        remove::cmd(),
        generate::cmd(),
    ]
}

pub fn exec(cmd: &str) -> Option<fn(&Config, &clap::ArgMatches) -> CmdResult> {
    let f = match cmd {
        "check" => check::exec,
        "describe" => describe::exec,
        "new" => new::exec,
        "remove" => remove::exec,
        "generate" => generate::exec,
        _ => return None,
    };
    Some(f)
}
