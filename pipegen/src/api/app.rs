use super::constants::{
    APP_OBJECT_NAME, BOOTSTRAP_FUNCTION_META, BOOTSTRAP_FUNCTION_NAME, BOOTSTRAP_MODULE_META_PATH,
    DEFAULT_APP_OBJECT, PIPEBASE_MAIN,
};
use super::context::ContextStore;
use super::meta::{Meta, MetaValue};
use super::package::PackageDependency;
use super::pipe::Pipe;
use super::utils::indent_literal;
use super::{Entity, EntityAccept, Object, VisitEntity};
use crate::api::{Block, DataType, Function, Rhs, Statement};
use crate::error::*;
use crate::ops::AppValidator;
use crate::ops::{AppDescriber, AppGenerator};
use crate::ops::{Describe, Generate, Validate};
use serde::Deserialize;
use std::collections::HashSet;
use std::path::Path;

#[derive(Deserialize, Debug, Clone)]
pub struct App {
    name: String,
    metas: Option<Vec<Meta>>,
    dependencies: Option<Vec<PackageDependency>>,
    cstores: Option<Vec<ContextStore>>,
    pipes: Vec<Pipe>,
    objects: Option<Vec<Object>>,
}

impl Entity for App {
    fn get_id(&self) -> String {
        self.name.to_owned()
    }

    fn list_dependency(&self) -> Vec<String> {
        let dependencies = self.get_package_dependency();
        dependencies
            .iter()
            .map(|dep| dep.get_modules().to_owned())
            .flatten()
            .collect()
    }

    fn to_literal(&self, indent: usize) -> String {
        // app metas
        let metas = self.get_metas();
        // create app object
        let app = Object::new(APP_OBJECT_NAME.to_owned(), metas.to_owned(), vec![]);
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
        // init context stores
        match self.cstores {
            Some(_) => (),
            None => self.cstores = Some(vec![]),
        }
    }

    fn has_dependency(&self, other: &PackageDependency) -> bool {
        let dependencies = self.dependencies.as_ref().unwrap();
        for dependency in dependencies {
            if dependency.eq(other) {
                return true;
            }
        }
        return false;
    }

    fn add_dependency(&mut self, dependency: PackageDependency) {
        let dependencies = self.dependencies.as_mut().unwrap();
        dependencies.push(dependency);
    }

    pub fn get_package_dependency(&self) -> &Vec<PackageDependency> {
        self.dependencies.as_ref().unwrap()
    }

    fn default_dependencies() -> Vec<PackageDependency> {
        vec![
            PackageDependency::new(
                "pipebase".to_owned(),
                Some("0.1.0".to_owned()),
                None,
                None,
                vec!["pipebase::*".to_owned()],
            ),
            PackageDependency::new(
                "tokio".to_owned(),
                Some("1.6.1".to_owned()),
                None,
                Some(vec!["full".to_owned()]),
                vec![],
            ),
            PackageDependency::new(
                "log".to_owned(),
                Some("0.4.14".to_owned()),
                None,
                None,
                vec![],
            ),
            PackageDependency::new(
                "env_logger".to_owned(),
                Some("0.8.4".to_owned()),
                None,
                None,
                vec![],
            ),
            PackageDependency::new(
                "serde".to_owned(),
                Some("1.0".to_owned()),
                None,
                Some(vec!["derive".to_owned()]),
                vec![
                    "serde::Serialize".to_owned(),
                    "serde::Deserialize".to_owned(),
                ],
            ),
        ]
    }

    pub(crate) fn get_use_modules_literal(&self, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        let mut use_module_lits: Vec<String> = Vec::new();
        for module_lit in self.list_dependency() {
            use_module_lits.push(format!("{}use {}", indent_lit, module_lit));
        }
        let mut use_module_lits = use_module_lits.join(";\n");
        use_module_lits.push(';');
        use_module_lits
    }

    pub(crate) fn get_bootstrap_function_literal(&self, indent: usize) -> String {
        let meta = Meta::Path {
            name: BOOTSTRAP_FUNCTION_META.to_owned(),
        };
        let rtype = DataType::Object(APP_OBJECT_NAME.to_owned());
        let block = Block::new(vec![Statement::new(
            None,
            Rhs::Expr(DEFAULT_APP_OBJECT.to_owned()),
        )]);
        let function = Function::new(
            BOOTSTRAP_FUNCTION_NAME.to_owned(),
            Some(meta),
            true,
            true,
            vec![],
            block,
            Some(rtype),
        );
        function.to_literal(indent)
    }

    pub(crate) fn get_main_function_literal(&self, indent: usize) -> String {
        let meta = Meta::List {
            name: PIPEBASE_MAIN.to_owned(),
            metas: vec![Meta::Value {
                name: BOOTSTRAP_MODULE_META_PATH.to_owned(),
                meta: MetaValue::Str {
                    value: self.get_app_module_name(),
                    raw: false,
                },
            }],
        };
        let block = Block::new(vec![Statement::new(
            None,
            Rhs::Expr("env_logger::init();".to_owned()),
        )]);
        let function = Function::new(
            "main".to_owned(),
            Some(meta),
            false,
            true,
            vec![],
            block,
            None,
        );
        function.to_literal(indent)
    }

    fn get_metas(&self) -> &Vec<Meta> {
        self.metas.as_ref().unwrap()
    }

    pub fn get_context_stores(&self) -> &Vec<ContextStore> {
        self.cstores.as_ref().expect("stores")
    }

    pub(crate) fn get_objects(&self) -> &Vec<Object> {
        self.objects.as_ref().expect("objects")
    }

    pub(crate) fn get_pipes(&self) -> &Vec<Pipe> {
        &self.pipes
    }

    pub(crate) fn get_app_module_name(&self) -> String {
        self.get_id()
    }

    pub fn print(&self) {
        println!("{}", self.generate())
    }

    pub fn generate(&self) -> String {
        let mut app_generator = AppGenerator::new(0);
        self.accept(&mut app_generator);
        app_generator.generate()
    }

    // generate pipelines contains pid
    pub fn generate_pipes(&self, pid: &str) -> Result<String> {
        let mut describer = AppDescriber::new();
        self.accept(&mut describer);
        // filter pipes with selected pipelines - partial generation
        let pipelines = describer.get_pipelines(pid)?;
        let selected_pipes: HashSet<String> = pipelines.into_iter().flatten().collect();
        let mut app_generator = AppGenerator::new(0);
        self.accept(&mut app_generator);
        app_generator.set_pipe_filter(selected_pipes);
        Ok(app_generator.generate())
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

    pub fn validate_cstores(&self) -> Result<()> {
        let mut validator = AppValidator::new("");
        self.accept(&mut validator);
        validator.validate_cstores()
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

    pub fn describe_pipe(&self, pid: &str) -> Result<String> {
        let mut describer = AppDescriber::new();
        self.accept(&mut describer);
        describer.describe_pipe(pid)
    }

    pub fn describe_pipe_graph(&self) -> Vec<String> {
        let mut describer = AppDescriber::new();
        self.accept(&mut describer);
        describer.describe_pipe_graph()
    }

    pub fn describe_pipelines(&self, pid: &str) -> Result<Vec<String>> {
        let mut describer = AppDescriber::new();
        self.accept(&mut describer);
        describer.describe_pipelines(pid)
    }

    pub fn describe_objects(&self) -> Vec<String> {
        let mut describer = AppDescriber::new();
        self.accept(&mut describer);
        describer.describe_objects()
    }

    pub fn describe_object(&self, oid: &str) -> Result<String> {
        let mut describer = AppDescriber::new();
        self.accept(&mut describer);
        describer.describe_object(oid)
    }

    pub fn describe(&self) -> Vec<String> {
        let mut describer = AppDescriber::new();
        self.accept(&mut describer);
        describer.describe()
    }
}
