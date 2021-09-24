[`cargo-pipe`] is a [`clap`] based command-line tool to generate and build [`pipebase`] app with manifest

## Installation
install Rust and Cargo (Minimum Supported Version of Rust 1.54.0)
```sh
curl https://sh.rustup.rs -sSf | sh
```
install [`rustfmt`]
```sh
rustup component add rustfmt
```
install `cargo-pipe` CLI
```sh
cargo install cargo-pipe
```

## Usage
```
cargo pipe --help
```

## Quick Start
go to [`resources`] folder
```
cargo pipe new && \
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o timer -r
```
run app
```
./timer
```
note that, any change to `pipe.yml` requires re-run `validate`, `generate`, `build` steps

## Validation & Debug
validate manifest only
```
cargo pipe validate -o -p
```
build with debug flag
```
cargo pipe build -d
...
     Warning struct is never constructed: `Bar`
     Warning 1 warning emitted
```
fix the warning by remove `objects` section
```
@@ -23,13 +23,6 @@ pipes:
     config:
       ty: PrinterConfig
     upstreams: [ "timer1", "timer2" ]
-objects:
-  - ty: Bar
-    fields:
-      - name: bar1
-        ty: Integer
-      - name: bar2
-        ty: String
```
since manifest changed, re-generate and build
```
cargo pipe generate && \
cargo pipe build -d
```

## Describe Manifest
list all pipes and objects
```
cargo pipe describe -a
...
    Describe pipes
      Result pipe: timer1, printer, timer2
    Describe objects
      Result objects: Bar
```
describe a pipe
```
cargo pipe describe -p timer1
...
    Describe pipe timer1
      Result
Name:   timer1
Type:   poller
Config: { type: TimerConfig, path: catalogs/timer1.yml }
Upstream: []
```
describe an object
```
cargo pipe describe -o Bar
...
    Describe object Bar
      Result
Type: Bar
Type Metas:
Fields:
Name   Type Metas
bar1    i32
bar2 String
```
describe pipelines cross a pipe
```
cargo pipe describe -l printer
...
    Describe pipelines for printer
      Result pipeline: timer2(u128) -> printer
      Result pipeline: timer1(u128) -> printer
```

[`cargo-pipe`]: https://github.com/pipebase/pipebase/tree/main/cargo-pipe
[`resources`]: https://github.com/pipebase/pipebase/tree/main/cargo-pipe/resources
[`clap`]: https://github.com/clap-rs/clap
[`rustfmt`]: https://github.com/rust-lang/rustfmt
[`pipebase`]: https://github.com/pipebase/pipebase