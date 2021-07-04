use super::do_cargo::*;
use crate::commands::check::CheckOptions;
use crate::print::Printer;
use crate::Config;
use std::path::PathBuf;
use std::process;

pub fn do_check(
    mut path_buf: PathBuf,
    warning: bool,
    verbose: bool,
    printer: &mut Printer,
) -> anyhow::Result<()> {
    path_buf.push(CARGO_MANIFEST_FILE);
    let manifest_path = path_buf.as_path();
    printer.status(&"Check", manifest_path.to_str().unwrap())?;
    let status_code = do_cargo_check(manifest_path, warning, verbose, printer)?;
    match status_code {
        0 => (),
        _ => process::exit(status_code),
    };
    Ok(())
}

pub fn do_exec(config: &Config, opts: &CheckOptions) -> anyhow::Result<()> {
    let mut printer = Printer::new();
    do_check(
        config.get_app_directory(opts.get_app_name()),
        opts.warning(),
        opts.verbose(),
        &mut printer,
    )
}
