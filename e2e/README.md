A collection of e2e test suites
## Run test
setup
```bash
# at project root directory
./e2e/setup.sh -d .
```
run apps
```bash
# at project root directory
docker-compose -f e2e/TEST_NAME/docker-compose.yml up -d
```
run validator
```bash
# at project root directory
./e2e/run.sh -f e2e/TEST_NAME/entry.json
```