use crate::client::DynamocDBClient;
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, IntoAttributes},
    export::Export,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DynamoDBWriterConfig {
    region: String,
    table: String,
}

impl FromPath for DynamoDBWriterConfig {}

impl ConfigInto<DynamoDBWriter> for DynamoDBWriterConfig {}

pub struct DynamoDBWriter {
    client: DynamocDBClient,
}

#[async_trait]
impl FromConfig<DynamoDBWriterConfig> for DynamoDBWriter {
    async fn from_config(config: DynamoDBWriterConfig) -> anyhow::Result<Self> {
        let client = DynamocDBClient::new(config.region, config.table);
        Ok(DynamoDBWriter { client })
    }
}

#[async_trait]
impl<T> Export<T, DynamoDBWriterConfig> for DynamoDBWriter
where
    T: IntoAttributes + Send + 'static,
{
    async fn export(&mut self, t: T) -> anyhow::Result<()> {
        self.client.put_item(t).await
    }
}
