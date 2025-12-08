use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use ts_rs::TS;

/// Zenoh mode enum for TypeScript
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
#[serde(rename_all = "lowercase")]
pub enum ZenohMode {
    Peer,
    Router,
    Client,
}

impl ZenohMode {
    pub fn as_str(&self) -> &str {
        match self {
            ZenohMode::Peer => "peer",
            ZenohMode::Router => "router",
            ZenohMode::Client => "client",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "peer" => Ok(ZenohMode::Peer),
            "router" => Ok(ZenohMode::Router),
            "client" => Ok(ZenohMode::Client),
            _ => Err(format!("Invalid mode: {}. Must be 'peer', 'router', or 'client'", s)),
        }
    }
}

/// Editable fields for Zenoh configuration.
/// This represents the subset of configuration that can be modified through the UI.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ZenohConfigEdit {
    /// Zenoh mode: "peer", "router", or "client"
    pub mode: ZenohMode,
}

impl ZenohConfigEdit {
    /// Create a new ZenohConfigEdit with validation
    pub fn new(mode: &str) -> Result<Self, String> {
        Ok(Self {
            mode: ZenohMode::from_str(mode)?,
        })
    }

    /// Extract editable fields from a zenoh::Config
    pub fn from_config(config: &zenoh::config::Config) -> Self {
        let mode = match config.mode() {
            Some(zenoh::config::WhatAmI::Peer) => ZenohMode::Peer,
            Some(zenoh::config::WhatAmI::Router) => ZenohMode::Router,
            Some(zenoh::config::WhatAmI::Client) => ZenohMode::Client,
            None => ZenohMode::Peer, // Default fallback
        };

        Self { mode }
    }

    /// Convert mode to zenoh::WhatAmI enum
    pub fn to_what_am_i(&self) -> zenoh::config::WhatAmI {
        match self.mode {
            ZenohMode::Peer => zenoh::config::WhatAmI::Peer,
            ZenohMode::Router => zenoh::config::WhatAmI::Router,
            ZenohMode::Client => zenoh::config::WhatAmI::Client,
        }
    }
}

/// Validated Zenoh configuration JSON.
/// This is a newtype wrapper that guarantees the JSON is valid for zenoh::Config.
/// It can ONLY be created through validation.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/", type = "Record<string, any>")]
pub struct ZenohConfigJson {
    /// Validated JSON representation of zenoh::Config
    #[serde(flatten)]
    config_json: JsonValue,
}

impl ZenohConfigJson {
    /// Create from JSON with validation
    pub fn from_json(json: JsonValue) -> Result<Self, String> {
        // Verify it's valid for zenoh::Config
        let _config: zenoh::config::Config = serde_json::from_value(json.clone())
            .map_err(|e| format!("Invalid zenoh config JSON: {}", e))?;

        Ok(Self { config_json: json })
    }

    /// Create a default config with specified mode and port
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

    /// Convert to zenoh::config::Config
    pub fn into_zenoh_config(self) -> Result<zenoh::config::Config, String> {
        serde_json::from_value(self.config_json)
            .map_err(|e| format!("Failed to deserialize zenoh config: {}", e))
    }

    /// Get a reference to the underlying JSON
    pub fn as_json(&self) -> &JsonValue {
        &self.config_json
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

