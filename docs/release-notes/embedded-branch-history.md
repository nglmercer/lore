---
type: feature
---

- `lore-server`: A host that embeds the server lists a branch's recent revisions in process with
  `EmbeddedServer::stores().repositories.branch_history`, the listing the branch-revision RPC runs,
  newest first with a continuation flag. A missing branch is reported as not found.
