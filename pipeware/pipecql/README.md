A [`pipebase`] plugin using [`scylla-rust-driver`]
### Pipe Configs
| type | example |
| ---- | ------- |
| `CqlWriterConfig` | [`ingest_cassandra`] |
| `CqlPreparedWriterConfig` | [`batch_ingest_cassandra`] |

[`pipebase`]: https://github.com/pipebase/pipebase
[`scylla-rust-driver`]: https://github.com/scylladb/scylla-rust-driver
[`ingest_cassandra`]: https://github.com/pipebase/pipebase/blob/main/examples/ingest_cassandra/catalogs/cql_writer.yml
[`batch_ingest_cassandra`]: https://github.com/pipebase/pipebase/blob/main/examples/batch_ingest_cassandra/catalogs/batch_cql_writer.yml

