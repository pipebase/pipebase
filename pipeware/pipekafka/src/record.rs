use pipebase::common::{GroupAs, Pair};
use rdkafka::message::ToBytes;
use serde::Serialize;
pub trait IntoKafkaRecord<K, T>
where
    K: Clone + ToBytes,
    T: GroupAs<K>,
{
    fn key(t: &T) -> K {
        t.group().to_owned()
    }

    fn serialize(t: &T) -> anyhow::Result<Vec<u8>>;

    fn convert(t: &T) -> anyhow::Result<Pair<Option<K>, Vec<u8>>> {
        let payload = Self::serialize(t)?;
        Ok(Pair::new(Some(Self::key(t)), payload))
    }
}

pub struct KafkaJsonRecordConverter {}

impl<K, T> IntoKafkaRecord<K, T> for KafkaJsonRecordConverter
where
    K: Clone + ToBytes,
    T: GroupAs<K> + Serialize,
{
    fn serialize(t: &T) -> anyhow::Result<Vec<u8>> {
        let bytes = serde_json::to_vec(t)?;
        Ok(bytes)
    }
}
