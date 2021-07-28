use pipebase::common::LeftRight;
use redis::{aio::PubSub, Client, Commands, Connection, FromRedisValue, RedisResult, ToRedisArgs};

pub struct RedisClient {
    client: Client,
    connection: Connection,
    // flag indicate whether to reconnect
    reconnect: bool,
}

impl RedisClient {
    pub fn new(url: String) -> anyhow::Result<Self> {
        let client = redis::Client::open(url)?;
        let connection = client.get_connection()?;
        Ok(RedisClient {
            client,
            connection,
            reconnect: false,
        })
    }

    pub fn get<K, V>(&mut self, k: K) -> RedisResult<Option<V>>
    where
        K: ToRedisArgs,
        V: FromRedisValue,
    {
        self.reconnect()?;
        match self.connection.get(k) {
            Ok(v) => Ok(v),
            Err(err) => {
                self.set_reconnect();
                Err(err)
            }
        }
    }

    pub fn set<K, V, P>(&mut self, p: P) -> RedisResult<()>
    where
        P: LeftRight<L = K, R = V>,
        K: ToRedisArgs + Clone,
        V: ToRedisArgs + Clone,
    {
        self.reconnect()?;
        let (k, v) = p.as_tuple();
        match self.connection.set::<K, V, ()>(k, v) {
            Ok(_) => Ok(()),
            Err(e) => {
                self.set_reconnect();
                Err(e)
            }
        }
    }

    pub fn set_all<K, V, T, U>(&mut self, entries: U) -> RedisResult<()>
    where
        K: ToRedisArgs,
        V: ToRedisArgs,
        T: LeftRight<L = K, R = V>,
        U: IntoIterator<Item = T>,
    {
        self.reconnect()?;
        let entries: Vec<(K, V)> = entries.into_iter().map(|entry| entry.as_tuple()).collect();
        match self.connection.set_multiple::<K, V, ()>(entries.as_slice()) {
            Ok(_) => Ok(()),
            Err(err) => {
                self.set_reconnect();
                Err(err)
            }
        }
    }

    pub fn publish<C, M, P>(&mut self, p: P) -> RedisResult<()>
    where
        C: ToRedisArgs,
        M: ToRedisArgs,
        P: LeftRight<L = C, R = M>,
    {
        self.reconnect()?;
        let (channel, message) = p.as_tuple();
        match self.connection.publish::<C, M, ()>(channel, message) {
            Ok(_) => Ok(()),
            Err(err) => {
                self.set_reconnect();
                Err(err)
            }
        }
    }

    pub async fn subscribe<C>(&mut self, channel: C) -> RedisResult<PubSub>
    where
        C: ToRedisArgs,
    {
        let conn = self.client.get_async_connection().await?;
        let mut pubsub = conn.into_pubsub();
        pubsub.subscribe(channel).await?;
        Ok(pubsub)
    }

    fn reconnect(&mut self) -> RedisResult<()> {
        if !self.reconnect {
            return Ok(());
        }
        self.connection = self.client.get_connection()?;
        self.reconnect = false;
        Ok(())
    }

    fn set_reconnect(&mut self) {
        self.reconnect = true;
    }
}
