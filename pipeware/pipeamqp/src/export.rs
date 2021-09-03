use super::connection::ConnectionConfig;
use async_trait::async_trait;
use lapin::{
    options::BasicPublishOptions, BasicProperties, Channel, Connection, ConnectionProperties,
};
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    export::Export,
};
use serde::Deserialize;
use tokio_amqp::LapinTokioExt;

#[derive(Deserialize)]
pub struct AmqpPublisherConfig {
    connection: ConnectionConfig,
    exchange: String,
    routing_key: String,
}

impl FromPath for AmqpPublisherConfig {}

impl ConfigInto<AmqpPublisher> for AmqpPublisherConfig {}

pub struct AmqpPublisher {
    channel: Channel,
    exchange: String,
    routing_key: String,
}

#[async_trait]
impl FromConfig<AmqpPublisherConfig> for AmqpPublisher {
    async fn from_config(config: AmqpPublisherConfig) -> anyhow::Result<Self> {
        let uri = config.connection.uri;
        let exchange = config.exchange;
        let routing_key = config.routing_key;
        let conn = Connection::connect(&uri, ConnectionProperties::default().with_tokio()).await?;
        let channel = conn.create_channel().await?;
        Ok(AmqpPublisher {
            channel,
            exchange,
            routing_key,
        })
    }
}

#[async_trait]
impl Export<Vec<u8>, AmqpPublisherConfig> for AmqpPublisher {
    async fn export(&mut self, payload: Vec<u8>) -> anyhow::Result<()> {
        let confirm = self
            .channel
            .basic_publish(
                &self.exchange,
                &self.routing_key,
                BasicPublishOptions::default(),
                payload,
                BasicProperties::default(),
            )
            .await?;
        confirm.await?;
        Ok(())
    }
}
