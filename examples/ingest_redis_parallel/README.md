Demo `IteratorReader` `RandomSelector` `RedisWriter` pipe
### Setup (terminal 1)
launch redis and app
```sh
docker-compose up -d
```
### Ingest Data
```sh
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
query redis (terminal 1)
```sh
docker exec redis redis-cli get "one" && \
docker exec redis redis-cli get "two" && \
docker exec redis redis-cli get "three" && \
docker exec redis redis-cli get "four"
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes