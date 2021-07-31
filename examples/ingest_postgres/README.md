Demo `PsqlWriter` pipe
### Prepare Postgres (terminal 1)
start postgres
```
docker-compose up -d
```
login container
```
docker exec -it postgres /bin/sh
```
psql login
```
psql -h localhost -p 5432 -U postgres -d postgres -W
Password:postgres
```
create table
```
CREATE TABLE IF NOT EXISTS records (
    key TEXT PRIMARY KEY,
    value   INTEGER
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
cargo pipe build -o psql -r
```
run app
```
./psql
```
### Ingest Data and Monitor (terminal 3)
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
query postgres (terminal 1)
```
SELECT * FROM records;
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes