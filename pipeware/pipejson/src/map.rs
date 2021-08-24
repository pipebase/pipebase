use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, GroupAs, Pair},
    map::Map,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::path::Path;

#[derive(Deserialize)]
pub struct JsonSerConfig {}

#[async_trait]
impl FromPath for JsonSerConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path> + Send,
    {
        Ok(JsonSerConfig {})
    }
}

#[async_trait]
impl ConfigInto<JsonSer> for JsonSerConfig {}

pub struct JsonSer {}

#[async_trait]
impl FromConfig<JsonSerConfig> for JsonSer {
    async fn from_config(_config: JsonSerConfig) -> anyhow::Result<Self> {
        Ok(JsonSer {})
    }
}

impl JsonSer {
    fn serialize<T: Serialize>(t: &T) -> anyhow::Result<Vec<u8>> {
        match serde_json::to_vec(t) {
            Ok(r) => Ok(r),
            Err(err) => Err(err.into()),
        }
    }
}

#[async_trait]
impl<T> Map<T, Vec<u8>, JsonSerConfig> for JsonSer
where
    T: Serialize + Send + Sync + 'static,
{
    async fn map(&mut self, t: T) -> anyhow::Result<Vec<u8>> {
        JsonSer::serialize(&t)
    }
}

#[derive(Deserialize)]
pub struct JsonDeserConfig {}

#[async_trait]
impl FromPath for JsonDeserConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path> + Send,
    {
        Ok(JsonDeserConfig {})
    }
}

#[async_trait]
impl ConfigInto<JsonDeser> for JsonDeserConfig {}

pub struct JsonDeser {}

#[async_trait]
impl FromConfig<JsonDeserConfig> for JsonDeser {
    async fn from_config(_config: JsonDeserConfig) -> anyhow::Result<Self> {
        Ok(JsonDeser {})
    }
}

impl JsonDeser {
    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<T> {
        let t: T = serde_json::from_slice::<T>(bytes)?;
        Ok(t)
    }
}

#[async_trait]
impl<T> Map<Vec<u8>, T, JsonDeserConfig> for JsonDeser
where
    T: DeserializeOwned + Sync,
{
    async fn map(&mut self, bytes: Vec<u8>) -> anyhow::Result<T> {
        JsonDeser::deserialize(bytes.as_slice())
    }
}

#[derive(Deserialize)]
pub struct JsonRecordSerConfig {}

#[async_trait]
impl FromPath for JsonRecordSerConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(JsonRecordSerConfig {})
    }
}

impl ConfigInto<JsonRecordSer> for JsonRecordSerConfig {}

#[async_trait]
impl FromConfig<JsonRecordSerConfig> for JsonRecordSer {
    async fn from_config(_config: JsonRecordSerConfig) -> anyhow::Result<Self> {
        Ok(JsonRecordSer {})
    }
}

pub struct JsonRecordSer {}

impl JsonRecordSer {
    fn serialize<K, R>(record: &R) -> anyhow::Result<Pair<K, Vec<u8>>>
    where
        R: GroupAs<K> + Serialize,
    {
        let bytes = serde_json::to_vec(record)?;
        let key = record.group();
        Ok(Pair::new(key, bytes))
    }
}

#[async_trait]
impl<K, R> Map<R, Pair<K, Vec<u8>>, JsonRecordSerConfig> for JsonRecordSer
where
    R: GroupAs<K> + Serialize + Send + 'static,
{
    async fn map(&mut self, data: R) -> anyhow::Result<Pair<K, Vec<u8>>> {
        Self::serialize(&data)
    }
}
