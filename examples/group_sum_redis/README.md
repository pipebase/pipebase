Demo summation with `RedisUnorderedGroupAddAggregator` pipe
### Setup (terminal 1)
launch redis and app
```sh
docker-compose up -d
docker logs -f app
```
### Ingest Data and Monitor Pipe (terminal 3)
ingest sample data
```sh
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
checkout terminal 1
```sh
[Pair("bar", 1), Pair("foo", 3)]
```
query redis (terminal 1)
```sh
docker exec redis redis-cli get "foo" && \
docker exec redis redis-cli get "bar"
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes