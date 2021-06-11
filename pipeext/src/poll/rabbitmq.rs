use async_trait::async_trait;
use lapin::options::BasicConsumeOptions;
use lapin::Channel;
use lapin::{
    options::BasicAckOptions, types::FieldTable, Connection, ConnectionProperties, Consumer,
};
use log::{error, info};
use pipebase::Poll;
use pipebase::{FromConfig, FromFile};
use serde::Deserialize;
use std::error::Error;
use std::result::Result;
use tokio_amqp::*;
pub struct RabbitMQConsumer {
    queue: String,
    consumer_tag: String,
    uri: String,
    options: BasicConsumeOptions,
    args: FieldTable,
    channel: Channel,
}

#[derive(Deserialize)]
pub struct RabbitMQConsumerConfig {
    pub queue: String,
    pub consumer_tag: String,
    pub uri: String,
}

impl FromFile for RabbitMQConsumerConfig {}

#[async_trait]
impl FromConfig<RabbitMQConsumerConfig> for RabbitMQConsumer {
    async fn from_config(config: &RabbitMQConsumerConfig) -> Result<Self, Box<dyn Error>> {
        let properties = ConnectionProperties::default().with_tokio_executor();
        info!("start connection ...");
        let conn = Connection::connect(&config.uri, properties).await?;
        info!("connected ...");
        let channel = conn.create_channel().await?;
        Ok(RabbitMQConsumer {
            queue: config.queue.to_owned(),
            consumer_tag: config.consumer_tag.to_owned(),
            uri: config.uri.to_owned(),
            options: BasicConsumeOptions::default(),
            args: FieldTable::default(),
            channel: channel,
        })
    }
}

#[async_trait]
impl Poll<Vec<u8>> for RabbitMQConsumer {
    async fn poll(&mut self) -> Result<Option<Vec<u8>>, Box<dyn Error + Send + Sync>> {
        let consumer = match self.create_consumer().await {
            Ok(consumer) => consumer,
            Err(err) => return Err(err.into()),
        };
        let next = consumer.into_iter().next();
        let delivery = match next {
            Some(delivery) => delivery,
            None => return Ok(None),
        };
        let delivery = match delivery {
            Ok((_, delivery)) => delivery,
            Err(err) => return Err(err.into()),
        };
        delivery.ack(BasicAckOptions::default()).await.ok();
        Ok(Some(delivery.data))
    }
}

impl RabbitMQConsumer {
    pub async fn create_consumer(&self) -> std::result::Result<Consumer, lapin::Error> {
        self.channel
            .basic_consume(
                &self.queue,
                &self.consumer_tag,
                self.options.to_owned(),
                self.args.to_owned(),
            )
            .await
    }
}

#[cfg(test)]
mod tests {

    use pipebase::{source, FromConfig, FromFile, Pipe, Source};
    use tokio::sync::mpsc::channel;
    use tokio::sync::mpsc::Receiver;

    use super::{RabbitMQConsumer, RabbitMQConsumerConfig};

    async fn on_receive(rx: &mut Receiver<Vec<u8>>) {
        let mut i: i32 = 0;
        while i < 10 {
            let message = rx.recv().await;
            match message {
                Some(message) => println!("message: {:#?}", String::from_utf8(message)),
                None => break,
            }
            i += 1;
        }
        println!("received all")
    }

    #[tokio::test]
    #[ignore]
    async fn test_consumer() {
        let (tx, mut rx) = channel::<Vec<u8>>(1024);
        let mut s = source!(
            "rbmq_consumer",
            "resources/catalogs/rabbitmq_consumer.yml",
            RabbitMQConsumerConfig,
            RabbitMQConsumer,
            [tx]
        );
        let jh0 = tokio::spawn(async move {
            s.run().await;
        });
        let jh1 = on_receive(&mut rx);
        tokio::join!(jh0, jh1);
    }
}
