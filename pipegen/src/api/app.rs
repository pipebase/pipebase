use super::EntityAccept;
use super::Object;
use super::VisitEntity;
use crate::api::pipe::Pipe;
use crate::api::DataField;
use crate::error::*;
use crate::operation::DataFieldValidator;
use crate::operation::ObjectDependencyValidator;
use crate::operation::ObjectIdValidator;
use crate::operation::PipeDependencyValidator;
use crate::operation::PipeGraphDescriber;
use crate::operation::PipeGraphValidator;
use crate::operation::{
    Describe, Generate, ObjectGenerator, PipeGenerator, PipeIdValidator, Validate,
};
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

    fn generate_entity<T: EntityAccept<G>, G: Generate<T> + VisitEntity<T>>(
        entity: &T,
        indent: usize,
    ) -> Option<String> {
        let mut generator = G::new(indent);
        entity.accept(&mut generator);
        generator.generate()
    }

    fn generate_entities<T: EntityAccept<G>, G: Generate<T> + VisitEntity<T>>(
        entities: &Vec<T>,
        indent: usize,
        join_sep: &str,
    ) -> String {
        let mut lits: Vec<String> = vec![];
        for entity in entities.as_slice() {
            match Self::generate_entity(entity, indent) {
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
        let objects_lit =
            Self::generate_entities::<Object, ObjectGenerator>(objects, indent, "\n\n");
        Some(objects_lit)
    }

    pub fn generate(&self) -> String {
        let mut sections: Vec<String> = vec![];
        match self.generate_objects(1) {
            Some(objects_lit) => sections.push(objects_lit),
            None => (),
        };
        sections.push(Self::generate_entities::<Pipe, PipeGenerator>(
            &(self.pipes),
            1,
            "\n",
        ));
        format!("mod {} {{\n{}\n}}", self.name, sections.join("\n\n"))
    }

    fn validate_entity<T: EntityAccept<V>, V: Validate<T> + VisitEntity<T>>(
        items: &Vec<T>,
        location: &str,
    ) -> Result<()> {
        let mut validator: V = V::new(location);
        for item in items {
            item.accept(&mut validator);
        }
        validator.validate();
        match validator.get_errors() {
            Some(errors) => Err(api_error(errors)),
            None => Ok(()),
        }
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

    fn init_describer<T: EntityAccept<A>, A: Describe<T> + VisitEntity<T>>(entities: &Vec<T>) -> A {
        let mut analyzer = A::new();
        for entity in entities {
            entity.accept(&mut &mut analyzer);
        }
        analyzer
    }

    pub fn get_pipe_describer(&self) -> PipeGraphDescriber {
        Self::init_describer::<Pipe, PipeGraphDescriber>(&self.pipes)
    }

    fn describe_pipes(&self) -> std::vec::IntoIter<std::string::String> {
        let mut describe = self.get_pipe_describer();
        describe.parse();
        describe.describe()
    }

    pub fn describe(&self) {
        let mut results = self.describe_pipes();
        loop {
            match results.next() {
                Some(result) => println!("{}", result),
                None => break,
            }
        }
    }
}
