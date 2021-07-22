use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct PackageDependency {
    package: String,
    version: Option<String>,
    path: Option<String>,
    git: Option<String>,
    branch: Option<String>,
    tag: Option<String>,
    features: Option<Vec<String>>,
    // module path used in app
    modules: Vec<String>,
}

impl PartialEq for PackageDependency {
    fn eq(&self, other: &Self) -> bool {
        self.package.eq(&other.get_package())
    }
}

impl Eq for PackageDependency {}

impl PackageDependency {
    pub(crate) fn new(
        package: String,
        version: Option<String>,
        path: Option<String>,
        git: Option<String>,
        branch: Option<String>,
        tag: Option<String>,
        features: Option<Vec<String>>,
        modules: Vec<String>,
    ) -> Self {
        PackageDependency {
            package,
            version,
            path,
            git,
            branch,
            tag,
            features,
            modules,
        }
    }

    pub fn get_package(&self) -> String {
        self.package.to_owned()
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

    pub(crate) fn get_modules(&self) -> &Vec<String> {
        &self.modules
    }
}
