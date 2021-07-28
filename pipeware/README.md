[`pipeware`] is repository for pipe plugins using third party SDK

## Plugins
Pipes
| pipe type | implementation | input / output | example |
| --------- | -------------- | -------------- | ------- |
| `Listener`  | WarpIngestionServer | doc | `ingest_*` |
| `Listener` | KafkaConsumer | doc | [`consume_kafka`] |
| `Listener` | KubeLogReader | doc | [`kube_log`] |
| `Listener` | KubeEventReader | doc | [`kube_event`] |
| `Listener` | RedisSubscriber | doc | [`pubsub_redis`] |
| `Poller` | SQSMessageReceiver | doc | [`consume_sqs`] |
| `Mapper` | JsonSer | doc |  |
| `Mapper` | JsonDeser | doc | `ingest_*` |
| `Mapper` | CsvSer | doc | [`convert_csv`] |
| `Mapper` | CsvDeser | doc |  |
| `Mapper` | RedisUnorderedGroupAddAggregator | doc | [`group_sum_redis`], [`group_avg_redis`], [`group_count_redis`] |
| `Mapper` | RocksDBUnorderedGroupAddAggregator | doc | [`group_sum_rocksdb`], [`group_avg_rocksdb`], [`group_count_rocksdb`] |
| `Mapper` | KafkaJsonRecordConverter | doc | [`ingest_kafka`] |
| `Mapper` | ReqwestGetter | doc | [`stripe_get_charge`] |
| `Mapper` | ReqwestQuery | doc | [`stripe_query_charge`] |
| `Exporter` | CqlWriter | doc | [`ingest_cassandra`] |
| `Exporter` | PsqlWriter | doc | [`ingest_postgres`] |
| `Exporter` | RedisStringWriter | doc | [`ingest_redis`] |
| `Exporter` | RedisPublisher | doc | [`pubsub_redis`] |
| `Exporter` | ReqwestPoster | doc | [`relay`], [`ingest_elasticsearch`] |
| `Exporter` | KafkaProducer | doc | [`ingest_kafka`] |
| `Exporter` | S3Writer | doc | [`upload_s3`] |
| `Exporter` | MySQLWriter | doc | [`ingest_mysql`] |


[`pipeware`]: https://github.com/pipebase/pipebase/tree/main/pipeware
[`group_sum_redis`]: https://github.com/pipebase/pipebase/tree/main/examples/group_sum_redis
[`group_avg_redis`]: https://github.com/pipebase/pipebase/tree/main/examples/group_avg_redis
[`group_count_redis`]: https://github.com/pipebase/pipebase/tree/main/examples/group_count_redis
[`group_sum_rocksdb`]: https://github.com/pipebase/pipebase/tree/main/examples/group_sum_rocksdb
[`group_avg_rocksdb`]: https://github.com/pipebase/pipebase/tree/main/examples/group_avg_rocksdb
[`group_count_rocksdb`]: https://github.com/pipebase/pipebase/tree/main/examples/group_count_rocksdb
[`ingest_cassandra`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_cassandra
[`ingest_postgres`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_postgres
[`ingest_redis`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_redis
[`relay`]: https://github.com/pipebase/pipebase/tree/main/examples/relay
[`consume_kafka`]: https://github.com/pipebase/pipebase/tree/main/examples/consume_kafka
[`ingest_kafka`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_kafka
[`kube_log`]: https://github.com/pipebase/pipebase/tree/main/examples/kube_log
[`kube_event`]: https://github.com/pipebase/pipebase/tree/main/examples/kube_event
[`convert_csv`]: https://github.com/pipebase/pipebase/tree/main/examples/convert_csv
[`upload_s3`]: https://github.com/pipebase/pipebase/tree/main/examples/upload_s3
[`ingest_mysql`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_mysql
[`ingest_elasticsearch`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_elasticsearch
[`stripe_get_charge`]: https://github.com/pipebase/pipebase/tree/main/examples/stripe_get_charge
[`stripe_query_charge`]: https://github.com/pipebase/pipebase/tree/main/examples/stripe_query_charge
[`consume_sqs`]: https://github.com/pipebase/pipebase/tree/main/examples/consume_sqs
[`pubsub_redis`]: https://github.com/pipebase/pipebase/tree/main/examples/pubsub_redis