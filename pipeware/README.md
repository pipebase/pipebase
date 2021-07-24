[`pipeware`] is repository for pipe plugins using third party SDK

## Plugins
Pipes
| pipe type | implementation | input / output | example |
| --------- | -------------- | -------------- | ------- |
| `Listener`  | WarpIngestionServer | doc | `ingest_*` |
| `Listener` | KafkaConsumer | doc | [`consume_kafka`] |
| `Listener` | KubeLogReader | doc | [`log_kube`] |
| `Mapper` | JsonSer | doc |  |
| `Mapper` | JsonDeser | doc | `ingest_*` |
| `Mapper` | CsvSer | doc | [`convert_csv`] |
| `Mapper` | CsvDeser | doc |  |
| `Mapper` | RedisUnorderedGroupAddAggregator | doc | [`group_sum_redis`] |
| `Mapper` | RocksDBUnorderedGroupAddAggregator | doc | [`group_sum_rocksdb`] |
| `Mapper` | KafkaJsonRecordConverter | doc | [`ingest_kafka`] |
| `Mapper` | ReqwestQuery | doc | [`query_stripe_charge`] |
| `Exporter` | CqlWriter | doc | [`ingest_cassandra`] |
| `Exporter` | PsqlWriter | doc | [`ingest_postgres`] |
| `Exporter` | RedisWriter | doc | [`ingest_redis`] |
| `Exporter` | ReqwestPoster | doc | [`relay`], [`ingest_elasticsearch`] |
| `Exporter` | KafkaProducer | doc | [`ingest_kafka`] |
| `Exporter` | S3Writer | doc | [`upload_s3`] |
| `Exporter` | MySQLWriter | doc | [`ingest_mysql`] |

[`pipeware`]: https://github.com/pipebase/pipebase/tree/main/pipeware
[`group_sum_redis`]: https://github.com/pipebase/pipebase/tree/main/examples/group_sum_redis
[`group_sum_rocksdb`]: https://github.com/pipebase/pipebase/tree/main/examples/group_sum_rocksdb
[`ingest_cassandra`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_cassandra
[`ingest_postgres`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_postgres
[`ingest_redis`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_redis
[`relay`]: https://github.com/pipebase/pipebase/tree/main/examples/relay
[`consume_kafka`]: https://github.com/pipebase/pipebase/tree/main/examples/consume_kafka
[`ingest_kafka`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_kafka
[`log_kube`]: https://github.com/pipebase/pipebase/tree/main/examples/log_kube
[`convert_csv`]: https://github.com/pipebase/pipebase/tree/main/examples/convert_csv
[`upload_s3`]: https://github.com/pipebase/pipebase/tree/main/examples/upload_s3
[`ingest_mysql`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_mysql
[`ingest_elasticsearch`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_elasticsearch
[`query_stripe_charge`]: https://github.com/pipebase/pipebase/tree/main/examples/query_stripe_charge