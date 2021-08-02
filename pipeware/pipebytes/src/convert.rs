use crate::{AsBytes, FromBytes};
use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use pipebase::common::{Averagef32, Count32};
use std::io::Cursor;

impl FromBytes for u32 {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let mut rdr = Cursor::new(bytes);
        let value = rdr.read_u32::<BigEndian>()?;
        Ok(value)
    }
}

impl AsBytes for u32 {
    fn as_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut wtr = vec![];
        wtr.write_u32::<BigEndian>(self.to_owned())?;
        Ok(wtr)
    }
}

impl FromBytes for Count32 {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let mut rdr = Cursor::new(bytes);
        let value = rdr.read_u32::<BigEndian>()?;
        Ok(Count32::new(value))
    }
}

impl AsBytes for Count32 {
    fn as_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut wtr = vec![];
        wtr.write_u32::<BigEndian>(self.get())?;
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

impl AsBytes for i32 {
    fn as_bytes(&self) -> anyhow::Result<Vec<u8>> {
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

impl AsBytes for u64 {
    fn as_bytes(&self) -> anyhow::Result<Vec<u8>> {
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

impl AsBytes for i64 {
    fn as_bytes(&self) -> anyhow::Result<Vec<u8>> {
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

impl AsBytes for f32 {
    fn as_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut wtr = vec![];
        wtr.write_f32::<BigEndian>(self.to_owned())?;
        Ok(wtr)
    }
}

impl FromBytes for Averagef32 {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let mut rdr = Cursor::new(bytes);
        let sum = rdr.read_f32::<BigEndian>()?;
        let count = rdr.read_f32::<BigEndian>()?;
        Ok(Averagef32::new(sum, count))
    }
}

impl AsBytes for Averagef32 {
    fn as_bytes(&self) -> anyhow::Result<Vec<u8>> {
        let mut wtr = vec![];
        wtr.write_f32::<BigEndian>(self.sum())?;
        wtr.write_f32::<BigEndian>(self.count())?;
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

impl AsBytes for f64 {
    fn as_bytes(&self) -> anyhow::Result<Vec<u8>> {
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

impl AsBytes for String {
    fn as_bytes(&self) -> anyhow::Result<Vec<u8>> {
        Ok(self.as_bytes().to_vec())
    }
}
