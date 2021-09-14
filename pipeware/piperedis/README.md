A [`pipebase`] plugin using [`redis-rs`]
### Pipe Configs
| type | example |
| ---- | ------- |
| `RedisPublisherConfig` | [`redis_pub`] |
| `RedisStringBatchWriterConfig` | [`batch_redis_writer`] |
| `RedisStringWriterConfig` | [`redis_writer`] |
| `RedisSubscriberConfig` | [`redis_sub`] |
| `RedisUnorderedGroupAddAggregatorConfig` | [`redis_group_add`] |

[`pipebase`]: https://github.com/pipebase/pipebase
[`redis-rs`]: https://github.com/mitsuhiko/redis-rs
[`batch_redis_writer`]: https://github.com/pipebase/pipebase/blob/main/examples/batch_ingest_redis/catalogs/batch_redis_writer.yml
[`redis_group_add`]: https://github.com/pipebase/pipebase/blob/main/examples/group_sum_redis/catalogs/redis_group_add.yml
[`redis_pub`]: https://github.com/pipebase/pipebase/blob/main/examples/pubsub_redis/catalogs/redis_pub.yml
[`redis_sub`]: https://github.com/pipebase/pipebase/blob/main/examples/pubsub_redis/catalogs/redis_sub.yml
[`redis_writer`]: https://github.com/pipebase/pipebase/blob/main/examples/ingest_redis/catalogs/redis_writer.yml