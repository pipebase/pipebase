use std::time::Duration;

use crate::client::{SQSClient, SQSClientConfig};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, Period},
    poll::Poll,
};
use serde::Deserialize;
use tokio::time::Interval;

#[derive(Deserialize)]
pub struct SQSReceiverConfig {
    client: SQSClientConfig,
    poll_period: Period,
}

impl FromPath for SQSReceiverConfig {}

pub struct SQSReceiver {
    client: SQSClient,
    poll_interval: Interval,
}

#[async_trait]
impl FromConfig<SQSReceiverConfig> for SQSReceiver {
    async fn from_config(config: SQSReceiverConfig) -> anyhow::Result<Self> {
        let client_config = config.client;
        Ok(SQSReceiver {
            client: SQSClient::new(client_config),
            poll_interval: tokio::time::interval(config.poll_period.into()),
        })
    }
}

impl SQSReceiver {
    async fn receive_message(&self) -> anyhow::Result<Vec<String>> {
        let msg_output = self.client.receive_message().await?;
        let messages = msg_output.messages.unwrap_or_default();
        let messages: Vec<String> = messages.into_iter().filter_map(|m| m.body).collect();
        Ok(messages)
    }
}
