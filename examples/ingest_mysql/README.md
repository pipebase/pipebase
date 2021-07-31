Demo `MySQLWriter` pipe
### Prepare MySQL (terminal 1)
start mysql
```
docker-compose up -d
```
login container
```
docker exec -it mysql /bin/sh
```
mysql login
```
mysql --user=foo --password foo
Enter password: foo
```
create table
```
CREATE TABLE IF NOT EXISTS records (
    `key` VARCHAR(64) NOT NULL PRIMARY KEY,
    `value` INTEGER
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
cargo pipe build -o mysql -r
```
run app
```
./mysql
```
### Ingest Data and Monitor (terminal 3)
ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
query mysql (terminal 1)
```
SELECT * FROM records;
```
open [browser](http://localhost:8000/v1/pipe) and list all pipes