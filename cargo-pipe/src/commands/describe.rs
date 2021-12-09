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
                .help("List all pipes and objects in pipe manifest"),
            Arg::new("graph")
                .short('g')
                .help("List basic pipe graph info: source / sink / disjoint components"),
            Arg::new("pipe")
                .short('p')
                .help("Describe pipe in pipe manifest")
                .takes_value(true),
            Arg::new("object")
                .short('o')
                .help("Describe object in pipe manifest")
                .takes_value(true),
            Arg::new("line")
                .short('l')
                .help("Describe pipelines in pipe manifest given pipe name")
                .takes_value(true),
        ])
}

pub fn exec(config: &Config, args: &clap::ArgMatches) -> CmdResult<()> {
    let all = args.is_present("all");
    let graph = args.is_present("graph");
    let pipe_name = args.value_of("pipe").map(|pipe_name| pipe_name.to_owned());
    let object_name = args
        .value_of("object")
        .map(|object_name| object_name.to_owned());
    let pipe_name_in_line = args.value_of("line").map(|pipe_name| pipe_name.to_owned());
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
