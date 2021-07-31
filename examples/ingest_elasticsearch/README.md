Demo `TextCollector` `ReqwestPoster` pipe
### Setup Elasticsearch (terminal 1)
Launch elasticsearch
```
docker-compose up -d
```
Create index
```
curl -X PUT localhost:9200/records
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
cargo pipe build -o ingest_es -r
```
Run app
```
./ingest_es
```
### Ingest Data (terminal 3)
Open new terminal and ingest sample data
```
for (( i=0; i < 10; i++ )) 
do
    curl -i -X POST \
    -H "Content-Type: application/json" \
    -d @resources/record_${i}.json \
    http://localhost:9000/v1/ingest
done
```
### Query Elasticsearch (terminal 1)
```
curl http://localhost:9200/records/_search?q=key:* | jq -r .hits.hits[]._source
{
  "key": "zero",
  "value": 0
}
{
  "key": "one",
  "value": 1
}
{
  "key": "two",
  "value": 2
}
...
```