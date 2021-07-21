[`cargo-pipe`] is a [`clap`] based command line tool to generate and build data integration app with manifest

## Installation
Install Rust and Cargo
```sh
curl https://sh.rustup.rs -sSf | sh
```
Install `cargo-pipe` CLI
```
cargo install pipe
```

## Usage
```
cargo pipe --help
```

## Quick Start
Go to resources folder
```
cargo pipe new && \
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o timer -r
```
Run app
```
./timer
```

[`cargo-pipe`]: https://github.com/pipebase/pipebase/tree/main/cargo-pipe
[`clap`]: https://github.com/clap-rs/clap