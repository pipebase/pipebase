use crate::commands::remove::RemoveOptions;
use crate::print::Printer;
use crate::Config;
use std::fs;
use std::path::PathBuf;

pub fn do_remove(path_buf: PathBuf, printer: &mut Printer) -> anyhow::Result<()> {
    let path = path_buf.as_path();
    printer.status(&"Remove", path.to_str().unwrap())?;
    match path.exists() {
        true => fs::remove_dir_all(path)?,
        false => (),
    };
    printer.status(&"Remove", "success")?;
    Ok(())
}

pub fn do_exec(config: &Config, opts: &RemoveOptions) -> anyhow::Result<()> {
    let mut printer = Printer::new();
    do_remove(config.get_app_directory(opts.get_app_name()), &mut printer)
}
