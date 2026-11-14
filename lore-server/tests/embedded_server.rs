// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! The server embedded in a host process. A binary of its own: the server's runtimes and log
//! callback are process-wide, and the host here is this test process.
use std::net::TcpListener;
use std::time::Duration;

use lore_server::server::EmbeddedServer;
use lore_server::server_config::ServerConfig;
use lore_server::settings::Settings;
use tokio::net::TcpStream;

/// A port that was free a moment ago. The server binds it itself, so the window between this
/// returning and the bind is the test's only race.
fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind an ephemeral port")
        .local_addr()
        .expect("the bound address")
        .port()
}

/// Settings that serve gRPC only, on `port`, over local stores under `root`.
fn settings_for(root: &std::path::Path, port: u16) -> (Settings, lore_storage::hash::StringHash) {
    settings_with_grpc(root, port, true)
}

/// Settings over local stores under `root`, with the gRPC endpoint on `port` when `grpc` is
/// set, and no endpoint at all otherwise.
fn settings_with_grpc(
    root: &std::path::Path,
    port: u16,
    grpc: bool,
) -> (Settings, lore_storage::hash::StringHash) {
    let root = root.to_str().expect("a UTF-8 path").replace('\\', "/");
    let overrides = format!(
        r#"
        [server.quic]
        enabled = false
        [server.http]
        enabled = false
        [server.grpc]
        enabled = {grpc}
        host = "127.0.0.1"
        port = {port}
        [immutable_store.local]
        path = "{root}/immutable"
        [mutable_store.local]
        path = "{root}/mutable"
        "#
    );
    Settings::from_overrides(&overrides).expect("embedded settings")
}

async fn accepts_connections(port: u16) -> bool {
    tokio::time::timeout(
        Duration::from_secs(2),
        TcpStream::connect(("127.0.0.1", port)),
    )
    .await
    .is_ok_and(|connected| connected.is_ok())
}

/// Waits up to ten seconds for the endpoint to accept connections. The server reports ready
/// once its endpoints are spawned, and an endpoint binds its port a moment after that.
async fn eventually_accepts_connections(port: u16) -> bool {
    for _ in 0..100 {
        if accepts_connections(port).await {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    false
}

/// The gate for the embedding lifecycle: start inside a host runtime, serve, drain, stop, and
/// start again on the same stores in the same process, without a nested-runtime failure or a
/// store claim left behind.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn the_server_serves_stops_and_restarts_on_the_same_stores() {
    let root = lore_base::test_util::TempDir::new("lore-embedded-");
    let port = free_port();

    for round in 0..2 {
        let (settings, hash) = settings_for(root.path(), port);
        let server = EmbeddedServer::start(settings, hash, ServerConfig::default())
            .await
            .unwrap_or_else(|error| panic!("round {round}: the server starts: {error:#}"));

        assert!(server.is_running(), "round {round}: running after start");
        assert!(
            eventually_accepts_connections(port).await,
            "round {round}: the gRPC endpoint accepts connections"
        );

        server
            .stop()
            .await
            .unwrap_or_else(|error| panic!("round {round}: the server stops: {error:#}"));
        assert!(
            !accepts_connections(port).await,
            "round {round}: the endpoint is closed after stop"
        );
    }
}

/// Settings that cannot be served fail the start with the cause, rather than leaving a handle
/// to a server that is not there.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_start_that_cannot_initialize_reports_the_error() {
    let root = lore_base::test_util::TempDir::new("lore-embedded-bad-");
    let blocker = root.path().join("not-a-directory");
    std::fs::write(&blocker, b"a file where the store directory should be").expect("write file");
    let (settings, hash) = settings_for(&blocker, free_port());

    let result = EmbeddedServer::start(settings, hash, ServerConfig::default()).await;

    assert!(result.is_err(), "a store under a regular file cannot open");
}

/// A host that wants the stores and its own authorization but no native listener runs the
/// server with every endpoint off: it must keep serving until stopped, and open no port.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_server_without_endpoints_keeps_serving_its_stores_until_stopped() {
    let root = lore_base::test_util::TempDir::new("lore-embedded-quiet-");
    let port = free_port();
    let (settings, hash) = settings_with_grpc(root.path(), port, false);

    let server = EmbeddedServer::start(settings, hash, ServerConfig::default())
        .await
        .expect("the server starts with no endpoint");
    // Long enough for an endpoint that was wrongly started to bind, and for a server that
    // wrongly ends at once to have ended.
    tokio::time::sleep(Duration::from_millis(500)).await;

    assert!(server.is_running(), "it keeps running with no endpoint");
    assert!(
        !accepts_connections(port).await,
        "a disabled gRPC endpoint opens no port"
    );
    assert!(
        server.stores().lock.is_some(),
        "the stores are handed to the host"
    );

    server.stop().await.expect("the server stops");
}
