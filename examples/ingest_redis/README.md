Demo `RedisStringWriter` pipe
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
cargo pipe build -o ingest_redis -r
```
run app
```
./ingest_redis
```
### Ingest Data (terminal 3)
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
query redis (terminal 1)
```
redis-cli get "foo"
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes