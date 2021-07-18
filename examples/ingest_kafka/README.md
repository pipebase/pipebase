Demo `KafkaProducer` pipe
### Setup Kafka (terminal 1)
Launch kafka and zookeeper
```
docker-compose up -d && \
docker exec -it kafka /bin/sh
```
Create topic
```
kafka-topics --create --topic records --bootstrap-server kafka:9092 --replication-factor 1 --partitions 1
```
Start consumer
```
kafka-console-consumer --topic records --bootstrap-server kafka:9092
```
### Build and Run (terminal 2)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o ingest_kafka -r
```
Run app
```
./producer_kafka
```
### Ingest data (terminal 3)
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```