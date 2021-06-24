mod json;

use serde::{de::DeserializeOwned, Serialize};

pub trait Ser {
    fn serialize<T: Serialize>(t: &T) -> anyhow::Result<Vec<u8>>;
}

pub trait Deser {
    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<T>;
}
