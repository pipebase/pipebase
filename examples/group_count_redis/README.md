Demo count with `RedisUnorderedGroupAddAggregator` pipe
### Setup Redis (terminal 1)
launch redis and app
```sh
docker-compose up -d
docker logs -f app
```
### Ingest Data and Monitor Pipe (terminal 2)
ingest sample data
```sh
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
checkout terminal 1
```sh
[Pair("bar", RedisCount32(Count32(2))), Pair("foo", RedisCount32(Count32(3)))]
```
query redis
```sh
docker exec redis redis-cli get "foo" && \
docker exec redis redis-cli get "bar"
```