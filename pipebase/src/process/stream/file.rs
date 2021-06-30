use std::{
    fs::File,
    io::{BufRead, BufReader},
    path::Path,
};

use crate::{ConfigInto, FromConfig, FromPath, Stream};
use async_trait::async_trait;
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

#[async_trait]
pub trait FileStreamer {
    fn new_reader<P: AsRef<Path>>(path: P) -> anyhow::Result<BufReader<File>> {
        let f = File::open(path)?;
        Ok(BufReader::new(f))
    }

    async fn stream_file<P: AsRef<Path> + Send>(&self, path: P) -> anyhow::Result<()>;

    async fn stream_eof(&self) -> anyhow::Result<()>;
}

#[derive(Deserialize)]
pub struct FileSplitStreamerConfig {
    pub delimiter: u8,
}

impl FromPath for FileSplitStreamerConfig {}

#[async_trait]
impl ConfigInto<FileSplitStreamer> for FileSplitStreamerConfig {}

pub struct FileSplitStreamer {
    delimiter: u8,
    tx: Option<Sender<Vec<u8>>>,
}

#[async_trait]
impl FileStreamer for FileSplitStreamer {
    async fn stream_eof(&self) -> anyhow::Result<()> {
        let tx = self.tx.as_ref().unwrap();
        tx.send(vec![]).await?;
        Ok(())
    }

    async fn stream_file<P>(&self, path: P) -> anyhow::Result<()>
    where
        P: AsRef<Path> + Send,
    {
        let reader = Self::new_reader(path)?;
        let mut iter = reader.split(self.delimiter);
        let tx = self.tx.as_ref().unwrap();
        loop {
            let bin = match iter.next() {
                Some(result) => result?,
                None => {
                    // EOF
                    self.stream_eof().await?;
                    break;
                }
            };
            tx.send(bin).await?;
        }
        Ok(())
    }
    /*
    async fn stream_lines<P>(&self, path: P) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        let reader = Self::new_reader(path)?;
        let mut lines_iter = reader.lines();
        let tx = self.tx.as_ref().unwrap();
        loop {
            let line = match lines_iter.next() {
                Some(line) => line?,
                None => return self.stream_eof().await
            };
            let bin: Vec<u8> = line.as_bytes().into_iter().collect();
        }
    }
    */
}

#[async_trait]
impl FromConfig<FileSplitStreamerConfig> for FileSplitStreamer {
    async fn from_config(config: &FileSplitStreamerConfig) -> anyhow::Result<Self> {
        Ok(FileSplitStreamer {
            delimiter: config.delimiter.to_owned(),
            tx: None,
        })
    }
}

#[async_trait]
impl<P> Stream<P, Vec<u8>, FileSplitStreamerConfig> for FileSplitStreamer
where
    P: AsRef<Path> + Send + 'static,
{
    async fn stream(&mut self, path: P) -> anyhow::Result<()> {
        self.stream_file(path).await
    }

    async fn set_sender(&mut self, sender: Sender<Vec<u8>>) {
        self.tx = Some(sender)
    }
}

#[cfg(test)]
mod file_split_streamer_tests {

    use std::path::PathBuf;

    use crate::*;
    use tokio::sync::mpsc::Sender;

    async fn populate_records(tx: Sender<PathBuf>, paths: Vec<PathBuf>) {
        for path in paths {
            let _ = tx.send(path).await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_file_split_streamer() {
        let (tx0, rx0) = channel!(PathBuf, 1024);
        let (tx1, mut rx1) = channel!(Vec<u8>, 1024);
        let mut pipe = streamer!(
            "file_space_split_streamer",
            "resources/catalogs/file_split_streamer.yml",
            FileSplitStreamerConfig,
            rx0,
            [tx1]
        );
        let f0 = populate_records(
            tx0,
            vec![PathBuf::from("resources/test_file_stream/test_file_0.txt")],
        );
        f0.await;
        spawn_join!(pipe);
        let word = rx1.recv().await.unwrap();
        assert_eq!("foo", String::from_utf8(word).unwrap());
        let word = rx1.recv().await.unwrap();
        assert_eq!("bar", String::from_utf8(word).unwrap());
        // eof
        let line = rx1.recv().await.unwrap();
        assert!(line.is_empty())
    }
}

#[derive(Deserialize)]
pub struct FileLineStreamerConfig {}

impl FromPath for FileLineStreamerConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(FileLineStreamerConfig {})
    }
}

#[async_trait]
impl ConfigInto<FileLineStreamer> for FileLineStreamerConfig {}

pub struct FileLineStreamer {
    tx: Option<Sender<String>>,
}

#[async_trait]
impl FromConfig<FileLineStreamerConfig> for FileLineStreamer {
    async fn from_config(_config: &FileLineStreamerConfig) -> anyhow::Result<Self> {
        Ok(FileLineStreamer { tx: None })
    }
}

#[async_trait]
impl FileStreamer for FileLineStreamer {
    async fn stream_eof(&self) -> anyhow::Result<()> {
        let tx = self.tx.as_ref().unwrap();
        tx.send(String::new()).await?;
        Ok(())
    }

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
                None => return self.stream_eof().await,
            };
            tx.send(line).await?;
        }
    }
}

#[async_trait]
impl<P> Stream<P, String, FileLineStreamerConfig> for FileLineStreamer
where
    P: AsRef<Path> + Send + 'static,
{
    async fn stream(&mut self, path: P) -> anyhow::Result<()> {
        self.stream_file(path).await
    }

    async fn set_sender(&mut self, sender: Sender<String>) {
        self.tx = Some(sender)
    }
}

#[cfg(test)]
mod file_line_streamer_tests {

    use std::path::PathBuf;

    use crate::*;
    use tokio::sync::mpsc::Sender;

    async fn populate_records(tx: Sender<PathBuf>, paths: Vec<PathBuf>) {
        for path in paths {
            let _ = tx.send(path).await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_file_line_streamer() {
        let (tx0, rx0) = channel!(PathBuf, 1024);
        let (tx1, mut rx1) = channel!(String, 1024);
        let mut pipe = streamer!("file_line_streamer", FileLineStreamerConfig, rx0, [tx1]);
        let f0 = populate_records(
            tx0,
            vec![PathBuf::from("resources/test_file_stream/test_file_1.txt")],
        );
        f0.await;
        spawn_join!(pipe);
        let line = rx1.recv().await.unwrap();
        assert_eq!("foo1 bar1", &line);
        let line = rx1.recv().await.unwrap();
        assert_eq!("foo2 bar2", &line);
        // eof
        let line = rx1.recv().await.unwrap();
        assert!(line.is_empty())
    }
}
