Demo `CqlWriter` pipe
### Prepare Cassandra (terminal 1)
start cassandra
```
docker-compose up -d
```
login container
```
docker exec -it cassandra /bin/sh
```
cql login
```
cqlsh
```
create keyspace
```
CREATE KEYSPACE IF NOT EXISTS test WITH REPLICATION = {'class' : 'SimpleStrategy', 'replication_factor' : 1};
USE test;
```
create table
```
CREATE TABLE IF NOT EXISTS records (
    key text PRIMARY KEY,
    value   int
);
```
### Build and Run (terminal 2)
init
```
cargo pipe new
```
build 
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o cql -r
```
run app
```
./cql
```
### Ingest Data and Monitor (terminal 3)
ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
query cassandra (terminal 1)
```
SELECT * FROM records;
 key | value
-----+-------
 foo |     1
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes