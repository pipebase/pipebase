Demo `KubeLogReader` `KafkaProducer` pipe
### Setup Kafka (terminal 1)
launch kafka and zookeeper
```
docker-compose up -d && \
docker exec -it kafka /bin/sh
```
create topic
```
kafka-topics --create --topic kube-log --bootstrap-server kafka:9092 --replication-factor 1 --partitions 1
```
start consumer
```
kafka-console-consumer --topic kube-log --bootstrap-server kafka:9092
```
### Kube client configuration
```
# catalogs/kube_log_reader.yml
namespace: YOUR_NAMESPACE
pod: YOUR_POD
container: YOUR_CONTAINER
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
cargo pipe build -o kube_log -r
```
run app
```
./kube_log
```
### Monitor log (terminal 1)
check container log ingest to kafka