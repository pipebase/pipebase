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
