use crate::client::MySQLClient;
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, Render},
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
