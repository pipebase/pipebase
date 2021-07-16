Demo `FileLineReader` `StringSplitter` `UnorderedGroupAddAggregator` pipes
### Build and Run (terminal 1)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o wc -r
```
Run app
```
./wc
Pair("languages.", Count32(1))
Pair("and", Count32(2))
Pair("run", Count32(1))
Pair("other", Count32(1))
Pair("blazingly", Count32(1))
```