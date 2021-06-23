use pipegen::api::App;
use std::path::Path;

use crate::print::Printer;

pub fn parse_pipe_manifest(manifest_path: &Path, printer: &mut Printer) -> anyhow::Result<App> {
    printer.status(&"Parse", manifest_path.to_str().unwrap())?;
    match App::parse(manifest_path) {
        Ok(app) => Ok(app),
        Err(err) => {
            printer.error(err.to_string())?;
            return Err(err.into());
        }
    }
}
