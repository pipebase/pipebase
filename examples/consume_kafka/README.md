Demo `KafkaConsumer` pipe
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
Publish record message
```
kafka-console-producer --topic records --bootstrap-server kafka:9092
> {"key": "foo", "value": 1}
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
cargo pipe build -o consume_kafka -r
```
Run app
```
./consume_kafka
Record { key: "foo", value: 1 }
```