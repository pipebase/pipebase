use crate::client::RedisClient;
use pipebase::GroupTable;
use redis::{FromRedisValue, ToRedisArgs};

pub struct RedisGroupTable<C> {
    cache: C,
    client: RedisClient,
}

impl<K, V, C> IntoIterator for RedisGroupTable<C>
where
    C: IntoIterator<Item = (K, V)>,
{
    type Item = (K, V);
    type IntoIter = C::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.cache.into_iter()
    }
}

impl<K, V, C> GroupTable<K, V> for RedisGroupTable<C>
where
    C: GroupTable<K, V> + Clone,
    K: ToRedisArgs + Clone,
    V: ToRedisArgs + FromRedisValue,
{
    fn contains_group(&mut self, gid: &K) -> anyhow::Result<bool> {
        if self.cache.contains_group(gid)? {
            return Ok(true);
        }
        // load on demand
        match self.client.get::<K, V>(gid.to_owned())? {
            Some(value) => {
                self.cache.insert_group(gid.to_owned(), value)?;
                Ok(true)
            }
            None => Ok(false),
        }
    }

    fn get_group(&mut self, gid: &K) -> anyhow::Result<Option<&mut V>> {
        if !self.contains_group(gid)? {
            return Ok(None);
        }
        self.cache.get_group(gid)
    }

    fn insert_group(&mut self, gid: K, v: V) -> anyhow::Result<Option<V>> {
        self.cache.insert_group(gid, v)
    }

    fn persist_groups(&mut self) -> anyhow::Result<()> {
        self.client.set_all(self.cache.to_owned())?;
        Ok(())
    }
}

impl<C> RedisGroupTable<C> {
    pub fn new(url: String, cache: C) -> anyhow::Result<Self> {
        Ok(RedisGroupTable {
            cache,
            client: RedisClient::new(url)?,
        })
    }
}
