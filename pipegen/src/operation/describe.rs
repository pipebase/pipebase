use crate::{
    api::{Entity, Pipe, VisitEntity},
    operation::utils::PipeGraph,
};

use std::fmt::{self, Display};

pub trait Describe {
    fn new() -> Self;
    fn parse(&mut self);
    fn describe(&self) -> Vec<String>;
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

impl Describe for PipeGraphDescriber {
    fn new() -> Self {
        PipeGraphDescriber {
            graph: PipeGraph::new(),
            pipe_ids: Vec::new(),
            results: Vec::new(),
        }
    }

    fn describe(&self) -> Vec<String> {
        let mut results: Vec<String> = Vec::new();
        for result in &self.results {
            results.push(format!("{}", result))
        }
        results
    }

    fn parse(&mut self) {
        self.results.clear();
        self.results.push(Box::new(self.display_source_pipe_ids()));
        self.results.push(Box::new(self.display_sink_pipe_ids()));
        for component in self.display_pipe_components() {
            self.results.push(Box::new(component))
        }
    }
}

impl PipeGraphDescriber {
    pub fn describe_pipelines(&self, pid: &str) -> Vec<String> {
        let mut results: Vec<String> = Vec::new();
        for pipeline in &self.display_pipelines(pid) {
            results.push(format!("{}", pipeline))
        }
        results
    }

    pub fn display_source_pipe_ids(&self) -> PipeIdsDisplay {
        PipeIdsDisplay {
            ids: self.get_source_pipe_ids(),
            sep: PIPE_LIST_SEP.to_owned(),
            label: Some(SOURCE_PIPE_LABEL.to_owned()),
        }
    }

    pub fn display_sink_pipe_ids(&self) -> PipeIdsDisplay {
        PipeIdsDisplay {
            ids: self.get_sink_pipe_ids(),
            sep: PIPE_LIST_SEP.to_owned(),
            label: Some(SINK_PIPE_LABEL.to_owned()),
        }
    }

    pub fn display_pipe_components(&self) -> Vec<PipeIdsDisplay> {
        let mut components_display: Vec<PipeIdsDisplay> = Vec::new();
        for component in self.get_pipe_components() {
            let component_display = PipeIdsDisplay {
                ids: component,
                sep: PIPE_LIST_SEP.to_owned(),
                label: Some(PIPE_COMPONENT_LABEL.to_owned()),
            };
            components_display.push(component_display);
        }
        components_display
    }

    pub fn display_pipelines(&self, pid: &str) -> Vec<PipeIdsDisplay> {
        let mut pipelines_display: Vec<PipeIdsDisplay> = Vec::new();
        for pipeline in self.get_pipelines(pid) {
            let pipeline_display = PipeIdsDisplay {
                ids: pipeline,
                sep: PIPE_DIRECT_SEP.to_owned(),
                label: Some(PIPELINE_LABEL.to_owned()),
            };
            pipelines_display.push(pipeline_display);
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
        assert!(self.graph.has_pipe(pid));
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

#[derive(Clone)]
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

#[cfg(test)]
mod tests {

    use crate::api::App;
    use std::path::Path;

    #[test]
    fn test_describe_timer_tick_pipe() {
        let manifest_path: &Path = Path::new("resources/manifest/print_timer_tick_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.describe();
    }

    #[test]
    fn test_describe_timer_tick_pipeline() {
        let manifest_path: &Path = Path::new("resources/manifest/print_timer_tick_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        let describer = app.get_pipe_describer();
        assert_eq!(2, describer.get_pipelines("printer").len());
        for pipeline in app.describe_pipelines("printer") {
            println!("{}", pipeline)
        }
    }
}
