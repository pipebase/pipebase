use std::{
    collections::{HashMap, HashSet},
    ops::Deref,
};

pub struct DirectedGraph {
    g: HashMap<String, HashSet<String>>,
    in_counts: HashMap<String, usize>,
}

impl DirectedGraph {
    pub fn new() -> DirectedGraph {
        DirectedGraph {
            g: HashMap::new(),
            in_counts: HashMap::new(),
        }
    }

    pub fn has_vertex(&self, id: &str) -> bool {
        self.g.contains_key(id)
    }

    fn add_vertex(&mut self, id: &str) {
        self.g.insert(id.to_owned(), HashSet::new());
        self.in_counts.insert(id.to_owned(), 0);
    }

    pub fn add_vertex_if_not_exists(&mut self, id: &str) {
        if !self.has_vertex(id) {
            self.add_vertex(id)
        }
    }

    // return true if add edge success
    pub fn add_edge(&mut self, src: &str, dst: &str) -> bool {
        match self.g.contains_key(dst) {
            true => (),
            false => return false,
        };
        let dsts = match self.g.get_mut(src) {
            Some(dsts) => dsts,
            None => return false,
        };
        if dsts.insert(dst.to_owned()) {
            *self.in_counts.get_mut(dst).unwrap() += 1;
            return true;
        }
        false
    }

    pub fn find_cycle(&self) -> Vec<String> {
        let mut candidates: Vec<String> = vec![];
        let mut in_counts = self.in_counts.to_owned();
        for (id, in_count) in &in_counts {
            if *in_count == 0 {
                candidates.push(id.to_owned());
            }
        }
        while !candidates.is_empty() {
            let src = candidates.pop().unwrap();
            for dst in self.g.get(&src).unwrap() {
                let count = in_counts.get_mut(dst).unwrap();
                *count -= 1;
                if *count == 0 {
                    candidates.push(dst.to_owned());
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

    pub fn find_components(&self) -> HashMap<String, Vec<String>> {
        let mut unions: HashMap<String, String> = HashMap::new();
        let mut ranks: HashMap<String, usize> = HashMap::new();
        for vertex in self.g.keys() {
            unions.insert(vertex.to_owned(), vertex.to_owned());
            ranks.insert(vertex.to_owned(), 0);
        }
        for (src, dsts) in &self.g {
            for dst in dsts {
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

    fn find(union: &HashMap<String, String>, vertex: &str) -> String {
        let mut vertex = vertex.to_owned();
        while union.get(&vertex).unwrap().deref() != vertex {
            vertex = union.get(&vertex).unwrap().to_owned();
        }
        vertex
    }

    pub fn find_source_vertex(&self) -> Vec<String> {
        let mut source_vertex = vec![];
        for (vertex, in_count) in &self.in_counts {
            if *in_count == 0 {
                source_vertex.push(vertex.to_owned());
            }
        }
        source_vertex
    }

    pub fn find_sink_vertex(&self) -> Vec<String> {
        let mut sink_vertex = vec![];
        for (src, dsts) in &self.g {
            if dsts.is_empty() {
                sink_vertex.push(src.to_owned());
            }
        }
        sink_vertex
    }
}

impl Default for DirectedGraph {
    fn default() -> Self {
        DirectedGraph {
            g: HashMap::new(),
            in_counts: HashMap::new(),
        }
    }
}
