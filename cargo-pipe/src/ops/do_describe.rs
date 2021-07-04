use super::do_check::do_check;
use super::utils::parse_pipe_manifest;
use crate::commands::check::CheckOptions;
use crate::commands::describe::DescribeOptions;
use crate::config::Config;
use crate::print::Printer;

pub fn do_exec(config: &Config, opts: &DescribeOptions) -> anyhow::Result<()> {
    let mut printer = Printer::new();
    let pipe_manifest_path = config.get_pipe_manifest_path();
    let app = parse_pipe_manifest(pipe_manifest_path.as_path(), &mut printer)?;
    // check first
    do_check(&app, &mut printer, &CheckOptions::default())?;
    if opts.describe_pipe() {
        for description in app.describe_pipes() {
            printer.result(description)?;
        }
    }
    // TODO: Describe object
    Ok(())
}
