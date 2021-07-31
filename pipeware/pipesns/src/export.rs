use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    export::Export,
};
use serde::Deserialize;

use crate::client::{SnsClient, Subscriber};

#[derive(Deserialize)]
pub struct SnsPublisherConfig {
    region: String,
    topic_arn: String,
    subscribers: Vec<Subscriber>,
}

impl FromPath for SnsPublisherConfig {}

impl ConfigInto<SnsPublisher> for SnsPublisherConfig {}

pub struct SnsPublisher {
    client: SnsClient,
}

#[async_trait]
impl FromConfig<SnsPublisherConfig> for SnsPublisher {
    async fn from_config(config: SnsPublisherConfig) -> anyhow::Result<Self> {
        let client = SnsClient::new(config.region, config.topic_arn, config.subscribers).await?;
        Ok(SnsPublisher { client })
    }
}

#[async_trait]
impl<T> Export<T, SnsPublisherConfig> for SnsPublisher
where
    T: Into<String> + Send + 'static,
{
    async fn export(&mut self, t: T) -> anyhow::Result<()> {
        self.client.publish(t).await
    }
}
