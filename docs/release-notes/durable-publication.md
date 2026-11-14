---
type: feature
---

- `lore-server`: `[server] durable_publication = true` flushes and syncs the stores before a
  branch advance is reported to the client, so a push the server acknowledged survives the process
  being killed. It is off by default, which keeps the stores' flush delay: a push is lost if the
  process is killed before the delay elapses. Each advance pays one flush.
