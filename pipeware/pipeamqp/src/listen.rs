use super::connection::ConnectionConfig;
use async_trait::async_trait;
use futures_util::stream::StreamExt;
use lapin::{
    options::{BasicAckOptions, BasicConsumeOptions},
    types::FieldTable,
    Channel, Connection, ConnectionProperties,
};
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    listen::Listen,
};
use serde::Deserialize;
use tokio::sync::mpsc::Sender;
use tokio_amqp::LapinTokioExt;

#[derive(Deserialize)]
pub struct AmqpConsumerConfig {
    connection: ConnectionConfig,
    queue: String,
    consumer_tag: String,
}

impl FromPath for AmqpConsumerConfig {}

impl ConfigInto<AmqpConsumer> for AmqpConsumerConfig {}

pub struct AmqpConsumer {
    channel: Channel,
    queue: String,
    consumer_tag: String,
    tx: Option<Sender<Vec<u8>>>,
}

#[async_trait]
impl FromConfig<AmqpConsumerConfig> for AmqpConsumer {
    async fn from_config(config: AmqpConsumerConfig) -> anyhow::Result<Self> {
        let uri = config.connection.uri;
        let queue = config.queue;
        let consumer_tag = config.consumer_tag;
        let connection =
            Connection::connect(&uri, ConnectionProperties::default().with_tokio()).await?;
        let channel = connection.create_channel().await?;
        Ok(AmqpConsumer {
            channel,
            queue,
            consumer_tag,
            tx: None,
        })
    }
}

#[async_trait]
impl Listen<Vec<u8>, AmqpConsumerConfig> for AmqpConsumer {
    async fn run(&mut self) -> anyhow::Result<()> {
        let mut consumer = self
            .channel
            .basic_consume(
                &self.queue,
                &self.consumer_tag,
                BasicConsumeOptions::default(),
                FieldTable::default(),
            )
            .await?;
        let tx = self.tx.as_ref().expect("sender not set for AmqpConsumer");
        while let Some(item) = consumer.next().await {
            let (_, delivery) = item?;
            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("failed to ack");
            tx.send(delivery.data).await?;
        }
        Ok(())
    }

    fn set_sender(&mut self, sender: Sender<Vec<u8>>) {
        self.tx = Some(sender)
    }
}
