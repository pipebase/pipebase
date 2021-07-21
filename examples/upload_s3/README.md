Demo `S3Writer` pipe
### Config S3
S3Writer config file
```
# catalogs/s3_writer.yml
region: MY_REGION
bucket: MY_BUCKET
directory: MY_S3_DIRECTORY_AS_KEY_PREFIX
filename_ext: json
```
S3 environment variable
```
export AWS_ACCESS_KEY_ID=
export AWS_SECRET_ACCESS_KEY=
```
### Build and Run (terminal 1)
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o s3 -r
```
Run app
```
./s3
```
`resources/transactions.json` uploaded to s3