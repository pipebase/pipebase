### Build and Run
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
Setup Redis
```
docker-compose up -d
```
Ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
Open [browser](http://localhost:8000/v1/pipe) and list all pipes