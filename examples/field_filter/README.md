Demo `FieldVisit` and `FilterMap` pipe
### Build and Run (terminal 1)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o field_filter -r
```
Run app
```
./field_filter
```
### Ingest Data (terminal 2)
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @records.json  \
http://localhost:9000/v1/ingest
```
Stdout in terminal 1
```
[Record { key: "three", value: 3 }, Record { key: "four", value: 4 }]
```
