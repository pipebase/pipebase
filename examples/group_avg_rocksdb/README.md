Demo average with `RedisUnorderedGroupAddAggregator` pipe
### Build and Run (terminal 1)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o avg_rocksdb -r
```
Run app
```
./avg_rocksdb
```
### Ingest Data and Monitor Pipe (terminal 2)
Open new terinal and ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
Checkout terminal 1
```
[Pair("foo", Averagef32(Averagef32(6.0, 3.0))), Pair("bar", Averagef32(Averagef32(15.0, 3.0)))]
```
Restart app, ingest one more time, result is stateful
```
[Pair("bar", Averagef32(30.0, 6.0)), Pair("foo", Averagef32(12.0, 6.0))]
```