use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use crate::ops::do_describe::do_exec;
use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("describe").args(vec![
        Arg::new("pipe")
            .short('p')
            .about("describe pipes in manifest"),
        Arg::new("object")
            .short('o')
            .about("describe objects in manifest"),
    ])
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult {
    let pipe = args.is_present("pipe");
    let object = args.is_present("object");
    let opts = DescribeOptions::new(pipe, object);
    do_exec(config, &opts)?;
    Ok(())
}

pub struct DescribeOptions {
    pipe: bool,
    object: bool,
}

impl DescribeOptions {
    pub fn new(pipe: bool, object: bool) -> Self {
        DescribeOptions {
            pipe: pipe,
            object: object,
        }
    }

    pub fn describe_pipe(&self) -> bool {
        self.pipe
    }
    pub fn describe_object(&self) -> bool {
        self.object
    }
}
