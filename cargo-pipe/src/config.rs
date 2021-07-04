use std::{env::current_dir, path::PathBuf};

const DEFAULT_PIPE_MANIFEST: &'static str = "pipe.yml";
const DEFAULT_APP_DIRECTORY: &'static str = "app";
const SRC_DIRECTORY: &'static str = "src";
const APP_MAIN: &'static str = "main.rs";

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

    pub fn get_app_directory(&self, app_name: Option<&String>) -> PathBuf {
        let mut app_directory = self.working_drectory.to_owned();
        let app_name = match app_name {
            Some(app_name) => app_name.as_str(),
            None => DEFAULT_APP_DIRECTORY,
        };
        app_directory.push(app_name);
        app_directory
    }

    pub fn get_app_main_path(&self, app_name: Option<&String>) -> PathBuf {
        let mut main_path = self.working_drectory.to_owned();
        let app_name = match app_name {
            Some(app_name) => app_name.as_str(),
            None => DEFAULT_APP_DIRECTORY,
        };
        main_path.push(app_name);
        main_path.push(SRC_DIRECTORY);
        main_path.push(APP_MAIN);
        main_path
    }
}
