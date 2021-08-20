Demo `InMemoryWindowCollector` `UnorderedGroupAddAggregator`
### Setup (terminal 1)
```sh
docker-compose up -d
docker logs -f app
```
### Ingest Data (terminal 2)
ingest data
```sh
for (( i=0; i < 10; i++ )) 
do
    curl -i -X POST \
    -H "Content-Type: application/json" \
    -d @resources/record_${i}.json \
    http://localhost:9000/v1/ingest && \
    sleep 1
done
```
check stdout of app logs in terminal 1