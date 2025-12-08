use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Configuration for creating a Zenoh runtime.
/// This stores the full JSON representation of zenoh::Config.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZenohConfig {
    /// Full JSON representation of zenoh::Config
    #[serde(flatten)]
    pub config_json: JsonValue,
}

impl ZenohConfig {
    /// Create a new ZenohConfig from JSON, verifying it's valid for zenoh::Config
    pub fn from_json(json: JsonValue) -> Result<Self, String> {
        // Try to deserialize into zenoh::Config to verify it's valid
        let _config: zenoh::config::Config = serde_json::from_value(json.clone())
            .map_err(|e| format!("Invalid zenoh config JSON: {}", e))?;

        Ok(Self {
            config_json: json,
        })
    }

    /// Create a default config with specified mode
    pub fn create_default(mode: &str, websocket_port: &str) -> Result<Self, String> {
        let mut config = zenoh::config::Config::default();

        // Parse and set mode
        let what_am_i = match mode {
            "peer" => zenoh::config::WhatAmI::Peer,
            "router" => zenoh::config::WhatAmI::Router,
            "client" => zenoh::config::WhatAmI::Client,
            _ => return Err(format!("Invalid mode: {}. Must be 'peer', 'router', or 'client'", mode)),
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

        // Set websocket port
        config
            .insert_json5(
                "plugins/remote_api/websocket_port",
                &format!(r#""{}""#, websocket_port),
            )
            .map_err(|e| format!("Failed to set websocket_port: {e}"))?;

        // Convert to JSON
        let config_json = serde_json::to_value(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;

        Ok(Self { config_json })
    }

    /// Convert sandbox::ZenohConfig to zenoh::config::Config
    pub fn into_zenoh_config(self) -> Result<zenoh::config::Config, String> {
        serde_json::from_value(self.config_json)
            .map_err(|e| format!("Failed to deserialize zenoh config: {}", e))
    }

    /// Get the mode from the config JSON
    pub fn get_mode(&self) -> Option<String> {
        self.config_json
            .get("mode")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Get the websocket port from the config JSON
    pub fn get_websocket_port(&self) -> Option<String> {
        self.config_json
            .get("plugins")
            .and_then(|p| p.get("remote_api"))
            .and_then(|ra| ra.get("websocket_port"))
            .and_then(|wp| wp.as_str())
            .map(|s| s.to_string())
    }
}
