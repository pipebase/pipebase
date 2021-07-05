use super::do_validate::do_validate;
use super::utils::parse_pipe_manifest;
use crate::commands::describe::DescribeOptions;
use crate::commands::validate::ValidateOptions;
use crate::config::Config;
use crate::print::Printer;

pub fn do_exec(config: &Config, opts: &DescribeOptions) -> anyhow::Result<()> {
    let mut printer = Printer::new();
    let pipe_manifest_path = config.get_pipe_manifest_path();
    let app = parse_pipe_manifest(pipe_manifest_path.as_path(), &mut printer)?;
    // check first
    do_validate(&app, &mut printer, &ValidateOptions::default())?;
    if opts.pipe() {
        for description in app.describe_pipes() {
            printer.result(description)?;
        }
    }
    // TODO: Describe object
    match opts.pipe_name() {
        Some(pipe_name) => {
            for line in app.describe_pipelines(pipe_name) {
                printer.result(line)?;
            }
        }
        None => (),
    };
    Ok(())
}
