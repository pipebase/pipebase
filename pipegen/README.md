[`pipegen`] parse `manifest`, contains pipe / custom data object specification, and generate code for data integration app

## Manifest Layout
A `manifest` is composed of following sections:
* `name`, name of application
* `dependencies`, list of crates the application dependes on
* [`pipes`], list of pipe definition
* [`objects`], list of custom data object definition
* `cstores`, list of pipe runtime context store definition

## Pipes
Pipes are the smallest runtime unit that you can create, example:
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
| name | pipe name in snake_case | true |
| ty | [`pipe type`] | true |
| config.ty | pipe config type | true |
| config.path | path to pipe config file | false |
| upstreams | list of upstream pipe names | false if `ty` is `Poller` or `Listener` |
| output | output data type (unnamed [`data field`]) | false if [`pipe type`] is `Exporter` |

Note that:
* pipes are wired as **directed acyclic graph** using upstreams
* upstreams of a pipe should have **same** output type, i.e a pipe's input type is **determined** in runtime
* pipe defines trait bounds for input, and upstreams' output should satisfy the constrait

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

## Objects
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
| ty | object type in CamelCase | true |
| metas | list of [`meta`]s per object | false |
| fields | list of [`data field`]s | true |

## Meta
Meta defines additional attributes of an object so that it fits trait bounds of a pipe's input 

## Data Field
| Field | Description | Required |
| ----- | ----------- | -------- |
| name | field name | false  |
| ty | [`data type`] | true |
| metas | list of [`meta`]s per field | false |
| is_boxed | field is boxed or not | false |
| is_optional | field is optional or not | false |
| is_public | field is public or not | false |

## Data Type
| Type | In Rust |
| ---- | ------- |
| `Boolean` | bool |
| `Character` | char |
| `String` | String |
| `Byte` | i8 |
| `UnsignedByte` | u8 |
| `Short` | i16 |
| `UnsignedShort` | u16 |
| `Integer` | i32 |
| `UnsignedInteger` | u32 |
| `Size` | size |
| `UnsignedSize` | usize |
| `Long` | i64 |
| `UnsignedLong` | u64 |
| `LongLong` | i128 |
| `UnsignedLongLong` | u128 |
| `Float` | f32 |
| `Double` | f64 |
| `PathBuf` | std::path::PathBuf |
| `Count32` | pipebase::Count32(pub u32) |
| `Averagef32` | pipebase::Averagef32(pub f32, pub f32) |
| `Vec` | Vec<T> |
| `Array` | [T; N] |
| `Tuple` | (T,) |
| `HashMap` | HashMap<K, V> |
| `HashSet` | HashSet<T> |
| `Pair` | pipebase::Pair<L, R>(pub L, pub R) |

[`data field`]: https://github.com/pipebase/pipebase/tree/main/pipegen#data-field
[`data type`]: https://github.com/pipebase/pipebase/tree/main/pipegen#data-type
[`meta`]: https://github.com/pipebase/pipebase/tree/main/pipegen#meta
[`objects`]: https://github.com/pipebase/pipebase/tree/main/pipegen#objects
[`pipegen`]: https://github.com/pipebase/pipebase/tree/main/pipegen
[`pipes`]: https://github.com/pipebase/pipebase/tree/main/pipegen#pipes
[`pipe type`]: https://github.com/pipebase/pipebase/tree/main/pipegen#pipe-type