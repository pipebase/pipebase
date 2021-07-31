Demo `Conversion` `CsvSer` `FileWriter` pipe
### Build and Run (terminal 1)
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o convert_csv -r
```
Create data folder where csv file dump
```
mkdir data
```
run app
```
./convert_csv
"data/Xj3ec8n3yrg83zbR"
```
open csv file under data folder