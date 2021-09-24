use super::do_validate::do_validate;
use super::utils::read_pipe_manifest;
use crate::print::Printer;
use crate::{
    commands::{describe::DescribeOptions, validate::ValidateOptions},
    config::Config,
    errors::CmdResult,
};
use pipegen::models::App;

fn do_describe_pipes(app: &App, printer: &mut Printer) -> CmdResult<()> {
    printer.status(&"Describe", "pipes")?;
    for description in app.describe_pipes() {
        printer.result(description)?;
    }
    Ok(())
}

fn do_describe_pipe_graph(app: &App, printer: &mut Printer) -> CmdResult<()> {
    printer.status(&"Describe", "pipe graph")?;
    for description in app.describe_pipe_graph() {
        printer.result(description)?;
    }
    Ok(())
}

fn do_describe_pipe(app: &App, pipe_name: &str, printer: &mut Printer) -> CmdResult<()> {
    printer.status(&"Describe", format!("pipe {}", pipe_name))?;
    let pipe_display = match app.describe_pipe(pipe_name) {
        Ok(pipe_display) => pipe_display,
        Err(err) => {
            printer.error(format!("describe pipe {} falied: {}", pipe_name, err))?;
            return Err(err.into());
        }
    };
    printer.result(format!("\n{}", pipe_display))?;
    Ok(())
}

fn do_describe_object(app: &App, object_name: &str, printer: &mut Printer) -> CmdResult<()> {
    printer.status(&"Describe", format!("object {}", object_name))?;
    let object_display = match app.describe_object(object_name) {
        Ok(object_display) => object_display,
        Err(err) => {
            printer.error(format!("describe object {} falied: {}", object_name, err))?;
            return Err(err.into());
        }
    };
    printer.result(format!("\n{}", object_display))?;
    Ok(())
}

fn do_describe_objects(app: &App, printer: &mut Printer) -> CmdResult<()> {
    printer.status(&"Describe", "objects")?;
    for description in app.describe_objects() {
        printer.result(description)?;
    }
    Ok(())
}

fn do_describe_pipelines(app: &App, pipe_name: &str, printer: &mut Printer) -> CmdResult<()> {
    printer.status(&"Describe", format!("pipelines for {}", pipe_name))?;
    let pipelines = match app.describe_pipelines(pipe_name) {
        Ok(pipelines) => pipelines,
        Err(err) => {
            printer.error(format!(
                "describe pipelines for {} failed: {}",
                pipe_name, err
            ))?;
            return Err(err.into());
        }
    };
    for line in pipelines {
        printer.result(line)?;
    }
    Ok(())
}

pub fn do_exec(config: &Config, opts: &DescribeOptions) -> CmdResult<()> {
    let mut printer = Printer::new();
    let pipe_manifest_path = config.get_pipe_manifest_path();
    let app = read_pipe_manifest(pipe_manifest_path.as_path(), &mut printer)?;
    // check first
    do_validate(&app, &mut printer, &ValidateOptions::default())?;
    // list all pipes and objects
    if opts.all() {
        do_describe_pipes(&app, &mut printer)?;
        do_describe_objects(&app, &mut printer)?;
    }
    if opts.graph() {
        do_describe_pipe_graph(&app, &mut printer)?;
    }
    if let Some(pipe_name) = opts.pipe_name() {
        do_describe_pipe(&app, pipe_name, &mut printer)?
    };
    // describe particular object
    if let Some(object_name) = opts.object_name() {
        do_describe_object(&app, object_name, &mut printer)?
    };
    // desribe pipelines for particular pipe
    if let Some(pipe_name) = opts.pipe_name_in_line() {
        do_describe_pipelines(&app, pipe_name, &mut printer)?;
    };
    Ok(())
}
