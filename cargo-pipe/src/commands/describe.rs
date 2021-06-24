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
    let opts = match (args.is_present("pipe"), args.is_present("object")) {
        (true, false) => DescribeOptions {
            pipe: true,
            object: false,
        },
        (false, true) => DescribeOptions {
            pipe: false,
            object: true,
        },
        (_, _) => DescribeOptions {
            pipe: true,
            object: true,
        },
    };
    do_exec(config, &opts)?;
    Ok(())
}

pub struct DescribeOptions {
    pipe: bool,
    object: bool,
}

impl DescribeOptions {
    pub fn describe_pipe(&self) -> bool {
        self.pipe
    }
    pub fn describe_object(&self) -> bool {
        self.object
    }
}
