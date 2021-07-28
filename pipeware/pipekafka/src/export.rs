use crate::config::{create_kafka_client, KafkaClientConfig, KafkaProducerClientConfig};
use async_trait::async_trait;
use pipebase::{
    common::{ConfigInto, FromConfig, FromPath, LeftRight},
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
impl<K, P, T> Export<T, KafkaProducerConfig> for KafkaProducer
where
    K: ToBytes + Send + Sync,
    P: ToBytes + Send + Sync,
    T: LeftRight<L = Option<K>, R = P> + Send + 'static,
{
    async fn export(&mut self, t: T) -> anyhow::Result<()> {
        let record = Self::create_record(&self.topic, &t);
        match self.client.send(record, self.queue_timeout).await {
            Ok(_) => Ok(()),
            Err((e, _)) => return Err(e.into()),
        }
    }
}

impl KafkaProducer {
    fn create_record<'a, K, P, T>(topic: &'a str, t: &'a T) -> FutureRecord<'a, K, P>
    where
        K: ToBytes,
        P: ToBytes,
        T: LeftRight<L = Option<K>, R = P>,
    {
        let key = t.left().as_ref();
        let payload = t.right();
        let record = FutureRecord::to(topic).payload(payload);
        match key {
            Some(key) => record.key(key),
            None => record,
        }
    }
}
