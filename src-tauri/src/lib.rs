use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};
use zenoh::{
    internal::{plugins::PluginsManager, runtime::Runtime, runtime::RuntimeBuilder},
    session::ZenohId,
};

// ============================================================================
// Modules
// ============================================================================

mod logs;

use logs::{clear_runtime_context, set_runtime_context, LogCaptureLayer, LogEntry, LogStorage};

// ============================================================================
// sandbox module - Data structures for Rust <-> TypeScript exchange
// ============================================================================

pub mod sandbox {
    use super::*;

    /// Configuration for creating a Zenoh runtime.
    /// This is the data structure exchanged between Rust and TypeScript.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct ZenohConfig {
        /// WebSocket port for the Remote API (default: 10000).
        /// Accepts a port number or a string with format `<local_ip>:<port_number>`.
        pub websocket_port: Option<String>,
    }

    impl ZenohConfig {
        /// Convert sandbox::ZenohConfig to zenoh::config::Config
        pub fn into_zenoh_config(self) -> Result<zenoh::config::Config, String> {
            let mut config = zenoh::config::Config::default();

            // Set mode to peer (default for sandbox)
            config
                .set_mode(Some(zenoh::config::WhatAmI::Peer))
                .map_err(|e| format!("Failed to set mode: {e:?}"))?;

            // Enable admin space
            config
                .adminspace
                .set_enabled(true)
                .map_err(|e| format!("Failed to enable adminspace: {e}"))?;

            // Enable loading plugins
            config
                .plugins_loading
                .set_enabled(true)
                .map_err(|e| format!("Failed to enable plugins loading: {e}"))?;

            // Add remote_api plugin configuration
            config
                .insert_json5("plugins/remote_api", "{}")
                .map_err(|e| format!("Failed to add remote_api plugin config: {e}"))?;

            // Set websocket port if provided
            if let Some(ws_port) = &self.websocket_port {
                config
                    .insert_json5(
                        "plugins/remote_api/websocket_port",
                        &format!(r#""{ws_port}""#),
                    )
                    .map_err(|e| format!("Failed to set websocket_port: {e}"))?;
            }

            Ok(config)
        }
    }
}

// ============================================================================
// State management for Zenoh runtimes
// ============================================================================

/// Holds all active Zenoh runtime runtimes
pub struct ZenohRuntimes {
    runtimes: RwLock<HashMap<ZenohId, (sandbox::ZenohConfig, Runtime)>>,
}

impl ZenohRuntimes {
    pub fn new() -> Self {
        Self {
            runtimes: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for ZenohRuntimes {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tauri commands
// ============================================================================

/// Create a new Zenoh runtime with the given configuration.
/// Returns the ZenohId as a string on success.
#[tauri::command]
async fn zenoh_runtime_start(
    config: sandbox::ZenohConfig,
    state: State<'_, ZenohRuntimes>,
) -> Result<String, String> {
    // Convert sandbox config to zenoh config
    let zenoh_config = config.clone().into_zenoh_config()?;

    // Create plugins manager and add Remote API plugin
    let mut plugins_mgr = PluginsManager::static_plugins_only();
    plugins_mgr.declare_static_plugin::<zenoh_plugin_remote_api::RemoteApiPlugin, &str>(
        "remote_api",
        true,
    );

    // Build the runtime
    let mut runtime = RuntimeBuilder::new(zenoh_config)
        .plugins_manager(plugins_mgr)
        .build()
        .await
        .map_err(|e| format!("Failed to build Zenoh runtime: {e}"))?;

    // Get the ZenohId before starting
    let zid = runtime.zid();

    // Set the runtime context for log capturing
    set_runtime_context(zid);

    // Start the runtime
    runtime
        .start()
        .await
        .map_err(|e| {
            clear_runtime_context();
            format!("Failed to start Zenoh runtime: {e}")
        })?;

    // Store the runtime in state
    {
        let mut runtimes = state.runtimes.write().await;
        runtimes.insert(zid, (config, runtime));
    }

    Ok(zid.to_string())
}

/// stop (close) a Zenoh runtime by its ZenohId string.
#[tauri::command]
async fn zenoh_runtime_stop(
    zid: String,
    runtimes_state: State<'_, ZenohRuntimes>,
    logs_state: State<'_, LogStorage>,
) -> Result<(), String> {
    // Parse the ZenohId from string
    let zenoh_id = ZenohId::from_str(&zid)
        .map_err(|e| format!("Invalid ZenohId '{}': {}", zid, e))?;

    // Remove from state and get the runtime
    let runtime = {
        let mut runtimes = runtimes_state.runtimes.write().await;
        runtimes
            .remove(&zenoh_id)
            .map(|(_, runtime)| runtime)
            .ok_or_else(|| format!("Zenoh runtime '{}' not found", zid))?
    };

    // Close the runtime explicitly
    runtime
        .close()
        .await
        .map_err(|e| format!("Failed to close Zenoh runtime: {e}"))?;

    // Clear logs for this runtime
    logs_state.clear_logs(&zenoh_id);

    // Clear runtime context if it matches
    if logs::get_runtime_context() == Some(zenoh_id) {
        clear_runtime_context();
    }

    Ok(())
}

/// List all active Zenoh runtime ZenohIds as strings.
#[tauri::command]
async fn zenoh_runtime_list(state: State<'_, ZenohRuntimes>) -> Result<Vec<String>, String> {
    let runtimes = state.runtimes.read().await;
    let zids: Vec<String> = runtimes.keys().map(|zid| zid.to_string()).collect();
    Ok(zids)
}

/// Get the configuration of a Zenoh runtime by its ZenohId string.
#[tauri::command]
async fn zenoh_runtime_config(
    zid: String,
    state: State<'_, ZenohRuntimes>,
) -> Result<sandbox::ZenohConfig, String> {
    // Parse the ZenohId from string
    let zenoh_id = ZenohId::from_str(&zid)
        .map_err(|e| format!("Invalid ZenohId '{}': {}", zid, e))?;

    // Get the config from state
    let runtimes = state.runtimes.read().await;
    let (config, _) = runtimes
        .get(&zenoh_id)
        .ok_or_else(|| format!("Zenoh runtime '{}' not found", zid))?;

    Ok(config.clone())
}

/// Get a page of logs from a specific zenoh runtime.
/// Page 0 returns the most recent logs.
#[tauri::command]
async fn zenoh_runtime_log(
    zid: String,
    page: usize,
    state: State<'_, LogStorage>,
) -> Result<Vec<LogEntry>, String> {
    // Parse the ZenohId from string
    let zenoh_id = ZenohId::from_str(&zid)
        .map_err(|e| format!("Invalid ZenohId '{}': {}", zid, e))?;

    Ok(state.get_page(&zenoh_id, page))
}

// ============================================================================
// Tauri application entry point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize log storage
    let log_storage = LogStorage::default();

    // Set up tracing subscriber with custom log capture layer
    let log_layer = LogCaptureLayer::new(log_storage.logs_ref(), 10_000);

    // Initialize tracing subscriber
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_level(true)
                .with_filter(tracing_subscriber::filter::LevelFilter::DEBUG),
        )
        .with(log_layer)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ZenohRuntimes::new())
        .manage(log_storage)
        .invoke_handler(tauri::generate_handler![
            zenoh_runtime_start,
            zenoh_runtime_stop,
            zenoh_runtime_list,
            zenoh_runtime_config,
            zenoh_runtime_log,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
