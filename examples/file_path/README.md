Demo `LocalFilePathVisitor` pipe
### Build and Run (terminal 1)
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o file_path -r
```
run app, scan files under directory in period
```
./file_path
"resources/file1.txt"
"resources/file0.txt"
"resources/file1.txt"
"resources/file0.txt"
```