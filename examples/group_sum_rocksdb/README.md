Demo `RocksDBUnorderedGroupAddAggregator` pipe
### Build and Run (terminal 1)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o sum_rocksdb -r
```
Run app
```
./sum_rocksdb
```
### Ingest Data and Monitor Pipe (terminal 2)
Ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
Group sum stdout (terminal 1)
```
[Pair("foo", 3), Pair("bar", 1)]
```
Open [browser](http://localhost:8000/v1/pipe) and list all pipes