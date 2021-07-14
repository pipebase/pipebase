use crate::client::RedisClient;
use async_trait::async_trait;
use pipebase::{ConfigInto, Export, FromConfig, FromPath, LeftRight};
use redis::ToRedisArgs;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RedisWriterConfig {
    // connection parameter: https://docs.rs/redis/0.20.2/redis/#connection-parameters
    url: String,
}

impl FromPath for RedisWriterConfig {}

impl ConfigInto<RedisWriter> for RedisWriterConfig {}

pub struct RedisWriter {
    client: RedisClient,
}

#[async_trait]
impl FromConfig<RedisWriterConfig> for RedisWriter {
    async fn from_config(config: &RedisWriterConfig) -> anyhow::Result<Self> {
        Ok(RedisWriter {
            client: RedisClient::new(&config.url)?,
        })
    }
}

#[async_trait]
impl<K, V, P> Export<P, RedisWriterConfig> for RedisWriter
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
