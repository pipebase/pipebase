Demo `RedisPublisher` `RedisSubscriber` pipe
### Setup Redis (terminal 1)
launch redis
```
docker-compose up -d
```
### Build and Run
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o pubsub_redis -r
```
run app
```
./pubsub_redis
```
### Ingest Data (terminal 2)
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
check stdout in terminal 1
```
"Hello World"
```