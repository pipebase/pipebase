Demo `Timer` pipe
### Build and Run (terminal 1)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o timer -r
```
Run app
```
./timer
0
1
2
3
4
5
6
7
8
9
```
