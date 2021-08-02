use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct PackageDependency {
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

impl PartialEq for PackageDependency {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.get_name())
    }
}

impl Eq for PackageDependency {}

impl PackageDependency {
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

pub(crate) fn default_tokio_package() -> PackageDependency {
    PackageDependency {
        name: "tokio".to_owned(),
        version: Some("1.6.1".to_owned()),
        path: None,
        git: None,
        branch: None,
        tag: None,
        features: Some(vec!["full".to_owned()]),
        package: None,
        modules: vec![],
    }
}

pub(crate) fn default_pipebase_package() -> PackageDependency {
    PackageDependency {
        name: "pipebase".to_owned(),
        version: Some("0.1.0".to_owned()),
        path: None,
        git: None,
        branch: None,
        tag: None,
        features: None,
        package: None,
        modules: vec!["pipebase::prelude::*".to_owned()],
    }
}

pub(crate) fn default_log_package() -> PackageDependency {
    PackageDependency {
        name: "log".to_owned(),
        version: Some("0.4.14".to_owned()),
        path: None,
        git: None,
        branch: None,
        tag: None,
        features: None,
        package: None,
        modules: vec![],
    }
}

pub(crate) fn default_env_log_package() -> PackageDependency {
    PackageDependency {
        name: "env_logger".to_owned(),
        version: Some("0.8.4".to_owned()),
        path: None,
        git: None,
        branch: None,
        tag: None,
        features: None,
        package: None,
        modules: vec![],
    }
}
