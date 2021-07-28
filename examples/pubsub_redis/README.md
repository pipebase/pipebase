Demo `RedisPublisher` `RedisSubscriber` pipe
### Setup Redis (terminal 1)
Launch redis
```
docker-compose up -d
```
### Build and Run (terminal 1)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o pubsub_redis -r
```
Run app
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