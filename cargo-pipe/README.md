[`cargo-pipe`] is a [`clap`] based command-line tool to generate and build data integration app with manifest

## Installation
install Rust and Cargo
```sh
curl https://sh.rustup.rs -sSf | sh
```
install `cargo-pipe` CLI
```
cargo install cargo-pipe
```

## Usage
```
cargo pipe --help
```

## Quick Start
go to resources folder
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
Note that, any change to `pipe.yml` requires re-run `validate`, `generate`, `build` steps

[`cargo-pipe`]: https://github.com/pipebase/pipebase/tree/main/cargo-pipe
[`clap`]: https://github.com/clap-rs/clap