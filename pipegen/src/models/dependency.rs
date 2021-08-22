use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Dependency {
    name: String,
    version: Option<String>,
    path: Option<String>,
    git: Option<String>,
    branch: Option<String>,
    tag: Option<String>,
    features: Option<Vec<String>>,
    package: Option<String>,
    // module path used in app
    modules: Vec<String>,
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.get_name())
    }
}

impl Eq for Dependency {}

impl Dependency {
    pub fn get_name(&self) -> String {
        self.name.to_owned()
    }

    pub fn get_version(&self) -> Option<String> {
        self.version.to_owned()
    }

    pub fn get_path(&self) -> Option<String> {
        self.path.to_owned()
    }

    pub fn get_git(&self) -> Option<String> {
        self.git.to_owned()
    }

    pub fn get_branch(&self) -> Option<String> {
        self.branch.to_owned()
    }

    pub fn get_tag(&self) -> Option<String> {
        self.tag.to_owned()
    }

    pub fn get_features(&self) -> Option<Vec<String>> {
        self.features.to_owned()
    }

    pub fn get_package(&self) -> Option<String> {
        self.package.to_owned()
    }

    pub(crate) fn get_modules(&self) -> &Vec<String> {
        &self.modules
    }
}

#[derive(Default)]
pub struct DependencyBuilder {
    name: Option<String>,
    version: Option<String>,
    path: Option<String>,
    git: Option<String>,
    branch: Option<String>,
    tag: Option<String>,
    features: Option<Vec<String>>,
    package: Option<String>,
    // module path used in app
    modules: Vec<String>,
}

impl DependencyBuilder {
    pub fn new() -> Self {
        DependencyBuilder::default()
    }

    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn path(mut self, path: String) -> Self {
        self.path = Some(path);
        self
    }

    pub fn git(mut self, git: String) -> Self {
        self.git = Some(git);
        self
    }

    pub fn branch(mut self, branch: String) -> Self {
        self.branch = Some(branch);
        self
    }

    pub fn tag(mut self, tag: String) -> Self {
        self.tag = Some(tag);
        self
    }

    pub fn features(mut self, features: Vec<String>) -> Self {
        self.features = Some(features);
        self
    }

    pub fn package(mut self, package: String) -> Self {
        self.package = Some(package);
        self
    }

    pub fn modules(mut self, modules: Vec<String>) -> Self {
        self.modules = modules;
        self
    }

    pub fn build(self) -> Dependency {
        Dependency {
            name: self.name.expect("dependency name not inited"),
            version: self.version,
            path: self.path,
            git: self.git,
            branch: self.branch,
            tag: self.tag,
            features: self.features,
            package: self.package,
            modules: self.modules,
        }
    }
}

pub(crate) fn default_tokio_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("tokio".to_owned())
        .version("1.6.1".to_owned())
        .features(vec!["full".to_owned()])
        .build()
}

pub(crate) fn default_pipebase_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipebase".to_owned())
        .version("0.1.4".to_owned())
        .modules(vec!["pipebase::prelude::*".to_owned()])
        .build()
}

pub(crate) fn default_log_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("log".to_owned())
        .version("0.4.14".to_owned())
        .build()
}

pub(crate) fn default_env_log_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("env_logger".to_owned())
        .version("0.8.4".to_owned())
        .build()
}
