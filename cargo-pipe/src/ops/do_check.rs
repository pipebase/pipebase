use super::utils::parse_pipe_manifest;
use crate::commands::check::CheckOptions;
use crate::print::Printer;
use crate::Config;
use pipegen::api::App;

fn check_cargo_toml() {}

pub fn do_check(app: &App, printer: &mut Printer, opts: &CheckOptions) -> anyhow::Result<()> {
    if opts.check_pipe() {
        printer.status(&"Check", "Pipes")?;
        match app.validate_pipes() {
            Ok(_) => (),
            Err(err) => {
                printer.error(err.to_string())?;
                return Err(err.into());
            }
        };
    }
    if opts.check_object() {
        printer.status(&"Check", "Objects")?;
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

pub fn do_exec(config: &Config, opts: &CheckOptions) -> anyhow::Result<()> {
    let mut printer = Printer::new();
    let pipe_manifest_path = config.get_pipe_manifest_path();
    let app = parse_pipe_manifest(pipe_manifest_path.as_path(), &mut printer)?;
    do_check(&app, &mut printer, opts)
}
