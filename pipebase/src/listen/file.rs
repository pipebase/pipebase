use async_trait::async_trait;
use serde::Deserialize;
use std::fs::{self, DirEntry};
use std::io;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::time::sleep;

use super::Listen;
use crate::common::{ConfigInto, FromConfig, FromPath, Period};

#[async_trait]
pub trait ListFile {
    /// list file in directory
    async fn list(&self) -> io::Result<Vec<PathBuf>>;
    async fn filter(&self, _entry: &DirEntry) -> bool {
        true
    }
}

#[derive(Clone, Deserialize)]
pub enum FilePathVisitMode {
    Once,
    Cron(Period),
}

#[derive(Clone, Deserialize)]
pub struct LocalFilePathVisitorConfig {
    pub root: String,
    pub mode: Option<FilePathVisitMode>,
}

impl FromPath for LocalFilePathVisitorConfig {}

#[async_trait]
impl ConfigInto<LocalFilePathVisitor> for LocalFilePathVisitorConfig {}

/// Visit file under directory and send file path
pub struct LocalFilePathVisitor {
    /// Root directory path
    root: PathBuf,
    /// Either Once ot Cron
    mode: FilePathVisitMode,
    /// Sender to notify downstreams
    tx: Option<Sender<PathBuf>>,
}

impl LocalFilePathVisitor {
    pub fn new(config: LocalFilePathVisitorConfig) -> Self {
        let mode = match config.mode {
            Some(mode) => mode,
            None => FilePathVisitMode::Once,
        };
        LocalFilePathVisitor {
            root: PathBuf::from(config.root),
            mode,
            tx: None,
        }
    }
}

#[async_trait]
impl FromConfig<LocalFilePathVisitorConfig> for LocalFilePathVisitor {
    async fn from_config(config: LocalFilePathVisitorConfig) -> anyhow::Result<Self> {
        Ok(LocalFilePathVisitor::new(config))
    }
}

#[async_trait]
impl ListFile for LocalFilePathVisitor {
    /// Recursive list file under directory
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
                if include {
                    file_paths.push(path)
                }
            }
        }
    }
}

impl LocalFilePathVisitor {
    async fn run_once(&mut self) -> anyhow::Result<()> {
        for path in self.list().await? {
            self.tx.as_ref().unwrap().send(path).await?;
        }
        Ok(())
    }

    async fn run_cron(&mut self, delay: Duration) -> anyhow::Result<()> {
        loop {
            self.run_once().await?;
            sleep(delay).await;
        }
    }
}

/// # Parameters
/// * PathBuf: output
#[async_trait]
impl Listen<PathBuf, LocalFilePathVisitorConfig> for LocalFilePathVisitor {
    async fn run(&mut self) -> anyhow::Result<()> {
        let period = match self.mode {
            FilePathVisitMode::Once => return self.run_once().await,
            FilePathVisitMode::Cron(ref period) => period.to_owned(),
        };
        self.run_cron(period.into()).await
    }

    fn set_sender(&mut self, sender: Sender<PathBuf>) {
        self.tx = Some(sender)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use std::collections::HashSet;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_list_folder() {
        let (tx, mut rx) = channel!(PathBuf, 1024);
        let mut pipe = listener!("file_visitor");
        join_pipes!([run_pipe!(
            pipe,
            LocalFilePathVisitorConfig,
            "resources/catalogs/local_file_visitor.yml",
            [tx]
        )]);
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
