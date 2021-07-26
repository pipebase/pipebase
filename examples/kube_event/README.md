Demo `KubeEventReader` pipe
### Setup Kafka (terminal 1)
Launch kafka and zookeeper
```
docker-compose up -d && \
docker exec -it kafka /bin/sh
```
Create topic
```
kafka-topics --create --topic kube-event --bootstrap-server kafka:9092 --replication-factor 1 --partitions 1
```
Start consumer
```
kafka-console-consumer --topic kube-event --bootstrap-server kafka:9092
```
### Kube client configuration
```
# catalogs/kube_log_reader.yml
namespace: YOUR_NAMESPACE # optional
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
cargo pipe build -o kube_event -r
```
Run app
```
./kube_event
```
### Monitor log (terminal 1)
Check kube events ingest to kafka