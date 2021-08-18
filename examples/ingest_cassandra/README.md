Demo `CqlWriter` pipe
### Setup
start cassandra and app
```sh
# app sleep 60 second for cassandra ready
docker-compose up -d
```
create keyspace
```sh
docker exec cassandra cqlsh -e "CREATE KEYSPACE IF NOT EXISTS test WITH REPLICATION = {'class' : 'SimpleStrategy', 'replication_factor' : 1}" localhost 9042
```
create table
```sh
docker exec cassandra cqlsh -e "CREATE TABLE IF NOT EXISTS test.records (key text PRIMARY KEY, value int)" localhost 9042
```
### Ingest Data
ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
query cassandra
```
docker exec cassandra cqlsh -e "SELECT key, value FROM test.records WHERE key = 'foo'" localhost 9042
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes