use super::do_cargo::*;
use super::do_remove::do_remove;
use crate::commands::new::NewOptions;
use crate::print::Printer;
use crate::Config;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn do_remove_app_if_exists(path: &Path, printer: &mut Printer) -> anyhow::Result<()> {
    do_remove(path.to_path_buf(), printer)
}

fn do_new_directory(path: &Path, printer: &mut Printer) -> anyhow::Result<()> {
    printer.status(&"New", path.to_str().unwrap())?;
    fs::create_dir(path)?;
    printer.status(&"New", "succeed")?;
    Ok(())
}

pub fn do_new(printer: &mut Printer, path_buf: PathBuf) -> anyhow::Result<()> {
    let path = path_buf.as_path();
    do_remove_app_if_exists(path, printer)?;
    do_new_directory(path, printer)?;
    // cargo init
    let status_code = do_cargo_init(path, printer)?;
    match status_code {
        0 => (),
        _ => process::exit(status_code),
    };
    Ok(())
}

pub fn do_exec(config: &Config, opts: &NewOptions) -> anyhow::Result<()> {
    let mut printer = Printer::new();
    do_new(&mut printer, config.get_app_directory(opts.get_app_name()))
}
