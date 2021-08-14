use crate::record::{IntoKafkaRecord, KafkaJsonRecordConverter};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, GroupAs, Pair},
    map::Map,
};
use rdkafka::message::ToBytes;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct KafkaJsonRecordConverterConfig {}

#[async_trait]
impl FromPath for KafkaJsonRecordConverterConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(KafkaJsonRecordConverterConfig {})
    }
}

impl ConfigInto<KafkaJsonRecordConverter> for KafkaJsonRecordConverterConfig {}

#[async_trait]
impl FromConfig<KafkaJsonRecordConverterConfig> for KafkaJsonRecordConverter {
    async fn from_config(_config: KafkaJsonRecordConverterConfig) -> anyhow::Result<Self> {
        Ok(KafkaJsonRecordConverter {})
    }
}

#[async_trait]
impl<K, T> Map<T, Pair<Option<K>, Vec<u8>>, KafkaJsonRecordConverterConfig>
    for KafkaJsonRecordConverter
where
    K: Clone + ToBytes,
    T: GroupAs<K> + Serialize + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Pair<Option<K>, Vec<u8>>> {
        Ok(Self::convert(&data)?)
    }
}

#[derive(Deserialize)]
pub struct KafkaUnsignedBytesRecordConverterConfig {}

#[async_trait]
impl FromPath for KafkaUnsignedBytesRecordConverterConfig {
    async fn from_path<P>(_path: P) -> anyhow::Result<Self>
    where
        P: AsRef<std::path::Path> + Send,
    {
        Ok(KafkaUnsignedBytesRecordConverterConfig {})
    }
}

impl ConfigInto<KafkaUnsignedBytesRecordConverter> for KafkaUnsignedBytesRecordConverterConfig {}

#[async_trait]
impl FromConfig<KafkaUnsignedBytesRecordConverterConfig> for KafkaUnsignedBytesRecordConverter {
    async fn from_config(_config: KafkaUnsignedBytesRecordConverterConfig) -> anyhow::Result<Self> {
        Ok(KafkaUnsignedBytesRecordConverter {})
    }
}

pub struct KafkaUnsignedBytesRecordConverter {}

#[async_trait]
impl Map<Vec<u8>, Pair<Option<String>, Vec<u8>>, KafkaUnsignedBytesRecordConverterConfig>
    for KafkaUnsignedBytesRecordConverter
{
    async fn map(&mut self, data: Vec<u8>) -> anyhow::Result<Pair<Option<String>, Vec<u8>>> {
        Ok(Pair::new(None, data))
    }
}
