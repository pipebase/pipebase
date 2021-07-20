use crate::byteops::{FromBytes, IntoBytes};
use pipebase::common::LeftRight;
use rocksdb::{DBWithThreadMode, SingleThreaded, WriteBatch, DB};

pub struct RocksDBClient {
    db: DBWithThreadMode<SingleThreaded>,
}

impl RocksDBClient {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let db = DB::open_default(path)?;
        Ok(RocksDBClient { db })
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
