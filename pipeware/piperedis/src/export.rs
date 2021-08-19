use crate::client::RedisClient;
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, LeftRight},
    export::Export,
};
use redis::ToRedisArgs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RedisStringWriterConfig {
    // connection parameter: https://docs.rs/redis/0.20.2/redis/#connection-parameters
    url: String,
}

impl FromPath for RedisStringWriterConfig {}

impl ConfigInto<RedisStringWriter> for RedisStringWriterConfig {}

pub struct RedisStringWriter {
    client: RedisClient,
}

#[async_trait]
impl FromConfig<RedisStringWriterConfig> for RedisStringWriter {
    async fn from_config(config: RedisStringWriterConfig) -> anyhow::Result<Self> {
        Ok(RedisStringWriter {
            client: RedisClient::new(config.url)?,
        })
    }
}

#[async_trait]
impl<K, V, P> Export<P, RedisStringWriterConfig> for RedisStringWriter
where
    P: LeftRight<L = K, R = V> + Send + 'static,
    K: ToRedisArgs + Clone + Send + 'static,
    V: ToRedisArgs + Clone + Send + 'static,
{
    async fn export(&mut self, p: P) -> anyhow::Result<()> {
        self.client.set(p)?;
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct RedisPublisherConfig {
    // connection parameter: https://docs.rs/redis/0.20.2/redis/#connection-parameters
    url: String,
}

impl FromPath for RedisPublisherConfig {}

impl ConfigInto<RedisPublisher> for RedisPublisherConfig {}

pub struct RedisPublisher {
    client: RedisClient,
}

#[async_trait]
impl FromConfig<RedisPublisherConfig> for RedisPublisher {
    async fn from_config(config: RedisPublisherConfig) -> anyhow::Result<Self> {
        Ok(RedisPublisher {
            client: RedisClient::new(config.url)?,
        })
    }
}

#[async_trait]
impl<K, V, P> Export<P, RedisPublisherConfig> for RedisPublisher
where
    P: LeftRight<L = K, R = V> + Send + 'static,
    K: ToRedisArgs + Clone + Send + 'static,
    V: ToRedisArgs + Clone + Send + 'static,
{
    async fn export(&mut self, p: P) -> anyhow::Result<()> {
        self.client.publish(p)?;
        Ok(())
    }
}

#[derive(Deserialize)]
pub struct RedisStringBatchWriterConfig {
    // connection parameter: https://docs.rs/redis/0.20.2/redis/#connection-parameters
    url: String,
}

impl FromPath for RedisStringBatchWriterConfig {}

impl ConfigInto<RedisStringBatchWriter> for RedisStringBatchWriterConfig {}

pub struct RedisStringBatchWriter {
    client: RedisClient,
}

#[async_trait]
impl FromConfig<RedisStringBatchWriterConfig> for RedisStringBatchWriter {
    async fn from_config(config: RedisStringBatchWriterConfig) -> anyhow::Result<Self> {
        Ok(RedisStringBatchWriter {
            client: RedisClient::new(config.url)?,
        })
    }
}

#[async_trait]
impl<K, V, P> Export<Vec<P>, RedisStringBatchWriterConfig> for RedisStringBatchWriter
where
    P: LeftRight<L = K, R = V> + Send + 'static,
    K: ToRedisArgs + Clone + Send + 'static,
    V: ToRedisArgs + Clone + Send + 'static,
{
    async fn export(&mut self, entries: Vec<P>) -> anyhow::Result<()> {
        self.client.set_all(entries)?;
        Ok(())
    }
}
