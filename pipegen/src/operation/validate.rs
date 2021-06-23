use crate::api::DataField;
use crate::api::Object;
use crate::api::{
    Entity, Pipe, VisitEntity, DATA_FIELD_ENTITY_ID_FIELD, OBJECT_ENTITY_ID_FIELD,
    PIPE_ENTITY_DEPENDENCY_FIELD, PIPE_ENTITY_ID_FIELD,
};
use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};

use super::utils::PipeGraph;

pub struct ValidationErrorDetailsDisplay {
    details: HashMap<String, String>,
}

impl ValidationErrorDetailsDisplay {
    pub fn new(details: HashMap<String, String>) -> Self {
        ValidationErrorDetailsDisplay { details }
    }
}

impl Display for ValidationErrorDetailsDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut buffer: Vec<String> = vec![];
        for (location, detail) in &self.details {
            buffer.push(format!("{} at {}", detail, location))
        }
        write!(f, "{}", buffer.join(",\n"))
    }
}

pub trait Validate<T> {
    fn new(location: &str) -> Self;
    fn validate(&mut self);
    // error location -> msg
    fn display_error_details(&self) -> Option<ValidationErrorDetailsDisplay>;
    fn details_to_display(
        details: HashMap<String, String>,
    ) -> Option<ValidationErrorDetailsDisplay> {
        match details.is_empty() {
            true => None,
            false => Some(ValidationErrorDetailsDisplay::new(details)),
        }
    }
}

pub struct PipeIdValidator {
    pub location: String,
    pub ids: Vec<String>,
    pub error_details: HashMap<String, String>,
}

impl VisitEntity<Pipe> for PipeIdValidator {
    fn visit(&mut self, pipe: &Pipe) {
        self.ids.push(pipe.get_id())
    }
}

impl Validate<Pipe> for PipeIdValidator {
    fn new(location: &str) -> Self {
        PipeIdValidator {
            location: location.to_owned(),
            ids: vec![],
            error_details: HashMap::new(),
        }
    }

    fn display_error_details(&self) -> Option<ValidationErrorDetailsDisplay> {
        Self::details_to_display(self.error_details.to_owned())
    }

    fn validate(&mut self) {
        // snake case validation
        let errors = validate_ids_with_predicate(
            &self.ids,
            &self.location,
            PIPE_ENTITY_ID_FIELD,
            "use snake_case",
            &is_snake_lower_case,
        );
        if !errors.is_empty() {
            self.error_details = errors;
            return;
        }
        let errors = validate_ids_uniqueness(
            &self.ids,
            &self.location,
            PIPE_ENTITY_ID_FIELD,
            "duplicated",
        );
        self.error_details = errors;
    }
}

pub struct PipeGraphValidator {
    pub location: String,
    pub graph: PipeGraph<Pipe>,
    pub index: HashMap<String, usize>,
    pub error_details: HashMap<String, String>,
}

impl VisitEntity<Pipe> for PipeGraphValidator {
    fn visit(&mut self, pipe: &Pipe) {
        self.graph.add_pipe(pipe, pipe.to_owned());
        self.index.insert(pipe.get_id(), self.index.len());
    }
}

impl Validate<Pipe> for PipeGraphValidator {
    fn new(location: &str) -> Self {
        PipeGraphValidator {
            location: location.to_owned(),
            graph: PipeGraph::new(),
            index: HashMap::new(),
            error_details: HashMap::new(),
        }
    }

    fn display_error_details(&self) -> Option<ValidationErrorDetailsDisplay> {
        Self::details_to_display(self.error_details.to_owned())
    }

    fn validate(&mut self) {
        for (pid, i) in &self.index {
            let pipe = self.graph.get_pipe_value(pid).unwrap();
            let location = format!("{}[{}].{}", self.location, i, PIPE_ENTITY_DEPENDENCY_FIELD);
            if pipe.is_source() {
                if self.graph.has_upstream_pipe(pid) {
                    self.error_details.insert(
                        location.to_owned(),
                        format!("found invalid upstream for source pipe"),
                    );
                }
                continue;
            }
            // non-source pipe must have upstream
            if !self.graph.has_upstream_pipe(pid) {
                self.error_details.insert(
                    location.to_owned(),
                    format!("no upstream found for downstream pipe"),
                );
                continue;
            }
            for upid in &self.graph.get_upstream_pipes(pid) {
                if !self.graph.has_pipe(upid) {
                    self.error_details
                        .insert(location.to_owned(), format!("upstream does not exists"));
                }
            }
        }
        if !self.error_details.is_empty() {
            // continue validation if previous error fixed
            return;
        }
        let cycle_vertex = self.graph.find_cycle();
        for pid in &cycle_vertex {
            let location = format!("{}[{}]", self.location, self.index.get(pid).unwrap());
            self.error_details
                .insert(location, "cycle detected".to_owned());
        }
    }
}

#[derive(Default)]
pub struct ObjectIdValidator {
    pub location: String,
    pub ids: Vec<String>,
    pub error_details: HashMap<String, String>,
}

impl VisitEntity<Object> for ObjectIdValidator {
    fn visit(&mut self, object: &Object) {
        self.ids.push(object.get_id())
    }
}

impl Validate<Object> for ObjectIdValidator {
    fn new(location: &str) -> Self {
        ObjectIdValidator {
            location: location.to_owned(),
            ..Default::default()
        }
    }

    fn display_error_details(&self) -> Option<ValidationErrorDetailsDisplay> {
        Self::details_to_display(self.error_details.to_owned())
    }

    fn validate(&mut self) {
        // camel case validation
        let errors = validate_ids_with_predicate(
            &self.ids,
            &self.location,
            OBJECT_ENTITY_ID_FIELD,
            "use CamelCase",
            &is_camel_case,
        );
        if !errors.is_empty() {
            self.error_details = errors;
            return;
        }
        let errors = validate_ids_uniqueness(
            &self.ids,
            &self.location,
            OBJECT_ENTITY_ID_FIELD,
            "duplicated",
        );
        self.error_details = errors;
    }
}

#[derive(Default)]
pub struct ObjectDependencyValidator {
    pub location: String,
    pub deps: HashMap<String, Vec<String>>,
    pub ids: Vec<String>,
    pub error_details: HashMap<String, String>,
}

impl VisitEntity<Object> for ObjectDependencyValidator {
    fn visit(&mut self, object: &Object) {
        let ref id = object.get_id();
        let dep = object.list_dependency();
        self.ids.push(id.to_owned());
        self.deps.insert(id.to_owned(), dep);
    }
}

impl Validate<Object> for ObjectDependencyValidator {
    fn new(location: &str) -> Self {
        ObjectDependencyValidator {
            location: location.to_owned(),
            ..Default::default()
        }
    }

    fn display_error_details(&self) -> Option<ValidationErrorDetailsDisplay> {
        Self::details_to_display(self.error_details.to_owned())
    }

    fn validate(&mut self) {
        for i in 0..self.ids.len() {
            let id = self.ids.get(i).unwrap();
            let mut j: usize = 0;
            for dep in self.deps.get(id).unwrap() {
                if !self.deps.contains_key(dep) {
                    let location = format!("{}[{}].fields[{}]", self.location, i, j);
                    self.error_details
                        .insert(location, "object dependency not found".to_owned());
                    j += 1;
                    continue;
                }
                // other check ...
                j += 1;
            }
        }
    }
}

#[derive(Default)]
pub struct DataFieldValidator {
    pub location: String,
    pub ids: Vec<String>,
    pub error_details: HashMap<String, String>,
}

impl VisitEntity<DataField> for DataFieldValidator {
    fn visit(&mut self, field: &DataField) {
        self.ids.push(field.get_id())
    }
}

impl Validate<DataField> for DataFieldValidator {
    fn new(location: &str) -> Self {
        DataFieldValidator {
            location: location.to_owned(),
            ..Default::default()
        }
    }

    fn display_error_details(&self) -> Option<ValidationErrorDetailsDisplay> {
        Self::details_to_display(self.error_details.to_owned())
    }

    fn validate(&mut self) {
        let errors = validate_ids_with_predicate(
            &self.ids,
            &self.location,
            DATA_FIELD_ENTITY_ID_FIELD,
            "empty",
            &is_non_empty,
        );
        if !errors.is_empty() {
            self.error_details = errors;
            return;
        }
        let errors = validate_ids_with_predicate(
            &self.ids,
            &self.location,
            DATA_FIELD_ENTITY_ID_FIELD,
            "use snake_case",
            &is_snake_lower_case,
        );
        if !errors.is_empty() {
            self.error_details = errors;
            return;
        }
        self.error_details = validate_ids_uniqueness(
            &self.ids,
            &self.location,
            DATA_FIELD_ENTITY_ID_FIELD,
            "duplicate",
        );
    }
}

fn validate_ids_with_predicate(
    ids: &Vec<String>,
    location: &str,
    id_field: &str,
    error_msg: &str,
    predicate: &dyn Fn(&str) -> bool,
) -> HashMap<String, String> {
    let mut errors: HashMap<String, String> = HashMap::new();
    for i in 0..ids.len() {
        let id = ids.get(i).unwrap();
        if !predicate(id) {
            let location = format!("{}[{}].{}", location, i, id_field);
            errors.insert(location, error_msg.to_owned());
        }
    }
    errors
}

fn validate_ids_uniqueness(
    ids: &Vec<String>,
    location: &str,
    id_field: &str,
    error_msg: &str,
) -> HashMap<String, String> {
    let mut errors: HashMap<String, String> = HashMap::new();
    let mut id_set: HashSet<String> = HashSet::new();
    for i in 0..ids.len() {
        let id = ids.get(i).unwrap();
        if id_set.contains(id) {
            let location = format!("{}[{}].{}", location, i, id_field);
            errors.insert(location, error_msg.to_owned());
            continue;
        }
        id_set.insert(id.to_owned());
    }
    errors
}

fn is_non_empty(s: &str) -> bool {
    !s.is_empty()
}

fn is_snake_lower_case(s: &str) -> bool {
    is_snake_case(s, false)
}

fn is_snake_case(s: &str, uppercase: bool) -> bool {
    // no leading underscore
    let mut underscore = true;
    let mut initial_char = true;
    for c in s.chars() {
        if initial_char && !c.is_ascii() {
            return false;
        }
        initial_char = false;
        if c.is_numeric() {
            underscore = false;
            continue;
        }
        if c.is_ascii() && c.is_ascii_uppercase() == uppercase {
            underscore = false;
            continue;
        }
        if c == '_' {
            if underscore {
                // consecutive underscore
                return false;
            }
            underscore = true;
            continue;
        }
        return false;
    }
    true
}

fn is_camel_case(s: &str) -> bool {
    let mut uppercase = false;
    let mut initial_char = true;
    for c in s.chars() {
        if initial_char && !c.is_ascii_uppercase() {
            // initial uppercase
            return false;
        }
        initial_char = false;
        if !(c.is_ascii() || c.is_numeric()) {
            return false;
        }
        if c.is_ascii_uppercase() {
            // no concecutive upper case
            if uppercase {
                return false;
            }
            uppercase = true;
            continue;
        }
        uppercase = false;
    }
    true
}

#[cfg(test)]
mod tests {

    use crate::api::App;
    use std::path::Path;

    #[test]
    fn test_bad_name_case_pipe() {
        let manifest_path = Path::new("resources/manifest/bad_name_case_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_duplicate_name_pipe() {
        let manifest_path = Path::new("resources/manifest/duplicate_name_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_invalid_source_dependency_pipe() {
        let manifest_path = Path::new("resources/manifest/invalid_source_dependency_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_non_exists_upstream_pipe() {
        let manifest_path = Path::new("resources/manifest/non_exists_upstream_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_cycle_dependency_pipe() {
        let manifest_path = Path::new("resources/manifest/cycle_dependency_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_bad_object_ty_case_pipe() {
        let manifest_path = Path::new("resources/manifest/bad_object_ty_case_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_duplicate_object_ty_pipe() {
        let manifest_path = Path::new("resources/manifest/duplicate_object_ty_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_unnamed_data_field_pipe() {
        let manifest_path = Path::new("resources/manifest/unnamed_data_field_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_duplicate_data_field_name_pipe() {
        let manifest_path = Path::new("resources/manifest/duplicate_data_field_name_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_non_exists_object_pipe() {
        let manifest_path = Path::new("resources/manifest/non_exists_object_pipe.yml");
        let app = App::parse(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }
}
