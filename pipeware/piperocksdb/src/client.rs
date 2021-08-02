use pipebase::common::LeftRight;
use pipebytes::{AsBytes, FromBytes, IntoBytes};
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
        K: AsBytes,
        V: FromBytes,
    {
        match self.db.get(key.as_bytes()?)? {
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
            let (key, value) = entry.into_tuple();
            batch.put(key.into_bytes()?, value.into_bytes()?);
        }
        self.db.write(batch)?;
        Ok(())
    }
}
