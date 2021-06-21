use crate::{
    api::{Entity, Pipe, VisitEntity},
    operation::utils::PipeGraph,
};

pub trait Describe<T> {
    fn new() -> Self;
    fn parse(&mut self);
    fn describe(&self) -> String;
}

pub struct PipeGraphDescriber {
    graph: PipeGraph<Pipe>,
    pipes: Vec<String>,
    results: Vec<String>,
}

impl VisitEntity<Pipe> for PipeGraphDescriber {
    fn visit(&mut self, pipe: &Pipe) {
        self.graph.add_pipe(pipe, pipe.to_owned());
        self.pipes.push(pipe.get_id().to_owned());
    }
}

impl Describe<Pipe> for PipeGraphDescriber {
    fn new() -> Self {
        PipeGraphDescriber {
            graph: PipeGraph::new(),
            pipes: Vec::new(),
            results: Vec::new(),
        }
    }

    fn describe(&self) -> String {
        self.results.join("")
    }

    fn parse(&mut self) {
        self.results.clear();
        self.parse_source_sink_vertices();
        self.section_sep();
        self.parse_components();
        self.section_sep();
        self.parse_all_vertices();
    }
}

impl PipeGraphDescriber {
    fn format_pipe_ids(pids: &Vec<String>, sep: &str, label: Option<&str>) -> String {
        let joined_pids = pids.join(sep);
        match label {
            Some(label) => format!("{}: {}\n", label, joined_pids),
            None => joined_pids,
        }
    }

    fn describe_all_pipes(&self) -> String {
        let mut all_pipes: Vec<String> = vec![];

        for i in 0..self.pipes.len() {
            let pipe = self.pipes.get(i).unwrap();
            all_pipes.push(format!(
                "{} - {}",
                i,
                self.graph.get_pipe_value(pipe).unwrap()
            ));
        }
        all_pipes.join("\n")
    }

    fn describe_pipelines(&self, pid: &str) -> Option<String> {
        if !self.graph.has_pipe(pid) {
            return None;
        }
        let pipelines = self.graph.search_pipelines(pid);
        if pipelines.is_empty() {
            return None;
        }
        let mut pipeline_results: Vec<String> = Vec::new();
        for pipeline in &pipelines {
            pipeline_results.push(Self::format_pipe_ids(
                pipeline,
                DESCRIBE_PIPE_VERTEX_CONNECT,
                None,
            ))
        }
        Some(pipeline_results.join("\n"))
    }

    fn parse_source_sink_vertices(&mut self) {
        let sources = self.graph.find_source_pipes();
        let sinks = self.graph.find_sink_pipes();
        self.results.push(Self::format_pipe_ids(
            &sources,
            DESCRIBE_PIPE_VERTEX_SEP,
            Some("source"),
        ));
        self.results.push(Self::format_pipe_ids(
            &sinks,
            DESCRIBE_PIPE_VERTEX_SEP,
            Some("sink"),
        ));
    }

    fn parse_components(&mut self) {
        let components = self.graph.find_components();
        for (union_vertex, vertices) in &components {
            let label = format!("union {}", union_vertex);
            self.results.push(Self::format_pipe_ids(
                vertices,
                DESCRIBE_PIPE_VERTEX_SEP,
                Some(&label),
            ));
        }
    }

    fn parse_all_vertices(&mut self) {
        self.results.push(self.describe_all_pipes())
    }

    fn section_sep(&mut self) {
        self.results.push(DESCRIBE_PIPE_SECTION_SEP.to_owned());
    }
}

const DESCRIBE_PIPE_VERTEX_SEP: &str = ", ";
const DESCRIBE_PIPE_SECTION_SEP: &str = "\n";
const DESCRIBE_PIPE_VERTEX_CONNECT: &str = " -> ";

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
        let analyzer = app.get_pipe_describer();
        let pipeline = analyzer.describe_pipelines("printer").unwrap();
        println!("{}", pipeline)
    }
}
