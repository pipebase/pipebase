use super::do_cargo::*;
use super::utils::*;
use crate::commands::generate::GenerateOptions;
use crate::print::Printer;
use crate::Config;
use pipegen::api::App;
use std::fs;
use std::path::PathBuf;
use std::process;

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
    // cargo format
    // pop main.rs
    path_buf.pop();
    // pop src
    path_buf.pop();
    path_buf.push(CARGO_MANIFEST_FILE);
    let manifest_path = path_buf.as_path();
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
    let app = parse_pipe_manifest(pipe_manifest_path.as_path(), &mut printer)?;
    let path_buf = config.get_app_main_path(opts.get_app_name());
    do_generate(&app, path_buf, opts, &mut printer)?;
    Ok(())
}
