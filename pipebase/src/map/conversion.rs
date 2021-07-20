use crate::{ConfigInto, Convert, FromConfig, FromPath};

use super::Map;
use async_trait::async_trait;
use serde::Deserialize;
use std::path::Path;

#[derive(Deserialize)]
pub struct ConversionConfig {}

#[async_trait]
impl FromPath for ConversionConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<Path> + Send,
    {
        Ok(ConversionConfig {})
    }
}

#[async_trait]
impl ConfigInto<Conversion> for ConversionConfig {}

/// Convert from type to type
pub struct Conversion {}

#[async_trait]
impl FromConfig<ConversionConfig> for Conversion {
    async fn from_config(_config: ConversionConfig) -> anyhow::Result<Self> {
        Ok(Conversion {})
    }
}

/// # Parameters
/// * T: input
/// * U: output
#[async_trait]
impl<T, U> Map<T, U, ConversionConfig> for Conversion
where
    T: Send + 'static,
    U: Convert<T>,
{
    async fn map(&mut self, data: T) -> anyhow::Result<U> {
        Ok(U::convert(data))
    }
}
