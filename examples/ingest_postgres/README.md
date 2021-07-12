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
psql -h localhost -p 5432 -U postgres -W postgres -d postgres
```
Create table
```
CREATE TABLE IF NOT EXISTS records (
    key TEXT PRIMARY KEY,
    value   INTEGER
);
```
### Build and Run (terminal 2)
Build 
```
cargo pipe new && \
cargo pipe generate && \
cargo pipe check && \
cargo pipe build -o psql \
```
Run app
```
./psql
```
### Ingest Data and Monitor
Ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
Query postgres in terminal 1
```
SELECT * FROM records
```
Open [browser](http://localhost:8000/v1/pipe) and list all pipes