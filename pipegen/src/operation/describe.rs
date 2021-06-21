use crate::{
    api::{Entity, Pipe, VisitEntity},
    operation::utils::PipeGraph,
};

use std::{
    collections::HashMap,
    fmt::{self, Display},
};

pub trait Describe<T> {
    fn new() -> Self;
    fn parse(&mut self);
    fn describe(&self) -> std::vec::IntoIter<std::string::String>;
}

pub struct PipeGraphDescriber {
    graph: PipeGraph<Pipe>,
    pipe_ids: Vec<String>,
    results: Vec<Box<dyn Display>>,
}

impl VisitEntity<Pipe> for PipeGraphDescriber {
    fn visit(&mut self, pipe: &Pipe) {
        self.graph.add_pipe(pipe, pipe.to_owned());
        self.pipe_ids.push(pipe.get_id().to_owned());
    }
}

impl Describe<Pipe> for PipeGraphDescriber {
    fn new() -> Self {
        PipeGraphDescriber {
            graph: PipeGraph::new(),
            pipe_ids: Vec::new(),
            results: Vec::new(),
        }
    }

    fn describe(&self) -> std::vec::IntoIter<std::string::String> {
        let mut results: Vec<String> = Vec::new();
        for result in &self.results {
            results.push(format!("{}", result))
        }
        results.into_iter()
    }

    fn parse(&mut self) {
        self.results.clear();
        self.results.push(self.display_source_pipe_ids());
        self.results.push(self.display_sink_pipe_ids());
        for component in self.display_pipe_components() {
            self.results.push(component)
        }
    }
}

impl PipeGraphDescriber {
    fn display_source_pipe_ids(&self) -> Box<PipeIdsDisplay> {
        Box::new(PipeIdsDisplay {
            ids: self.get_source_pipe_ids(),
            sep: PIPE_LIST_SEP.to_owned(),
            label: Some(SOURCE_PIPE_LABEL.to_owned()),
        })
    }

    fn display_sink_pipe_ids(&self) -> Box<PipeIdsDisplay> {
        Box::new(PipeIdsDisplay {
            ids: self.get_sink_pipe_ids(),
            sep: PIPE_LIST_SEP.to_owned(),
            label: Some(SINK_PIPE_LABEL.to_owned()),
        })
    }

    fn display_pipe_components(&self) -> Vec<Box<PipeIdsDisplay>> {
        let mut components_display: Vec<Box<PipeIdsDisplay>> = Vec::new();
        for component in self.get_pipe_components() {
            let component_display = PipeIdsDisplay {
                ids: component,
                sep: PIPE_LIST_SEP.to_owned(),
                label: Some(PIPE_COMPONENT_LABEL.to_owned()),
            };
            components_display.push(Box::new(component_display));
        }
        components_display
    }

    fn display_pipelines(&self, pid: &str) -> Vec<Box<PipeIdsDisplay>> {
        let mut pipelines_display: Vec<Box<PipeIdsDisplay>> = Vec::new();
        for pipeline in self.get_pipelines(pid) {
            let pipeline_display = PipeIdsDisplay {
                ids: pipeline,
                sep: PIPE_DIRECT_SEP.to_owned(),
                label: Some(PIPELINE_LABEL.to_owned()),
            };
            pipelines_display.push(Box::new(pipeline_display));
        }
        pipelines_display
    }

    pub fn get_source_pipe_ids(&self) -> Vec<String> {
        self.graph.find_source_pipes()
    }

    pub fn get_sink_pipe_ids(&self) -> Vec<String> {
        self.graph.find_sink_pipes()
    }

    pub fn get_pipelines(&self, pid: &str) -> Vec<Vec<String>> {
        if !self.graph.has_pipe(pid) {
            return Vec::new();
        }
        self.graph.search_pipelines(pid)
    }

    pub fn get_pipe_components(&self) -> Vec<Vec<String>> {
        let mut components: Vec<Vec<String>> = Vec::new();
        for component in self.graph.find_components().values() {
            components.push(component.to_owned())
        }
        components
    }
}

pub struct PipeIdsDisplay {
    ids: Vec<String>,
    sep: String,
    label: Option<String>,
}

impl fmt::Display for PipeIdsDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ids_lit = self.ids.join(&self.sep);
        match self.label {
            Some(ref label) => write!(f, "{}: {}", label, ids_lit),
            None => write!(f, "{}", ids_lit),
        }
    }
}

const SOURCE_PIPE_LABEL: &str = "source";
const SINK_PIPE_LABEL: &str = "sink";
const PIPE_COMPONENT_LABEL: &str = "union";
const PIPELINE_LABEL: &str = "pipeline";
const PIPE_LIST_SEP: &str = ", ";
const PIPE_DIRECT_SEP: &str = " -> ";
const DISPLAY_NEWLINE: &str = "\n";

#[cfg(test)]
mod tests {

    use crate::api::App;

    #[test]
    fn test_describe_timer_tick_pipe() {
        let manifest_path = "resources/manifest/print_timer_tick_pipe.yml";
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.describe();
    }

    #[test]
    fn test_describe_timer_tick_pipeline() {
        let manifest_path = "resources/manifest/print_timer_tick_pipe.yml";
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        let describer = app.get_pipe_describer();
        for pipeline in describer.display_pipelines("printer") {
            println!("{}", pipeline)
        }
    }
}
