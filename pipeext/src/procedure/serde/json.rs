use super::{Deser, Ser};
use async_trait::async_trait;
use pipebase::{ConfigInto, FromConfig, FromFile, Procedure};
use serde::Deserialize;
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;
use std::result::Result;

#[derive(Deserialize)]
pub struct JsonSerConfig {}

impl FromFile for JsonSerConfig {
    fn from_file(_path: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(JsonSerConfig {})
    }
}

#[async_trait]
impl ConfigInto<JsonSer> for JsonSerConfig {}

pub struct JsonSer {}

#[async_trait]
impl FromConfig<JsonSerConfig> for JsonSer {
    async fn from_config(
        _config: &JsonSerConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(JsonSer {})
    }
}

impl Ser for JsonSer {
    fn serialize<T: Serialize>(t: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        match serde_json::to_vec(t) {
            Ok(r) => Ok(r),
            Err(err) => Err(err.into()),
        }
    }
}

#[async_trait]
impl<T: Serialize + Sync> Procedure<T, Vec<u8>, JsonSerConfig> for JsonSer {
    async fn process(&mut self, t: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        JsonSer::serialize(t)
    }
}

#[derive(Deserialize)]
pub struct JsonDeserConfig {}

impl FromFile for JsonDeserConfig {
    fn from_file(_path: &str) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(JsonDeserConfig {})
    }
}

#[async_trait]
impl ConfigInto<JsonDeser> for JsonDeserConfig {}

pub struct JsonDeser {}

#[async_trait]
impl FromConfig<JsonDeserConfig> for JsonDeser {
    async fn from_config(
        _config: &JsonDeserConfig,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        Ok(JsonDeser {})
    }
}

impl Deser for JsonDeser {
    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Box<dyn Error>> {
        match serde_json::from_slice::<T>(bytes) {
            Ok(t) => Ok(t),
            Err(err) => Err(err.into()),
        }
    }
}

#[async_trait]
impl<T: DeserializeOwned + Sync> Procedure<Vec<u8>, T, JsonDeserConfig> for JsonDeser {
    async fn process(&mut self, bytes: &Vec<u8>) -> Result<T, Box<dyn Error>> {
        JsonDeser::deserialize(bytes.as_slice())
    }
}
