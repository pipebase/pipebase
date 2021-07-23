Demo `MySQLWriter` pipe
### Prepare MySQL (terminal 1)
Start mysql
```
docker-compose up -d
```
Login container
```
docker exec -it mysql /bin/sh
```
MySQL login
```
mysql --user=foo --password foo
Enter password: foo
```
Create table
```
CREATE TABLE IF NOT EXISTS records (
    `key` VARCHAR(64) NOT NULL PRIMARY KEY,
    `value` INTEGER
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
cargo pipe build -o mysql -r
```
Run app
```
./mysql
```
### Ingest Data and Monitor
Open new terinal and ingest sample data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
Query mysql (terminal 1)
```
SELECT * FROM records;
```
Open [browser](http://localhost:8000/v1/pipe) and list all pipes