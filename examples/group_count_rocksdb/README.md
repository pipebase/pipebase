Demo count with `RocksDBUnorderedGroupAddAggregator` pipe
### Build and Run (terminal 1)
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o cnt_rocksdb -r
```
run app
```
./cnt_rocksdb
```
### Ingest Data and Monitor Pipe (terminal 2)
ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
group count stdout (terminal 1)
```
[Pair("bar", Count32(2)), Pair("foo", Count32(3))]
```
kill app and restart, ingest one more time
```
[Pair("bar", Count32(4)), Pair("foo", Count32(6))]
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes