use std::collections::HashSet;
use std::fmt::Display;

use crate::models::{Entity, EntityAccept, VisitEntity};
use serde::Deserialize;
use strum::{Display, EnumString};

use super::data::{data_ty_to_literal, DataType};
use super::default_mqtt_dependency;
use super::dependency::{
    default_avro_dependency, default_cql_dependency, default_csv_dependency,
    default_dynamodb_dependency, default_json_dependency, default_kafka_dependency,
    default_kube_dependency, default_mysql_dependency, default_psql_dependency,
    default_redis_dependency, default_reqwest_dependency, default_rocksdb_dependency,
    default_s3_dependency, default_sns_dependency, default_sqs_dependency, default_warp_dependency,
    Dependency, UseCrate,
};
use super::meta::{meta_to_literal, meta_value_str, meta_value_usize, Meta};

#[derive(Clone, Display, EnumString, PartialEq, Debug, Deserialize)]
pub enum PipeType {
    #[strum(to_string = "listener")]
    Listener,
    #[strum(to_string = "poller")]
    Poller,
    #[strum(to_string = "mapper")]
    Mapper,
    #[strum(to_string = "collector")]
    Collector,
    #[strum(to_string = "selector")]
    Selector,
    #[strum(to_string = "exporter")]
    Exporter,
    #[strum(to_string = "streamer")]
    Streamer,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PipeConfig {
    ty: String,
    path: Option<String>,
}

impl PipeConfig {
    pub fn get_path(&self) -> Option<&String> {
        self.path.as_ref()
    }
    pub fn get_config_type(&self) -> &String {
        &self.ty
    }
}

impl Display for PipeConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.path {
            Some(ref path) => write!(f, "{{ type: {}, path: {} }}", self.ty, path),
            None => write!(f, "{{ type: {} }}", self.ty),
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct Pipe {
    name: String,
    ty: PipeType,
    config: PipeConfig,
    // pipe channel buffer
    buffer: Option<usize>,
    // upstream pipe names
    upstreams: Option<Vec<String>>,
    // output data type
    output: Option<DataType>,
}

impl Pipe {
    pub fn init(&mut self) {
        if self.upstreams.is_none() {
            self.upstreams = Some(Vec::new())
        }
    }

    pub fn is_source(&self) -> bool {
        matches!(&self.ty, PipeType::Listener | PipeType::Poller)
    }

    pub fn is_sink(&self) -> bool {
        matches!(&self.ty, PipeType::Exporter)
    }

    fn get_name_meta(&self) -> Meta {
        meta_value_str("name", &self.name, false)
    }

    fn get_type_meta(&self) -> Meta {
        meta_value_str("ty", &self.ty.to_string(), false)
    }

    fn get_config_meta(&self) -> Meta {
        let mut config_metas = vec![meta_value_str("ty", self.config.get_config_type(), false)];
        if let Some(path) = self.config.get_path() {
            config_metas.push(meta_value_str("path", path, false));
        };
        Meta::List {
            name: "config".to_owned(),
            metas: config_metas,
        }
    }

    fn get_upstream_meta(&self) -> Option<Meta> {
        let upstreams = self.upstreams.as_ref().expect("upstreams not inited");
        if upstreams.is_empty() {
            return None;
        }
        Some(meta_value_str("upstream", &upstreams.join(", "), false))
    }

    fn get_output_data_type_meta(&self) -> Option<Meta> {
        let output = match self.output {
            Some(ref output) => output,
            None => return None,
        };
        Some(meta_value_str("output", &data_ty_to_literal(output), false))
    }

    pub(crate) fn get_output_data_type(&self) -> Option<&DataType> {
        self.output.as_ref()
    }

    pub(crate) fn has_output(&self) -> bool {
        self.output.is_some()
    }

    pub(crate) fn get_channel_buffer_meta(&self) -> Option<Meta> {
        let buffer = match self.buffer {
            Some(ref buffer) => buffer,
            None => return None,
        };
        Some(meta_value_usize("buffer", buffer))
    }

    pub fn filter_upstreams(&mut self, pipe_id_filter: &HashSet<String>) {
        let upstreams = match self.upstreams {
            Some(ref upstreams) => upstreams,
            None => return,
        };
        self.upstreams = Some(
            upstreams
                .to_owned()
                .into_iter()
                .filter(|id| pipe_id_filter.contains(id))
                .collect(),
        )
    }
}

impl Entity for Pipe {
    fn get_id(&self) -> String {
        self.name.to_owned()
    }

    fn list_dependency(&self) -> Vec<String> {
        match self.upstreams {
            Some(ref upstreams) => upstreams.to_owned(),
            None => vec![],
        }
    }

    // to pipe meta
    fn to_literal(&self, indent: usize) -> String {
        let mut metas: Vec<Meta> = vec![
            self.get_name_meta(),
            self.get_type_meta(),
            self.get_config_meta(),
        ];
        if let Some(meta) = self.get_upstream_meta() {
            metas.push(meta)
        };
        if let Some(meta) = self.get_output_data_type_meta() {
            metas.push(meta)
        };
        if let Some(meta) = self.get_channel_buffer_meta() {
            metas.push(meta)
        };
        let meta = Meta::List {
            name: "pipe".to_owned(),
            metas,
        };
        meta_to_literal(&meta, indent)
    }
}

impl Display for Pipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Name:   {}", self.name)?;
        writeln!(f, "Type:   {}", self.ty)?;
        writeln!(f, "Config: {}", self.config)?;
        let upstream = match self.upstreams {
            Some(ref upstreams) => upstreams.join(", "),
            None => "".to_owned(),
        };
        writeln!(f, "Upstream: [{}]", upstream)
    }
}

impl<V: VisitEntity<Pipe>> EntityAccept<V> for Pipe {}

impl UseCrate for Pipe {
    fn get_crate(&self) -> Option<Dependency> {
        let config_ty = self.config.get_config_type().as_str();
        match config_ty {
            "AvroDeserConfig" | "AvroSerConfig" => Some(default_avro_dependency()),
            "CqlPreparedWriterConfig" | "CqlWriterConfig" => Some(default_cql_dependency()),
            "CsvDeserConfig" | "CsvSerConfig" => Some(default_csv_dependency()),
            "JsonDeserConfig" | "JsonRecordSerConfig" | "JsonSerConfig" => {
                Some(default_json_dependency())
            }
            "KafkaConsumerConfig" | "KafkaPartitionedProducerConfig" | "KafkaProducerConfig" => {
                Some(default_kafka_dependency())
            }
            "KubeEventReaderConfig" | "KubeLogReaderConfig" => Some(default_kube_dependency()),
            "MySQLPreparedWriterConfig" | "MySQLWriterConfig" => Some(default_mysql_dependency()),
            "PsqlPreparedWriterConfig" | "PsqlWriterConfig" => Some(default_psql_dependency()),
            "RedisPublisherConfig"
            | "RedisStringBatchWriterConfig"
            | "RedisStringWriterConfig"
            | "RedisSubscriberConfig"
            | "RedisUnorderedGroupAddAggregatorConfig" => Some(default_redis_dependency()),
            "ReqwestGetterConfig" | "ReqwestPosterConfig" | "ReqwestQueryConfig" => {
                Some(default_reqwest_dependency())
            }
            "RocksDBUnorderedGroupAddAggregatorConfig" => Some(default_rocksdb_dependency()),
            "WarpIngestionServerConfig" => Some(default_warp_dependency()),
            "DynamoDBWriterConfig" => Some(default_dynamodb_dependency()),
            "S3WriterConfig" => Some(default_s3_dependency()),
            "SnsPublisherConfig" => Some(default_sns_dependency()),
            "SqsMessageReceiverConfig" => Some(default_sqs_dependency()),
            "MqttPublisherConfig" | "MqttSubscriberConfig" => Some(default_mqtt_dependency()),
            _ => None,
        }
    }
}
