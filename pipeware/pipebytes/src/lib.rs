mod convert;

pub use convert::*;

pub trait FromBytes: Sized {
    fn from_bytes(bytes: Vec<u8>) -> anyhow::Result<Self>;
}

pub trait AsBytes {
    fn as_bytes(&self) -> anyhow::Result<Vec<u8>>;
}
