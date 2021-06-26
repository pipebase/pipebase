use super::context::ContextStore;
use super::dependency::Dependency;
use super::meta::Meta;
use super::pipe::Pipe;
use super::utils::indent_literal;
use super::{Entity, EntityAccept, Object, VisitEntity};
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
        let cstore = match self.get_context_store() {
            Some(cstore) => cstore.as_data_field(),
            None => panic!("App's context store is None"),
        };
        // create app object
        let app = Object::new("App".to_owned(), metas, vec![cstore]);
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
        Self::init_app(&mut app);
        Ok(app)
    }

    fn init_app(app: &mut App) {
        // init app dependencies
        for default_dependency in Self::default_dependencies() {
            if !app.has_dependency(&default_dependency) {
                app.add_dependency(default_dependency)
            }
        }
        // init context store
        match app.get_context_store() {
            Some(_) => (),
            None => {
                let default_cstore = ContextStore::new("pipe_contexts".to_owned());
                app.set_context_store(default_cstore);
            }
        };
    }

    pub fn has_dependency(&self, other: &Dependency) -> bool {
        let dependencies = match self.dependencies {
            Some(ref dependencies) => dependencies,
            None => return false,
        };
        for dependency in dependencies {
            if dependency.eq(other) {
                return true;
            }
        }
        return false;
    }

    pub fn add_dependency(&mut self, dependency: Dependency) {
        match self.dependencies {
            Some(ref mut dependencies) => dependencies.push(dependency),
            None => self.dependencies = Some(vec![dependency]),
        }
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

    pub fn get_use_modules_lit(&self, indent: usize) -> String {
        let indent_lit = indent_literal(indent);
        let mut use_module_lits: Vec<String> = Vec::new();
        for module_lit in self.list_dependency() {
            use_module_lits.push(format!("{}use {}", indent_lit, module_lit));
        }
        use_module_lits.push("".to_owned());
        use_module_lits.join(";\n")
    }

    // TODO: function lit replaced with function entity or procedure derive
    pub fn get_new_app_function_lit(&self, indent: usize) -> String {
        let cstore_name = match self.cstore {
            Some(ref cstore) => cstore.get_name(),
            None => panic!("App's App's context store is None"),
        };
        let new_app_lit = format!(
            "{}App {{\n{}{}: std::collections::HashMap::new(),\n{}}}",
            indent_literal(indent + 1),
            indent_literal(indent + 2),
            cstore_name,
            indent_literal(indent + 1)
        );
        format!(
            "{}pub fn app() -> App {{\n{}\n{}}}",
            indent_literal(indent),
            new_app_lit,
            indent_literal(indent)
        )
    }

    pub fn get_bootstrap_app_function_lit(&self, indent: usize) -> String {
        let new_app_and_bootstap_lit = format!(
            "{}self::app().bootstrap().await;",
            indent_literal(indent + 1)
        );
        format!(
            "{}pub async fn bootstrap() {{\n{}\n{}}}",
            indent_literal(indent),
            new_app_and_bootstap_lit,
            indent_literal(indent)
        )
    }

    pub fn get_main_function_lit(&self, indent: usize) -> String {
        let tokio_main_lit = format!("{}#[tokio::main]", indent_literal(indent));
        let bootstrap_app_lit = format!(
            "{}{}::bootstrap().await;",
            indent_literal(indent + 1),
            self.name
        );
        format!(
            "{}\n{}async fn main() {{\n{}\n{}}}",
            tokio_main_lit,
            indent_literal(indent),
            bootstrap_app_lit,
            indent_literal(indent)
        )
    }

    pub fn get_metas(&self) -> Vec<Meta> {
        match self.metas {
            Some(ref metas) => metas.to_owned(),
            None => vec![Meta::List {
                name: "derive".to_owned(),
                metas: vec![
                    Meta::Path {
                        name: "Bootstrap".to_owned(),
                    },
                    Meta::Path {
                        name: "ContextStore".to_owned(),
                    },
                ],
            }],
        }
    }

    pub fn get_context_store(&self) -> Option<&ContextStore> {
        self.cstore.as_ref()
    }

    pub fn set_context_store(&mut self, cstore: ContextStore) {
        self.cstore = Some(cstore)
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
