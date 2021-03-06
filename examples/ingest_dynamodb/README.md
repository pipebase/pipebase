Demo `DynamoDBWriter`
### Create DynamoDB Table (terminal 1)
aws environment variable
```
export AWS_ACCESS_KEY_ID=
export AWS_SECRET_ACCESS_KEY=
export AWS_DEFAULT_REGION=
```
create `records` table for test
```
aws dynamodb create-table --attribute-definitions AttributeName=id,AttributeType=S AttributeName=value,AttributeType=N \
    --key-schema AttributeName=id,KeyType=HASH AttributeName=value,KeyType=RANGE \
    --table-name records \
    --billing-mode PAY_PER_REQUEST
```
### Build and Run (terminal 2)
init
```
cargo pipe new
```
build
```
cargo pipe validate -o -p && \
cargo pipe generate && \
cargo pipe build -o ingest_dynamodb -r
```
run app
```
./ingest_dynamodb
```
### Ingest Data (terminal 3)
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @record.json  \
http://localhost:9000/v1/ingest
```
### Query DynamoDB (terminal 1)
```
aws dynamodb query --table-name records \
    --key-condition-expression "id = :id" \
    --expression-attribute-values  '{":id":{"S":"foo"}}'
{
    "Items": [
        {
            "id": {
                "S": "foo"
            },
            "value": {
                "N": "1"
            }
        }
    ],
    "Count": 1,
    "ScannedCount": 1,
    "ConsumedCapacity": null
}
```