Demo `KubeLogReader` `KafkaProducer` pipe
### Kube client configuration
config kube log reader
```
# catalogs/kube_log_reader.yml
namespace: YOUR_NAMESPACE
pod: YOUR_POD
container: YOUR_CONTAINER
```
mount local `.kube` in `docker-compose.yml`
```
volumes:
    - /PATH/TO/LOCAL/.KUBE:/opt/app/.kube
```
### Setup (terminal 1)
launch kafka, zookeeper, app and create *kube-log* topic
```
docker-compose up -d
```
start consumer
```
docker exec -it kafka kafka-console-consumer --topic kube-log --bootstrap-server kafka:9092
```
