Demo email notification `SnsPublisher` pipe
### Create topic and subscription (terminal 1)
export aws environment variable
```
export AWS_DEFAULT_REGION=
export AWS_ACCESS_KEY_ID=
export AWS_SECRET_ACCESS_KEY=
```
create topic
```
aws sns create-topic --name YOUR_TOPIC
{
    "TopicArn": "YOUR_TOPIC_ARN"
}
```
create subscription and confirm subscription email
```
aws sns subscribe --topic-arn 'YOUR_TOPIC_ARN' \
    --protocol 'email' \
    --notification-endpoint 'YOUR_EMAIL'
{
    "SubscriptionArn": "pending confirmation"
}
```
fill sns publisher config
```
# catalogs/sns_publisher.yml
topic_arn: YOUR_TOPIC_ARN
region: YOUR_REGION
subscribers:
  - protocol: email
    endpoint: YOUR_EMAIL
```
### Build and Run
Init
```
cargo pipe new
```
Build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o sns -r
```
Run app
```
./sns
```
### Ingest data
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @message.json  \
http://localhost:9000/v1/ingest
```
check your email inbox
```
AWS Notifications
Hello World
```