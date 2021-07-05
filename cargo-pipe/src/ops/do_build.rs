use super::do_cargo::*;
use super::utils::*;
use crate::commands::build::BuildOptions;
use crate::print::Printer;
use crate::Config;
use std::path::PathBuf;
use std::process;

pub fn do_build(
    path_buf: PathBuf,
    opts: &BuildOptions,
    printer: &mut Printer,
) -> anyhow::Result<()> {
    let manifest_path = path_buf.as_path();
    printer.status(&"Build", manifest_path.to_str().unwrap())?;
    let partial = opts.partial();
    if partial {
        set_pipe_meta_flags("-Pskip");
    }
    let status_code = do_cargo_build(manifest_path, printer)?;
    match status_code {
        0 => (),
        _ => process::exit(status_code),
    };
    printer.status(&"Build", "succeed")?;
    Ok(())
}

pub fn do_copy_binary(from: PathBuf, to: PathBuf, printer: &mut Printer) -> anyhow::Result<()> {
    printer.status(&"Copy", from.to_str().unwrap())?;
    let size = std::fs::copy(from, to)?;
    printer.status(&"Copied", format!("size: {} Mb", size / 1024 / 1024))?;
    Ok(())
}

pub fn do_exec(config: &Config, opts: &BuildOptions) -> anyhow::Result<()> {
    let mut printer = Printer::new();
    do_build(
        config.get_app_manifest(opts.get_app_name()),
        opts,
        &mut printer,
    )?;
    let from = config.get_target_release_app_binary(opts.get_app_name());
    let to = config.get_run_app_binary(opts.get_app_name());
    do_copy_binary(from, to, &mut printer)
}
