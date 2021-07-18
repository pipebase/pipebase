use crate::constants::*;
use pipebase::Period;
use rdkafka::{
    client::ClientContext,
    config::{ClientConfig, FromClientConfigAndContext, RDKafkaLogLevel},
    error::KafkaResult,
};
use std::collections::HashMap;

use serde::Deserialize;

/// [reference](https://docs.confluent.io/3.2.1/clients/librdkafka/CONFIGURATION_8md.html)
/// Kafka client base config for consumer and producer
#[derive(Clone, Deserialize)]
pub struct KafkaClientConfig {
    bootstrap_servers: String,
}

impl From<KafkaClientConfig> for HashMap<&str, String> {
    fn from(config: KafkaClientConfig) -> Self {
        let mut params = HashMap::new();
        params.insert(BOOTSTRAP_SERVERS, config.bootstrap_servers.to_owned());
        params
    }
}

/// Kafka client config for consumer
#[derive(Clone, Deserialize)]
pub struct KafkaConsumerClientConfig {
    group_id: Option<String>,
    enable_partition_eof: Option<bool>,
    session_timeout_ms: Option<u32>,
    enable_auto_commit: Option<bool>,
    topics: Vec<String>,
}

impl KafkaConsumerClientConfig {
    pub fn get_topics(&self) -> Vec<&str> {
        self.topics.iter().map(|t| t.as_str()).collect()
    }
}

impl From<KafkaConsumerClientConfig> for HashMap<&str, String> {
    fn from(config: KafkaConsumerClientConfig) -> Self {
        let mut params: HashMap<&str, String> = HashMap::new();
        match config.group_id {
            Some(ref group_id) => params.insert(GROUP_ID, group_id.to_owned()),
            None => None,
        };
        match config.enable_partition_eof {
            Some(ref enable_partition_eof) => {
                params.insert(ENABLE_PARTITION_EOF, format!("{}", enable_partition_eof))
            }
            None => None,
        };
        match config.session_timeout_ms {
            Some(ref session_timeout_ms) => {
                params.insert(SESSION_TIMEOUT_MS, format!("{}", session_timeout_ms))
            }
            None => None,
        };
        match config.enable_auto_commit {
            Some(ref enable_auto_commit) => {
                params.insert(ENABLE_AUTO_COMMIT, format!("{}", enable_auto_commit))
            }
            None => None,
        };
        params
    }
}

/// Kafka client config for producer
#[derive(Clone, Deserialize)]
pub struct KafkaProducerClientConfig {
    topic: String,
    queue_timeout: Period,
    message_timeout_ms: Option<u32>,
}

impl KafkaProducerClientConfig {
    pub fn get_topic(&self) -> &str {
        self.topic.as_str()
    }

    pub fn get_queue_timeout(&self) -> Period {
        self.queue_timeout.to_owned()
    }
}

impl From<KafkaProducerClientConfig> for HashMap<&str, String> {
    fn from(config: KafkaProducerClientConfig) -> Self {
        let mut params: HashMap<&str, String> = HashMap::new();
        match config.message_timeout_ms {
            Some(ref message_timeout_ms) => {
                params.insert(MESSAGE_TIMEOUT_MS, format!("{}", message_timeout_ms))
            }
            None => None,
        };
        params
    }
}

/// Create kafka consumer or producer with context
pub fn create_kafka_client<C, T>(paras: HashMap<&str, String>, context: C) -> KafkaResult<T>
where
    C: ClientContext,
    T: FromClientConfigAndContext<C>,
{
    let mut config = ClientConfig::new();
    for (prop, value) in paras {
        config.set(prop, value);
    }
    config.set_log_level(RDKafkaLogLevel::Error);
    config.create_with_context::<C, T>(context)
}
