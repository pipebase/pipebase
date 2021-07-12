### Installation
Install Rust and Cargo
```sh
curl https://sh.rustup.rs -sSf | sh
```
Install `cargo-pipe` CLI
```
cargo install pipe
```
### Build and Run
Create new project
```
cargo pipe new
```
Generate app
```
cargo pipe generate
```
Build app
```
cargo pipe build -o ingest
```
Run app
```
./ingest
```
### Ingest Data and Monitor Pipe
Setup Redis
```
docker-compose up -d
```
Ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
Open [browser](http://localhost:8000/v1/pipe) and list all pipes