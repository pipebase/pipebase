Demo `LocalFilePathVisitor` pipe
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
cargo pipe build -o file_path -r
```
Run app
Cron scan files under directory
```
./file_path
"resources/file1.txt"
"resources/file0.txt"
"resources/file1.txt"
"resources/file0.txt"
```