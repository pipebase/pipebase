use async_trait::async_trait;
use pipebase::common::{ConfigInto, FromConfig, FromPath};
use redis::{Commands, Connection, FromRedisValue, ToRedisArgs};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RedisClientConfig {
    // connection parameter: https://docs.rs/redis/0.20.2/redis/#connection-parameters
    url: String,
}

impl FromPath for RedisClientConfig {}

impl ConfigInto<RedisClient> for RedisClientConfig {}

pub struct RedisClient {
    connection: Connection,
}

impl RedisClient {
    pub fn new(url: String) -> anyhow::Result<Self> {
        let client = redis::Client::open(url)?;
        let connection = client.get_connection()?;
        Ok(RedisClient { connection })
    }

    pub fn get<K, V>(&mut self, k: K) -> anyhow::Result<Option<V>>
    where
        K: ToRedisArgs,
        V: FromRedisValue,
    {
        let value = self.connection.get(k)?;
        Ok(value)
    }
}

#[async_trait]
impl FromConfig<RedisClientConfig> for RedisClient {
    async fn from_config(config: RedisClientConfig) -> anyhow::Result<Self> {
        RedisClient::new(config.url)
    }
}
