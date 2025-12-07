use serde::{Deserialize, Serialize};

/// Configuration for creating a Zenoh runtime.
/// This is the data structure exchanged between Rust and TypeScript.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZenohConfig {
    /// Zenoh mode: "peer", "router", or "client"
    pub mode: String,
    /// WebSocket port for the Remote API
    pub websocket_port: String,
}

impl ZenohConfig {
    /// Convert sandbox::ZenohConfig to zenoh::config::Config
    pub fn into_zenoh_config(self) -> Result<zenoh::config::Config, String> {
        let mut config = zenoh::config::Config::default();

        // Parse and set mode
        let what_am_i = match self.mode.as_str() {
            "peer" => zenoh::config::WhatAmI::Peer,
            "router" => zenoh::config::WhatAmI::Router,
            "client" => zenoh::config::WhatAmI::Client,
            _ => return Err(format!("Invalid mode: {}. Must be 'peer', 'router', or 'client'", self.mode)),
        };

        config
            .set_mode(Some(what_am_i))
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

        // Set websocket port from the provided config
        config
            .insert_json5(
                "plugins/remote_api/websocket_port",
                &format!(r#""{}""#, self.websocket_port),
            )
            .map_err(|e| format!("Failed to set websocket_port: {e}"))?;

        Ok(config)
    }
}
