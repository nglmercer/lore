---
type: feature
---

- `lore-server`: A host that embeds the server creates repositories in the running engine with
  `EmbeddedServer::stores().repositories`, which runs the same create as the `RepositoryCreate`
  RPC: a repeated create of the same id and name succeeds, the same id with another name is
  `already_exists`, and an invalid name is refused before anything is written. It performs no
  authorization of its own, so the host decides who may create.
