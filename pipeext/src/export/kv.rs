use async_trait::async_trait;
use pipebase::{ConfigInto, Export, FromConfig, FromPath, LeftRight};
use redis::{Client, Commands, Connection, RedisResult, ToRedisArgs};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RedisExporterConfig {
    url: String,
}

impl FromPath for RedisExporterConfig {}

impl ConfigInto<RedisExporter> for RedisExporterConfig {}

pub struct RedisExporter {
    client: Client,
    connection: Connection,
    // flag indicate whether to reconnect
    reconnect: bool,
}

#[async_trait]
impl FromConfig<RedisExporterConfig> for RedisExporter {
    async fn from_config(config: &RedisExporterConfig) -> anyhow::Result<Self> {
        let client = redis::Client::open(config.url.to_owned())?;
        let connection = client.get_connection()?;
        Ok(RedisExporter {
            client,
            connection,
            reconnect: false,
        })
    }
}

#[async_trait]
impl<K, V, P> Export<P, RedisExporterConfig> for RedisExporter
where
    P: LeftRight<L = K, R = V> + Send + 'static,
    K: ToRedisArgs + Clone + Send + 'static,
    V: ToRedisArgs + Clone + Send + 'static,
{
    async fn export(&mut self, p: P) -> anyhow::Result<()> {
        // reconnect if necessary
        self.reconnect().await?;
        match self.set(p).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                self.reconnect = true;
                return Err(e.into());
            }
        };
    }
}

impl RedisExporter {
    async fn set<K, V, P>(&mut self, p: P) -> RedisResult<()>
    where
        P: LeftRight<L = K, R = V>,
        K: ToRedisArgs + Clone,
        V: ToRedisArgs + Clone,
    {
        let k = p.left().to_owned();
        let v = p.right().to_owned();
        let _ = self.connection.set(k, v)?;
        Ok(())
    }

    async fn reconnect(&mut self) -> RedisResult<()> {
        if !self.reconnect {
            return Ok(());
        }
        self.connection = self.client.get_connection()?;
        Ok(())
    }
}
