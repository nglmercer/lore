// SPDX-FileCopyrightText: 2026 Epic Games, Inc.
// SPDX-License-Identifier: MIT
//! Server configuration for derived binaries and embedding hosts.
//!
//! This module provides [`ServerConfig`], the configuration struct that derived
//! server binaries pass to [`server_main()`](crate::server::server_main), and that
//! hosts embedding the server pass to
//! [`EmbeddedServer::start()`](crate::server::EmbeddedServer::start), to inject
//! plugins, hooks, resource detectors and repository policy.

use std::sync::Arc;

use crate::authnz::repository_authorizer::RepositoryAuthorizer;
use crate::authnz::repository_catalog::RepositoryCatalog;
use crate::hooks::HookRegistrationContext;
use crate::hooks::HookRegistry;
use crate::plugins::PluginRegistry;
use crate::telemetry::resource_provider::ResourceDetectorProvider;

/// Callback type for hook registration.
///
/// Derived crates provide closures that register their hook factories with the
/// hook registry, using the registration context for runtime dependencies
/// (e.g., notification sender).
pub type HookRegistrationCallback =
    Box<dyn FnOnce(&mut HookRegistry, &HookRegistrationContext) + Send>;

/// Configuration passed to [`server_main()`](crate::server::server_main) by
/// derived binaries.
///
/// This struct allows derived crates to inject plugins, hooks, and resource
/// detectors without the server library needing compile-time knowledge of them.
///
/// # Example
///
/// ```
/// use lore_server::server_config::ServerConfig;
///
/// // Base server with no plugins
/// let config = ServerConfig::default();
/// ```
pub struct ServerConfig {
    /// Pre-populated plugin registry with all desired plugins registered.
    pub plugin_registry: PluginRegistry,

    /// Callbacks for registering hooks. Called after the notification system is
    /// initialized so that [`HookRegistrationContext`] has a valid notification
    /// sender.
    pub hook_registration_callbacks: Vec<HookRegistrationCallback>,

    /// Optional provider for telemetry resource detectors (e.g., AWS, Nomad).
    /// When `None`, no environment-specific resource detectors are added.
    pub resource_detector_provider: Option<Box<dyn ResourceDetectorProvider>>,

    /// Repository policy supplied by the host. When `Some`, it answers every repository
    /// access check on every enabled transport instead of the authorizer the configuration
    /// would select, so the gRPC, QUIC and HTTP endpoints all enforce one policy. When
    /// `None`, the authorizer is selected from `[server.auth]` as before.
    pub repository_authorizer: Option<Arc<dyn RepositoryAuthorizer>>,

    /// Repository listing supplied by the host. When `None`, the catalog is selected from
    /// the configuration as before. A host that supplies an authorizer will normally supply
    /// the matching catalog, so a listing shows only what the authorizer would allow.
    pub repository_catalog: Option<Arc<dyn RepositoryCatalog>>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            plugin_registry: PluginRegistry::new(),
            hook_registration_callbacks: vec![],
            resource_detector_provider: None,
            repository_authorizer: None,
            repository_catalog: None,
        }
    }
}
