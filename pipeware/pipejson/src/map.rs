use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
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
