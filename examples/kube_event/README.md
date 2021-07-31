Demo `KubeEventReader` pipe
### Setup Kafka (terminal 1)
launch kafka and zookeeper
```
docker-compose up -d && \
docker exec -it kafka /bin/sh
```
create topic
```
kafka-topics --create --topic kube-event --bootstrap-server kafka:9092 --replication-factor 1 --partitions 1
```
start consumer
```
kafka-console-consumer --topic kube-event --bootstrap-server kafka:9092
```
### Kube client configuration
```
# catalogs/kube_log_reader.yml
namespace: YOUR_NAMESPACE # optional
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
cargo pipe build -o kube_event -r
```
run app
```
./kube_event
```
### Monitor log (terminal 1)
check kube events ingest to kafka