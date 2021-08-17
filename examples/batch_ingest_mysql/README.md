Demo `MySQLPreparedWriter` pipe
### Setup (terminal 1)
start mysql and app
```sh
docker-compose up -d
```
create table
```sh
docker exec -e MYSQL_PWD=foo mysql mysql --host=localhost --port=3306 --user=foo --database=foo --execute="CREATE TABLE IF NOT EXISTS records ( \`key\` VARCHAR(64) NOT NULL PRIMARY KEY, \`value\` INTEGER );"
```
### Ingest Data
ingest sample data
```sh
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
query mysql
```sh
docker exec -e MYSQL_PWD=foo mysql mysql --host=localhost --port=3306 --user=foo --database=foo --execute="SELECT \`key\`, \`value\` FROM records"
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes