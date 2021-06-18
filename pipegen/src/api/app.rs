use super::Object;
use crate::api::pipe::Pipe;
use crate::api::DataField;
use crate::error::*;
use crate::operation::DataFieldValidator;
use crate::operation::ObjectDependencyValidator;
use crate::operation::ObjectIdValidator;
use crate::operation::PipeDependencyValidator;
use crate::operation::PipeGraphValidator;
use crate::operation::{Generate, ObjectGenerator, PipeGenerator, PipeIdValidator, Validate};
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct App {
    pub name: String,
    pub pipes: Vec<Pipe>,
    pub objects: Option<Vec<Object>>,
}

impl App {
    pub fn parse(api_manifest_path: &str) -> Result<App> {
        let file = match std::fs::File::open(api_manifest_path) {
            Ok(file) => file,
            Err(err) => return Err(io_error(err)),
        };
        let app = match serde_yaml::from_reader::<std::fs::File, Self>(file) {
            Ok(app) => app,
            Err(err) => return Err(yaml_error(err)),
        };
        Ok(app)
    }

    pub fn print(&self) {
        println!("{}", self.generate())
    }

    fn generate_lits<T, G: Generate<T>>(items: &Vec<T>, indent: usize, join_sep: &str) -> String {
        let mut lits: Vec<String> = vec![];
        for item in items.as_slice() {
            match G::do_generate(item, indent) {
                Some(lit) => lits.push(lit),
                None => continue,
            }
        }
        lits.join(join_sep)
    }

    fn generate_objects(&self, indent: usize) -> Option<String> {
        let objects = match self.objects {
            Some(ref objects) => objects,
            None => return None,
        };
        let objects_lit = Self::generate_lits::<Object, ObjectGenerator>(objects, indent, "\n\n");
        Some(objects_lit)
    }

    pub fn generate(&self) -> String {
        let mut sections: Vec<String> = vec![];
        match self.generate_objects(1) {
            Some(objects_lit) => sections.push(objects_lit),
            None => (),
        };
        sections.push(Self::generate_lits::<Pipe, PipeGenerator>(
            &(self.pipes),
            1,
            "\n",
        ));
        format!("mod {} {{\n{}\n}}", self.name, sections.join("\n\n"))
    }

    fn validate_entity<T, V: Validate<T>>(items: &Vec<T>, location: &str) -> Result<()> {
        V::do_validate(items, location)
    }

    fn validate_pipes(&self) -> Result<()> {
        Self::validate_entity::<Pipe, PipeIdValidator>(&self.pipes, "pipes")?;
        Self::validate_entity::<Pipe, PipeDependencyValidator>(&self.pipes, "pipes")?;
        Self::validate_entity::<Pipe, PipeGraphValidator>(&self.pipes, "pipes")
    }

    fn validate_objects(&self) -> Result<()> {
        let objects = match self.objects {
            Some(ref objects) => objects,
            None => return Ok(()),
        };
        Self::validate_entity::<Object, ObjectIdValidator>(objects, "objects")?;
        for i in 0..objects.len() {
            let object = objects.get(i).unwrap();
            let location = format!("objects[{}].fields", i);
            Self::validate_entity::<DataField, DataFieldValidator>(&object.fields, &location)?;
        }
        Self::validate_entity::<Object, ObjectDependencyValidator>(objects, "objects")?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        self.validate_pipes()?;
        self.validate_objects()
    }
}
