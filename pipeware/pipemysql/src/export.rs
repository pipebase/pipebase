use crate::client::MySQLClient;
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, IntoAttributes, Render},
    export::Export,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct MySQLWriterConfig {
    url: String,
}

impl FromPath for MySQLWriterConfig {}

impl ConfigInto<MySQLWriter> for MySQLWriterConfig {}

pub struct MySQLWriter {
    client: MySQLClient,
}

#[async_trait]
impl FromConfig<MySQLWriterConfig> for MySQLWriter {
    async fn from_config(config: MySQLWriterConfig) -> anyhow::Result<Self> {
        Ok(MySQLWriter {
            client: MySQLClient::new(&config.url),
        })
    }
}

#[async_trait]
impl<T> Export<T, MySQLWriterConfig> for MySQLWriter
where
    T: Render + Send + 'static,
{
    async fn export(&mut self, t: T) -> anyhow::Result<()> {
        self.client.execute(t).await
    }
}

#[derive(Deserialize)]
pub struct MySQLPreparedWriterConfig {
    url: String,
    statement: String,
}

impl FromPath for MySQLPreparedWriterConfig {}

impl ConfigInto<MySQLPreparedWriter> for MySQLPreparedWriterConfig {}

pub struct MySQLPreparedWriter {
    client: MySQLClient,
    statement: String,
}

#[async_trait]
impl FromConfig<MySQLPreparedWriterConfig> for MySQLPreparedWriter {
    async fn from_config(config: MySQLPreparedWriterConfig) -> anyhow::Result<Self> {
        Ok(MySQLPreparedWriter {
            client: MySQLClient::new(&config.url),
            statement: config.statement,
        })
    }
}

#[async_trait]
impl<T> Export<Vec<T>, MySQLPreparedWriterConfig> for MySQLPreparedWriter
where
    T: IntoAttributes + Send + 'static,
{
    async fn export(&mut self, items: Vec<T>) -> anyhow::Result<()> {
        let statement = self.statement.to_owned();
        self.client.prepare_execute(statement, items).await
    }
}
