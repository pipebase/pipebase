Demo `SQSMessageReceiver` pipe
### Configuration
Config sqs
```
# catalogs/sqs_msg_receiver.yml
client:
  url: https://sqs.REGION.amazonaws.com/ACCOUNTID/QUEUE_NAME
  message_attribute_names: [Topic, Code]
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
### Populate SQS Message (terminal 2)
```
aws sqs send-message --queue-url "https://sqs.REGION.amazonaws.com/ACCOUNTID/QUEUE_NAME" --message-body '{"msg": "Hello World Zero"}' --message-attributes file://attributes_0.json
aws sqs send-message --queue-url "https://sqs.REGION.amazonaws.com/ACCOUNTID/QUEUE_NAME" --message-body '{"msg": "Hello World One"}' --message-attributes file://attributes_1.json
```
checkout stdout in terminal 1
```
("{\"msg\": \"Good Morning\"}", {"Code": SQSMessageAttributeValue { ty: "Number", data: String("1") }, "Topic": SQSMessageAttributeValue { ty: "String", data: String("Pipe") }})
```