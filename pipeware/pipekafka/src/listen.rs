use crate::config::{create_kafka_client, KafkaClientConfig, KafkaConsumerClientConfig};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    listen::Listen,
};
use rdkafka::{
    consumer::{CommitMode, Consumer, DefaultConsumerContext, StreamConsumer},
    Message,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::iter::FromIterator;
use tokio::sync::mpsc::Sender;

#[derive(Clone, Deserialize)]
pub struct KafkaConsumerConfig {
    base: KafkaClientConfig,
    consumer: KafkaConsumerClientConfig,
}

impl From<KafkaConsumerConfig> for HashMap<&str, String> {
    fn from(config: KafkaConsumerConfig) -> Self {
        let mut params: HashMap<&str, String> = config.base.into();
        let cparams: HashMap<&str, String> = config.consumer.into();
        params.extend(cparams);
        params
    }
}

impl FromPath for KafkaConsumerConfig {}

impl ConfigInto<KafkaConsumer> for KafkaConsumerConfig {}

type DefaultStreamConsumer = StreamConsumer<DefaultConsumerContext>;
pub struct KafkaConsumer {
    client: DefaultStreamConsumer,
    tx: Option<Sender<Vec<u8>>>,
}

#[async_trait]
impl FromConfig<KafkaConsumerConfig> for KafkaConsumer {
    async fn from_config(config: KafkaConsumerConfig) -> anyhow::Result<Self> {
        let params: HashMap<&str, String> = config.to_owned().into();
        let topics = config.consumer.get_topics();
        let consumer = create_kafka_client::<DefaultConsumerContext, DefaultStreamConsumer>(
            params,
            DefaultConsumerContext,
        )?;
        consumer.subscribe(&topics)?;
        Ok(KafkaConsumer {
            client: consumer,
            tx: None,
        })
    }
}

#[async_trait]
impl Listen<Vec<u8>, KafkaConsumerConfig> for KafkaConsumer {
    async fn run(&mut self) -> anyhow::Result<()> {
        self.do_run().await
    }

    fn set_sender(&mut self, sender: Sender<Vec<u8>>) {
        self.tx = Some(sender)
    }
}

impl KafkaConsumer {
    async fn do_run(&mut self) -> anyhow::Result<()> {
        let tx = self.tx.as_ref().expect("sender not found");
        loop {
            let message = self.client.recv().await?;
            self.client.commit_message(&message, CommitMode::Async)?;
            let bytes = match message.payload() {
                Some(bytes) => bytes,
                None => continue,
            };
            let bytes = Vec::from_iter(bytes.to_owned());
            tx.send(bytes).await?;
        }
    }
}
