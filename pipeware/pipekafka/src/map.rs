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
impl<K, T> Map<T, Pair<K, Vec<u8>>, KafkaJsonRecordConverterConfig> for KafkaJsonRecordConverter
where
    K: Clone + ToBytes + ?Sized,
    T: GroupAs<K> + Serialize + Send + 'static,
{
    async fn map(&mut self, data: T) -> anyhow::Result<Pair<K, Vec<u8>>> {
        Ok(Self::convert(&data)?)
    }
}
