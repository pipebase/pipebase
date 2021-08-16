Demo `PsqlPreparedWriter` pipe
### Setup (terminal 1)
start postgres and app
```sh
docker-compose up -d
```
create table
```sh
docker exec postgres psql -h localhost -p 5432 -U postgres -d postgres -w -c "CREATE TABLE IF NOT EXISTS records ( key TEXT PRIMARY KEY, value INTEGER )"
```
### Ingest Data
```sh
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
query postgres
```sh
docker exec postgres psql -h localhost -p 5432 -U postgres -d postgres -w -c "SELECT key, value FROM records"
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes