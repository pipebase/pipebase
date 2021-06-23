use std::{env::current_dir, path::PathBuf};

const DEFAULT_MANIFEST_FILE: &'static str = "pipe.yml";
pub struct Config {
    manifest_path: PathBuf,
}

impl Config {
    pub fn new(manifest_path: Option<&str>) -> anyhow::Result<Self> {
        let config = match manifest_path {
            Some(path) => Config {
                manifest_path: PathBuf::from(path),
            },
            None => Config {
                manifest_path: {
                    let mut path = current_dir()?;
                    path.push(DEFAULT_MANIFEST_FILE);
                    path
                },
            },
        };
        Ok(config)
    }
}
