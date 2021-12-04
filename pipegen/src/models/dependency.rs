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
        .version("0.1.10".to_owned())
        .modules(vec!["pipebase::prelude::*".to_owned()])
        .build()
}

pub(crate) fn default_tracing_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("tracing".to_owned())
        .version("0.1".to_owned())
        .build()
}

pub(crate) fn default_avro_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipeavro".to_owned())
        .version("0.1.3".to_owned())
        .modules(vec!["pipeavro::*".to_owned()])
        .build()
}

pub(crate) fn default_cql_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipecql".to_owned())
        .version("0.2.1".to_owned())
        .modules(vec!["pipecql::*".to_owned()])
        .build()
}

pub(crate) fn default_csv_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipecsv".to_owned())
        .version("0.1.4".to_owned())
        .modules(vec!["pipecsv::*".to_owned()])
        .build()
}

pub(crate) fn default_json_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipejson".to_owned())
        .version("0.1.5".to_owned())
        .modules(vec!["pipejson::*".to_owned()])
        .build()
}

pub(crate) fn default_kafka_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipekafka".to_owned())
        .version("0.2.2".to_owned())
        .modules(vec!["pipekafka::*".to_owned()])
        .build()
}

pub(crate) fn default_kube_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipekube".to_owned())
        .version("0.1.4".to_owned())
        .modules(vec!["pipekube::*".to_owned()])
        .build()
}

pub(crate) fn default_mysql_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipemysql".to_owned())
        .version("0.2.1".to_owned())
        .modules(vec!["pipemysql::*".to_owned()])
        .build()
}

pub(crate) fn default_psql_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipepsql".to_owned())
        .version("0.2.1".to_owned())
        .modules(vec!["pipepsql::*".to_owned()])
        .build()
}

pub(crate) fn default_redis_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("piperedis".to_owned())
        .version("0.1.5".to_owned())
        .modules(vec!["piperedis::*".to_owned()])
        .build()
}

pub(crate) fn default_reqwest_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipereqwest".to_owned())
        .version("0.1.4".to_owned())
        .modules(vec!["pipereqwest::*".to_owned()])
        .build()
}

pub(crate) fn default_rocksdb_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("piperocksdb".to_owned())
        .version("0.1.4".to_owned())
        .modules(vec!["piperocksdb::*".to_owned()])
        .build()
}

pub(crate) fn default_warp_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipewarp".to_owned())
        .version("0.1.4".to_owned())
        .modules(vec!["pipewarp::*".to_owned()])
        .build()
}

pub(crate) fn default_dynamodb_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipedynamodb".to_owned())
        .git("https://github.com/pipebase/pipebase.git".to_owned())
        .package("pipedynamodb".to_owned())
        .modules(vec!["pipedynamodb::*".to_owned()])
        .build()
}

pub(crate) fn default_s3_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipes3".to_owned())
        .git("https://github.com/pipebase/pipebase.git".to_owned())
        .package("pipes3".to_owned())
        .modules(vec!["pipes3::*".to_owned()])
        .build()
}

pub(crate) fn default_sns_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipesns".to_owned())
        .git("https://github.com/pipebase/pipebase.git".to_owned())
        .package("pipesns".to_owned())
        .modules(vec!["pipesns::*".to_owned()])
        .build()
}

pub(crate) fn default_sqs_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipesqs".to_owned())
        .git("https://github.com/pipebase/pipebase.git".to_owned())
        .package("pipesqs".to_owned())
        .modules(vec!["pipesqs::*".to_owned()])
        .build()
}

pub(crate) fn default_mqtt_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipemqtt".to_owned())
        .version("0.1.4".to_owned())
        .modules(vec!["pipemqtt::*".to_owned()])
        .build()
}

pub(crate) fn default_amqp_dependency() -> Dependency {
    DependencyBuilder::new()
        .name("pipeamqp".to_owned())
        .version("0.1.2".to_owned())
        .modules(vec!["pipeamqp::*".to_owned()])
        .build()
}

pub(crate) trait UseCrate: Sized {
    fn accept_crate_visitor(&self, visitor: &mut CrateVisitor) {
        visitor.visit(self)
    }

    fn get_crate(&self) -> Option<Dependency>;
}

pub(crate) struct CrateVisitor {
    dependencies: Vec<Dependency>,
}

impl CrateVisitor {
    pub(crate) fn new() -> Self {
        CrateVisitor {
            dependencies: vec![],
        }
    }

    fn visit<T>(&mut self, entity: &T)
    where
        T: UseCrate,
    {
        if let Some(dependency) = entity.get_crate() {
            self.dependencies.push(dependency)
        }
    }
}

impl IntoIterator for CrateVisitor {
    type Item = Dependency;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.dependencies.into_iter()
    }
}
