use super::constants::{
    APP_OBJECT_NAME, BOOTSTRAP_FUNCTION_META, BOOTSTRAP_FUNCTION_NAME, BOOTSTRAP_MODULE_META_PATH,
    DEFAULT_APP_OBJECT, PIPEBASE_MAIN, TRACING_INSTRUMENT,
};
use super::context::ContextStore;
use super::dependency::{CrateVisitor, Dependency, UseCrate};
use super::error::ErrorHandler;
use super::meta::{metas_to_literal, Meta, MetaValue};
use super::pipe::Pipe;
use super::utils::indent_literal;
use super::{Entity, EntityAccept, Object, VisitEntity};
use crate::error::*;
use crate::models::{
    default_pipebase_dependency, default_tokio_dependency, default_tracing_dependency, Block,
    DataType, FunctionBuilder, Rhs, Statement,
};
use crate::ops::AppValidator;
use crate::ops::{AppDescriber, AppGenerator};
use crate::ops::{Describe, Generate, Validate};
use serde::Deserialize;
use std::collections::HashSet;

#[derive(Deserialize, Debug, Clone)]
pub struct App {
    name: String,
    metas: Option<Vec<Meta>>,
    dependencies: Option<Vec<Dependency>>,
    cstores: Option<Vec<ContextStore>>,
    error: Option<ErrorHandler>,
    pipes: Vec<Pipe>,
    objects: Option<Vec<Object>>,
}

impl Entity for App {
    fn get_id(&self) -> String {
        self.name.to_owned()
    }

    fn list_dependency(&self) -> Vec<String> {
        let dependencies = self.get_dependencies();
        dependencies
            .iter()
            .flat_map(|dep| dep.get_modules().to_owned())
            .collect()
    }

    fn to_literal(&self, indent: usize) -> String {
        // create app object
        let app = Object::new(APP_OBJECT_NAME.to_owned(), vec![], vec![]);
        app.to_literal(indent)
    }
}

impl<V> EntityAccept<V> for App where V: VisitEntity<Self> {}

impl App {
    pub fn from_path<P>(manifest_path: P) -> Result<App>
    where
        P: AsRef<std::path::Path>,
    {
        let file = match std::fs::File::open(manifest_path) {
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

    pub fn from_buffer(manifest_buffer: &[u8]) -> Result<App> {
        let mut app = match serde_yaml::from_slice::<Self>(manifest_buffer) {
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
        for dependency in self.visit_dependencies() {
            if !self.has_dependency(&dependency) {
                self.add_dependency(dependency)
            }
        }
        // init metas
        match self.metas {
            Some(_) => (),
            None => self.metas = Some(Self::default_metas()),
        };
        // init pipes
        for pipe in self.pipes.as_mut_slice() {
            pipe.init();
        }
        // init objects
        match self.objects {
            Some(_) => (),
            None => self.objects = Some(vec![]),
        }
        for object in self.objects.as_mut().unwrap() {
            object.init();
        }
        // init context stores
        match self.cstores {
            Some(_) => (),
            None => self.cstores = Some(vec![]),
        }
    }

    fn has_dependency(&self, other: &Dependency) -> bool {
        let dependencies = self.dependencies.as_ref().unwrap();
        for dependency in dependencies {
            if dependency.eq(other) {
                return true;
            }
        }
        false
    }

    fn add_dependency(&mut self, dependency: Dependency) {
        let dependencies = self.dependencies.as_mut().unwrap();
        dependencies.push(dependency);
    }

    pub fn get_dependencies(&self) -> &Vec<Dependency> {
        self.dependencies.as_ref().unwrap()
    }

    // visit config crate dependency
    fn visit_dependencies(&self) -> Vec<Dependency> {
        let mut visitor = CrateVisitor::new();
        for pipe in &self.pipes {
            pipe.accept_crate_visitor(&mut visitor)
        }
        if let Some(cstores) = self.cstores.as_ref() {
            for cstore in cstores {
                cstore.accept_crate_visitor(&mut visitor)
            }
        }
        if let Some(error_handler) = self.error.as_ref() {
            error_handler.accept_crate_visitor(&mut visitor)
        }
        let mut all_dependencies: Vec<Dependency> = visitor.into_iter().collect();
        all_dependencies.extend(Self::default_dependencies());
        all_dependencies
    }

    fn default_dependencies() -> Vec<Dependency> {
        vec![
            default_pipebase_dependency(),
            default_tokio_dependency(),
            default_tracing_dependency(),
        ]
    }

    fn default_metas() -> Vec<Meta> {
        vec![Meta::List {
            name: "derive".to_owned(),
            metas: vec![
                Meta::Path {
                    name: "Bootstrap".to_owned(),
                },
                Meta::Path {
                    name: "Default".to_owned(),
                },
            ],
        }]
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
        let function = FunctionBuilder::new()
            .name(BOOTSTRAP_FUNCTION_NAME.to_owned())
            .meta(meta)
            .public(true)
            .asynchronous(true)
            .block(block)
            .rtype(rtype)
            .build();
        function.to_literal(indent)
    }

    pub(crate) fn get_main_function_literal(&self, indent: usize) -> String {
        let metas = vec![
            Meta::List {
                name: PIPEBASE_MAIN.to_owned(),
                metas: vec![Meta::Value {
                    name: BOOTSTRAP_MODULE_META_PATH.to_owned(),
                    meta: MetaValue::Str {
                        value: self.get_app_module_name(),
                        raw: false,
                    },
                }],
            },
            Meta::Path {
                name: TRACING_INSTRUMENT.to_owned(),
            },
        ];
        let block = Block::new(vec![]);
        let function = FunctionBuilder::new()
            .name("main".to_owned())
            .metas(metas)
            .public(false)
            .asynchronous(true)
            .block(block)
            .build();
        function.to_literal(indent)
    }

    pub(crate) fn get_app_metas_lit(&self, indent: usize) -> String {
        let metas = self.metas.as_ref().expect("app metas not inited");
        metas_to_literal(metas, indent)
    }

    pub(crate) fn get_context_stores(&self) -> &Vec<ContextStore> {
        self.cstores.as_ref().expect("stores")
    }

    pub(crate) fn get_error_handler(&self) -> Option<&ErrorHandler> {
        self.error.as_ref()
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
        self.accept_entity_visitor(&mut app_generator);
        app_generator.generate()
    }

    // generate pipelines contains pid
    pub fn generate_pipes(&self, pid: &str) -> Result<String> {
        let mut describer = AppDescriber::new();
        self.accept_entity_visitor(&mut describer);
        // filter pipes with selected component - partial generation
        let component = describer.get_pipe_component(pid)?;
        let selected_pipes: HashSet<String> = component.into_iter().collect();
        let mut app_generator = AppGenerator::new(0);
        self.accept_entity_visitor(&mut app_generator);
        app_generator.set_pipe_filter(selected_pipes);
        Ok(app_generator.generate())
    }

    pub fn validate_pipes(&self) -> Result<()> {
        let mut validator = AppValidator::new("");
        self.accept_entity_visitor(&mut validator);
        validator.validate_pipes()
    }

    pub fn validate_objects(&self) -> Result<()> {
        let mut validator = AppValidator::new("");
        self.accept_entity_visitor(&mut validator);
        validator.validate_objects()
    }

    pub fn validate_cstores(&self) -> Result<()> {
        let mut validator = AppValidator::new("");
        self.accept_entity_visitor(&mut validator);
        validator.validate_cstores()
    }

    pub fn validate(&self) -> Result<()> {
        let mut validator = AppValidator::new("");
        self.accept_entity_visitor(&mut validator);
        validator.validate()
    }

    pub fn describe_pipes(&self) -> Vec<String> {
        let mut describer = AppDescriber::new();
        self.accept_entity_visitor(&mut describer);
        describer.describe_pipes()
    }

    pub fn describe_pipe(&self, pid: &str) -> Result<String> {
        let mut describer = AppDescriber::new();
        self.accept_entity_visitor(&mut describer);
        describer.describe_pipe(pid)
    }

    pub fn describe_pipe_graph(&self) -> Vec<String> {
        let mut describer = AppDescriber::new();
        self.accept_entity_visitor(&mut describer);
        describer.describe_pipe_graph()
    }

    pub fn describe_pipelines(&self, pid: &str) -> Result<Vec<String>> {
        let mut describer = AppDescriber::new();
        self.accept_entity_visitor(&mut describer);
        describer.describe_pipelines(pid)
    }

    pub fn describe_objects(&self) -> Vec<String> {
        let mut describer = AppDescriber::new();
        self.accept_entity_visitor(&mut describer);
        describer.describe_objects()
    }

    pub fn describe_object(&self, oid: &str) -> Result<String> {
        let mut describer = AppDescriber::new();
        self.accept_entity_visitor(&mut describer);
        describer.describe_object(oid)
    }

    pub fn describe(&self) -> Vec<String> {
        let mut describer = AppDescriber::new();
        self.accept_entity_visitor(&mut describer);
        describer.describe()
    }
}
