use pipebase::common::{Convert, GroupAs};
use rdkafka::message::ToBytes;
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct KafkaRecord<K, P>
where
    K: ToBytes,
    P: ToBytes,
{
    pub key: Option<K>,
    pub payload: P,
}

/// Convert bytes into payload only record
impl<K> Convert<Vec<u8>> for KafkaRecord<K, Vec<u8>>
where
    K: ToBytes,
{
    fn convert(rhs: Vec<u8>) -> Self {
        KafkaRecord::new(None, rhs)
    }
}

impl<K, P> KafkaRecord<K, P>
where
    K: ToBytes,
    P: ToBytes,
{
    fn new(key: Option<K>, payload: P) -> Self {
        KafkaRecord { key, payload }
    }
}

pub trait IntoKafkaRecord<K, T>
where
    K: Clone + ToBytes,
    T: GroupAs<K>,
{
    fn key(t: &T) -> K {
        t.group().to_owned()
    }

    fn serialize(t: &T) -> anyhow::Result<Vec<u8>>;

    fn convert(t: &T) -> anyhow::Result<KafkaRecord<K, Vec<u8>>> {
        let payload = Self::serialize(t)?;
        Ok(KafkaRecord::new(Some(Self::key(t)), payload))
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
