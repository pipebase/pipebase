Demo `KubeEventReader` pipe
### Kube client configuration
config kube event reader
```
# catalogs/kube_event_reader.yml
namespace: YOUR_NAMESPACE # optional
```
mount local `.kube` in `docker-compose.yml`
```
volumes:
    - /PATH/TO/LOCAL/.KUBE:/opt/app/.kube
```
### Setup (terminal 1)
launch kafka, zookeeper, app and create *kube-event* topic
```
docker-compose up -d
```
start consumer
```
docker exec -it kafka kafka-console-consumer --topic kube-event --bootstrap-server kafka:9092
```
