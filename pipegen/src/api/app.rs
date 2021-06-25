use super::meta::{Meta, MetaValue};
use super::{DataType, Entity, EntityAccept, Object, VisitEntity};
use crate::api::pipe::Pipe;
use crate::api::DataField;
use crate::error::*;
use crate::ops::DataFieldValidator;
use crate::ops::ObjectDependencyValidator;
use crate::ops::ObjectIdValidator;
use crate::ops::PipeGraphDescriber;
use crate::ops::PipeGraphValidator;
use crate::ops::{Describe, Generate, ObjectGenerator, PipeGenerator, PipeIdValidator, Validate};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct ContextStore {
    name: String,
    methods: HashMap<String, String>,
    data_ty: DataType,
}

impl ContextStore {
    pub fn new(name: String) -> Self {
        let mut methods = HashMap::new();
        methods.insert("get".to_owned(), "get".to_owned());
        methods.insert("insert".to_owned(), "insert".to_owned());
        let data_ty = DataType::HashMap {
            key_data_ty: Box::new(DataType::String),
            value_data_ty: Box::new(DataType::Object("Arc<RwLock<Context>>".to_owned())),
        };
        ContextStore {
            name: name,
            methods: methods,
            data_ty: data_ty,
        }
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    pub fn get_methods(&self) -> &HashMap<String, String> {
        &self.methods
    }

    fn methods_as_meta(&self) -> Meta {
        let mut method_metas = vec![];
        for (name, value) in &self.methods {
            method_metas.push(Meta::Value {
                name: name.to_owned(),
                meta: MetaValue::Str(value.to_owned()),
            });
        }
        Meta::List {
            name: "method".to_owned(),
            metas: method_metas,
        }
    }

    fn get_meta(&self) -> Meta {
        Meta::List {
            name: "cstore".to_owned(),
            metas: vec![self.methods_as_meta()],
        }
    }

    pub fn as_data_field(&self) -> DataField {
        DataField::new_named_field(
            self.data_ty.to_owned(),
            self.name.to_owned(),
            vec![self.get_meta()],
            false,
            false,
        )
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct App {
    name: String,
    metas: Option<Vec<Meta>>,
    cstore: Option<ContextStore>,
    dependencies: Option<Vec<String>>,
    pipes: Vec<Pipe>,
    objects: Option<Vec<Object>>,
}

impl Entity for App {
    fn get_id(&self) -> String {
        self.name.to_owned()
    }

    fn list_dependency(&self) -> Vec<String> {
        match self.dependencies {
            Some(ref dependencies) => dependencies.to_owned(),
            None => vec![],
        }
    }

    fn to_literal(&self, indent: usize) -> String {
        // app metas
        let metas = self.get_metas();
        // app object fields
        let cstore = self.get_context_store().as_data_field();
        // create app object
        let app = Object::new(self.get_id(), metas, vec![cstore]);
        app.to_literal(indent)
    }
}

impl<V> EntityAccept<V> for App where V: VisitEntity<Self> {}

impl App {
    pub fn parse(api_manifest_path: &Path) -> Result<App> {
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

    pub fn get_metas(&self) -> Vec<Meta> {
        match self.metas {
            Some(ref metas) => metas.to_owned(),
            None => vec![Meta::List {
                name: "derive".to_owned(),
                metas: vec![
                    Meta::Path {
                        name: "Boostrap".to_owned(),
                    },
                    Meta::Path {
                        name: "ContextStore".to_owned(),
                    },
                ],
            }],
        }
    }

    pub fn get_context_store(&self) -> ContextStore {
        match self.cstore {
            Some(ref cstore) => cstore.to_owned(),
            None => ContextStore::new("pipe_contexts".to_owned()),
        }
    }

    pub fn print(&self) {
        println!("{}", self.generate())
    }

    pub fn generate_entity<T: EntityAccept<G>, G: Generate<T> + VisitEntity<T>>(
        entity: &T,
        indent: usize,
    ) -> Option<String> {
        let mut generator = G::new(indent);
        entity.accept(&mut generator);
        generator.generate()
    }

    pub fn generate_entities<T: EntityAccept<G>, G: Generate<T> + VisitEntity<T>>(
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

    pub fn generate_objects(&self, indent: usize) -> Option<String> {
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
        sections.push(self.to_literal(1));
        format!("mod {} {{\n{}\n}}", self.name, sections.join("\n\n"))
    }

    pub fn validate_entity<T: EntityAccept<V>, V: Validate<T> + VisitEntity<T>>(
        items: &Vec<T>,
        location: &str,
    ) -> Result<()> {
        let mut validator: V = V::new(location);
        for item in items {
            item.accept(&mut validator);
        }
        validator.validate();
        match validator.display_error_details() {
            Some(details) => Err(api_error(details)),
            None => Ok(()),
        }
    }

    pub fn validate_pipes(&self) -> Result<()> {
        Self::validate_entity::<Pipe, PipeIdValidator>(&self.pipes, "pipes")?;
        Self::validate_entity::<Pipe, PipeGraphValidator>(&self.pipes, "pipes")
    }

    pub fn validate_objects(&self) -> Result<()> {
        let objects = match self.objects {
            Some(ref objects) => objects,
            None => return Ok(()),
        };
        Self::validate_entity::<Object, ObjectIdValidator>(objects, "objects")?;
        for i in 0..objects.len() {
            let object = objects.get(i).unwrap();
            let location = format!("objects[{}].fields", i);
            Self::validate_entity::<DataField, DataFieldValidator>(object.get_fields(), &location)?;
        }
        Self::validate_entity::<Object, ObjectDependencyValidator>(objects, "objects")?;
        Ok(())
    }

    pub fn validate(&self) -> Result<()> {
        self.validate_pipes()?;
        self.validate_objects()
    }

    pub fn init_describer<T: EntityAccept<A>, A: Describe + VisitEntity<T>>(
        entities: &Vec<T>,
    ) -> A {
        let mut describer = A::new();
        for entity in entities {
            entity.accept(&mut &mut describer);
        }
        describer.parse();
        describer
    }

    pub fn get_pipe_describer(&self) -> PipeGraphDescriber {
        Self::init_describer::<Pipe, PipeGraphDescriber>(&self.pipes)
    }

    pub fn describe_pipes(&self) -> Vec<String> {
        let describe = self.get_pipe_describer();
        describe.describe()
    }

    pub fn describe_pipelines(&self, pid: &str) -> Vec<String> {
        let mut describe = self.get_pipe_describer();
        describe.parse();
        describe.describe_pipelines(pid)
    }

    pub fn describe(&self) {
        let results = self.describe_pipes();
        for result in results {
            println!("{}", result)
        }
    }
}
