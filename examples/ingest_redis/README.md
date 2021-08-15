Demo `RedisStringWriter` pipe
### Setup (terminal 1)
launch redis and app
```sh
docker-compose up -d
docker logs -f app
```
### Ingest Data (terminal 3)
```sh
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
query redis (terminal 1)
```sh
docker exec redis redis-cli get "foo"
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes
### Shutdown Pipes and App
```sh
curl -i -X POST http://localhost:9000/v1/shutdown
```
refresh browser all pipes in `done` state, then stop context server and app exit
```sh
curl -i -X POST http://localhost:8000/v1/shutdown
```