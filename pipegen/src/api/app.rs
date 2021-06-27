use super::context::ContextStore;
use super::dependency::Dependency;
use super::meta::{Meta, MetaValue};
use super::pipe::Pipe;
use super::utils::indent_literal;
use super::{Entity, EntityAccept, Object, VisitEntity};
use crate::api::{Block, DataType, Function, Rhs, Statement};
use crate::error::*;
use crate::ops::AppValidator;
use crate::ops::{AppDescriber, AppGenerator};
use crate::ops::{Describe, Generate, Validate};
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct App {
    name: String,
    metas: Option<Vec<Meta>>,
    cstore: Option<ContextStore>,
    dependencies: Option<Vec<Dependency>>,
    pipes: Vec<Pipe>,
    objects: Option<Vec<Object>>,
}

impl Entity for App {
    fn get_id(&self) -> String {
        self.name.to_owned()
    }

    fn list_dependency(&self) -> Vec<String> {
        let dependencies = match self.dependencies {
            Some(ref dependencies) => dependencies,
            None => return Vec::new(),
        };
        let mut modules: Vec<String> = Vec::new();
        for dependency in dependencies {
            modules.extend(dependency.get_modules().to_owned());
        }
        modules
    }

    fn to_literal(&self, indent: usize) -> String {
        // app metas
        let metas = self.get_metas();
        // app object fields
        let cstore = self.get_context_store().as_data_field();
        // create app object
        let app = Object::new("App".to_owned(), metas.to_owned(), vec![cstore]);
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
        let mut app = match serde_yaml::from_reader::<std::fs::File, Self>(file) {
            Ok(app) => app,
            Err(err) => return Err(yaml_error(err)),
        };
        app.init();
        Ok(app)
    }

    fn init(&mut self) {
        // init all fields as Some
        // init app dependencies
        match self.dependencies {
            Some(_) => (),
            None => self.dependencies = Some(Vec::new()),
        };
        for default_dependency in Self::default_dependencies() {
            if !self.has_dependency(&default_dependency) {
                self.add_dependency(default_dependency)
            }
        }
        // init context store
        match self.cstore {
            Some(_) => (),
            None => self.cstore = Some(ContextStore::new("pipe_contexts".to_owned())),
        };
        // init metas
        match self.metas {
            Some(_) => (),
            None => {
                self.metas = Some(vec![Meta::List {
                    name: "derive".to_owned(),
                    metas: vec![
                        Meta::Path {
                            name: "Bootstrap".to_owned(),
                        },
                        Meta::Path {
                            name: "ContextStore".to_owned(),
                        },
                        Meta::Path {
                            name: "Default".to_owned(),
                        },
                    ],
                }])
            }
        };
        // init objects
        match self.objects {
            Some(_) => (),
            None => self.objects = Some(vec![]),
        }
    }

    pub fn has_dependency(&self, other: &Dependency) -> bool {
        let dependencies = self.dependencies.as_ref().unwrap();
        for dependency in dependencies {
            if dependency.eq(other) {
                return true;
            }
        }
        return false;
    }

    pub fn add_dependency(&mut self, dependency: Dependency) {
        let dependencies = self.dependencies.as_mut().unwrap();
        dependencies.push(dependency);
    }

    pub fn default_dependencies() -> Vec<Dependency> {
        vec![
            Dependency::new(
                "pipebase".to_owned(),
                Some("0.1.0".to_owned()),
                None,
                None,
                vec!["pipebase::*".to_owned()],
            ),
            Dependency::new(
                "tokio".to_owned(),
                Some("1.6.1".to_owned()),
                None,
                Some(vec!["full".to_owned()]),
                vec![],
            ),
            Dependency::new(
                "log".to_owned(),
                Some("0.4.14".to_owned()),
                None,
                None,
                vec![],
            ),
        ]
    }

    pub fn get_use_modules_literal(&self, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        let mut use_module_lits: Vec<String> = Vec::new();
        for module_lit in self.list_dependency() {
            use_module_lits.push(format!("{}use {}", indent_lit, module_lit));
        }
        use_module_lits.push("".to_owned());
        use_module_lits.join(";\n")
    }

    pub fn get_bootstrap_function_literal(&self, indent: usize) -> String {
        let meta = Meta::Path {
            name: "bootstrap".to_owned(),
        };
        let rtype = DataType::Object("App".to_owned());
        let block = Block::new(vec![Statement::new(
            None,
            Rhs::Expr("App::default()".to_owned()),
        )]);
        let function = Function::new(
            "bootstrap".to_owned(),
            Some(meta),
            true,
            true,
            vec![],
            block,
            Some(rtype),
        );
        function.to_literal(indent)
    }

    pub fn get_main_function_literal(&self, indent: usize) -> String {
        let meta = Meta::List {
            name: "pipebase::main".to_owned(),
            metas: vec![Meta::Value {
                name: "bootstrap".to_owned(),
                meta: MetaValue::Str(self.get_id()),
            }],
        };
        let function = Function::new(
            "main".to_owned(),
            Some(meta),
            false,
            true,
            vec![],
            Block::new(vec![]),
            None,
        );
        function.to_literal(indent)
    }

    pub fn get_metas(&self) -> &Vec<Meta> {
        self.metas.as_ref().unwrap()
    }

    pub fn get_context_store(&self) -> &ContextStore {
        self.cstore.as_ref().unwrap()
    }

    pub fn get_objects(&self) -> &Vec<Object> {
        self.objects.as_ref().unwrap()
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
