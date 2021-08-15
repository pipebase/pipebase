Demo `KafkaProducer` pipe
### Setup (terminal 1)
launch kafka, zookeeper and app
```
docker-compose up -d
```
consume topic
```
docker exec -it kafka kafka-console-consumer --topic records --bootstrap-server kafka:9092
```
### Ingest data (terminal 2)
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
### Consumer stdout (terminal 1)
```
{"key":"foo","value":1}
```
