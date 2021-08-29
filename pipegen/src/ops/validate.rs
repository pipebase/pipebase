use crate::models::{
    App, ContextStore, DataField, Entity, EntityAccept, Object, Pipe, VisitEntity,
    CONTEXT_STORE_ENTITY_ID_FIELD, DATA_FIELD_ENTITY_ID_FIELD, OBJECT_ENTITY_ID_FIELD,
    PIPE_ENTITY_DEPENDENCY_FIELD, PIPE_ENTITY_ID_FIELD, PIPE_OUTPUT_FIELD,
};

use crate::error::{api_error, Result};
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

pub trait Validate {
    fn new(location: &str) -> Self;
    fn validate(&mut self) -> Result<()>;
    /// check any error details collected
    fn check(details: &HashMap<String, String>) -> Result<()> {
        match details.is_empty() {
            true => Ok(()),
            false => Err(api_error(ValidationErrorDetailsDisplay::new(
                details.to_owned(),
            ))),
        }
    }
}

pub struct PipeIdValidator {
    pub location: String,
    pub ids: Vec<String>,
}

impl VisitEntity<Pipe> for PipeIdValidator {
    fn visit(&mut self, pipe: &Pipe) {
        self.ids.push(pipe.get_id())
    }
}

impl Validate for PipeIdValidator {
    fn new(location: &str) -> Self {
        PipeIdValidator {
            location: location.to_owned(),
            ids: vec![],
        }
    }

    fn validate(&mut self) -> Result<()> {
        // snake case validation
        let errors = validate_ids_with_predicate(
            &self.ids,
            &self.location,
            PIPE_ENTITY_ID_FIELD,
            "use snake_case",
            &is_snake_lower_case,
        );
        Self::check(&errors)?;
        let errors = validate_ids_uniqueness(
            &self.ids,
            &self.location,
            PIPE_ENTITY_ID_FIELD,
            "duplicated",
        );
        Self::check(&errors)
    }
}

pub struct PipeOutputValidator {
    location: String,
    pipes: Vec<Pipe>,
}

impl VisitEntity<Pipe> for PipeOutputValidator {
    fn visit(&mut self, pipe: &Pipe) {
        self.pipes.push(pipe.to_owned())
    }
}

impl Validate for PipeOutputValidator {
    fn new(location: &str) -> Self {
        PipeOutputValidator {
            location: location.to_owned(),
            pipes: Vec::new(),
        }
    }

    fn validate(&mut self) -> Result<()> {
        let mut errors: HashMap<String, String> = HashMap::new();
        for (i, pipe) in self.pipes.iter().enumerate() {
            let location = format!("{}[{}].{}", self.location, i, PIPE_OUTPUT_FIELD);
            if pipe.is_sink() && pipe.has_output() {
                errors.insert(location, String::from("found invalid output for sink pipe"));
                continue;
            }
            if !pipe.is_sink() && !pipe.has_output() {
                errors.insert(location, String::from("pipe output not found"));
            }
        }
        Self::check(&errors)?;
        Ok(())
    }
}

pub struct PipeGraphValidator {
    pub location: String,
    pub graph: PipeGraph<Pipe>,
    pub index: HashMap<String, usize>,
}

impl VisitEntity<Pipe> for PipeGraphValidator {
    fn visit(&mut self, pipe: &Pipe) {
        self.graph.add_pipe(pipe, pipe.to_owned());
        self.index.insert(pipe.get_id(), self.index.len());
    }
}

impl Validate for PipeGraphValidator {
    fn new(location: &str) -> Self {
        PipeGraphValidator {
            location: location.to_owned(),
            graph: PipeGraph::new(),
            index: HashMap::new(),
        }
    }

    fn validate(&mut self) -> Result<()> {
        let mut errors: HashMap<String, String> = HashMap::new();
        for (pid, i) in &self.index {
            let pipe = self.graph.get_pipe_value(pid).unwrap();
            let location = format!("{}[{}].{}", self.location, i, PIPE_ENTITY_DEPENDENCY_FIELD);
            if pipe.is_source() {
                if self.graph.has_upstream_pipe(pid) {
                    errors.insert(
                        location.to_owned(),
                        String::from("found invalid upstream for source pipe"),
                    );
                }
                continue;
            }
            // non-source pipe must have upstream
            if !self.graph.has_upstream_pipe(pid) {
                errors.insert(
                    location.to_owned(),
                    "no upstream found for downstream pipe".to_string(),
                );
                continue;
            }
            for upid in self.graph.get_upstream_pipes(pid) {
                if !self.graph.has_pipe(upid) {
                    errors.insert(location.to_owned(), "upstream does not exists".to_string());
                }
            }
        }
        Self::check(&errors)?;
        let cycle_vertex = self.graph.find_cycle();
        for pid in &cycle_vertex {
            let location = format!("{}[{}]", self.location, self.index.get(pid).unwrap());
            errors.insert(location, "cycle detected".to_owned());
        }
        Self::check(&errors)
    }
}

#[derive(Default)]
pub struct ObjectIdValidator {
    pub location: String,
    pub ids: Vec<String>,
}

impl VisitEntity<Object> for ObjectIdValidator {
    fn visit(&mut self, object: &Object) {
        self.ids.push(object.get_id())
    }
}

impl Validate for ObjectIdValidator {
    fn new(location: &str) -> Self {
        ObjectIdValidator {
            location: location.to_owned(),
            ..Default::default()
        }
    }

    fn validate(&mut self) -> Result<()> {
        // camel case validation
        let errors = validate_ids_with_predicate(
            &self.ids,
            &self.location,
            OBJECT_ENTITY_ID_FIELD,
            "use CamelCase",
            &is_camel_case,
        );
        Self::check(&errors)?;
        let errors = validate_ids_uniqueness(
            &self.ids,
            &self.location,
            OBJECT_ENTITY_ID_FIELD,
            "duplicated",
        );
        Self::check(&errors)
    }
}

#[derive(Default)]
pub struct ObjectDependencyValidator {
    pub location: String,
    pub deps: HashMap<String, Vec<String>>,
    pub ids: Vec<String>,
}

impl VisitEntity<Object> for ObjectDependencyValidator {
    fn visit(&mut self, object: &Object) {
        let id = &object.get_id();
        let dep = object.list_dependency();
        self.ids.push(id.to_owned());
        self.deps.insert(id.to_owned(), dep);
    }
}

impl Validate for ObjectDependencyValidator {
    fn new(location: &str) -> Self {
        ObjectDependencyValidator {
            location: location.to_owned(),
            ..Default::default()
        }
    }

    fn validate(&mut self) -> Result<()> {
        let mut errors: HashMap<String, String> = HashMap::new();
        for i in 0..self.ids.len() {
            let id = self.ids.get(i).unwrap();
            let mut j: usize = 0;
            for dep in self.deps.get(id).unwrap() {
                if !self.deps.contains_key(dep) {
                    let location = format!("{}[{}].fields[{}]", self.location, i, j);
                    errors.insert(location, "object dependency not found".to_owned());
                    j += 1;
                    continue;
                }
                // other check ...
                j += 1;
            }
        }
        Self::check(&errors)
    }
}

#[derive(Default)]
pub struct DataFieldValidator {
    pub location: String,
    pub ids: Vec<String>,
}

impl VisitEntity<DataField> for DataFieldValidator {
    fn visit(&mut self, field: &DataField) {
        self.ids.push(field.get_id())
    }
}

impl Validate for DataFieldValidator {
    fn new(location: &str) -> Self {
        DataFieldValidator {
            location: location.to_owned(),
            ..Default::default()
        }
    }

    fn validate(&mut self) -> Result<()> {
        let errors = validate_ids_with_predicate(
            &self.ids,
            &self.location,
            DATA_FIELD_ENTITY_ID_FIELD,
            "empty",
            &is_non_empty,
        );
        Self::check(&errors)?;
        let errors = validate_ids_with_predicate(
            &self.ids,
            &self.location,
            DATA_FIELD_ENTITY_ID_FIELD,
            "use snake_case",
            &is_snake_lower_case,
        );
        Self::check(&errors)?;
        let errors = validate_ids_uniqueness(
            &self.ids,
            &self.location,
            DATA_FIELD_ENTITY_ID_FIELD,
            "duplicate",
        );
        Self::check(&errors)
    }
}

#[derive(Default)]
pub struct ContextStoreIdValidator {
    pub location: String,
    pub ids: Vec<String>,
}

impl VisitEntity<ContextStore> for ContextStoreIdValidator {
    fn visit(&mut self, cstore: &ContextStore) {
        self.ids.push(cstore.get_id())
    }
}

impl Validate for ContextStoreIdValidator {
    fn new(location: &str) -> Self {
        ContextStoreIdValidator {
            location: location.to_owned(),
            ..Default::default()
        }
    }

    fn validate(&mut self) -> Result<()> {
        // camel case validation
        let errors = validate_ids_with_predicate(
            &self.ids,
            &self.location,
            CONTEXT_STORE_ENTITY_ID_FIELD,
            "use snake_case",
            &is_snake_lower_case,
        );
        Self::check(&errors)?;
        let errors = validate_ids_uniqueness(
            &self.ids,
            &self.location,
            CONTEXT_STORE_ENTITY_ID_FIELD,
            "duplicated",
        );
        Self::check(&errors)
    }
}

pub struct AppValidator {
    app: Option<App>,
}

impl VisitEntity<App> for AppValidator {
    fn visit(&mut self, app: &App) {
        self.app = Some(app.to_owned())
    }
}

impl Validate for AppValidator {
    fn new(_location: &str) -> Self {
        AppValidator { app: None }
    }

    fn validate(&mut self) -> Result<()> {
        self.validate_pipes()?;
        self.validate_objects()?;
        self.validate_cstores()
    }
}

impl AppValidator {
    fn get_app(&self) -> &App {
        self.app.as_ref().unwrap()
    }

    fn validate_entities<T: EntityAccept<V>, V: Validate + VisitEntity<T>>(
        items: &[T],
        location: &str,
    ) -> Result<()> {
        let mut validator: V = V::new(location);
        for item in items {
            item.accept_entity_visitor(&mut validator);
        }
        validator.validate()
    }

    pub fn validate_pipes(&self) -> Result<()> {
        let pipes = self.get_app().get_pipes();
        Self::validate_entities::<Pipe, PipeIdValidator>(pipes, "pipes")?;
        Self::validate_entities::<Pipe, PipeOutputValidator>(pipes, "pipes")?;
        Self::validate_entities::<Pipe, PipeGraphValidator>(pipes, "pipes")
    }

    pub fn validate_objects(&self) -> Result<()> {
        let objects = self.get_app().get_objects();
        Self::validate_entities::<Object, ObjectIdValidator>(objects, "objects")?;
        for i in 0..objects.len() {
            let object = objects.get(i).unwrap();
            let location = format!("objects[{}].fields", i);
            Self::validate_entities::<DataField, DataFieldValidator>(
                object.get_fields(),
                &location,
            )?;
        }
        Self::validate_entities::<Object, ObjectDependencyValidator>(objects, "objects")?;
        Ok(())
    }

    pub fn validate_cstores(&self) -> Result<()> {
        let cstores = self.get_app().get_context_stores();
        Self::validate_entities::<ContextStore, ContextStoreIdValidator>(cstores, "cstores")?;
        Ok(())
    }
}

fn validate_ids_with_predicate(
    ids: &[String],
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
    ids: &[String],
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

    use crate::models::App;
    use std::path::Path;

    #[test]
    fn test_bad_name_case_pipe() {
        let manifest_path = Path::new("resources/manifest/bad_name_case_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_duplicate_name_pipe() {
        let manifest_path = Path::new("resources/manifest/duplicate_name_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_invalid_source_dependency_pipe() {
        let manifest_path = Path::new("resources/manifest/invalid_source_dependency_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_non_exists_upstream_pipe() {
        let manifest_path = Path::new("resources/manifest/non_exists_upstream_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_no_upstream_downstream_pipe() {
        let manifest_path = Path::new("resources/manifest/no_upstream_downstream_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_cycle_dependency_pipe() {
        let manifest_path = Path::new("resources/manifest/cycle_dependency_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_bad_object_ty_case_pipe() {
        let manifest_path = Path::new("resources/manifest/bad_object_ty_case_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_duplicate_object_ty_pipe() {
        let manifest_path = Path::new("resources/manifest/duplicate_object_ty_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_unnamed_data_field_pipe() {
        let manifest_path = Path::new("resources/manifest/unnamed_data_field_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_duplicate_data_field_name_pipe() {
        let manifest_path = Path::new("resources/manifest/duplicate_data_field_name_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_non_exists_object_pipe() {
        let manifest_path = Path::new("resources/manifest/non_exists_object_pipe.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_invalid_exporter_output() {
        let manifest_path = Path::new("resources/manifest/invalid_exporter_output.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }

    #[test]
    fn test_pipe_output_not_found() {
        let manifest_path = Path::new("resources/manifest/pipe_output_not_found.yml");
        let app = App::read(manifest_path).unwrap();
        let e = app.validate().expect_err("expect invalid");
        println!("{}", e)
    }
}
