Demo `Printer` pipe
### Build and Run (terminal 1)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe check && \
cargo pipe build -o printer -r
```
Run app
```
./printer
```
### Ingest Data (terminal 2)
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
Stdout in terminal 1
```
Record { key: "foo", value: 1 }
```