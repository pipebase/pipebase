use crate::{
    config::{create_kafka_client, KafkaClientConfig, KafkaProducerClientConfig},
    record::KafkaRecord,
};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath},
    export::Export,
};
use rdkafka::{
    message::ToBytes,
    producer::{DefaultProducerContext, FutureProducer, FutureRecord},
    util::TokioRuntime,
};
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};

#[derive(Clone, Deserialize)]
pub struct KafkaProducerConfig {
    base: KafkaClientConfig,
    producer: KafkaProducerClientConfig,
}

impl From<KafkaProducerConfig> for HashMap<&str, String> {
    fn from(config: KafkaProducerConfig) -> Self {
        let mut params: HashMap<&str, String> = config.base.into();
        let pparams: HashMap<&str, String> = config.producer.into();
        params.extend(pparams);
        params
    }
}

impl FromPath for KafkaProducerConfig {}

impl ConfigInto<KafkaProducer> for KafkaProducerConfig {}

type DefaultAsyncProducer = FutureProducer<DefaultProducerContext, TokioRuntime>;
pub struct KafkaProducer {
    client: DefaultAsyncProducer,
    queue_timeout: Duration,
    topic: String,
}

#[async_trait]
impl FromConfig<KafkaProducerConfig> for KafkaProducer {
    async fn from_config(config: KafkaProducerConfig) -> anyhow::Result<Self> {
        let params: HashMap<&str, String> = config.to_owned().into();
        let producer: DefaultAsyncProducer = create_kafka_client::<
            DefaultProducerContext,
            DefaultAsyncProducer,
        >(params, DefaultProducerContext)?;
        let queue_timeout: Duration = config.producer.get_queue_timeout().into();
        let topic = config.producer.get_topic().to_owned();
        Ok(KafkaProducer {
            client: producer,
            queue_timeout,
            topic,
        })
    }
}

#[async_trait]
impl<K, P> Export<KafkaRecord<K, P>, KafkaProducerConfig> for KafkaProducer
where
    K: ToBytes + Send + Sync + 'static,
    P: ToBytes + Send + Sync + 'static,
{
    async fn export(&mut self, r: KafkaRecord<K, P>) -> anyhow::Result<()> {
        let record = self.create_record(&r);
        match self.client.send(record, self.queue_timeout).await {
            Ok(_) => Ok(()),
            Err((e, _)) => return Err(e.into()),
        }
    }
}

impl KafkaProducer {
    fn create_record<'a, K, P>(&'a self, r: &'a KafkaRecord<K, P>) -> FutureRecord<K, P>
    where
        K: ToBytes,
        P: ToBytes,
    {
        let key = r.key.as_ref();
        let ref payload = r.payload;
        let future_record = FutureRecord::to(self.topic.as_str()).payload(payload);
        match key {
            Some(key) => future_record.key(key),
            None => future_record,
        }
    }
}
