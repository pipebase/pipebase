A [`pipebase`] plugin using [`rust-postgres`]
### Pipe Configs
| type | example |
| ---- | ------- |
| `PsqlWriterConfig` | [`ingest_postgres`] |
| `PsqlPreparedWriterConfig` | [`batch_ingest_postgres`] |

[`pipebase`]: https://github.com/pipebase/pipebase
[`rust-postgres`]: https://github.com/sfackler/rust-postgres
[`ingest_postgres`]: https://github.com/pipebase/pipebase/blob/main/examples/ingest_postgres/catalogs/psql_writer.yml
[`batch_ingest_postgres`]: https://github.com/pipebase/pipebase/blob/main/examples/batch_ingest_postgres/catalogs/batch_psql_writer.yml