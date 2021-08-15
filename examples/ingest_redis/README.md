Demo `RedisStringWriter` pipe
### Setup (terminal 1)
launch redis and app
```
docker-compose up -d
docker logs -f app
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
docker exec redis redis-cli get "foo"
1
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes
### Shutdown Pipes and App
```
curl -i -X POST http://localhost:9000/v1/shutdown
```
refresh browser all pipes in `done` state, then further stop context server and app exit
```
curl -i -X POST http://localhost:8000/v1/shutdown
```