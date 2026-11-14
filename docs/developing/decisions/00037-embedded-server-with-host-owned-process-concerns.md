---
status: accepted
date: 2026-11-14
deciders: Fork maintainers
---

# ADR-00037: Embedded server with host-owned process concerns

## Context and Problem Statement

`lore_server::server::server_main` is a process entry point. It parses the command line,
loads settings from the working directory and the environment, installs the process-wide
tracing subscriber and the signal listeners, runs on Lore's process-wide runtimes, and ends by
shutting those runtimes down. A host application that embeds Lore as a library, such as a web
backend that already owns its own logging, signals and runtime, cannot call it: the subscriber
install fails or replaces the host's, the host loses control of its signals, and the runtime
shutdown is terminal for the whole process (`SHUTDOWN_STARTED` is never cleared), so the host
could not stop and start the server again.

How does a host run the server, with its stores, its authorization and every enabled endpoint,
while keeping what is the process's to own?

## Decision Drivers

- Standalone behaviour must not change, and `server_main` must keep running the same
  initialization as the embedded path, so there is one place to maintain it.
- The host supplies repository policy once, and every enabled transport enforces it.
- A host must be able to stop the server and start it again on the same stores in one process.
- The change must stay small enough to compare with upstream.

## Considered Options

- Run `server_main` on a thread of the host process.
- Copy the startup sequence into the host.
- Parameterize `async_main`, the body of `server_main`, by who owns the process concerns, and
  expose a lifecycle handle over it.

## Decision Outcome

Chosen option: "Parameterize `async_main` and expose a lifecycle handle", because it is the only
option that keeps one startup sequence and lets the host keep what it owns.

`async_main` takes a `ServeMode`: whether to install the tracing subscriber, whether to listen
for signals, the shutdown channel and a readiness notification. `server_main` passes the mode of
a whole process. `EmbeddedServer::start` passes a mode that installs nothing, listens for
nothing and reports readiness, and runs the server as a task on Lore's core runtime, as the
standalone server does.

`ServerConfig` gains an optional `RepositoryAuthorizer` and `RepositoryCatalog`. When set, they
replace the ones the configuration would select, on every transport, because the server already
threads one authorizer into the gRPC, QUIC and HTTP endpoints. `Settings::from_overrides` builds
settings without reading the environment or the working directory and without printing them,
since they may hold credentials.

The embedded server hands the host the stores it serves (`EmbeddedServer::stores`), so the host
can run engine operations on the authoritative stores in process. It keeps serving when no
endpoint is enabled, until the host stops it: the standalone server exits in that case, because it
has nothing else to do. The public gRPC endpoint now honours `[server.grpc] enabled`, which it
ignored; the shipped defaults set it to `true`, so a standalone server on the defaults is
unchanged.

`[server] durable_publication` makes a branch advance flush the content store and then the
mutable store, with sync, before the advance is reported. A host that tells its own callers a
publication succeeded needs it: a measured `SIGKILL` right after an acknowledged push lost the
revision with the default flush delay, and survived with a graceful stop or a kill after the delay.
It is off by default and costs one flush per advance. The mutable store never schedules a delayed
flush when its delay is zero, so the delay cannot be tuned away.

`EmbeddedServer::stop` requests shutdown, waits for the endpoints to drain within
`connection_close_timeout_seconds`, flushes the stores, awaits guarded tasks and releases the
claim on the local stores. It does not shut down Lore's runtimes: they are process-wide and
their shutdown is terminal, so they end with the host process.

### Consequences

- Good, because the standalone server and the embedded one share one startup sequence.
- Good, because a host restarts the server in one process, which `tests/embedded_server.rs`
  covers.
- Bad, because Lore's two runtimes and their threads stay for the life of the process after
  `stop`.
- Bad, because the standalone server releases its store claim only after its runtime has dropped
  request tasks that outlive their connection, and an embedded stop cannot wait for that, so a
  task still running when `stop` returns could use a store the claim no longer covers. Such a
  task cannot start new work once the endpoints are closed.
- Bad, because "ready" means initialization finished and the endpoints are spawned, not that
  every port is bound: each endpoint binds as its own task. A bind failure ends the server, which
  `is_running` and `stop` report.

## More Information

Implemented in `lore-server/src/server.rs` (`ServeMode`, `EmbeddedServer`),
`lore-server/src/server_config.rs` and `lore-server/src/settings.rs`. Tested in
`lore-server/tests/embedded_server.rs`: serve, stop and restart on the same stores in one
process; a failed initialization reports its cause; and a server with no endpoint keeps serving
until stopped.
