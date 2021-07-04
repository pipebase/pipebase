use super::do_cargo::*;
use super::do_remove::do_remove;
use super::utils::*;
use crate::commands::new::NewOptions;
use crate::print::Printer;
use crate::Config;
use pipegen::api::App;
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

fn do_apply_additional_dependency(
    app: &App,
    toml_path: &Path,
    printer: &mut Printer,
) -> anyhow::Result<()> {
    printer.status("New", "add manifest dependencies")?;
    // fetch package dependency
    let additionals = app.get_package_dependency();
    let toml_content = fs::read_to_string(toml_path)?;
    let mut manifest = toml::from_str::<PipeTomlManifest>(&toml_content)?;
    manifest.init();
    for add in additionals.to_owned() {
        manifest.add_dependency(add.get_package(), add.into());
    }
    let toml_content = toml::to_string(&manifest)?;
    fs::write(toml_path, toml_content)?;
    printer.status("New", "manifest dependencies added")?;
    Ok(())
}

pub fn do_new(app: &App, printer: &mut Printer, mut path_buf: PathBuf) -> anyhow::Result<()> {
    let path = path_buf.as_path();
    do_remove_app_if_exists(path, printer)?;
    do_new_directory(path, printer)?;
    // cargo init
    let status_code = do_cargo_init(path, printer)?;
    match status_code {
        0 => (),
        _ => process::exit(status_code),
    };
    // replace dependency in cargo.toml
    path_buf.push(CARGO_MANIFEST_FILE);
    do_apply_additional_dependency(app, path_buf.as_path(), printer)?;
    Ok(())
}

pub fn do_exec(config: &Config, opts: &NewOptions) -> anyhow::Result<()> {
    let mut printer = Printer::new();
    let pipe_manifest_path = config.get_pipe_manifest_path();
    let app = parse_pipe_manifest(pipe_manifest_path.as_path(), &mut printer)?;
    do_new(
        &app,
        &mut printer,
        config.get_app_directory(opts.get_app_name()),
    )
}
