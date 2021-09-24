use super::utils::read_pipe_manifest;
use crate::{
    commands::validate::ValidateOptions, config::Config, errors::CmdResult, print::Printer,
};
use pipegen::models::App;

pub fn do_validate(app: &App, printer: &mut Printer, opts: &ValidateOptions) -> CmdResult<()> {
    if opts.pipe() {
        printer.status(&"Validate", "pipes")?;
        match app.validate_pipes() {
            Ok(_) => (),
            Err(err) => {
                printer.error(err.to_string())?;
                return Err(err.into());
            }
        };
    }
    if opts.object() {
        printer.status(&"Validate", "objects")?;
        match app.validate_objects() {
            Ok(_) => (),
            Err(err) => {
                printer.error(err.to_string())?;
                return Err(err.into());
            }
        };
    }
    if opts.cstore() {
        printer.status(&"Validate", "cstores")?;
        match app.validate_cstores() {
            Ok(_) => (),
            Err(err) => {
                printer.error(err.to_string())?;
                return Err(err.into());
            }
        };
    }
    printer.status(&"Validate", "pass")
}

pub fn do_exec(config: &Config, opts: &ValidateOptions) -> CmdResult<()> {
    let mut printer = Printer::new();
    let pipe_manifest_path = config.get_pipe_manifest_path();
    let app = read_pipe_manifest(pipe_manifest_path.as_path(), &mut printer)?;
    do_validate(&app, &mut printer, opts)
}
