[`pipeware`] is repository for pipe plugins using third party SDK

## Plugins
Pipes
| pipe type | implementation | input / output | example |
| --------- | -------------- | -------------- | ------- |
| `Listener`  | WarpIngestionServer | doc | `ingest_*` |
| `Listener` | KafkaConsumer | doc | [`consume_kafka`] |
| `Mapper` | JsonSer | doc |  |
| `Mapper` | JsonDeser | doc | `ingest_*` |
| `Mapper` | RedisUnorderedGroupAddAggregator | doc | [`group_sum_redis`] |
| `Mapper` | RocksDBUnorderedGroupAddAggregator | doc | [`group_sum_rocksdb`] |
| `Mapper` | KafkaJsonRecordConverter | doc | [`ingest_kafka`] |
| `Exporter` | CqlWriter | doc | [`ingest_cassandra`] |
| `Exporter` | PsqlWriter | doc | [`ingest_postgres`] |
| `Exporter` | RedisWriter | doc | [`ingest_redis`] |
| `Exporter` | ReqwestPoster | doc | [`relay`] |
| `Exporter` | KafkaProducer | doc | [`ingest_kafka`] |

[`pipeware`]: https://github.com/pipebase/pipebase/tree/main/pipeware
[`group_sum_redis`]: https://github.com/pipebase/pipebase/tree/main/examples/group_sum_redis
[`group_sum_rocksdb`]: https://github.com/pipebase/pipebase/tree/main/examples/group_sum_rocksdb
[`ingest_cassandra`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_cassandra
[`ingest_postgres`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_postgres
[`ingest_redis`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_redis
[`relay`]: https://github.com/pipebase/pipebase/tree/main/examples/relay
[`consume_kafka`]: https://github.com/pipebase/pipebase/tree/main/examples/consume_kafka
[`ingest_kafka`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_kafka