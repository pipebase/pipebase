use pipebase::LeftRight;
use redis::{Client, Commands, Connection, RedisResult, ToRedisArgs};

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

    pub fn set<K, V, P>(&mut self, p: P) -> RedisResult<()>
    where
        P: LeftRight<L = K, R = V>,
        K: ToRedisArgs + Clone,
        V: ToRedisArgs + Clone,
    {
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

    pub fn reconnect(&mut self) -> RedisResult<()> {
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
