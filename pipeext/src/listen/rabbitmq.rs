use async_trait::async_trait;
use lapin::options::BasicConsumeOptions;
use lapin::Channel;
use lapin::{
    options::BasicAckOptions, types::FieldTable, Connection, ConnectionProperties, Consumer,
};
use log::info;
use pipebase::ConfigInto;
use pipebase::Listen;
use pipebase::{FromConfig, FromPath};
use serde::Deserialize;
use tokio::sync::mpsc::Sender;
use tokio_amqp::*;

pub struct RabbitMQConsumer {
    queue: String,
    consumer_tag: String,
    uri: String,
    options: BasicConsumeOptions,
    args: FieldTable,
    channel: Channel,
    sender: Option<Sender<Vec<u8>>>,
}

#[derive(Deserialize)]
pub struct RabbitMQConsumerConfig {
    pub queue: String,
    pub consumer_tag: String,
    pub uri: String,
}

impl FromPath for RabbitMQConsumerConfig {}

#[async_trait]
impl ConfigInto<RabbitMQConsumer> for RabbitMQConsumerConfig {}

#[async_trait]
impl FromConfig<RabbitMQConsumerConfig> for RabbitMQConsumer {
    async fn from_config(config: &RabbitMQConsumerConfig) -> anyhow::Result<Self> {
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
            sender: None,
        })
    }
}

#[async_trait]
impl Listen<Vec<u8>, RabbitMQConsumerConfig> for RabbitMQConsumer {
    async fn run(&mut self) -> anyhow::Result<()> {
        info!("creating consumer ...");
        let consumer = match self.create_consumer().await {
            Ok(consumer) => consumer,
            Err(err) => return Err(err.into()),
        };
        //  TODO: Message lag one record behind
        for bytes in consumer.into_iter() {
            let data = match bytes {
                Ok((_, delivery)) => {
                    let data = delivery.data.to_owned();
                    delivery.ack(BasicAckOptions::default()).await.ok();
                    data
                }
                Err(err) => return Err(err.into()),
            };
            self.sender.as_ref().unwrap().send(data).await?;
        }
        Ok(())
    }

    fn set_sender(&mut self, sender: Sender<Vec<u8>>) {
        self.sender = Some(sender)
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

    use pipebase::{listener, FromPath, Listener, Pipe};
    use tokio::sync::mpsc::channel;
    use tokio::sync::mpsc::Receiver;

    use super::RabbitMQConsumerConfig;
    // use std::{println as info, println as error};

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

    #[ignore]
    #[tokio::test]
    async fn test_listen_rabbitmq() {
        let (tx, mut rx) = channel::<Vec<u8>>(1024);
        let mut s = listener!(
            "rbmq_consumer",
            "resources/catalogs/rabbitmq_consumer.yml",
            RabbitMQConsumerConfig,
            [tx]
        );
        println!("consumer built ...");
        let jh0 = tokio::spawn(async move {
            let _ = s.run().await;
        });
        let jh1 = on_receive(&mut rx);
        let _ = tokio::join!(jh0, jh1);
    }
}
