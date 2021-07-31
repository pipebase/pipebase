Demo summation with `RocksDBUnorderedGroupAddAggregator` pipe
### Build and Run (terminal 1)
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o sum_rocksdb -r
```
run app
```
./sum_rocksdb
```
### Ingest Data and Monitor Pipe (terminal 2)
ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
group sum stdout (terminal 1)
```
[Pair("foo", 3), Pair("bar", 1)]
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes