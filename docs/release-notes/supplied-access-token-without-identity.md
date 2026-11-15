---
type: fix
---

- `lore-transport`: Fixed a client that presents its own access token without naming an identity
  sending no authorization at all, so the server refused the call as not authenticated. The
  supplied token is now the authorization token, as it already was when an identity was named.
