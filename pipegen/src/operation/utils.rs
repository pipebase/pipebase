use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

type GraphPath = Vec<String>;

pub struct GraphPaths {
    paths: HashMap<String, HashMap<String, Vec<GraphPath>>>,
}

impl GraphPaths {
    pub fn new() -> Self {
        GraphPaths {
            paths: HashMap::new(),
        }
    }

    pub fn add_path(&mut self, src: &str, dst: &str, path: GraphPath) {
        if !self.paths.contains_key(src) {
            self.paths.insert(src.to_owned(), HashMap::new());
        }
        let paths = self.paths.get_mut(src).unwrap();
        if !paths.contains_key(dst) {
            paths.insert(dst.to_owned(), Vec::new());
        }
        paths.get_mut(dst).unwrap().push(path);
    }

    pub fn get_paths(&self, src: &str, dst: &str) -> Option<Vec<GraphPath>> {
        let paths = match self.paths.get(src) {
            Some(paths) => paths,
            None => return None,
        };
        match paths.get(dst) {
            Some(paths) => Some(paths.to_owned()),
            None => return None,
        }
    }
}

pub struct Vertex<T: Clone> {
    in_vertices: HashSet<String>,
    out_vertices: HashSet<String>,
    value: Option<T>,
}

impl<T: Clone> Vertex<T> {
    pub fn new(value: Option<T>) -> Self {
        Vertex {
            in_vertices: HashSet::new(),
            out_vertices: HashSet::new(),
            value: value,
        }
    }

    pub fn get_in_vertices(&self) -> HashSet<String> {
        self.in_vertices.to_owned()
    }

    pub fn get_in_vertex_count(&self) -> usize {
        self.in_vertices.len()
    }

    pub fn get_out_vertices(&self) -> HashSet<String> {
        self.out_vertices.to_owned()
    }

    pub fn get_out_vertex_count(&self) -> usize {
        self.out_vertices.len()
    }

    pub fn get_value(&self) -> Option<T> {
        self.value.to_owned()
    }

    pub fn set_value(&mut self, value: T) {
        self.value = Some(value)
    }

    pub fn add_in_vertex(&mut self, vid: String) -> bool {
        self.in_vertices.insert(vid)
    }

    pub fn add_out_vertex(&mut self, vid: String) -> bool {
        self.out_vertices.insert(vid)
    }
}

pub struct DirectedGraph<T: Clone> {
    vertices: HashMap<String, Vertex<T>>,
}

impl<T: Clone> DirectedGraph<T> {
    pub fn new() -> Self {
        DirectedGraph {
            vertices: HashMap::new(),
        }
    }

    pub fn has_vertex(&self, id: &str) -> bool {
        self.vertices.contains_key(id)
    }

    fn add_vertex(&mut self, id: String) {
        self.vertices.insert(id, Vertex::new(None));
    }

    pub fn add_vertex_if_not_exists(&mut self, id: String) {
        if !self.has_vertex(&id) {
            self.add_vertex(id)
        }
    }

    // return true if add edge success
    pub fn add_edge(&mut self, src: &str, dst: &str) -> bool {
        if !self.vertices.contains_key(src) {
            return false;
        }
        if !self.vertices.contains_key(dst) {
            return false;
        }
        let mut success = true;
        success = success
            & self
                .vertices
                .get_mut(src)
                .unwrap()
                .add_out_vertex(dst.to_owned());
        success = success
            & self
                .vertices
                .get_mut(dst)
                .unwrap()
                .add_in_vertex(src.to_owned());
        success
    }

    pub fn set_value(&mut self, vid: &str, value: T) -> bool {
        if !self.has_vertex(vid) {
            return false;
        }
        self.vertices.get_mut(vid).unwrap().set_value(value);
        true
    }

    pub fn get_value(&self, vid: &str) -> Option<T> {
        if !self.has_vertex(vid) {
            return None;
        }
        self.vertices.get(vid).unwrap().get_value()
    }

    pub fn find_cycle(&self) -> Vec<String> {
        let mut candidates: Vec<String> = vec![];
        let mut in_counts: HashMap<String, usize> = HashMap::new();
        for (vid, vertex) in &self.vertices {
            let in_count = vertex.get_in_vertex_count();
            in_counts.insert(vid.to_owned(), in_count);
            if in_count == 0 {
                candidates.push(vid.to_owned());
            }
        }
        while !candidates.is_empty() {
            let vid = candidates.pop().unwrap();
            for out_vid in &self.vertices.get(&vid).unwrap().get_out_vertices() {
                let count = in_counts.get_mut(out_vid).unwrap();
                *count -= 1;
                if *count == 0 {
                    candidates.push(out_vid.to_owned());
                }
            }
        }
        let mut cycle_vertex: Vec<String> = vec![];
        for (id, count) in &in_counts {
            if *count > 0 {
                cycle_vertex.push(id.to_owned());
            }
        }
        cycle_vertex
    }

    fn get_unions(&self) -> HashMap<String, String> {
        let mut unions: HashMap<String, String> = HashMap::new();
        let mut ranks: HashMap<String, usize> = HashMap::new();
        for vertex in self.vertices.keys() {
            unions.insert(vertex.to_owned(), vertex.to_owned());
            ranks.insert(vertex.to_owned(), 0);
        }
        for (src, vertex) in &self.vertices {
            let dsts = vertex.get_out_vertices();
            for dst in &dsts {
                let u_src = Self::find(&unions, src);
                let u_dst = Self::find(&unions, dst);
                if u_src != u_dst {
                    let u_src_rk = ranks.get(&u_src).unwrap();
                    let u_dst_rk = ranks.get(&u_dst).unwrap();
                    if u_src_rk > u_dst_rk {
                        let u_dst = unions.get_mut(&u_dst).unwrap();
                        *u_dst = u_src;
                    } else {
                        if u_src_rk == u_dst_rk {
                            *ranks.get_mut(&u_dst).unwrap() += 1;
                        }
                        let u_src = unions.get_mut(&u_src).unwrap();
                        *u_src = u_dst;
                    }
                }
            }
        }
        unions
    }

    pub fn find_components(&self) -> HashMap<String, Vec<String>> {
        let unions = self.get_unions();
        let mut components: HashMap<String, Vec<String>> = HashMap::new();
        for (vertex, union_vertex) in &unions {
            if !components.contains_key(union_vertex) {
                components.insert(union_vertex.to_owned(), Vec::new());
            }
            components
                .get_mut(union_vertex)
                .unwrap()
                .push(vertex.to_owned());
        }
        components
    }

    // find component contains vertex
    pub fn find_component(&self, vid: &str) -> Vec<String> {
        let unions = self.get_unions();
        let union = unions.get(vid).unwrap();
        let mut component: Vec<String> = Vec::new();
        for (v, u) in &unions {
            if u == union {
                component.push(v.to_owned());
            }
        }
        component
    }

    fn find(union: &HashMap<String, String>, vertex: &str) -> String {
        let mut vertex = vertex.to_owned();
        while union.get(&vertex).unwrap().deref() != vertex {
            vertex = union.get(&vertex).unwrap().to_owned();
        }
        vertex
    }

    pub fn find_source_vertices(&self) -> Vec<String> {
        let mut source_vertex = vec![];
        for (vid, vertex) in &self.vertices {
            if vertex.get_in_vertex_count() == 0 {
                source_vertex.push(vid.to_owned());
            }
        }
        source_vertex
    }

    pub fn find_sink_vertices(&self) -> Vec<String> {
        let mut sink_vertex = vec![];
        for (vid, vertex) in &self.vertices {
            if vertex.get_out_vertex_count() == 0 {
                sink_vertex.push(vid.to_owned());
            }
        }
        sink_vertex
    }

    pub fn has_in_vertex(&self, vid: &str) -> bool {
        assert!(self.has_vertex(vid));
        self.vertices.get(vid).unwrap().get_in_vertex_count() > 0
    }

    pub fn get_in_vertices(&self, vid: &str) -> HashSet<String> {
        assert!(self.has_vertex(vid));
        self.vertices.get(vid).unwrap().get_in_vertices()
    }

    pub fn has_out_vertex(&self, vid: &str) -> bool {
        assert!(self.has_vertex(vid));
        self.vertices.get(vid).unwrap().get_out_vertex_count() > 0
    }

    pub fn get_out_vertices(&self, vid: &str) -> HashSet<String> {
        assert!(self.has_vertex(vid));
        self.vertices.get(vid).unwrap().get_out_vertices()
    }

    pub fn find_paths(
        &self,
        src: &str,
        dst: &str,
        visited: &mut HashSet<String>,
        cache: &mut GraphPaths,
    ) -> Option<Vec<GraphPath>> {
        if src == dst {
            let path: GraphPath = vec![src.to_owned()];
            return Some(vec![path]);
        }
        if !visited.insert(src.to_owned()) {
            return cache.get_paths(src, dst);
        }
        for next in &self.vertices.get(src).unwrap().get_out_vertices() {
            let paths = match self.find_paths(next, dst, visited, cache) {
                None => continue,
                Some(paths) => paths,
            };
            // src, next ... dst
            for path in &paths {
                let mut new_path = vec![src.to_owned()];
                new_path.extend(path.to_owned());
                cache.add_path(src, dst, new_path);
            }
        }
        cache.get_paths(src, dst)
    }
}

use crate::api::{Entity, Pipe};
pub struct PipeGraph<T: Clone> {
    graph: DirectedGraph<T>,
}

impl<T: Clone> PipeGraph<T> {
    pub fn new() -> Self {
        PipeGraph {
            graph: DirectedGraph::new(),
        }
    }

    pub fn add_pipe(&mut self, pipe: &Pipe, value: T) {
        let ref id = pipe.get_id();
        self.graph.add_vertex_if_not_exists(id.to_owned());
        self.graph.set_value(id, value);
        let deps = pipe.list_dependency();
        for dep in &deps {
            self.graph.add_vertex_if_not_exists(dep.to_owned());
            self.graph.add_edge(dep, id);
        }
    }

    pub fn has_pipe(&self, pid: &str) -> bool {
        if !self.graph.has_vertex(pid) {
            return false;
        }
        self.graph.get_value(pid).is_some()
    }

    pub fn find_source_pipes(&self) -> Vec<String> {
        self.graph.find_source_vertices()
    }

    pub fn find_sink_pipes(&self) -> Vec<String> {
        self.graph.find_sink_vertices()
    }

    pub fn find_components(&self) -> HashMap<String, Vec<String>> {
        self.graph.find_components()
    }

    pub fn find_component(&self, vid: &str) -> Vec<String> {
        self.graph.find_component(vid)
    }

    pub fn find_cycle(&self) -> Vec<String> {
        self.graph.find_cycle()
    }

    fn find_paths(
        &self,
        src: &str,
        dst: &str,
        visited: &mut HashSet<String>,
        cache: &mut GraphPaths,
    ) -> Option<Vec<GraphPath>> {
        self.graph.find_paths(src, dst, visited, cache)
    }

    pub fn has_upstream_pipe(&self, pid: &str) -> bool {
        self.graph.has_in_vertex(pid)
    }

    pub fn get_upstream_pipes(&self, pid: &str) -> HashSet<String> {
        self.graph.get_in_vertices(pid)
    }

    pub fn has_downstream_pipe(&self, pid: &str) -> bool {
        self.graph.has_out_vertex(pid)
    }

    pub fn get_downstream_pipes(&self, pid: &str) -> HashSet<String> {
        self.graph.get_out_vertices(pid)
    }

    pub fn get_pipe_value(&self, pid: &str) -> Option<T> {
        assert!(self.has_pipe(pid));
        self.graph.get_value(pid)
    }

    fn connect_path(left_path: Vec<String>, right_path: Vec<String>) -> GraphPath {
        if left_path.is_empty() {
            return right_path;
        }
        if right_path.is_empty() {
            return left_path;
        }
        // validate connection point
        assert!(left_path.last().unwrap() == right_path.get(0).unwrap());
        let mut connected_path: GraphPath = GraphPath::new();
        connected_path.extend(left_path);
        connected_path.extend(right_path[1..].to_owned());
        connected_path
    }

    fn connect_all(left_paths: Vec<GraphPath>, right_paths: Vec<GraphPath>) -> Vec<GraphPath> {
        let mut connected_path: Vec<GraphPath> = Vec::new();
        for left_path in &left_paths {
            for right_path in &right_paths {
                connected_path.push(Self::connect_path(
                    left_path.to_owned(),
                    right_path.to_owned(),
                ))
            }
        }
        connected_path
    }

    // search pipelines given pipe id
    pub fn search_pipelines(&self, pid: &str) -> Vec<GraphPath> {
        let ref vertics = self.find_component(pid);
        let srcs: Vec<String> = vertics
            .to_owned()
            .into_iter()
            .filter(|vid| self.has_upstream_pipe(vid))
            .collect();
        let sinks: Vec<String> = vertics
            .to_owned()
            .into_iter()
            .filter(|vid| self.has_downstream_pipe(vid))
            .collect();
        let mut pipelines: Vec<GraphPath> = Vec::new();
        for src in &srcs {
            for sink in &sinks {
                let src_to_pid = match self.find_pipeline(src, pid) {
                    Some(src_to_pid) => src_to_pid,
                    None => continue,
                };
                let pid_to_sink = match self.find_pipeline(pid, sink) {
                    Some(pid_to_sink) => pid_to_sink,
                    None => continue,
                };
                pipelines.extend(Self::connect_all(src_to_pid, pid_to_sink));
            }
        }
        pipelines
    }

    pub fn find_pipeline(&self, src: &str, dst: &str) -> Option<Vec<GraphPath>> {
        let mut visited: HashSet<String> = HashSet::new();
        let mut cache = GraphPaths::new();
        self.find_paths(src, dst, &mut visited, &mut cache)
    }
}
