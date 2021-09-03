Demo `AmqpPublisher` `AmqpConsumer` pipe
### Setup (terminal 1)
launch rabbitmq and app
```sh
docker-compose up -d
docker logs -f app
```
### Ingest Data (terminal 2)
```sh
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
check stdout in terminal 1
```
Record { key: "foo", value: 1 }
```