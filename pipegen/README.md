[`pipegen`] parse `manifest`, contains pipe / custom data object specification, and generate code for data integration app

## Manifest Layout
A `manifest` is composed of:
| Field | Description | Required |
| ----- | ----------- | -------- |
| `name` | name of application | true |
| `dependencies` | list of crates the application dependes on | false |
| `pipes` | list of [`pipe`] definition | true |
| `objects` | list of custom data [`object`] definition | false |
| `cstores` | list of pipe runtime [`context store`] definition | false |
| `error` | pipe [`error handler`] definition | false |

Tips: compose manifest with YAML language support and [`schema`] setting

## Dependency
App dependency, similar as [`cargo dependencies`]
example:
```
dependencies:
  - name: pipebase
    version: 0.1.0
    modules: ["pipebase::prelude::*"]
```
Specification
| Field | Description | Required |
| ----- | ----------- | -------- |
| `name` | crate name | true |
| `version` | crate version | false |
| `path` | local crate path | false |
| `git` | git repository url | false |
| `branch` | git repository branch | false |
| `tag` | git repository tag | false |
| `features` | [`cargo features`] | false |
| `package` | package in [`cargo workspace`] | false |
| `modules` | list of used modules | true |

## Pipe
Pipes are the smallest runtime unit to create, example:
```
name: timer1
ty: Poller
config:
  ty: TimerConfig
  path: catalogs/timer.yml
output:
  ty: UnsignedLongLong
```
Specification
| Field | Description | Required |
| ----- | ----------- | -------- |
| `name` | pipe name in snake_case | true |
| `ty` | [`pipe type`] | true |
| `config.ty` | pipe config type | true |
| `config.path` | path to pipe config file | false |
| `upstreams` | list of upstream pipe names | false if `ty` is `Poller` or `Listener` |
| `output` | output [`data type`] | false if [`pipe type`] is `Exporter` |

Note that:
* pipes are wired as **directed acyclic graph** with upstreams
* upstreams of a pipe should have **same** output type, i.e a pipe's input type is **determined** in runtime
* pipe defines trait bounds for input, upstreams' output should satisfy the constraint

## Pipe Type
| Type | Description | #upstreams | #downstreams |
| ---- | ----------- | ---------- | ------------ |
| `Listener` | listen data at local | 0 | 1+ |
| `Poller` | poll data at remote | 0 | 1+ |
| `Mapper` | transform input | 1+ | 1+ |
| `Collector` | batch input  | 1+ | 1+ |
| `Streamer` | stream batched input | 1+ | 1+ |
| `Selector` | send input to a subset of downstream | 1+ | 1+ |
| `Exporter` | export input to remote | 1+ | 0 |

## Object
Cutstom data object transferred in pipeline, example:
```
ty: Record
metas:
  - derives: [Clone, Debug, Deserialize]
fields:
  - name: key
    ty: String
  - name: value
    ty: UnsignedInteger
```
Specification
| Field | Description | Required |
| ----- | ----------- | -------- |
| `ty` | object type in CamelCase | true |
| `metas` | list of [`meta`]s per object | false |
| `fields` | list of [`data field`]s | true |

## Meta
Meta defines additional attributes of an object so that it satisfy trait bounds of a pipe's input. See example [`fix_left_right`], [`fix_convert`] understand when and how to use metas

## Data Field
| Field | Description | Required |
| ----- | ----------- | -------- |
| `name` | field name | false  |
| `ty` | [`data type`] | true |
| `metas` | list of [`meta`]s per field | false |
| `is_public` | field is public or not | false |

## Data Type
| Type | In Rust |
| ---- | ------- |
| `Boolean` | `bool` |
| `Character` | `char` |
| `String` | `String` |
| `Byte` | `i8` |
| `UnsignedByte` | `u8` |
| `Short` | `i16` |
| `UnsignedShort` | `u16` |
| `Integer` | `i32` |
| `UnsignedInteger` | `u32` |
| `Size` | `size` |
| `UnsignedSize` | `usize` |
| `Long` | `i64` |
| `UnsignedLong` | `u64` |
| `LongLong` | `i128` |
| `UnsignedLongLong` | `u128` |
| `Float` | `f32` |
| `Double` | `f64` |
| `PathBuf` | `std::path::PathBuf` |
| `Count32` | `pipebase::common::Count32` |
| `Averagef32` | `pipebase::common::Averagef32` |
| `Box` | `Box<T>` |
| `Option` | `Option<T>` |
| `Vec` | `Vec<T>` |
| `Array` | `[T; N]` |
| `Tuple` | `(T,)` |
| `HashMap` | `HashMap<K, V>` |
| `HashSet` | `HashSet<T>` |
| `Pair` | `pipebase::common::Pair<L, R>` |

## Context Store
Store pipe runtime contexts including: `pipe name`, [`pipe state`], `total run`, `failure run`

## Pipe State
| State | Pipe Type |
| ----- | --------- |
| `Init` | all |
| `Receive` | except `Poller` |
| `Poll` | `Poller` |
| `Map` | `Mapper` |
| `Send` | except `Exporter` |
| `Export` | `Exporter` |
| `Done` | all |

## Error Handler
Listen errors from pipes, example [`error_printer`]

[`data field`]: https://github.com/pipebase/pipebase/tree/main/pipegen#data-field
[`data type`]: https://github.com/pipebase/pipebase/tree/main/pipegen#data-type
[`meta`]: https://github.com/pipebase/pipebase/tree/main/pipegen#meta
[`object`]: https://github.com/pipebase/pipebase/tree/main/pipegen#object
[`pipegen`]: https://github.com/pipebase/pipebase/tree/main/pipegen
[`pipe`]: https://github.com/pipebase/pipebase/tree/main/pipegen#pipe
[`pipe type`]: https://github.com/pipebase/pipebase/tree/main/pipegen#pipe-type
[`context store`]: https://github.com/pipebase/pipebase/tree/main/pipegen#context-store
[`error handler`]: https://github.com/pipebase/pipebase/tree/main/pipegen#error-handler
[`pipe state`]: https://github.com/pipebase/pipebase/tree/main/pipegen#pipe-state
[`fix_left_right`]: https://github.com/pipebase/pipebase/tree/main/examples/fix_left_right
[`fix_convert`]: https://github.com/pipebase/pipebase/tree/main/examples/fix_convert
[`error_printer`]: https://github.com/pipebase/pipebase/tree/main/examples/error_printer
[`cargo dependencies`]: https://doc.rust-lang.org/cargo/guide/dependencies.html#dependencies
[`cargo features`]: https://doc.rust-lang.org/cargo/reference/features.html
[`cargo workspace`]: https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html
[`schema`]: https://github.com/pipebase/schema