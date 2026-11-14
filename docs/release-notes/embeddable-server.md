---
type: feature
---

- `lore-server`: A host application can embed the server as a library with `EmbeddedServer`,
  which starts it inside the host's runtime, leaves the tracing subscriber and the process
  signals to the host, and stops and restarts it in the same process.
- `lore-server`: `ServerConfig` accepts a repository authorizer and catalog from the host, which
  every enabled transport then enforces in place of the ones the configuration selects.
- `lore-server`: `Settings::from_overrides` builds settings from TOML without reading the
  environment or the working directory and without printing them.
- `lore-server`: The public gRPC endpoint now honours `[server.grpc] enabled`. It previously
  started whatever the setting said. The shipped default is `true`, so a server on the defaults is
  unchanged; set it to `false` to run without the endpoint.
