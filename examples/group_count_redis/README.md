Demo count with `RedisUnorderedGroupAddAggregator` pipe
### Setup Redis (terminal 1)
launch redis
```
docker-compose up -d
```
login container
```
docker exec -it redis /bin/sh
```
### Build and Run (terminal 2)
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o cnt_redis -r
```
run app
```
./cnt_redis
```
### Ingest Data and Monitor Pipe (terminal 3)
ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
checkout terminal 2
```
[Pair("bar", RedisCount32(Count32(2))), Pair("foo", RedisCount32(Count32(3)))]
```
query redis (terminal 1)
```
redis-cli get "foo" && \
redis-cli get "bar"
"3"
"2"
```