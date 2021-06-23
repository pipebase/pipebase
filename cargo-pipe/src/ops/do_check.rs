use crate::commands::check::CheckOptions;
use crate::print::Printer;
use crate::Config;
use pipegen::api::App;
use std::path::Path;

fn check_cargo_toml() {}

fn parse_pipe_manifest(manifest_path: &Path) -> anyhow::Result<App> {
    Ok(App::parse(manifest_path)?)
}

pub fn do_exec(config: &Config, opts: &CheckOptions) -> anyhow::Result<()> {
    let mut printer = Printer::new();
    let pipe_manifest_path = config.get_pipe_manifest_path();
    printer.status(&"Parse", pipe_manifest_path.to_str().unwrap())?;
    let app = match parse_pipe_manifest(pipe_manifest_path.as_path()) {
        Ok(app) => app,
        Err(err) => {
            printer.error(err.to_string())?;
            return Err(err);
        }
    };
    if opts.check_pipe() {
        printer.status(&"Check", "pipes")?;
        match app.validate_pipes() {
            Ok(_) => (),
            Err(err) => {
                printer.error(err.to_string())?;
                return Err(err.into());
            }
        };
    }
    if opts.check_object() {
        printer.status(&"check", "objects")?;
        match app.validate_objects() {
            Ok(_) => (),
            Err(err) => {
                printer.error(err.to_string())?;
                return Err(err.into());
            }
        };
    }
    Ok(())
}
