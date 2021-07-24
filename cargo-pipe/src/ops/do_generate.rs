use super::do_cargo::*;
use super::utils::*;
use crate::commands::generate::GenerateOptions;
use crate::print::Printer;
use crate::Config;
use pipegen::models::App;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn do_apply_additional_dependency(
    app: &App,
    toml_path: &Path,
    printer: &mut Printer,
) -> anyhow::Result<()> {
    printer.status("Generate", "add toml manifest dependencies")?;
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
    printer.status("Generate", "toml manifest dependencies added")?;
    Ok(())
}

pub fn do_generate(
    app: &App,
    mut path_buf: PathBuf,
    opts: &GenerateOptions,
    printer: &mut Printer,
) -> anyhow::Result<()> {
    let main_path = path_buf.as_path();
    printer.status(&"Generate", main_path.to_str().unwrap())?;
    let contents = match opts.get_pipe_name() {
        Some(pipe_name) => app.generate_pipes(pipe_name)?,
        None => app.generate(),
    };
    fs::write(main_path, contents)?;
    // pop main.rs
    path_buf.pop();
    // pop src
    path_buf.pop();
    path_buf.push(CARGO_MANIFEST_FILE);
    // replace dependency in cargo.toml
    do_apply_additional_dependency(app, path_buf.as_path(), printer)?;
    let manifest_path = path_buf.as_path();
    // cargo format
    let status_code = do_cargo_fmt(manifest_path, printer)?;
    match status_code {
        0 => (),
        _ => process::exit(status_code),
    };
    printer.status(&"Generate", "succeed")
}

pub fn do_exec(config: &Config, opts: &GenerateOptions) -> anyhow::Result<()> {
    let mut printer = Printer::new();
    let pipe_manifest_path = config.get_pipe_manifest_path();
    let app = read_pipe_manifest(pipe_manifest_path.as_path(), &mut printer)?;
    let path_buf = config.get_app_main_path(opts.get_app_name());
    do_generate(&app, path_buf, opts, &mut printer)?;
    Ok(())
}
