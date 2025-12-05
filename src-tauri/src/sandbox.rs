use serde::{Deserialize, Serialize};

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
