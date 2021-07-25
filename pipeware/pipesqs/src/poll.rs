use std::time::Duration;

use crate::client::{SQSClient, SQSClientConfig};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, Period},
    poll::{Poll, PollResponse},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SQSReceiverConfig {
    client: SQSClientConfig,
    initial_delay: Period,
    interval: Period,
}

impl FromPath for SQSReceiverConfig {}

impl ConfigInto<SQSReceiver> for SQSReceiverConfig {}

pub struct SQSReceiver {
    client: SQSClient,
    initial_delay: Duration,
    interval: Duration,
}

#[async_trait]
impl FromConfig<SQSReceiverConfig> for SQSReceiver {
    async fn from_config(config: SQSReceiverConfig) -> anyhow::Result<Self> {
        let client_config = config.client;
        Ok(SQSReceiver {
            client: SQSClient::new(client_config),
            initial_delay: config.initial_delay.into(),
            interval: config.interval.into(),
        })
    }
}

#[async_trait]
impl Poll<Vec<String>, SQSReceiverConfig> for SQSReceiver {
    async fn poll(&mut self) -> anyhow::Result<PollResponse<Vec<String>>> {
        let messages = self.receive_message().await?;
        if messages.is_empty() {
            return Ok(PollResponse::PollResult(None));
        }
        Ok(PollResponse::PollResult(Some(messages)))
    }

    fn get_initial_delay(&self) -> Duration {
        self.initial_delay.to_owned()
    }

    fn get_interval(&self) -> tokio::time::Interval {
        let interval = self.interval.to_owned();
        tokio::time::interval(interval)
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
