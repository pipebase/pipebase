use std::fmt::Debug;

use crate::client::RedisClient;
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    listen::Listen,
};
use redis::FromRedisValue;
use serde::Deserialize;
use tokio::sync::mpsc::Sender;

#[derive(Deserialize)]
pub struct RedisSubscriberConfig {
    channel: String,
    url: String,
}

impl FromPath for RedisSubscriberConfig {}

impl<T> ConfigInto<RedisSubscriber<T>> for RedisSubscriberConfig where T: FromRedisValue {}

pub struct RedisSubscriber<T>
where
    T: FromRedisValue,
{
    client: RedisClient,
    channel: String,
    tx: Option<Sender<T>>,
}

#[async_trait]
impl<T> FromConfig<RedisSubscriberConfig> for RedisSubscriber<T>
where
    T: FromRedisValue,
{
    async fn from_config(config: RedisSubscriberConfig) -> anyhow::Result<Self> {
        Ok(RedisSubscriber {
            client: RedisClient::new(config.url)?,
            channel: config.channel,
            tx: None,
        })
    }
}

#[async_trait]
impl<T> Listen<T, RedisSubscriberConfig> for RedisSubscriber<T>
where
    T: Debug + FromRedisValue + Send + Sync + 'static,
{
    async fn run(&mut self) -> anyhow::Result<()> {
        self.do_run().await
    }

    fn set_sender(&mut self, sender: Sender<T>) {
        self.tx = Some(sender)
    }
}

impl<T> RedisSubscriber<T>
where
    T: Debug + FromRedisValue + Send + Sync + 'static,
{
    async fn do_run(&mut self) -> anyhow::Result<()> {
        let mut pubsub = self.client.subscribe(&self.channel)?;
        let tx = self
            .tx
            .as_ref()
            .expect("sender not inited for redis subscriber");
        loop {
            let msg = pubsub.get_message()?;
            let payload: T = msg.get_payload()?;
            tx.send(payload).await?;
        }
    }
}
