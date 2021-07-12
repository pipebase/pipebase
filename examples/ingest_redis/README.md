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
Build
```
cargo pipe new && \
cargo pipe generate && \
cargo pipe check && \
cargo pipe build -o ingest_redis
```
Run app
```
./ingest_redis
```
### Ingest Data and Monitor Pipe 
Open new terinal and ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
Query Redis (terminal 1)
```
redis-cli get "foo"
```
Open [browser](http://localhost:8000/v1/pipe) and list all pipes