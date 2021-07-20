use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    map::Map,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize)]
pub struct CsvSerConfig {}

#[async_trait]
impl FromPath for CsvSerConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path> + Send,
    {
        Ok(CsvSerConfig {})
    }
}

#[async_trait]
impl ConfigInto<CsvSer> for CsvSerConfig {}

pub struct CsvSer {}

#[async_trait]
impl FromConfig<CsvSerConfig> for CsvSer {
    async fn from_config(_config: CsvSerConfig) -> anyhow::Result<Self> {
        Ok(CsvSer {})
    }
}

impl CsvSer {
    fn serialize<I, T>(t: T) -> anyhow::Result<Vec<u8>>
    where
        I: Serialize,
        T: IntoIterator<Item = I>,
    {
        let mut wtr = csv::Writer::from_writer(vec![]);
        for item in t.into_iter() {
            wtr.serialize(item)?;
        }
        let bytes = wtr.into_inner()?;
        Ok(bytes)
    }
}

#[async_trait]
impl<I, T> Map<T, Vec<u8>, CsvSerConfig> for CsvSer
where
    I: Serialize,
    T: IntoIterator<Item = I> + Send + Sync + 'static,
{
    async fn map(&mut self, t: T) -> anyhow::Result<Vec<u8>> {
        CsvSer::serialize(t)
    }
}

#[derive(Deserialize)]
pub struct CsvDeserConfig {}

#[async_trait]
impl FromPath for CsvDeserConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path> + Send,
    {
        Ok(CsvDeserConfig {})
    }
}

#[async_trait]
impl ConfigInto<CsvDeser> for CsvDeserConfig {}

pub struct CsvDeser {}

#[async_trait]
impl FromConfig<CsvDeserConfig> for CsvDeser {
    async fn from_config(_config: CsvDeserConfig) -> anyhow::Result<Self> {
        Ok(CsvDeser {})
    }
}

impl CsvDeser {
    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<Vec<T>> {
        let mut rdr = csv::Reader::from_reader(bytes);
        let mut records: Vec<T> = Vec::new();
        for result in rdr.deserialize::<T>() {
            let record = result?;
            records.push(record);
        }
        Ok(records)
    }
}

#[async_trait]
impl<T> Map<Vec<u8>, Vec<T>, CsvDeserConfig> for CsvDeser
where
    T: DeserializeOwned + Sync,
{
    async fn map(&mut self, bytes: Vec<u8>) -> anyhow::Result<Vec<T>> {
        CsvDeser::deserialize(bytes.as_slice())
    }
}
