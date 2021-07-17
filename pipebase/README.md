[`pipebase`] is a [`tokio`] based runtime library for data integration app. It provides basic [`pipes`] implemented with rust standard library.

## Basic Pipes
Here is a list of basic pipes built in `pipebase`
| pipe type | implementation | input / output | example |
| --------- | -------------- | -------------- | ------- |
| `Exporter` | `Printer` | doc | [`printer`] |
| `Poller` | `Timer` | doc | [`timer`] |
| `Listener` | `LocalFilePathVisitor` | doc | [`file_path`] |
| `Streamer` | `FileLineReader` | doc | [`stateless_word_count`] |
| `Streamer` | `IteratorReader` | doc | [`stateless_word_count`] |
| `Mapper` | `FieldVisit` | doc | [`field_filter`] |
| `Mapper` | `FilterMap` | doc | [`field_filter`] |
| `Mapper` | `StringSplitter` | doc | [`stateless_word_count`] |
| `Mapper` | `Projection` | doc | [`project_file`] |
| `Mapper` | `FileReader` | doc | [`project_file`] |

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