use crate::client::RocksDBClient;
use pipebase::common::GroupTable;
use pipebytes::{FromBytes, IntoBytes};
pub struct RocksDBGroupTable<C> {
    cache: C,
    client: RocksDBClient,
}

impl<K, V, C> IntoIterator for RocksDBGroupTable<C>
where
    C: IntoIterator<Item = (K, V)>,
{
    type Item = (K, V);
    type IntoIter = C::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.cache.into_iter()
    }
}

impl<K, V, C> GroupTable<K, V> for RocksDBGroupTable<C>
where
    C: GroupTable<K, V> + Clone,
    K: IntoBytes + Clone,
    V: IntoBytes + FromBytes,
{
    fn contains_group(&mut self, gid: &K) -> anyhow::Result<bool> {
        if self.cache.contains_group(gid)? {
            return Ok(true);
        }
        // load on demand
        match self.client.get::<K, V>(gid)? {
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
        self.client.put_all(self.cache.to_owned())
    }
}

impl<C> RocksDBGroupTable<C> {
    pub fn new(path: &str, cache: C) -> anyhow::Result<Self> {
        let client = RocksDBClient::new(path)?;
        Ok(RocksDBGroupTable { cache, client })
    }
}
