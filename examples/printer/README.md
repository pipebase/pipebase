Demo `Printer` pipe
### Build and Run (terminal 1)
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe check && \
cargo pipe build -o printer -r
```
run app
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
stdout in terminal 1
```
Record { key: "foo", value: 1 }
```