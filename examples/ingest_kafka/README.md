Demo `KafkaProducer` pipe
### Setup Kafka (terminal 1)
launch kafka and zookeeper
```
docker-compose up -d && \
docker exec -it kafka /bin/sh
```
create topic
```
kafka-topics --create --topic records --bootstrap-server kafka:9092 --replication-factor 1 --partitions 1
```
start consumer
```
kafka-console-consumer --topic records --bootstrap-server kafka:9092
```
### Build and Run (terminal 2)
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o ingest_kafka -r
```
run app
```
./ingest_kafka
```
### Ingest data (terminal 3)
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```