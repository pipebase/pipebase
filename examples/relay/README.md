Demo `ReqwestPoster` pipe
### Build Sender
Init
```
cargo pipe -d send new
```
Build
```
cargo pipe -d send validate -o -p && \
cargo pipe -d send generate && \
cargo pipe -d send build -o sender -r
```
### Build Receiver
Init
```
cargo pipe -d receive new
```
Build
```
cargo pipe -d receive validate -o -p && \
cargo pipe -d receive generate && \
cargo pipe -d receive build -o receiver -r
```
### Run sender (terminal 1)
```
./sender
```
### Run receiver (terminal 2)
```
./receiver
```
### Ingest data (terminal 3)
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
Monitor terminal 2
```
$ ./receiver
Record { key: "foo", value: 1 }
```