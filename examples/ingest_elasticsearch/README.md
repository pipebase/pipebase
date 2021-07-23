Demo `InMemoryBagCollector` `ReqwestPoster` pipe
### Setup Elasticsearch and Kibana (terminal 1)
Launch elasticsearch and kibana
```
docker-compose up -d
```
Create index
```
curl -XPUT localhost:9200/records
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
### Ingest Data
Open new terinal and ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record0.json  \
-d @record1.json  \
http://localhost:9000/v1/ingest
```