use crate::Map;
use async_trait::async_trait;
use serde::Deserialize;
use std::{io::Read, path::Path};

use crate::{ConfigInto, FromConfig, FromPath};

#[async_trait]
pub trait ReadFile {
    async fn read_all<P>(path: P) -> anyhow::Result<Vec<u8>>
    where
        P: AsRef<Path> + Send;
}

#[derive(Deserialize)]
pub struct LocalFileReaderConfig {}

impl FromPath for LocalFileReaderConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path>,
    {
        Ok(LocalFileReaderConfig {})
    }
}

impl ConfigInto<LocalFileReader> for LocalFileReaderConfig {}

pub struct LocalFileReader {}

#[async_trait]
impl FromConfig<LocalFileReaderConfig> for LocalFileReader {
    async fn from_config(_config: &LocalFileReaderConfig) -> anyhow::Result<Self> {
        Ok(LocalFileReader {})
    }
}

#[async_trait]
impl ReadFile for LocalFileReader {
    async fn read_all<P>(path: P) -> anyhow::Result<Vec<u8>>
    where
        P: AsRef<Path> + Send,
    {
        let mut file = std::fs::File::open(path)?;
        let mut buffer: Vec<u8> = Vec::new();
        file.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

#[async_trait]
impl<T> Map<T, Vec<u8>, LocalFileReaderConfig> for LocalFileReader
where
    T: AsRef<Path> + Send + 'static,
{
    async fn map(&mut self, path: T) -> anyhow::Result<Vec<u8>> {
        Self::read_all(path).await
    }
}

#[cfg(test)]
mod tests {

    use std::path::PathBuf;

    use crate::*;
    use tokio::sync::mpsc::Sender;

    async fn populate_records(tx: Sender<PathBuf>, paths: Vec<PathBuf>) {
        for path in paths {
            let _ = tx.send(path).await.unwrap();
        }
    }

    #[tokio::test]
    async fn test_read_file() {
        let (tx0, rx0) = channel!(PathBuf, 1024);
        let (tx1, mut rx1) = channel!(Vec<u8>, 1024);
        let mut pipe = mapper!("file_reader", LocalFileReaderConfig, rx0, [tx1]);
        let f0 = populate_records(
            tx0,
            vec![PathBuf::from("resources/test_file_folder/test_file_0.txt")],
        );
        f0.await;
        spawn_join!(pipe);
        let bin = rx1.recv().await.expect("expect file binary");
        let content = String::from_utf8(bin).expect("expect utf8 encoded");
        assert_eq!("foo bar", &content)
    }
}
