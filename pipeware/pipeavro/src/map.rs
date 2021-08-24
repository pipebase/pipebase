use async_trait::async_trait;
use avro_rs::{from_value, Codec, Reader, Schema, Writer};
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    map::Map,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Deserialize)]
enum Compression {
    Null,
    Deflate,
    Snappy,
}

fn get_codec(compression: Compression) -> Codec {
    match compression {
        Compression::Null => Codec::Null,
        Compression::Deflate => Codec::Deflate,
        Compression::Snappy => Codec::Snappy,
    }
}

#[derive(Deserialize)]
pub struct AvroSerConfig {
    compression: Compression,
    schema: String,
}

impl FromPath for AvroSerConfig {}

impl ConfigInto<AvroSer> for AvroSerConfig {}

pub struct AvroSer {
    codec: Codec,
    schema: Schema,
}

#[async_trait]
impl FromConfig<AvroSerConfig> for AvroSer {
    async fn from_config(config: AvroSerConfig) -> anyhow::Result<Self> {
        Ok(AvroSer {
            codec: get_codec(config.compression),
            schema: Schema::parse_str(&config.schema)?,
        })
    }
}

impl AvroSer {
    fn serialize<T: Serialize>(
        items: Vec<T>,
        schema: &Schema,
        codec: Codec,
    ) -> anyhow::Result<Vec<u8>> {
        let mut writer = Writer::with_codec(schema, Vec::new(), codec);
        for item in items {
            writer.append_ser(item)?;
        }
        Ok(writer.into_inner()?)
    }
}

#[async_trait]
impl<T> Map<Vec<T>, Vec<u8>, AvroSerConfig> for AvroSer
where
    T: Serialize + Send + 'static,
{
    async fn map(&mut self, data: Vec<T>) -> anyhow::Result<Vec<u8>> {
        let schema = &self.schema;
        let codec = self.codec;
        Self::serialize(data, schema, codec)
    }
}

#[derive(Deserialize)]
pub struct AvroDeserConfig {}

#[async_trait]
impl FromPath for AvroDeserConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(AvroDeserConfig {})
    }
}

impl ConfigInto<AvroDeser> for AvroDeserConfig {}

pub struct AvroDeser {}

#[async_trait]
impl FromConfig<AvroDeserConfig> for AvroDeser {
    async fn from_config(_config: AvroDeserConfig) -> anyhow::Result<Self> {
        Ok(AvroDeser {})
    }
}

impl AvroDeser {
    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> anyhow::Result<Vec<T>> {
        let reader = Reader::new(bytes)?;
        let mut items: Vec<T> = vec![];
        for value in reader {
            items.push(from_value::<T>(&value?)?);
        }
        Ok(items)
    }
}

#[async_trait]
impl<T> Map<Vec<u8>, Vec<T>, AvroDeserConfig> for AvroDeser
where
    T: DeserializeOwned,
{
    async fn map(&mut self, data: Vec<u8>) -> anyhow::Result<Vec<T>> {
        Self::deserialize(&data)
    }
}
