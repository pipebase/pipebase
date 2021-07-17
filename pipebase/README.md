[`pipebase`] is a [`tokio`] based runtime library for data integration app. It provides basic [`pipes`] implemented with rust standard library.

## Basic Pipes
Here is a list of basic pipes built in `pipebase`
| config type | pipe type | input / output | example |
| ----------- | --------- | -------------- | ------- |
| `PrinterConfig` | `Exporter` | doc | [`printer`] |
| `TimerConfig` | `Poller` | doc | [`timer`] |
| `LocalFilePathVisitorConfig` | `Listener` | doc | [`file_path`] |
| `FileLineReaderConfig` | `Streamer` | doc | [`stateless_word_count`] |
| `IteratorReaderConfig` | `Streamer` | doc | [`stateless_word_count`] |
| `FieldVisitConfig` | `Mapper` | doc | [`field_filter`] |
| `FilterMapConfig` | `Mapper` | doc | [`field_filter`] |
| `StringSplitterConfig` | `Mapper` | doc | [`stateless_word_count`] |
| `ProjectionConfig` | `Mapper` | doc | [`project_file`] |
| `FileReaderConfig` | `Mapper` | doc | [`project_file`] |

[`pipebase`]: https://github.com/pipebase/pipebase/tree/main/pipebase
[`tokio`]: https://github.com/tokio-rs/tokio
[`pipes`]: https://github.com/pipebase/pipebase/tree/main/pipegen#pipes
[`pipe type`]: https://github.com/pipebase/pipebase/tree/main/pipegen#pipe-type
[`printer`]: https://github.com/pipebase/pipebase/tree/main/examples/printer
[`timer`]: https://github.com/pipebase/pipebase/tree/main/examples/timer
[`field_filter`]: https://github.com/pipebase/pipebase/tree/main/examples/field_filter
[`file_path`]: https://github.com/pipebase/pipebase/tree/main/examples/file_path
[`stateless_word_count`]: https://github.com/pipebase/pipebase/tree/main/examples/stateless_word_count
[`project_file`]: https://github.com/pipebase/pipebase/tree/main/examples/project_file