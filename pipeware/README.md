[`pipeware`] is repository for pipe plugins using third party SDK

## Plugins
Pipes
| pipe type | implementation | example |
| --------- | -------------- | ------- |
| `Listener`  | WarpIngestionServer | `ingest_*` |
| `Listener` | KafkaConsumer | [`consume_kafka`] |
| `Listener` | KubeLogReader | [`kube_log`] |
| `Listener` | KubeEventReader | [`kube_event`] |
| `Listener` | RedisSubscriber | [`pubsub_redis`] |
| `Poller` | SqsMessageReceiver | [`consume_sqs`] |
| `Mapper` | JsonSer |  |
| `Mapper` | JsonDeser | `ingest_*` |
| `Mapper` | CsvSer | [`convert_csv`] |
| `Mapper` | CsvDeser |  |
| `Mapper` | RedisUnorderedGroupAddAggregator | [`group_sum_redis`], [`group_avg_redis`], [`group_count_redis`] |
| `Mapper` | RocksDBUnorderedGroupAddAggregator | [`group_sum_rocksdb`], [`group_avg_rocksdb`], [`group_count_rocksdb`] |
| `Mapper` | KafkaJsonRecordConverter | [`ingest_kafka`] |
| `Mapper` | ReqwestGetter | [`stripe_get_charge`] |
| `Mapper` | ReqwestQuery | [`stripe_query_charge`] |
| `Exporter` | CqlWriter | [`ingest_cassandra`] |
| `Exporter` | PsqlWriter | [`ingest_postgres`] |
| `Exporter` | RedisStringWriter | [`ingest_redis`] |
| `Exporter` | RedisPublisher | [`pubsub_redis`] |
| `Exporter` | ReqwestPoster | [`relay`], [`ingest_elasticsearch`] |
| `Exporter` | KafkaProducer | [`ingest_kafka`] |
| `Exporter` | S3Writer | [`upload_s3`] |
| `Exporter` | MySQLWriter | [`ingest_mysql`] |
| `Exporter` | DynamoDBWriter | [`ingest_dynamodb`] |
| `Exporter` | SnsPublisher | [`pubsub_sns`] |

Context Stores
| implementation | example |
| -------------- | ------- |
| WarpContextServer | `ingest_*` |

Error Handlers
| implementation | example |
| -------------- | ------- |
| SnsPipeErrorHandler | [`error_sns_publisher`] |

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
[`ingest_dynamodb`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_dynamodb
[`pubsub_sns`]: https://github.com/pipebase/pipebase/tree/main/examples/pubsub_sns
[`error_sns_publisher`]: https://github.com/pipebase/pipebase/tree/main/examples/error_sns_publisher