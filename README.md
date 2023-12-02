# Goals
hmq-tui sets out to give easy access to [HiveMQ  REST API](https://docs.hivemq.com/hivemq/4.21/rest-api/specification/).

```
hmq-tui -h localhost:8888
```

# Setup

Generate REST API client code (requires `openapi-generator`):
```
cd hivemq-openapi
openapi-generator generate -i hivemq-4.21.0-openapi.yaml -g rust --additional-properties=useSingleRequestParameter=true
```
