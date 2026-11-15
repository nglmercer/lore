---
type: feature
---

- `lore-server`: A host that embeds the server reads a branch's head in process with
  `EmbeddedServer::stores().repositories.branch_head`, which resolves the branch name and reads the
  head the branch RPCs read. A branch with no revision, or a branch that does not exist, reads as
  absent rather than as an error.
