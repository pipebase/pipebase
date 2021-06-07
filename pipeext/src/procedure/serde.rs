use async_trait::async_trait;
use pipebase::Procedure;
use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;
use std::result::Result;
trait Ser {
    fn serialize<T: Serialize>(t: &T) -> Result<Vec<u8>, Box<dyn Error>>;
}

trait Deser {
    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Box<dyn Error>>;
}

struct JsonSer {}

impl Ser for JsonSer {
    fn serialize<T: Serialize>(t: &T) -> Result<Vec<u8>, Box<dyn Error>> {
        match serde_json::to_vec(t) {
            Ok(r) => Ok(r),
            Err(err) => Err(err.into()),
        }
    }
}

#[async_trait]
impl<T: Serialize + Send + Sync + 'static> Procedure<T, Vec<u8>> for JsonSer {
    async fn process(&self, t: T) -> Result<Vec<u8>, Box<dyn Error>> {
        JsonSer::serialize(&t)
    }
}

struct JsonDeser {}

impl Deser for JsonDeser {
    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Box<dyn Error>> {
        match serde_json::from_slice::<T>(bytes) {
            Ok(t) => Ok(t),
            Err(err) => Err(err.into()),
        }
    }
}

#[async_trait]
impl<T: DeserializeOwned + Send + Sync + 'static> Procedure<Vec<u8>, T> for JsonDeser {
    async fn process(&self, bytes: Vec<u8>) -> Result<T, Box<dyn Error>> {
        JsonDeser::deserialize(bytes.as_slice())
    }
}
