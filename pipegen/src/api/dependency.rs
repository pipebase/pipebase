use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct Dependency {
    package: String,
    version: Option<String>,
    path: Option<String>,
    features: Option<Vec<String>>,
    // module path used in app
    modules: Vec<String>,
}

impl PartialEq for Dependency {
    fn eq(&self, other: &Self) -> bool {
        self.package.eq(other.get_package())
    }
}

impl Eq for Dependency {}

impl Dependency {
    pub(crate) fn new(
        package: String,
        version: Option<String>,
        path: Option<String>,
        features: Option<Vec<String>>,
        modules: Vec<String>,
    ) -> Self {
        Dependency {
            package: package,
            version: version,
            path: path,
            features: features,
            modules: modules,
        }
    }

    pub(crate) fn get_package(&self) -> &String {
        &self.package
    }

    pub(crate) fn get_version(&self) -> Option<&String> {
        self.version.as_ref()
    }

    pub(crate) fn get_path(&self) -> Option<&String> {
        self.path.as_ref()
    }

    pub(crate) fn get_modules(&self) -> &Vec<String> {
        &self.modules
    }
}
