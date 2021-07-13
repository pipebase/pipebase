### Build and Run (terminal 1)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe check && \
cargo pipe build -o sum_rocksdb -r
```
Run app
```
./sum_rocksdb
```
### Ingest Data and Monitor Pipe 
Open new terinal and ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
Group sum stdout (terminal 1)
```

```
Open [browser](http://localhost:8000/v1/pipe) and list all pipes