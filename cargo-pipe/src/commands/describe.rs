use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use crate::ops::do_describe::do_exec;
use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("describe")
        .about("Describe pipes and objects in pipe manifest")
        .args(vec![
            Arg::new("pipe")
                .short('p')
                .about("Describe pipes in pipe manifest"),
            Arg::new("object")
                .short('o')
                .about("Describe objects in pipe manifest"),
            Arg::new("line")
                .short('l')
                .about("Describe pipelines in pipe manifest given pipe name")
                .takes_value(true),
        ])
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult {
    let pipe = args.is_present("pipe");
    let object = args.is_present("object");
    let pipe_name = match args.value_of("line") {
        Some(pipe_name) => Some(pipe_name.to_owned()),
        None => None,
    };
    let opts = DescribeOptions::new(pipe, object, pipe_name);
    do_exec(config, &opts)?;
    Ok(())
}

pub struct DescribeOptions {
    pipe: bool,
    object: bool,
    pipe_name: Option<String>,
}

impl DescribeOptions {
    pub fn new(pipe: bool, object: bool, pipe_name: Option<String>) -> Self {
        DescribeOptions {
            pipe,
            object,
            pipe_name,
        }
    }

    pub fn pipe(&self) -> bool {
        self.pipe
    }

    pub fn object(&self) -> bool {
        self.object
    }

    pub fn pipe_name(&self) -> Option<&String> {
        self.pipe_name.as_ref()
    }
}
