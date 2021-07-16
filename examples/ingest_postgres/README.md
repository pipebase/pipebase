Demo `PsqlWriter` pipe
### Prepare Postgres (terminal 1)
Start postgres
```
docker-compose up -d
```
Login container
```
docker exec -it postgres /bin/sh
```
Psql login
```
psql -h localhost -p 5432 -U postgres -d postgres -W
Password:postgres
```
Create table
```
CREATE TABLE IF NOT EXISTS records (
    key TEXT PRIMARY KEY,
    value   INTEGER
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
cargo pipe build -o psql -r
```
Run app
```
./psql
```
### Ingest Data and Monitor
Open new terinal and ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
Query postgres (terminal 1)
```
SELECT * FROM records;
```
Open [browser](http://localhost:8000/v1/pipe) and list all pipes