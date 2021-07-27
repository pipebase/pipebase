use byteorder::{BigEndian, ReadBytesExt, WriteBytesExt};
use crate::{FromBytes, IntoBytes};
use pipebase::common::{Averagef32, Count32};
use std::io::Cursor;

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

impl FromBytes for Count32 {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let mut rdr = Cursor::new(bytes);
        let value = rdr.read_u32::<BigEndian>()?;
        Ok(Count32::new(value))
    }
}

impl IntoBytes for Count32 {
    fn into_bytes(&self) -> anyhow::Result<Vec<u8>> {
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

impl FromBytes for Averagef32 {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self> {
        let mut rdr = Cursor::new(bytes);
        let sum = rdr.read_f32::<BigEndian>()?;
        let count = rdr.read_f32::<BigEndian>()?;
        Ok(Averagef32::new(sum, count))
    }
}

impl IntoBytes for Averagef32 {
    fn into_bytes(&self) -> anyhow::Result<Vec<u8>> {
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