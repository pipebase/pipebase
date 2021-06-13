mod json;

use serde::{de::DeserializeOwned, Serialize};
use std::error::Error;
use std::result::Result;
pub trait Ser {
    fn serialize<T: Serialize>(t: &T) -> Result<Vec<u8>, Box<dyn Error>>;
}

pub trait Deser {
    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, Box<dyn Error>>;
}
