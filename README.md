# Goals
hmq-tui sets out to give easy access to [HiveMQ  REST API](https://docs.hivemq.com/hivemq/4.21/rest-api/specification/).

# Libaries
[ratatui](https://github.com/ratatui-org/ratatui)
[openapi-generator](https://github.com/OpenAPITools/openapi-generator/blob/master/docs/generators/rust.md)

# Setup

Generate REST API client code (requires `openapi-generator`):
```
cd hivemq-openapi
openapi-generator generate -i hivemq-4.21.0-openapi.yaml -g rust --additional-properties=useSingleRequestParameter=true
```

# Milestones

- [ ] Retrieve a snapshot of all available clients
- [ ] Retrieve the client details of a given client id
- [ ] Have a expendable UI mock for the TUI
- [ ] Implement the TUI frontend & wire in the REST API calls
- [ ] CI/CD the TUI
