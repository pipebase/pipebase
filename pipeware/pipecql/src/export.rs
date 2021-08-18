use crate::client::CqlClient;
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, IntoAttributes, Render},
    export::Export,
};
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
    async fn from_config(config: CqlWriterConfig) -> anyhow::Result<Self> {
        Ok(CqlWriter {
            client: CqlClient::new(config.hostname).await?,
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

#[derive(Deserialize)]
pub struct CqlPreparedWriterConfig {
    hostname: String,
    statement: String,
}

impl FromPath for CqlPreparedWriterConfig {}

impl ConfigInto<CqlPreparedWriter> for CqlPreparedWriterConfig {}

pub struct CqlPreparedWriter {
    client: CqlClient,
    statement: String,
}

#[async_trait]
impl FromConfig<CqlPreparedWriterConfig> for CqlPreparedWriter {
    async fn from_config(config: CqlPreparedWriterConfig) -> anyhow::Result<Self> {
        Ok(CqlPreparedWriter {
            client: CqlClient::new(config.hostname).await?,
            statement: config.statement,
        })
    }
}

#[async_trait]
impl<T> Export<Vec<T>, CqlPreparedWriterConfig> for CqlPreparedWriter
where
    T: IntoAttributes + Send + Sync + 'static,
{
    async fn export(&mut self, items: Vec<T>) -> anyhow::Result<()> {
        let statement = self.statement.to_owned();
        self.client.prepare_execute(statement, items).await
    }
}
