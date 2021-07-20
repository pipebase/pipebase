Demo `Projection` `FileReader` pipe
### Build and Run (terminal 1)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o project_file -r
```
Run app
```
./project_file
[Record { key: "foo", value: 3.0 }, Record { key: "bar", value: 3.0 }]
```