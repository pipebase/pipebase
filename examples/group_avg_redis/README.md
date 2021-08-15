Demo average with `RedisUnorderedGroupAddAggregator` pipe
### Setup (terminal 1)
launch redis and app
```sh
docker-compose up -d
docker logs -f app
```
### Ingest Data (terminal 2)
ingest sample data **twice**
```sh
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
Checkout terminal 1
```sh
[Pair("foo", RedisAveragef32(Averagef32(6.0, 3.0))), Pair("bar", RedisAveragef32(Averagef32(15.0, 3.0)))]
[Pair("foo", RedisAveragef32(Averagef32(12.0, 6.0))), Pair("bar", RedisAveragef32(Averagef32(30.0, 6.0)))]
```
