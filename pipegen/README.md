[`pipegen`] parse `manifest`, contains pipe / custom data object specification, and generate code

## Specifications
A `manifest` is composed of following sections:
* `name`, name of application
* `dependencies`, list of crates the application dependes on
* `pipes`, list of pipe definition
* `objects`, list of custom data object definition
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
  data_ty: UnsignedLongLong
```
Specification
| Field | Description | Required |
| ----- | ----------- | -------- |
| name | pipe name in snake_case | true |
| ty | pipe type | true |
| config.ty | pipe config type | true |
| config.path | path to pipe config file | false |
| upstreams | list of upstream pipe names | false if `ty` is `Poller` or `Listener` |
| output | output data type | false if `ty` is `Exporter` |

## Objects

[`pipegen`]: https://github.com/pipebase/pipebase/tree/main/pipegen