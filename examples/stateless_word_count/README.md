Demo `FileLineReader` `StringSplitter` `IteratorStreamer` `UnorderedGroupAddAggregator` pipes
### Build and Run (terminal 1)
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o wc -r
```
run app
```
./wc
Pair("languages.", Count32(1))
Pair("and", Count32(2))
Pair("run", Count32(1))
Pair("other", Count32(1))
Pair("blazingly", Count32(1))
```