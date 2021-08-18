Demo `TextCollector` `ReqwestPoster` pipe
### Setup
launch elasticsearch and app
```sh
docker-compose up -d
```
create index
```sh
curl -X PUT localhost:9200/records
```
### Ingest Data
ingest sample data
```sh
for (( i=0; i < 10; i++ )) 
do
    curl -i -X POST \
    -H "Content-Type: application/json" \
    -d @resources/record_${i}.json \
    http://localhost:9000/v1/ingest
done
```
query elasticsearch
```sh
curl http://localhost:9200/records/_search?q=key:* | jq -r .hits.hits[]._source
```