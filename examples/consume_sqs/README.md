Demo `SQSMessageReceiver` pipe
### Configuration
Config sqs
```
# catalogs/sqs_msg_receiver.yml
client:
  url: 
initial_delay:
  Secs: 1
interval:
  Secs: 30
```
SQS environment variable
```
export AWS_ACCESS_KEY_ID=
export AWS_SECRET_ACCESS_KEY=
export AWS_DEFAULT_REGION=
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
cargo pipe build -o sqs -r
```
Run app
```
./sqs
```
### Populate SQS Message
```
aws sqs send-message --queue-url "https://sqs.REGION.amazonaws.com/*" --message-body '{"msg": "Hello World"}' --message-attributes file://attribute.json
```