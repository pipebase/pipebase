use crate::client::PsqlClient;
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, Render},
    export::Export,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PsqlWriterConfig {
    params: String,
}

impl FromPath for PsqlWriterConfig {}

impl ConfigInto<PsqlWriter> for PsqlWriterConfig {}

pub struct PsqlWriter {
    client: PsqlClient,
}

#[async_trait]
impl FromConfig<PsqlWriterConfig> for PsqlWriter {
    async fn from_config(config: PsqlWriterConfig) -> anyhow::Result<Self> {
        Ok(PsqlWriter {
            // TODO: Support Tls
            client: PsqlClient::new(config.params).await?,
        })
    }
}

#[async_trait]
impl<T> Export<T, PsqlWriterConfig> for PsqlWriter
where
    T: Render + Send + 'static,
{
    async fn export(&mut self, t: T) -> anyhow::Result<()> {
        self.client.execute(t).await
    }
}
