use crate::{
    api::{Entity, Pipe, VisitEntity},
    operation::utils::DirectedGraph,
};
use std::collections::HashMap;
pub trait Analyze<T> {
    fn new() -> Self;
    fn analyze(&mut self);
    fn get_result(&self) -> String;
}

pub struct PipeGraphAnalyzer {
    graph: DirectedGraph<usize>,
    order: usize,
    results: Vec<String>,
}

impl VisitEntity<Pipe> for PipeGraphAnalyzer {
    fn visit(&mut self, pipe: &Pipe) {
        let ref id = pipe.get_id();
        self.graph.add_vertex_if_not_exists(id.to_owned());
        self.graph.set_value(id, self.order);
        self.order += 1;
        let deps = pipe.list_dependency();
        for dep in &deps {
            self.graph.add_vertex_if_not_exists(dep.to_owned());
            self.graph.add_edge(dep, id);
        }
    }
}

impl Analyze<Pipe> for PipeGraphAnalyzer {
    fn new() -> Self {
        PipeGraphAnalyzer {
            graph: DirectedGraph::new(),
            order: 0,
            results: Vec::new(),
        }
    }

    fn get_result(&self) -> String {
        self.results.join("")
    }

    fn analyze(&mut self) {
        self.results.clear();
        self.collect_source_sink_vertices();
        self.analyze_section_sep();
        self.collect_components();
        self.analyze_section_sep();
        self.collect_all_vertices();
    }
}

impl PipeGraphAnalyzer {
    fn show_vertices(&self, label: &str, vertices: &Vec<String>, sep: &str) -> String {
        let vertices = self.pipe_ids_to_orders_literal(vertices);
        format!("{}: {}\n", label, vertices.join(sep))
    }

    fn show_all_vertices_in_order(&self) -> String {
        let mut all_vertices: Vec<String> = vec![];
        let mut order_lookup: HashMap<usize, String> = HashMap::new();
        for (id, value) in self.graph.get_values() {
            assert!(order_lookup.insert(value, id).is_none());
        }
        for i in 0..self.order {
            all_vertices.push(format!("{}: {}", i, order_lookup.get(&i).unwrap()));
        }
        all_vertices.join("\n")
    }

    fn collect_source_sink_vertices(&mut self) {
        let sources = self.graph.find_source_vertex();
        let sinks = self.graph.find_sink_vertex();
        self.results
            .push(self.show_vertices("source", &sources, ANALYZE_VERTEX_SEP));
        self.results
            .push(self.show_vertices("sink", &sinks, ANALYZE_VERTEX_SEP));
    }

    fn collect_components(&mut self) {
        let components = self.graph.find_components();
        for (union_vertex, vertices) in &components {
            let label = format!("union {}", self.pipe_id_to_order(union_vertex));
            self.results
                .push(self.show_vertices(&label, vertices, ANALYZE_VERTEX_SEP));
        }
    }

    fn collect_all_vertices(&mut self) {
        self.results.push(self.show_all_vertices_in_order())
    }

    fn analyze_section_sep(&mut self) {
        self.results.push(ANALYZE_SECTION_SEP.to_owned());
    }

    fn pipe_ids_to_orders_literal(&self, ids: &Vec<String>) -> Vec<String> {
        self.pipe_ids_to_orders(ids)
            .into_iter()
            .map(|id| id.to_string())
            .collect()
    }

    fn pipe_ids_to_orders(&self, ids: &Vec<String>) -> Vec<usize> {
        ids.into_iter()
            .map(|id| self.pipe_id_to_order(id))
            .collect()
    }

    fn pipe_id_to_order(&self, id: &str) -> usize {
        self.graph.get_value(id).unwrap()
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
