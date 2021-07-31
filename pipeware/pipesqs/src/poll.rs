use crate::{
    client::{SqsClient, SqsClientConfig},
    message::SqsMessage,
};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, Period},
    poll::{Poll, PollResponse},
};
use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
pub struct SqsMessageReceiverConfig {
    client: SqsClientConfig,
    initial_delay: Period,
    interval: Period,
}

impl FromPath for SqsMessageReceiverConfig {}

impl ConfigInto<SqsMessageReceiver> for SqsMessageReceiverConfig {}

pub struct SqsMessageReceiver {
    client: SqsClient,
    initial_delay: Duration,
    interval: Duration,
}

#[async_trait]
impl FromConfig<SqsMessageReceiverConfig> for SqsMessageReceiver {
    async fn from_config(config: SqsMessageReceiverConfig) -> anyhow::Result<Self> {
        let client_config = config.client;
        Ok(SqsMessageReceiver {
            client: SqsClient::new(client_config),
            initial_delay: config.initial_delay.into(),
            interval: config.interval.into(),
        })
    }
}

#[async_trait]
impl Poll<Vec<SqsMessage>, SqsMessageReceiverConfig> for SqsMessageReceiver {
    async fn poll(&mut self) -> anyhow::Result<PollResponse<Vec<SqsMessage>>> {
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

impl SqsMessageReceiver {
    async fn receive_message(&self) -> anyhow::Result<Vec<SqsMessage>> {
        let msg_output = self.client.receive_message().await?;
        let messages = msg_output.messages.unwrap_or_default();
        let mut sqs_messages: Vec<SqsMessage> = Vec::new();
        for message in messages {
            let sqs_message = self.client.handle_message(message).await;
            sqs_messages.push(sqs_message);
        }
        Ok(sqs_messages)
    }
}
