use crate::utils::{FromBytes, IntoBytes};
use pipebase::{GroupTable, LeftRight};
use rocksdb::{DBWithThreadMode, SingleThreaded, WriteBatch, DB};

pub struct RocksDBGroupTable<C> {
    cache: C,
    db: DBWithThreadMode<SingleThreaded>,
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
        match self.get::<K, V>(gid)? {
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
        self.put_all(self.cache.to_owned())
    }
}

impl<C> RocksDBGroupTable<C> {
    pub fn new(path: String, cache: C) -> anyhow::Result<Self> {
        let db = DB::open_default(path)?;
        Ok(RocksDBGroupTable { cache: cache, db })
    }

    pub fn get<K, V>(&self, key: &K) -> anyhow::Result<Option<V>>
    where
        K: IntoBytes,
        V: FromBytes,
    {
        match self.db.get(key.into_bytes()?)? {
            Some(bytes) => Ok(Some(V::from_bytes(bytes)?)),
            None => Ok(None),
        }
    }

    pub fn put_all<K, V, T, U>(&mut self, entries: U) -> anyhow::Result<()>
    where
        K: IntoBytes,
        V: IntoBytes,
        T: LeftRight<L = K, R = V>,
        U: IntoIterator<Item = T>,
    {
        let mut batch = WriteBatch::default();
        for entry in entries.into_iter() {
            let key = entry.left();
            let value = entry.right();
            batch.put(key.into_bytes()?, value.into_bytes()?);
        }
        self.db.write(batch)?;
        Ok(())
    }
}

pub struct RedisGroupTable<C> {
    cache: C,
}
