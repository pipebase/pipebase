use crate::config::Config;
use crate::errors::CmdResult;

pub(crate) mod build;
pub(crate) mod check;
pub(crate) mod describe;
pub(crate) mod generate;
pub(crate) mod new;
pub(crate) mod remove;
pub(crate) mod validate;

pub type Cmd = clap::App<'static>;

pub fn cmds() -> Vec<Cmd> {
    vec![
        validate::cmd(),
        describe::cmd(),
        new::cmd(),
        remove::cmd(),
        generate::cmd(),
        check::cmd(),
        build::cmd(),
    ]
}

pub fn exec(cmd: &str) -> Option<fn(&Config, &clap::ArgMatches) -> CmdResult> {
    let f = match cmd {
        "validate" => validate::exec,
        "describe" => describe::exec,
        "new" => new::exec,
        "remove" => remove::exec,
        "generate" => generate::exec,
        "check" => check::exec,
        "build" => build::exec,
        _ => return None,
    };
    Some(f)
}
