use crate::commands::Cmd;
use crate::config::Config;
use crate::errors::CmdResult;
use crate::ops::do_describe::do_exec;
use clap::Arg;

pub fn cmd() -> Cmd {
    Cmd::new("describe")
        .about("Describe pipe manifest")
        .args(vec![
            Arg::new("all")
                .short('a')
                .about("List all pipes and objects in pipe manifest"),
            Arg::new("graph")
                .short('g')
                .about("List basic pipe graph info: source / sink / disjoint components"),
            Arg::new("pipe")
                .short('p')
                .about("Describe pipe in pipe manifest")
                .takes_value(true),
            Arg::new("object")
                .short('o')
                .about("Describe object in pipe manifest")
                .takes_value(true),
            Arg::new("line")
                .short('l')
                .about("Describe pipelines in pipe manifest given pipe name")
                .takes_value(true),
        ])
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult {
    let all = args.is_present("all");
    let graph = args.is_present("graph");
    let pipe_name = match args.value_of("pipe") {
        Some(pipe_name) => Some(pipe_name.to_owned()),
        None => None,
    };
    let object_name = match args.value_of("object") {
        Some(object_name) => Some(object_name.to_owned()),
        None => None,
    };
    let pipe_name_in_line = match args.value_of("line") {
        Some(pipe_name) => Some(pipe_name.to_owned()),
        None => None,
    };
    let opts = DescribeOptions::new(all, graph, pipe_name, object_name, pipe_name_in_line);
    do_exec(config, &opts)?;
    Ok(())
}

pub struct DescribeOptions {
    all: bool,
    graph: bool,
    pipe_name: Option<String>,
    object_name: Option<String>,
    pipe_name_in_line: Option<String>,
}

impl DescribeOptions {
    pub fn new(
        all: bool,
        graph: bool,
        pipe_name: Option<String>,
        object_name: Option<String>,
        pipe_name_in_line: Option<String>,
    ) -> Self {
        DescribeOptions {
            all,
            graph,
            pipe_name,
            object_name,
            pipe_name_in_line,
        }
    }

    pub fn all(&self) -> bool {
        self.all
    }

    pub fn graph(&self) -> bool {
        self.graph
    }

    pub fn pipe_name(&self) -> Option<&String> {
        self.pipe_name.as_ref()
    }

    pub fn object_name(&self) -> Option<&String> {
        self.object_name.as_ref()
    }

    pub fn pipe_name_in_line(&self) -> Option<&String> {
        self.pipe_name_in_line.as_ref()
    }
}
