---
type: fix
---

- `lore-transport`: Fixed a client that presents its own credentials being refused. Without a named
  identity, a supplied access token was dropped and no authorization was sent, so the server refused
  the call as not authenticated. Storage sessions against a server that advertises no authentication
  also sent no token, so every repository-scoped push and clone was refused as not authorized. A
  supplied access token is now sent as the authorization token, and a supplied identity token as
  the authentication token.
