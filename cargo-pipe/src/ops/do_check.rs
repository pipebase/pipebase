use super::do_cargo::*;
use super::do_validate::do_validate;
use super::utils::*;
use crate::commands::check::CheckOptions;
use crate::commands::validate::ValidateOptions;
use crate::print::Printer;
use crate::Config;
use std::process;

pub fn do_check(config: &Config, opts: &CheckOptions, printer: &mut Printer) -> anyhow::Result<()> {
    // validate pipe manifest as prerequiste
    let pipe_manifest_path = config.get_pipe_manifest_path();
    let app = read_pipe_manifest(pipe_manifest_path.as_path(), printer)?;
    do_validate(&app, printer, &ValidateOptions::default())?;
    // cargo check
    let manifest_path_buf = config.get_app_manifest(opts.get_app_name());
    let manifest_path = manifest_path_buf.as_path();
    printer.status(&"Check", manifest_path.to_str().unwrap())?;
    let warning = opts.warning();
    let verbose = opts.verbose();
    let status_code = do_cargo_check(manifest_path, warning, verbose, printer)?;
    match status_code {
        0 => (),
        _ => process::exit(status_code),
    };
    Ok(())
}

pub fn do_exec(config: &Config, opts: &CheckOptions) -> anyhow::Result<()> {
    let mut printer = Printer::new();
    do_check(config, opts, &mut printer)
}
