use super::meta::{Meta, MetaValue};
use super::{DataType, Entity, EntityAccept, Object, VisitEntity};
use crate::api::pipe::Pipe;
use crate::api::DataField;
use crate::error::*;
use crate::ops::AppValidator;
use crate::ops::{AppDescriber, AppGenerator};
use crate::ops::{Describe, Generate, Validate};
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

    pub fn get_objects(&self) -> Option<&Vec<Object>> {
        self.objects.as_ref()
    }

    pub fn get_pipes(&self) -> &Vec<Pipe> {
        &self.pipes.as_ref()
    }

    pub fn print(&self) {
        match self.generate() {
            Some(lit) => println!("{}", lit),
            None => (),
        }
    }

    pub fn generate(&self) -> Option<String> {
        let mut app_generator = AppGenerator::new(0);
        self.accept(&mut app_generator);
        app_generator.generate()
    }

    pub fn validate_pipes(&self) -> Result<()> {
        let mut validator = AppValidator::new("");
        self.accept(&mut validator);
        validator.validate_pipes()
    }

    pub fn validate_objects(&self) -> Result<()> {
        let mut validator = AppValidator::new("");
        self.accept(&mut validator);
        validator.validate_objects()
    }

    pub fn validate(&self) -> Result<()> {
        let mut validator = AppValidator::new("");
        self.accept(&mut validator);
        validator.validate()
    }

    pub fn describe_pipes(&self) -> Vec<String> {
        let mut describer = AppDescriber::new();
        self.accept(&mut describer);
        describer.describe_pipes()
    }

    pub fn describe_pipelines(&self, pid: &str) -> Vec<String> {
        let mut describer = AppDescriber::new();
        self.accept(&mut describer);
        describer.describe_pipelines(pid)
    }

    pub fn describe(&self) -> Vec<String> {
        let mut describer = AppDescriber::new();
        self.accept(&mut describer);
        describer.describe()
    }
}
