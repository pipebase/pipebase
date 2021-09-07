use crate::client::{PsqlClient, PsqlClientConfig};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, IntoAttributes, Render},
    export::Export,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct PsqlWriterConfig {
    client: PsqlClientConfig,
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
            client: PsqlClient::new(config.client).await?,
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

#[derive(Deserialize)]
pub struct PsqlPreparedWriterConfig {
    client: PsqlClientConfig,
    statement: String,
}

impl FromPath for PsqlPreparedWriterConfig {}

impl ConfigInto<PsqlPreparedWriter> for PsqlPreparedWriterConfig {}

pub struct PsqlPreparedWriter {
    client: PsqlClient,
    statement: String,
}

#[async_trait]
impl FromConfig<PsqlPreparedWriterConfig> for PsqlPreparedWriter {
    async fn from_config(config: PsqlPreparedWriterConfig) -> anyhow::Result<Self> {
        Ok(PsqlPreparedWriter {
            client: PsqlClient::new(config.client).await?,
            statement: config.statement,
        })
    }
}

#[async_trait]
impl<T> Export<Vec<T>, PsqlPreparedWriterConfig> for PsqlPreparedWriter
where
    T: IntoAttributes + Send + 'static,
{
    async fn export(&mut self, items: Vec<T>) -> anyhow::Result<()> {
        let statement = self.statement.to_owned();
        self.client.prepare_execute(statement, items).await
    }
}
