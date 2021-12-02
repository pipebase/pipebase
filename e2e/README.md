A collection of e2e test suites
## Run Test
setup
```sh
# at project root directory
./e2e/setup.sh -d .
```
run apps
```sh
# at project root directory
docker-compose -f e2e/TEST_NAME/docker-compose.yml up -d
```
run test
```sh
# at project root directory
cargo test --package e2e_TEST_NAME --features itest
```