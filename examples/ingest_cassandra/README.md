### Prepare Cassandra (terminal 1)
Start cassandra
```
docker-compose up -d
```
Login container
```
docker exec -it cassandra /bin/sh
```
Cql login
```
cqlsh
```
Create keyspace
```
CREATE KEYSPACE IF NOT EXISTS test WITH REPLICATION = {'class' : 'SimpleStrategy', 'replication_factor' : 1};
USE test;
```
Create table
```
CREATE TABLE IF NOT EXISTS records (
    key text PRIMARY KEY,
    value   int
);
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
cargo pipe check && \
cargo pipe build -o cql -r
```
Run app
```
./cql
```
### Ingest Data and Monitor (terminal 3)
Ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
Query cassandra (terminal 1)
```
SELECT * FROM records;
 key | value
-----+-------
 foo |     1
```
Open [browser](http://localhost:8000/v1/pipe) and list all pipes