use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use super::Stream;
use crate::common::{ConfigInto, FromConfig, FromPath};
use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

#[async_trait]
pub trait FileStreamReader {
    fn new_reader<P: AsRef<Path>>(path: P) -> anyhow::Result<BufReader<File>> {
        let f = File::open(path)?;
        Ok(BufReader::new(f))
    }

    async fn stream_file<P: AsRef<Path> + Send>(&self, path: P) -> anyhow::Result<()>;
}

#[derive(Deserialize)]
pub struct FileSplitReaderConfig {
    pub delimiter: u8,
}

impl FromPath for FileSplitReaderConfig {}

#[async_trait]
impl ConfigInto<FileSplitReader> for FileSplitReaderConfig {}

/// Stream file splits
pub struct FileSplitReader {
    /// Delimite to split file
    delimiter: u8,
    /// Sender to notify downstreams
    tx: Option<Sender<Vec<u8>>>,
}

#[async_trait]
impl FileStreamReader for FileSplitReader {
    async fn stream_file<P>(&self, path: P) -> anyhow::Result<()>
    where
        P: AsRef<Path> + Send,
    {
        let reader = Self::new_reader(path)?;
        let iter = reader.split(self.delimiter);
        let tx = self.tx.as_ref().unwrap();
        for result in iter {
            let bin = result?;
            tx.send(bin).await?;
        }
        Ok(())
    }
}

#[async_trait]
impl FromConfig<FileSplitReaderConfig> for FileSplitReader {
    async fn from_config(config: FileSplitReaderConfig) -> anyhow::Result<Self> {
        Ok(FileSplitReader {
            delimiter: config.delimiter,
            tx: None,
        })
    }
}

/// # Parameters
/// * P, file path: input
/// * Vec<u8>, bytes: output
#[async_trait]
impl<P> Stream<P, Vec<u8>, FileSplitReaderConfig> for FileSplitReader
where
    P: AsRef<Path> + Send + 'static,
{
    async fn stream(&mut self, path: P) -> anyhow::Result<()> {
        self.stream_file(path).await
    }

    fn set_sender(&mut self, sender: Sender<Vec<u8>>) {
        self.tx = Some(sender)
    }
}

#[cfg(test)]
mod file_split_streamer_tests {

    use std::path::PathBuf;

    use crate::prelude::*;

    #[tokio::test]
    async fn test_file_split_streamer() {
        let (tx0, rx0) = channel!(PathBuf, 1024);
        let (tx1, mut rx1) = channel!(Vec<u8>, 1024);
        let mut pipe = streamer!("file_space_split_streamer");
        let f0 = populate_records(
            tx0,
            vec![PathBuf::from("resources/test_file_stream/test_file_0.txt")],
        );
        f0.await;
        join_pipes!([run_pipe!(
            pipe,
            FileSplitReaderConfig,
            "resources/catalogs/file_split_streamer.yml",
            [tx1],
            rx0
        )]);
        let word = rx1.recv().await.unwrap();
        assert_eq!("foo", String::from_utf8(word).unwrap());
        let word = rx1.recv().await.unwrap();
        assert_eq!("bar", String::from_utf8(word).unwrap());
    }
}

#[derive(Deserialize)]
pub struct FileLineReaderConfig {}

#[async_trait]
impl FromPath for FileLineReaderConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(FileLineReaderConfig {})
    }
}

#[async_trait]
impl ConfigInto<FileLineReader> for FileLineReaderConfig {}

/// Stream file lines
pub struct FileLineReader {
    /// Sender to notify downstreams
    tx: Option<Sender<String>>,
}

#[async_trait]
impl FromConfig<FileLineReaderConfig> for FileLineReader {
    async fn from_config(_config: FileLineReaderConfig) -> anyhow::Result<Self> {
        Ok(FileLineReader { tx: None })
    }
}

#[async_trait]
impl FileStreamReader for FileLineReader {
    async fn stream_file<P>(&self, path: P) -> anyhow::Result<()>
    where
        P: AsRef<Path> + Send,
    {
        let reader = Self::new_reader(path)?;
        let mut lines_iter = reader.lines();
        let tx = self.tx.as_ref().unwrap();
        loop {
            let line = match lines_iter.next() {
                Some(line) => line?,
                None => {
                    // EOF
                    return Ok(());
                }
            };
            tx.send(line).await?;
        }
    }
}

#[async_trait]
impl<P> Stream<P, String, FileLineReaderConfig> for FileLineReader
where
    P: AsRef<Path> + Send + 'static,
{
    /// Input: P, file path
    /// Output: String, file line
    async fn stream(&mut self, path: P) -> anyhow::Result<()> {
        self.stream_file(path).await
    }

    fn set_sender(&mut self, sender: Sender<String>) {
        self.tx = Some(sender)
    }
}

#[cfg(test)]
mod file_line_streamer_tests {

    use std::path::PathBuf;

    use crate::prelude::*;

    #[tokio::test]
    async fn test_file_line_streamer() {
        let (tx0, rx0) = channel!(PathBuf, 1024);
        let (tx1, mut rx1) = channel!(String, 1024);
        let mut pipe = streamer!("file_line_streamer");
        let f0 = populate_records(
            tx0,
            vec![PathBuf::from("resources/test_file_stream/test_file_1.txt")],
        );
        f0.await;
        join_pipes!([run_pipe!(pipe, FileLineReaderConfig, "", [tx1], rx0)]);
        let line = rx1.recv().await.unwrap();
        assert_eq!("foo1 bar1", &line);
        let line = rx1.recv().await.unwrap();
        assert_eq!("foo2 bar2", &line);
    }
}
