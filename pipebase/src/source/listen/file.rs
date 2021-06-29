use crate::Listen;
use async_trait::async_trait;
use serde::Deserialize;
use std::fs::{self, DirEntry};
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;

use crate::{ConfigInto, FromConfig, FromPath};

#[async_trait]
pub trait ListFile {
    // list file in directory
    async fn list(&self) -> io::Result<Vec<PathBuf>>;
    async fn filter(&self, _entry: &DirEntry) -> bool {
        true
    }
}

#[derive(Clone, Deserialize)]
pub struct LocalFileVisitorConfig {
    // root directory path
    pub root: String,
}

impl FromPath for LocalFileVisitorConfig {}

#[async_trait]
impl ConfigInto<LocalFileVisitor> for LocalFileVisitorConfig {}

pub struct LocalFileVisitor {
    // root directory path
    pub root: PathBuf,
    tx: Option<Arc<Sender<PathBuf>>>,
}

impl LocalFileVisitor {
    pub fn new(config: &LocalFileVisitorConfig) -> Self {
        LocalFileVisitor {
            root: PathBuf::from(&config.root),
            tx: None,
        }
    }
}

#[async_trait]
impl FromConfig<LocalFileVisitorConfig> for LocalFileVisitor {
    async fn from_config(config: &LocalFileVisitorConfig) -> anyhow::Result<Self> {
        Ok(LocalFileVisitor::new(config))
    }
}

#[async_trait]
impl ListFile for LocalFileVisitor {
    async fn list(&self) -> io::Result<Vec<PathBuf>> {
        let dir = match self.root.is_dir() {
            true => self.root.to_owned(),
            false => return Ok(vec![]),
        };
        let mut dirs = vec![dir];
        let mut file_paths: Vec<PathBuf> = Vec::new();
        loop {
            let dir = match dirs.pop() {
                Some(dir) => dir,
                None => return Ok(file_paths),
            };
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                let include = match path.is_dir() {
                    true => {
                        dirs.push(path);
                        continue;
                    }
                    false => self.filter(&entry).await,
                };
                match include {
                    true => file_paths.push(path),
                    _ => (),
                };
            }
        }
    }
}

#[async_trait]
impl Listen<PathBuf, LocalFileVisitorConfig> for LocalFileVisitor {
    async fn run(&mut self) -> anyhow::Result<()> {
        // list once
        for path in self.list().await? {
            self.tx.as_ref().unwrap().send(path).await?;
        }
        Ok(())
    }

    async fn set_sender(&mut self, sender: std::sync::Arc<tokio::sync::mpsc::Sender<PathBuf>>) {
        self.tx = Some(sender)
    }
}

#[cfg(test)]
mod tests {
    use crate::*;
    use std::collections::HashSet;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_list_folder() {
        let (tx, mut rx) = channel!(PathBuf, 1024);
        let mut pipe = listener!(
            "file_visitor",
            "resources/catalogs/local_file_visitor.yml",
            LocalFileVisitorConfig,
            [tx]
        );
        spawn_join!(pipe);
        let mut all_expected_files: HashSet<PathBuf> = HashSet::new();
        all_expected_files.insert(PathBuf::from(
            "resources/test_file_folder/sub_folder/test_file_0.txt",
        ));
        all_expected_files.insert(PathBuf::from("resources/test_file_folder/test_file_0.txt"));
        all_expected_files.insert(PathBuf::from("resources/test_file_folder/test_file_1.txt"));
        let mut actual_files_total: usize = 0;
        loop {
            let file_path = match rx.recv().await {
                Some(file_path) => file_path,
                None => break,
            };
            actual_files_total += 1;
            assert!(all_expected_files.contains(&file_path))
        }
        assert_eq!(all_expected_files.len(), actual_files_total)
    }
}
