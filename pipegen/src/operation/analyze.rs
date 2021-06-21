use crate::{
    api::{Entity, Pipe, VisitEntity},
    operation::utils::PipeGraph,
};

pub trait Analyze<T> {
    fn new() -> Self;
    fn analyze(&mut self);
    fn get_result(&self) -> String;
}

pub struct PipeGraphAnalyzer {
    graph: PipeGraph<Pipe>,
    pipes: Vec<String>,
    results: Vec<String>,
}

impl VisitEntity<Pipe> for PipeGraphAnalyzer {
    fn visit(&mut self, pipe: &Pipe) {
        self.graph.add_pipe(pipe, pipe.to_owned());
        self.pipes.push(pipe.get_id().to_owned());
    }
}

impl Analyze<Pipe> for PipeGraphAnalyzer {
    fn new() -> Self {
        PipeGraphAnalyzer {
            graph: PipeGraph::new(),
            pipes: Vec::new(),
            results: Vec::new(),
        }
    }

    fn get_result(&self) -> String {
        self.results.join("")
    }

    fn analyze(&mut self) {
        self.results.clear();
        self.collect_source_sink_vertices();
        self.section_sep();
        self.collect_components();
        self.section_sep();
        self.collect_all_vertices();
    }
}

impl PipeGraphAnalyzer {
    fn show_pipes(&self, label: &str, vertices: &Vec<String>, sep: &str) -> String {
        format!("{}: {}\n", label, vertices.join(sep))
    }

    fn show_all_pipes(&self) -> String {
        let mut all_vertices: Vec<String> = vec![];

        for i in 0..self.pipes.len() {
            let pipe = self.pipes.get(i).unwrap();
            all_vertices.push(format!("{} - {}", i, self.graph.get_value(pipe).unwrap()));
        }
        all_vertices.join("\n")
    }

    fn collect_source_sink_vertices(&mut self) {
        let sources = self.graph.find_source_vertex();
        let sinks = self.graph.find_sink_vertex();
        self.results
            .push(self.show_pipes("source", &sources, ANALYZE_VERTEX_SEP));
        self.results
            .push(self.show_pipes("sink", &sinks, ANALYZE_VERTEX_SEP));
    }

    fn collect_components(&mut self) {
        let components = self.graph.find_components();
        for (union_vertex, vertices) in &components {
            let label = format!("union {}", union_vertex);
            self.results
                .push(self.show_pipes(&label, vertices, ANALYZE_VERTEX_SEP));
        }
    }

    fn collect_all_vertices(&mut self) {
        self.results.push(self.show_all_pipes())
    }

    fn section_sep(&mut self) {
        self.results.push(ANALYZE_SECTION_SEP.to_owned());
    }
}

// const ANALYZE_BASIC_INFO: &str = "basic";
const ANALYZE_VERTEX_SEP: &str = ", ";
const ANALYZE_SECTION_SEP: &str = "\n";

#[cfg(test)]
mod tests {

    use crate::api::App;

    #[test]
    fn test_analyze_timer_tick_pipe() {
        let manifest_path = "resources/manifest/print_timer_tick_pipe.yml";
        let app = App::parse(manifest_path).unwrap();
        app.validate().expect("expect valid");
        app.analyze()
    }
}
