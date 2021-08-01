Demo `PipeErrorPrinter`
### Build and Run (terminal 1)
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o error_printer -r
```
run app
```
./error_printer
```
### Ingest Data (terminal 2)
populate record with missing key
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @bad_record.json  \
http://localhost:9000/v1/ingest
```
stdout in terminal 1
```
[Error] pipe: 'json', details: 'Error("missing field `key`", line: 1, column: 13)'
```