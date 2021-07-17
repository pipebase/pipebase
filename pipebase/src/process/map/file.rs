use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use crate::error::Result;
use crate::{ConfigInto, FromConfig, FromPath, Map};
use async_trait::async_trait;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::fs;

const DEFAULT_FILENAME_LENGTH: usize = 16;

#[derive(Deserialize)]
pub struct FileWriterConfig {
    directory: String,
    filename_length: Option<usize>,
}

impl FromPath for FileWriterConfig {}

impl ConfigInto<FileWriter> for FileWriterConfig {}

/// Create and Write files under directory
pub struct FileWriter {
    directory: PathBuf,
    /// Random file name length
    filename_length: usize,
}

#[async_trait]
impl FromConfig<FileWriterConfig> for FileWriter {
    async fn from_config(config: &FileWriterConfig) -> anyhow::Result<Self> {
        Ok(FileWriter {
            directory: PathBuf::from(&config.directory),
            filename_length: config.filename_length.unwrap_or(DEFAULT_FILENAME_LENGTH),
        })
    }
}

#[async_trait]
impl Map<Vec<u8>, PathBuf, FileWriterConfig> for FileWriter {
    /// Input: Vec<u8>, bytes
    /// Output: PathBuf, file path
    async fn map(&mut self, data: Vec<u8>) -> anyhow::Result<PathBuf> {
        let path = self.write_all(data)?;
        Ok(path)
    }
}

impl FileWriter {
    fn write_all(&self, data: Vec<u8>) -> Result<PathBuf> {
        let mut path = self.directory.to_owned();
        let filename: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(self.filename_length)
            .map(char::from)
            .collect();
        path.push(filename);
        let mut wrt = BufWriter::new(fs::File::create(path.as_path())?);
        wrt.write_all(data.as_slice())?;
        wrt.flush()?;
        Ok(path)
    }
}

#[derive(Deserialize)]
pub struct FileReaderConfig {}

#[async_trait]
impl FromPath for FileReaderConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(FileReaderConfig {})
    }
}

impl ConfigInto<FileReader> for FileReaderConfig {}

/// Read file
pub struct FileReader {}

#[async_trait]
impl FromConfig<FileReaderConfig> for FileReader {
    async fn from_config(_config: &FileReaderConfig) -> anyhow::Result<Self> {
        Ok(FileReader {})
    }
}

/// # Parameters
/// * P, file path: Input
/// * Vec<u8>, bytes: Output
#[async_trait]
impl<P> Map<P, Vec<u8>, FileReaderConfig> for FileReader
where
    P: AsRef<Path> + Send + 'static,
{
    async fn map(&mut self, path: P) -> anyhow::Result<Vec<u8>> {
        let bytes = self.read_all(path)?;
        Ok(bytes)
    }
}

impl FileReader {
    fn read_all<P>(&self, path: P) -> Result<Vec<u8>>
    where
        P: AsRef<Path>,
    {
        let mut rdr = BufReader::new(fs::File::open(path)?);
        let mut buffer: Vec<u8> = Vec::new();
        rdr.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

#[cfg(test)]
mod file_rw_tests {

    use std::fs;
    use std::path::PathBuf;

    use crate::*;

    #[tokio::test]
    async fn test_file_rw() {
        fs::create_dir("resources/data").expect("failed to create data directory");
        let (tx0, rx0) = channel!(Vec<u8>, 1024);
        let (tx1, rx1) = channel!(PathBuf, 1024);
        let (tx2, mut rx2) = channel!(Vec<u8>, 1024);
        let mut wrt = mapper!("writer");
        let mut rdr = mapper!("reader");
        let wrt = run_pipe!(
            wrt,
            FileWriterConfig,
            "resources/catalogs/file_writer.yml",
            [tx1],
            rx0
        );
        let rdr = run_pipe!(rdr, FileReaderConfig, [tx2], rx1);
        let content = String::from("test message for test_file_rw");
        let f0 = populate_records(tx0, vec![content.to_owned().into_bytes()]);
        f0.await;
        join_pipes!([wrt, rdr]);
        let bytes = rx2.recv().await.unwrap();
        let content_recv = String::from_utf8(bytes).expect("utf8 decode failure");
        assert_eq!(content, content_recv);
        fs::remove_dir_all("resources/data").expect("clean data folder failed");
    }
}
