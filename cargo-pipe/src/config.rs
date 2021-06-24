use std::{env::current_dir, path::PathBuf};

const DEFAULT_PIPE_MANIFEST: &'static str = "pipe.yml";
pub struct Config {
    working_drectory: PathBuf,
}

impl Config {
    pub fn new(directory: Option<&str>) -> anyhow::Result<Self> {
        let config = match directory {
            Some(directory) => Config {
                working_drectory: PathBuf::from(directory),
            },
            None => Config {
                working_drectory: current_dir()?,
            },
        };
        Ok(config)
    }

    pub fn get_pipe_manifest_path(&self) -> PathBuf {
        let mut manifest_path = self.working_drectory.to_owned();
        manifest_path.push(DEFAULT_PIPE_MANIFEST);
        manifest_path
    }
}
