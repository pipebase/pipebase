use super::utils::*;
use crate::commands::init::InitOptions;
use crate::print::Printer;
use crate::Config;
use pipegen::api::App;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn do_init_directory(path: &Path, printer: &mut Printer) -> anyhow::Result<()> {
    printer.status(&"Init", path.to_str().unwrap())?;
    match path.exists() {
        true => fs::remove_dir_all(path)?,
        false => (),
    };
    fs::create_dir(path)?;
    printer.status(&"Init", "succeed")?;
    Ok(())
}

fn cargo_binary() -> OsString {
    match std::env::var_os("CARGO") {
        Some(cargo) => cargo,
        None => "cargo".to_owned().into(),
    }
}

fn do_cargo_init(path: &Path, printer: &mut Printer) -> anyhow::Result<bool> {
    printer.status(&"Cargo", "init")?;
    let mut cmd = Command::new(cargo_binary());
    cmd.arg("init").arg(path);
    let output = cmd.output()?;
    let succeed = match output.status.success() {
        true => {
            let stdout = String::from_utf8(output.stdout)?;
            printer.status(&"Cargo", format!("init succeed, {}", stdout))?;
            true
        }
        false => {
            let stderr = String::from_utf8(output.stderr)?;
            printer.error(format!("cargo init failed {}", stderr))?;
            false
        }
    };
    Ok(succeed)
}

fn do_apply_additional_dependency(
    app: &App,
    toml_path: &Path,
    printer: &mut Printer,
) -> anyhow::Result<()> {
    printer.status("Manifest", "add dependencies")?;
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
    printer.status("Manifest", "add dependencies succeed")?;
    Ok(())
}

pub fn do_init(
    app: &App,
    printer: &mut Printer,
    mut init_pathbuf: PathBuf,
    _opts: &InitOptions,
) -> anyhow::Result<()> {
    let init_path = init_pathbuf.as_path();
    do_init_directory(init_path, printer)?;
    // cargo init
    let succeed = do_cargo_init(init_path, printer)?;
    match succeed {
        false => return Ok(()),
        true => (),
    };
    // replace dependency in cargo.toml
    init_pathbuf.push("cargo.toml");
    do_apply_additional_dependency(app, init_pathbuf.as_path(), printer)?;
    Ok(())
}

pub fn do_exec(config: &Config, opts: &InitOptions) -> anyhow::Result<()> {
    let mut printer = Printer::new();
    let pipe_manifest_path = config.get_pipe_manifest_path();
    let app = parse_pipe_manifest(pipe_manifest_path.as_path(), &mut printer)?;
    do_init(&app, &mut printer, config.get_init_directory(), opts)
}
