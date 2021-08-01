use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, PipeError},
    error::HandleError,
};
use serde::Deserialize;

use crate::client::{SnsClient, Subscriber};

#[derive(Deserialize)]
pub struct SnsPipeErrorPublisherConfig {
    region: String,
    topic_arn: String,
    subscribers: Vec<Subscriber>,
}

impl FromPath for SnsPipeErrorPublisherConfig {}

impl ConfigInto<SnsPipeErrorPublisher> for SnsPipeErrorPublisherConfig {}

pub struct SnsPipeErrorPublisher {
    client: SnsClient,
}

#[async_trait]
impl FromConfig<SnsPipeErrorPublisherConfig> for SnsPipeErrorPublisher {
    async fn from_config(config: SnsPipeErrorPublisherConfig) -> anyhow::Result<Self> {
        let client = SnsClient::new(config.region, config.topic_arn, config.subscribers).await?;
        Ok(SnsPipeErrorPublisher { client })
    }
}

#[async_trait]
impl HandleError<SnsPipeErrorPublisherConfig> for SnsPipeErrorPublisher {
    async fn handle_error(&mut self, pipe_error: PipeError) -> anyhow::Result<()> {
        let error_message = Self::pipe_error_as_text(pipe_error);
        self.client.publish(error_message).await
    }
}

impl SnsPipeErrorPublisher {
    fn pipe_error_as_text(pipe_error: PipeError) -> String {
        format!("{:#?}", pipe_error)
    }
}
