Demo `KafkaConsumer` pipe
### Setup
Launch kafka, zookeeper and app
```sh
docker-compose up -d
```
Publish record message
```sh
docker exec -it kafka kafka-console-producer --topic records --bootstrap-server localhost:9092
> {"key": "foo", "value": 1}
# CTRL+C
```
### Monitor App
```sh
docker logs app
Record { key: "foo", value: 1 }
```