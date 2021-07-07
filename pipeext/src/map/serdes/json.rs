use super::{Deser, Ser};
use async_trait::async_trait;
use pipebase::{ConfigInto, FromConfig, FromPath, Map};
use serde::Deserialize;
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;

#[derive(Deserialize)]
pub struct JsonSerConfig {}

impl FromPath for JsonSerConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(JsonSerConfig {})
    }
}

#[async_trait]
impl ConfigInto<JsonSer> for JsonSerConfig {}

pub struct JsonSer {}

#[async_trait]
impl FromConfig<JsonSerConfig> for JsonSer {
    async fn from_config(_config: &JsonSerConfig) -> anyhow::Result<Self> {
        Ok(JsonSer {})
    }
}

impl Ser for JsonSer {
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

impl FromPath for JsonDeserConfig {
    fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path>,
    {
        Ok(JsonDeserConfig {})
    }
}

#[async_trait]
impl ConfigInto<JsonDeser> for JsonDeserConfig {}

pub struct JsonDeser {}

#[async_trait]
impl FromConfig<JsonDeserConfig> for JsonDeser {
    async fn from_config(_config: &JsonDeserConfig) -> anyhow::Result<Self> {
        Ok(JsonDeser {})
    }
}

impl Deser for JsonDeser {
    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<T> {
        match serde_json::from_slice::<T>(bytes) {
            Ok(t) => Ok(t),
            Err(err) => Err(err.into()),
        }
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
