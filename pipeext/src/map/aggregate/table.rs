use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use pipebase::{GroupTable, LeftRight};
use rocksdb::{DBWithThreadMode, SingleThreaded, WriteBatch, DB};
use std::io::Cursor;

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

pub trait FromBytes: Sized {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self>;
}

pub trait IntoBytes {
    fn into_bytes(&self) -> anyhow::Result<Vec<u8>>;
}

impl FromBytes for u32 {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let mut rdr = Cursor::new(bytes);
        let value = rdr.read_u32::<BigEndian>()?;
        Ok(value)
    }
}

impl IntoBytes for u32 {
    fn into_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut wtr = vec![];
        wtr.write_u32::<BigEndian>(self.to_owned())?;
        Ok(wtr)
    }
}

impl FromBytes for i32 {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let mut rdr = Cursor::new(bytes);
        let value = rdr.read_i32::<BigEndian>()?;
        Ok(value)
    }
}

impl IntoBytes for i32 {
    fn into_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut wtr = vec![];
        wtr.write_i32::<BigEndian>(self.to_owned())?;
        Ok(wtr)
    }
}

impl FromBytes for u64 {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let mut rdr = Cursor::new(bytes);
        let value = rdr.read_u64::<BigEndian>()?;
        Ok(value)
    }
}

impl IntoBytes for u64 {
    fn into_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut wtr = vec![];
        wtr.write_u64::<BigEndian>(self.to_owned())?;
        Ok(wtr)
    }
}

impl FromBytes for i64 {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let mut rdr = Cursor::new(bytes);
        let value = rdr.read_i64::<BigEndian>()?;
        Ok(value)
    }
}

impl IntoBytes for i64 {
    fn into_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut wtr = vec![];
        wtr.write_i64::<BigEndian>(self.to_owned())?;
        Ok(wtr)
    }
}

impl FromBytes for f32 {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let mut rdr = Cursor::new(bytes);
        let value = rdr.read_f32::<BigEndian>()?;
        Ok(value)
    }
}

impl IntoBytes for f32 {
    fn into_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut wtr = vec![];
        wtr.write_f32::<BigEndian>(self.to_owned())?;
        Ok(wtr)
    }
}

impl FromBytes for f64 {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let mut rdr = Cursor::new(bytes);
        let value = rdr.read_f64::<BigEndian>()?;
        Ok(value)
    }
}

impl IntoBytes for f64 {
    fn into_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut wtr = vec![];
        wtr.write_f64::<BigEndian>(self.to_owned())?;
        Ok(wtr)
    }
}

impl FromBytes for String {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let s = String::from_utf8(bytes)?;
        Ok(s)
    }
}

impl IntoBytes for String {
    fn into_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(self.as_bytes().to_vec())
    }
}
