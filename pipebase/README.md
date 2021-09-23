[`pipebase`] is a [`tokio`] based runtime library for data integration app. It provides basic [`pipe`]s implemented with rust standard library.

## Basic Pipes
list of basic pipes built in `pipebase`
| pipe type | implementation | example |
| --------- | -------------- | ------- |
| `Exporter` | `Printer` | [`printer`] |
| `Poller` | `Timer` | [`timer`] |
| `Listener` | `LocalFilePathVisitor` | [`file_path`] |
| `Streamer` | `FileLineReader` | [`stateless_word_count`] |
| `Streamer` | `IteratorReader` | [`stateless_word_count`] |
| `Mapper` | `FieldVisit` | [`field_filter`] |
| `Mapper` | `FilterMap` | [`field_filter`] |
| `Mapper` | `StringSplitter` | [`stateless_word_count`] |
| `Mapper` | `Projection` | [`project_file`] |
| `Mapper` | `FileReader` | [`project_file`] |
| `Mapper` | `FileWriter` | [`convert_csv`] |
| `Mapper` | `Conversion` | [`convert_csv`] |
| `Selector` | `RandomSelector` | [`ingest_redis_parallel`] |
| `Collector` | `TextCollector` | [`ingest_elasticsearch`] |
| `Collector` | `InMemoryWindowCollector` | [`moving_average`] |

[`pipebase`]: https://github.com/pipebase/pipebase/tree/main/pipebase
[`tokio`]: https://github.com/tokio-rs/tokio
[`pipe`]: https://github.com/pipebase/pipebase/tree/main/pipegen#pipe
[`pipe type`]: https://github.com/pipebase/pipebase/tree/main/pipegen#pipe-type
[`printer`]: https://github.com/pipebase/pipebase/tree/main/examples/printer
[`timer`]: https://github.com/pipebase/pipebase/tree/main/examples/timer
[`field_filter`]: https://github.com/pipebase/pipebase/tree/main/examples/field_filter
[`file_path`]: https://github.com/pipebase/pipebase/tree/main/examples/file_path
[`stateless_word_count`]: https://github.com/pipebase/pipebase/tree/main/examples/stateless_word_count
[`project_file`]: https://github.com/pipebase/pipebase/tree/main/examples/project_file
[`convert_csv`]: https://github.com/pipebase/pipebase/tree/main/examples/convert_csv
[`ingest_redis_parallel`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_redis_parallel
[`ingest_elasticsearch`]: https://github.com/pipebase/pipebase/tree/main/examples/ingest_elasticsearch
[`moving_average`]: https://github.com/pipebase/pipebase/tree/main/examples/moving_average