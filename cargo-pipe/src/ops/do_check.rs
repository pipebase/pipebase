use super::do_cargo::*;
use super::do_validate::do_validate;
use super::utils::*;
use crate::{
    commands::{check::CheckOptions, validate::ValidateOptions},
    config::Config,
    errors::{cargo_error, CmdResult},
    print::Printer,
};

pub fn do_check(config: &Config, opts: &CheckOptions, printer: &mut Printer) -> CmdResult<()> {
    // validate pipe manifest as prerequiste
    let pipe_manifest_path = config.get_pipe_manifest_path();
    let app = read_pipe_manifest(pipe_manifest_path.as_path(), printer)?;
    do_validate(&app, printer, &ValidateOptions::default())?;
    // cargo check
    let manifest_path_buf = config.get_app_manifest(opts.get_app_name());
    let manifest_path = manifest_path_buf.as_path();
    printer.status(&"Check", manifest_path.to_str().unwrap())?;
    let verbose = opts.verbose();
    let debug = opts.debug();
    let status_code = do_cargo_check(manifest_path, verbose, debug, printer)?;
    match status_code {
        0 => (),
        _ => return Err(cargo_error("check", status_code)),
    };
    Ok(())
}

pub fn do_exec(config: &Config, opts: &CheckOptions) -> CmdResult<()> {
    let mut printer = Printer::new();
    do_check(config, opts, &mut printer)
}
