Demo `Projection` `FileReader` pipe
### Build and Run (terminal 1)
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o project -r
```
run app
```
./project
[Record { key: "foo", value: 3.0 }, Record { key: "bar", value: 3.0 }]
```