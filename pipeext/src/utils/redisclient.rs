use pipebase::LeftRight;
use redis::{Client, Commands, Connection, FromRedisValue, RedisResult, ToRedisArgs};

pub struct RedisClient {
    client: Client,
    connection: Connection,
    // flag indicate whether to reconnect
    reconnect: bool,
}

impl RedisClient {
    pub fn new(url: &str) -> anyhow::Result<Self> {
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
        let k = p.left().to_owned();
        let v = p.right().to_owned();
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
