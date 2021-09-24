use crate::{commands::remove::RemoveOptions, config::Config, errors::CmdResult, print::Printer};
use std::fs;
use std::path::PathBuf;

pub fn do_remove(path_buf: PathBuf, printer: &mut Printer) -> CmdResult<()> {
    let path = path_buf.as_path();
    printer.status(&"Remove", path.to_str().unwrap())?;
    match path.exists() {
        true => fs::remove_dir_all(path)?,
        false => (),
    };
    printer.status(&"Remove", "success")?;
    Ok(())
}

pub fn do_exec(config: &Config, opts: &RemoveOptions) -> CmdResult<()> {
    let mut printer = Printer::new();
    do_remove(config.get_app_directory(opts.get_app_name()), &mut printer)
}
