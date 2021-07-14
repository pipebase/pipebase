use crate::client::CqlClient;
use async_trait::async_trait;
use pipebase::{ConfigInto, Export, FromConfig, FromPath, Render};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CqlWriterConfig {
    hostname: String,
}

impl FromPath for CqlWriterConfig {}

impl ConfigInto<CqlWriter> for CqlWriterConfig {}

pub struct CqlWriter {
    client: CqlClient,
}

#[async_trait]
impl FromConfig<CqlWriterConfig> for CqlWriter {
    async fn from_config(config: &CqlWriterConfig) -> anyhow::Result<Self> {
        Ok(CqlWriter {
            client: CqlClient::new(&config.hostname).await?,
        })
    }
}

#[async_trait]
impl<T> Export<T, CqlWriterConfig> for CqlWriter
where
    T: Render + Send + Sync + 'static,
{
    async fn export(&mut self, t: T) -> anyhow::Result<()> {
        self.client.execute(t).await
    }
}
