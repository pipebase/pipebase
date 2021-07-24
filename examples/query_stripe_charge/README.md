Query strpie charge by customer id with `ReqwestQuery` pipe
### Fill in stripe credentials / customer id
```
# catalogs/reqwest_query.yml
base_url: https://api.stripe.com/v1/charges
headers:
  Content-Type: application/json
basic_auth:
  username: sk_test*
  password: *
```
```
# query.json
{
    "query": {
        "customer": "cus_*"
    }
}
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
cargo pipe build -o query_stripe -r
```
Run app
```
./query_stripe
```

### Query (terminal 2)
```
curl -i -X POST \
-H "Content-Type: application/json" \
-d @query.json  \
http://localhost:9000/v1/ingest
```
checkout terminal 1, charge for customer stdout
```
[Charge { id: "CHARGE_ID", amount: AMOUNT }]
```

### Stripe API reference
[`List all charges`]: https://stripe.com/docs/api/charges/list
