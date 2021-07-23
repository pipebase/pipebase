Demo `IteratorReader` `RandomSelector` `RedisWriter` pipe
### Setup Redis (terminal 1)
Launch redis
```
docker-compose up -d
```
Login container
```
docker exec -it redis /bin/sh
```
### Build and Run (terminal 2)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o ingest_redis_parallel -r
```
Run app
```
./ingest_redis_parallel
```
### Ingest Data (terminal 3)
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
Query Redis (terminal 1)
```
redis-cli get "one"
redis-cli get "two"
redis-cli get "three"
redis-cli get "four"
```
Open [browser](http://localhost:8000/v1/pipe) and list all pipes