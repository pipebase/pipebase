Demo count with `RocksDBUnorderedGroupAddAggregator` pipe
### Build and Run (terminal 1)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o cnt_rocksdb -r
```
Run app
```
./cnt_rocksdb
```
### Ingest Data and Monitor Pipe (terminal 2)
Ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
Group count stdout (terminal 1)
```
[Pair("bar", Count32(2)), Pair("foo", Count32(3))]
```
Kill app and restart, ingest one more time
```
[Pair("bar", Count32(4)), Pair("foo", Count32(6))]
```
Open [browser](http://localhost:8000/v1/pipe) and list all pipes