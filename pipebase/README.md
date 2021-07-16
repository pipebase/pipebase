[`pipebase`] is a [`tokio`] based runtime library for data integration app. It provides basic [`pipes`] implemented with rust standard library.

## Basic Pipes
Here is a list of basic pipes built in `pipebase`
| config type | pipe type | input / trait bounds | output | example |
| ----------- | --------- | ----- | ------ | ------- |
| `PrinterConfig` | `Exporter`  | `Debug` | No output | [`printer`] |
| `TimerConfig` | `Poller` | No input | `u128` | [`timer`] |
| `LocalFilePathVisitorConfig` | `Listener` | No input | `PathBuf` | [`file_path`] |

[`pipebase`]: https://github.com/pipebase/pipebase/tree/main/pipebase
[`tokio`]: https://github.com/tokio-rs/tokio
[`pipes`]: https://github.com/pipebase/pipebase/tree/main/pipegen#pipes
[`pipe type`]: https://github.com/pipebase/pipebase/tree/main/pipegen#pipe-type
[`printer`]: https://github.com/pipebase/pipebase/tree/main/examples/printer
[`timer`]: https://github.com/pipebase/pipebase/tree/main/examples/timer
[`file_path`]: https://github.com/pipebase/pipebase/tree/main/examples/file_path
