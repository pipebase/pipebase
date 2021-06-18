use crate::error::Result;
use crate::{
    api::{
        Entity, Pipe, VisitEntity, OBJECT_ENTITY_ID_FIELD, PIPE_ENTITY_DEPENDENCY_FIELD,
        PIPE_ENTITY_ID_FIELD,
    },
    error::api_error,
};
use std::collections::{HashMap, HashSet};
use std::ops::Deref;

pub trait Validate<T> {
    fn validate(&mut self);
    // error location -> msg
    fn get_errors(&self) -> Option<HashMap<String, String>>;
    fn do_validate(items: &Vec<T>, location: &str) -> Result<()>;
}

pub struct PipeIdValidator {
    pub location: String,
    pub ids: Vec<String>,
    pub errors: HashMap<String, String>,
}

impl VisitEntity<Pipe> for PipeIdValidator {
    fn visit(&mut self, pipe: &Pipe) {
        self.ids.push(pipe.get_id())
    }
}

impl Validate<Pipe> for PipeIdValidator {
    fn get_errors(&self) -> Option<HashMap<String, String>> {
        match self.errors.is_empty() {
            true => None,
            false => Some(self.errors.to_owned()),
        }
    }

    fn validate(&mut self) {
        let mut invalid = false;
        let mut i = 0;
        // snake case validation
        for id in self.ids.as_slice() {
            if !is_snake_case(id, false) {
                let location = format!("{}[{}].{}", self.location, i, PIPE_ENTITY_ID_FIELD);
                self.errors.insert(
                    location,
                    format!("use snake_case for {}", PIPE_ENTITY_ID_FIELD),
                );
                invalid = true;
            }
            i += 1;
        }
        if invalid {
            return;
        }
        i = 0;
        // duplicated id validation
        let set: HashSet<String> = HashSet::new();
        for id in self.ids.as_slice() {
            if set.contains(id) {
                let location = format!("{}[{}].{}", self.location, i, PIPE_ENTITY_ID_FIELD);
                self.errors
                    .insert(location, format!("duplicated {}", PIPE_ENTITY_ID_FIELD));
            }
            i += 1;
        }
    }

    fn do_validate(pipes: &Vec<Pipe>, location: &str) -> Result<()> {
        let mut validator = PipeIdValidator {
            location: location.to_owned(),
            ids: vec![],
            errors: HashMap::new(),
        };
        for pipe in pipes {
            validator.visit(pipe)
        }
        validator.validate();
        match validator.get_errors() {
            Some(errors) => Err(api_error(errors)),
            None => Ok(()),
        }
    }
}

#[derive(Default)]
pub struct PipeDependencyValidator {
    pub location: String,
    pub ids: Vec<String>,
    pub is_source: HashMap<String, bool>,
    // dependency pipe id -> other pipe ids
    pub deps: HashMap<String, Vec<String>>,
    pub errors: HashMap<String, String>,
}

impl VisitEntity<Pipe> for PipeDependencyValidator {
    fn visit(&mut self, pipe: &Pipe) {
        let ref id = pipe.get_id();
        self.ids.push(id.to_owned());
        self.is_source.insert(id.to_owned(), pipe.is_source());
        if !self.deps.contains_key(id) {
            self.deps.insert(id.to_owned(), vec![]);
        }
        self.deps
            .get_mut(id)
            .unwrap()
            .extend(pipe.list_dependency());
    }
}

impl Validate<Pipe> for PipeDependencyValidator {
    fn get_errors(&self) -> Option<HashMap<String, String>> {
        if self.errors.is_empty() {
            return None;
        }
        Some(self.errors.to_owned())
    }

    fn validate(&mut self) {
        let mut pipe_id_set: HashSet<String> = HashSet::new();
        for id in self.ids.as_slice() {
            pipe_id_set.insert(id.to_owned());
        }
        let mut i: usize = 0;
        for id in self.ids.as_slice() {
            let location = format!("{}[{}].{}", self.location, i, PIPE_ENTITY_DEPENDENCY_FIELD);
            match self.is_source.get(id).unwrap() {
                true => match self.deps.get(id).unwrap().is_empty() {
                    true => (),
                    false => {
                        self.errors
                            .insert(location, format!("found invalid upstream for source pipe"));
                    }
                },
                false => {
                    let deps = self.deps.get(id).unwrap();
                    match deps.is_empty() {
                        true => {
                            self.errors
                                .insert(location, format!("no upstream found for downstream pipe"));
                        }
                        false => {
                            for dep in deps {
                                match pipe_id_set.contains(dep) {
                                    true => (),
                                    false => {
                                        self.errors.insert(
                                            location.to_owned(),
                                            format!("upstream does not exists"),
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            };
            i += 1;
        }
    }

    fn do_validate(pipes: &Vec<Pipe>, location: &str) -> Result<()> {
        let mut validator = PipeDependencyValidator {
            location: location.to_owned(),
            ..Default::default()
        };
        for pipe in pipes {
            validator.visit(pipe);
        }
        validator.validate();
        match validator.get_errors() {
            Some(errors) => Err(api_error(errors)),
            None => Ok(()),
        }
    }
}

#[derive(Default)]
pub struct PipeGraphValidator {
    // all pipe has at most one upstream pipe
    // in-edge count <= 1
    pub location: String,
    pub graph: HashMap<String, Vec<String>>,
    pub sources: Vec<String>,
    pub visited: HashMap<String, bool>,
    pub positions: HashMap<String, usize>,
    pub errors: HashMap<String, String>,
}

impl VisitEntity<Pipe> for PipeGraphValidator {
    fn visit(&mut self, pipe: &Pipe) {
        let ref id = pipe.get_id();
        self.positions.insert(id.to_owned(), self.positions.len());
        self.visited.insert(id.to_owned(), false);
        let deps = pipe.list_dependency();
        if deps.len() == 0 {
            // it's source pipe - listener / poller
            self.graph.insert(id.to_owned(), vec![]);
            self.sources.push(id.to_owned());
            return;
        }
        for upstream_id in deps.as_slice() {
            if !self.graph.contains_key(upstream_id) {
                self.graph.insert(upstream_id.to_owned(), vec![]);
            }
            self.graph.get_mut(upstream_id).unwrap().push(id.to_owned())
        }
    }
}

impl Validate<Pipe> for PipeGraphValidator {
    fn get_errors(&self) -> Option<HashMap<String, String>> {
        if self.errors.len() > 0 {
            return Some(self.errors.to_owned());
        }
        None
    }

    fn validate(&mut self) {
        // topology sort
        while !self.sources.is_empty() {
            let ref src = self.sources.pop().unwrap();
            let old = self.visited.insert(src.to_owned(), true);
            assert_eq!(false, old.unwrap());
            let dsts = match self.graph.get(src) {
                Some(dsts) => dsts,
                None => continue, // sink (ex: exporter) pipe does not have downstream
            };
            for dst in dsts {
                // all pipe has at most one in edge
                self.sources.push(dst.to_owned())
            }
        }
        for (id, visited) in &self.visited {
            if !visited {
                let location = format!("{}[{}]", self.location, self.positions.get(id).unwrap());
                self.errors.insert(location, "cycle detected".to_owned());
            }
        }
    }

    fn do_validate(pipes: &Vec<Pipe>, location: &str) -> Result<()> {
        let mut validator = PipeGraphValidator {
            location: location.to_owned(),
            ..Default::default()
        };
        for pipe in pipes {
            validator.visit(pipe)
        }
        validator.validate();
        match validator.get_errors() {
            Some(errors) => Err(api_error(errors)),
            None => Ok(()),
        }
    }
}

fn is_snake_case(s: &str, uppercase: bool) -> bool {
    // no leading underscore
    let mut underscore = true;
    for c in s.chars() {
        if c.is_ascii_uppercase() != uppercase {
            // non uniform upper or lower case
            return false;
        }
        if c == '_' {
            if underscore {
                // consecutive underscore
                return false;
            }
            underscore = true;
            continue;
        }
        underscore = false
    }
    true
}

fn is_camel_case(s: &str) -> bool {
    let mut uppercase = false;
    let mut i: usize = 0;
    for c in s.chars() {
        if i == 0 && !c.is_ascii_uppercase() {
            // initial uppercase
            return false;
        }
        if c.is_ascii_uppercase() {
            // no concecutive upper case
            if uppercase {
                return false;
            }
            uppercase = true;
            i += 1;
            continue;
        }
        uppercase = false;
        i += 1;
    }
    true
}

#[cfg(test)]
mod tests {

    use crate::api::App;

    #[test]
    fn test_bad_name_case_pipe() {
        let manifest_path = "resources/manifest/bad_name_case_pipe.yml";
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_invalid_source_dependency_pipe() {
        let manifest_path = "resources/manifest/invalid_source_dependency_pipe.yml";
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_non_exists_upstream_pipe() {
        let manifest_path = "resources/manifest/non_exists_upstream_pipe.yml";
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_cycle_dependency_pipe() {
        let manifest_path = "resources/manifest/cycle_dependency_pipe.yml";
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }
}
