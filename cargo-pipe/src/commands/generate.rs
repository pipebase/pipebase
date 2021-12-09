use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use crate::ops::do_generate::do_exec;
use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("generate")
        .about("Generate pipe app main")
        .args(vec![
            Arg::new("name")
                .short('n')
                .help("Specify the app name")
                .takes_value(true),
            Arg::new("line")
                .short('l')
                .help("Specify the pipe name, so that only pipelines contain pipe generated")
                .takes_value(true),
        ])
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult<()> {
    let app_name = args.value_of("name").map(|app_name| app_name.to_owned());
    let pipe_name = args.value_of("line").map(|pipe_name| pipe_name.to_owned());
    let opts = GenerateOptions::new(app_name, pipe_name);
    do_exec(config, &opts)?;
    Ok(())
}

pub struct GenerateOptions {
    app_name: Option<String>,
    pipe_name: Option<String>,
}

impl GenerateOptions {
    pub fn new(app_name: Option<String>, pipe_name: Option<String>) -> Self {
        GenerateOptions {
            app_name,
            pipe_name,
        }
    }

    pub fn get_app_name(&self) -> Option<&String> {
        self.app_name.as_ref()
    }

    pub fn get_pipe_name(&self) -> Option<&String> {
        self.pipe_name.as_ref()
    }
}
