use crate::{
    errors::CmdResult,
    ops::do_cargo::{
        CARGO_APP_MAIN, CARGO_DEBUG_DIRECTORY, CARGO_MANIFEST_FILE, CARGO_RELEASE_DIRECTORY,
        CARGO_SRC_DIRECTORY, CARGO_TARGET_DIRECTORY,
    },
};
use std::{env::current_dir, path::PathBuf};

const DEFAULT_PIPE_MANIFEST: &str = "pipe.yml";
const DEFAULT_APP_DIRECTORY: &str = "app";

pub struct Config {
    working_drectory: PathBuf,
    manifest: String,
}

impl Config {
    pub fn new(directory: Option<&str>, manifest: Option<&str>) -> CmdResult<Self> {
        let working_drectory = match directory {
            Some(directory) => PathBuf::from(directory),
            None => current_dir()?,
        };
        let manifest = String::from(manifest.unwrap_or(DEFAULT_PIPE_MANIFEST));
        Ok(Config {
            working_drectory,
            manifest,
        })
    }

    pub fn get_pipe_manifest_path(&self) -> PathBuf {
        let mut manifest_path = self.working_drectory.to_owned();
        manifest_path.push(&self.manifest);
        manifest_path
    }

    pub fn get_app_directory(&self, app_name: Option<&String>) -> PathBuf {
        let mut app_directory = self.working_drectory.to_owned();
        let app_name = &Self::get_app_name_or_default(app_name);
        app_directory.push(app_name);
        app_directory
    }

    pub fn get_app_manifest(&self, app_name: Option<&String>) -> PathBuf {
        let mut manifest = self.working_drectory.to_owned();
        let app_name = &Self::get_app_name_or_default(app_name);
        manifest.push(app_name);
        manifest.push(CARGO_MANIFEST_FILE);
        manifest
    }

    pub fn get_app_main_path(&self, app_name: Option<&String>) -> PathBuf {
        let mut main_path = self.working_drectory.to_owned();
        let app_name = &Self::get_app_name_or_default(app_name);
        main_path.push(app_name);
        main_path.push(CARGO_SRC_DIRECTORY);
        main_path.push(CARGO_APP_MAIN);
        main_path
    }

    pub fn get_target_release_app_binary(&self, app_name: Option<&String>) -> PathBuf {
        let mut app_binary = self.working_drectory.to_owned();
        let app_name = &Self::get_app_name_or_default(app_name);
        app_binary.push(app_name);
        app_binary.push(CARGO_TARGET_DIRECTORY);
        app_binary.push(CARGO_RELEASE_DIRECTORY);
        app_binary.push(app_name);
        app_binary
    }

    pub fn get_target_debug_app_binary(&self, app_name: Option<&String>) -> PathBuf {
        let mut app_binary = self.working_drectory.to_owned();
        let app_name = &Self::get_app_name_or_default(app_name);
        app_binary.push(app_name);
        app_binary.push(CARGO_TARGET_DIRECTORY);
        app_binary.push(CARGO_DEBUG_DIRECTORY);
        app_binary.push(app_name);
        app_binary
    }

    pub fn get_app_name_or_default(app_name: Option<&String>) -> String {
        match app_name {
            Some(app_name) => app_name.to_owned(),
            None => DEFAULT_APP_DIRECTORY.to_owned(),
        }
    }

    pub fn get_run_app_binary(&self, app_name: Option<&String>) -> PathBuf {
        let mut app_binary = self.working_drectory.to_owned();
        let app_name = &Self::get_app_name_or_default(app_name);
        app_binary.push(format!("run_{}", app_name));
        app_binary
    }
}
