use crate::error::Result;
use crate::{
    api::{Entity, Pipe, VisitEntity, OBJECT_ENTITY_ID_FIELD, PIPE_ENTITY_ID_FIELD},
    error::api_error,
};
use std::collections::{HashMap, HashSet};

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
