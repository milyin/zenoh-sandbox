use std::{collections::HashMap, str::FromStr};

use serde::{Deserialize, Serialize};
use tauri::State;
use tokio::sync::RwLock;
use zenoh::{
    internal::{plugins::PluginsManager, runtime::Runtime, runtime::RuntimeBuilder},
    session::ZenohId,
};

// ============================================================================
// sandbox module - Data structures for Rust <-> TypeScript exchange
// ============================================================================

pub mod sandbox {
    use super::*;

    /// Configuration for creating a Zenoh instance.
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
// State management for Zenoh instances
// ============================================================================

/// Holds all active Zenoh runtime instances
pub struct ZenohInstances {
    instances: RwLock<HashMap<ZenohId, (sandbox::ZenohConfig, Runtime)>>,
}

impl ZenohInstances {
    pub fn new() -> Self {
        Self {
            instances: RwLock::new(HashMap::new()),
        }
    }
}

impl Default for ZenohInstances {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Tauri commands
// ============================================================================

/// Create a new Zenoh instance with the given configuration.
/// Returns the ZenohId as a string on success.
#[tauri::command]
async fn zenoh_instance_invoke(
    config: sandbox::ZenohConfig,
    state: State<'_, ZenohInstances>,
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

    // Start the runtime
    runtime
        .start()
        .await
        .map_err(|e| format!("Failed to start Zenoh runtime: {e}"))?;

    // Get the ZenohId
    let zid = runtime.zid();

    // Store the runtime in state
    {
        let mut instances = state.instances.write().await;
        instances.insert(zid, (config, runtime));
    }

    Ok(zid.to_string())
}

/// Dismiss (close) a Zenoh instance by its ZenohId string.
#[tauri::command]
async fn zenoh_instance_dismiss(
    zid: String,
    state: State<'_, ZenohInstances>,
) -> Result<(), String> {
    // Parse the ZenohId from string
    let zenoh_id = ZenohId::from_str(&zid)
        .map_err(|e| format!("Invalid ZenohId '{}': {}", zid, e))?;

    // Remove from state and get the runtime
    let runtime = {
        let mut instances = state.instances.write().await;
        instances
            .remove(&zenoh_id)
            .map(|(_, runtime)| runtime)
            .ok_or_else(|| format!("Zenoh instance '{}' not found", zid))?
    };

    // Close the runtime explicitly
    runtime
        .close()
        .await
        .map_err(|e| format!("Failed to close Zenoh runtime: {e}"))?;

    Ok(())
}

/// List all active Zenoh instance ZenohIds as strings.
#[tauri::command]
async fn zenoh_instance_list(state: State<'_, ZenohInstances>) -> Result<Vec<String>, String> {
    let instances = state.instances.read().await;
    let zids: Vec<String> = instances.keys().map(|zid| zid.to_string()).collect();
    Ok(zids)
}

/// Get the configuration of a Zenoh instance by its ZenohId string.
#[tauri::command]
async fn zenoh_instance_config(
    zid: String,
    state: State<'_, ZenohInstances>,
) -> Result<sandbox::ZenohConfig, String> {
    // Parse the ZenohId from string
    let zenoh_id = ZenohId::from_str(&zid)
        .map_err(|e| format!("Invalid ZenohId '{}': {}", zid, e))?;

    // Get the config from state
    let instances = state.instances.read().await;
    let (config, _) = instances
        .get(&zenoh_id)
        .ok_or_else(|| format!("Zenoh instance '{}' not found", zid))?;

    Ok(config.clone())
}

// ============================================================================
// Tauri application entry point
// ============================================================================

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(ZenohInstances::new())
        .invoke_handler(tauri::generate_handler![
            zenoh_instance_invoke,
            zenoh_instance_dismiss,
            zenoh_instance_list,
            zenoh_instance_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
