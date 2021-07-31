Demo `Timer` pipe
### Build and Run (terminal 1)
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o timer -r
```
run app
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
