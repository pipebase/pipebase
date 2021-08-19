Demo `RedisStringBatchWriter` pipe
### Setup
launch redis and app
```sh
docker-compose up -d
docker logs -f app
```
### Ingest Data
```sh
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
query redis
```sh
docker exec redis redis-cli get "foo" && \
docker exec redis redis-cli get "bar"
```