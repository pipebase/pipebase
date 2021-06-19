use std::collections::{HashMap, HashSet};

pub struct Graph {
    g: HashMap<String, HashSet<String>>,
}

impl Graph {
    pub fn new() -> Graph {
        Graph { g: HashMap::new() }
    }

    pub fn has_vertex(&self, id: &str) -> bool {
        self.g.contains_key(id)
    }

    pub fn add_vertex(&mut self, id: &str) {
        self.g.insert(id.to_owned(), HashSet::new());
    }

    pub fn add_vertex_if_not_exists(&mut self, id: &str) {
        if !self.has_vertex(id) {
            self.add_vertex(id)
        }
    }

    // return true if add edge success
    pub fn add_edge(&mut self, src: &str, dst: &str) -> bool {
        let dsts = match self.g.get_mut(src) {
            Some(dsts) => dsts,
            None => return false,
        };
        dsts.insert(dst.to_owned())
    }

    pub fn find_cycle(&self) -> Vec<String> {
        let mut candidates: Vec<String> = vec![];
        let mut in_count: HashMap<String, usize> = HashMap::new();
        // collect in edge count
        for (_, dsts) in &self.g {
            for dst in dsts {
                match in_count.get_mut(dst) {
                    Some(count) => *count += 1,
                    None => {
                        in_count.insert(dst.to_owned(), 1);
                    }
                };
            }
        }
        // find source vertex
        for id in self.g.keys() {
            if !in_count.contains_key(id) {
                candidates.push(id.to_owned());
            }
        }
        while !candidates.is_empty() {
            let src = candidates.pop().unwrap();
            for dst in self.g.get(&src).unwrap() {
                let count = in_count.get_mut(dst).unwrap();
                *count -= 1;
                if *count == 0 {
                    candidates.push(dst.to_owned());
                }
            }
        }
        let mut cycle_vertex: Vec<String> = vec![];
        for (id, count) in &in_count {
            if *count > 0 {
                cycle_vertex.push(id.to_owned());
            }
        }
        cycle_vertex
    }
}

impl Default for Graph {
    fn default() -> Self {
        Graph { g: HashMap::new() }
    }
}
