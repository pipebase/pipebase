use crate::client::{SQSClient, SQSClientConfig, SQSMessageAttributeValue};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, Period},
    poll::{Poll, PollResponse},
};
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};

#[derive(Deserialize)]
pub struct SQSMessageReceiverConfig {
    client: SQSClientConfig,
    initial_delay: Period,
    interval: Period,
}

impl FromPath for SQSMessageReceiverConfig {}

impl ConfigInto<SQSMessageReceiver> for SQSMessageReceiverConfig {}

pub struct SQSMessageReceiver {
    client: SQSClient,
    initial_delay: Duration,
    interval: Duration,
}

#[async_trait]
impl FromConfig<SQSMessageReceiverConfig> for SQSMessageReceiver {
    async fn from_config(config: SQSMessageReceiverConfig) -> anyhow::Result<Self> {
        let client_config = config.client;
        Ok(SQSMessageReceiver {
            client: SQSClient::new(client_config),
            initial_delay: config.initial_delay.into(),
            interval: config.interval.into(),
        })
    }
}

#[async_trait]
impl Poll<Vec<(String, HashMap<String, SQSMessageAttributeValue>)>, SQSMessageReceiverConfig>
    for SQSMessageReceiver
{
    async fn poll(
        &mut self,
    ) -> anyhow::Result<PollResponse<Vec<(String, HashMap<String, SQSMessageAttributeValue>)>>>
    {
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

impl SQSMessageReceiver {
    async fn receive_message(
        &self,
    ) -> anyhow::Result<Vec<(String, HashMap<String, SQSMessageAttributeValue>)>> {
        let msg_output = self.client.receive_message().await?;
        let messages = msg_output.messages.unwrap_or_default();
        let messages: Vec<(String, HashMap<String, SQSMessageAttributeValue>)> = messages
            .into_iter()
            .map(|m| SQSClient::handle_message(m))
            .collect();
        Ok(messages)
    }
}
