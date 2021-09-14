A [`pipebase`] plugin using [`rust-rdkafka`]
### Pipe Configs
| type | example |
| ---- | ------- |
| `KafkaConsumerConfig` | [`kafka_consumer`] |
| `KafkaPartitionedProducerConfig` | TBD |
| `KafkaProducerConfig` | [`kafka_producer`] |

[`pipebase`]: https://github.com/pipebase/pipebase
[`rust-rdkafka`]: https://github.com/fede1024/rust-rdkafka
[`kafka_consumer`]: https://github.com/pipebase/pipebase/blob/main/examples/consume_kafka/catalogs/kafka_consumer.yml
[`kafka_producer`]: https://github.com/pipebase/pipebase/blob/main/examples/ingest_kafka/catalogs/kafka_producer.yml