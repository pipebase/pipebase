use crate::print::Printer;
use pipegen::api::App;
use pipegen::api::PackageDependency;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

pub(crate) fn parse_pipe_manifest(
    manifest_path: &Path,
    printer: &mut Printer,
) -> anyhow::Result<App> {
    printer.status(&"Parse", manifest_path.to_str().unwrap())?;
    let app = match App::parse(manifest_path) {
        Ok(app) => app,
        Err(err) => {
            printer.error(err.to_string())?;
            return Err(err.into());
        }
    };
    printer.status(&"Parse", "succeed")?;
    Ok(app)
}

#[derive(Deserialize, Serialize, Debug)]
pub struct PipeTomlProject {
    name: String,
    version: String,
    authors: Option<Vec<String>>,
    edition: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct PipeTomlDependency {
    version: Option<String>,
    path: Option<String>,
    features: Option<Vec<String>>,
}

impl From<PackageDependency> for PipeTomlDependency {
    fn from(pd: PackageDependency) -> Self {
        PipeTomlDependency {
            version: pd.get_version(),
            path: pd.get_path(),
            features: pd.get_features(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PipeTomlWorkspace {}

impl PipeTomlWorkspace {
    pub fn new() -> Self {
        PipeTomlWorkspace {}
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub struct PipeTomlManifest {
    package: Option<PipeTomlProject>,
    dependencies: Option<HashMap<String, PipeTomlDependency>>,
    workspace: Option<PipeTomlWorkspace>,
}

impl PipeTomlManifest {
    pub fn init(&mut self) {
        self.dependencies = Some(HashMap::new());
        self.workspace = Some(PipeTomlWorkspace::new());
    }

    pub fn add_dependency(&mut self, package: String, dependency: PipeTomlDependency) {
        let dependencies = self.dependencies.as_mut().unwrap();
        dependencies.insert(package, dependency);
    }
}
