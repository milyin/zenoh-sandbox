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

impl From<zenoh::config::WhatAmI> for ZenohMode {
    fn from(what_am_i: zenoh::config::WhatAmI) -> Self {
        match what_am_i {
            zenoh::config::WhatAmI::Peer => ZenohMode::Peer,
            zenoh::config::WhatAmI::Router => ZenohMode::Router,
            zenoh::config::WhatAmI::Client => ZenohMode::Client,
        }
    }
}

impl From<ZenohMode> for zenoh::config::WhatAmI {
    fn from(mode: ZenohMode) -> Self {
        match mode {
            ZenohMode::Peer => zenoh::config::WhatAmI::Peer,
            ZenohMode::Router => zenoh::config::WhatAmI::Router,
            ZenohMode::Client => zenoh::config::WhatAmI::Client,
        }
    }
}

impl Default for ZenohMode {
    fn default() -> Self {
        zenoh::config::WhatAmI::default().into()
    }
}

/// Editable fields for Zenoh configuration.
/// This represents the JSON5 string representation of the user-edited config.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ZenohConfigEdit {
    /// JSON5 string representation of the editable config
    pub content: String,
}

impl ZenohConfigEdit {
    /// Validate and parse the JSON content into a zenoh::Config
    pub fn to_config(&self) -> Result<zenoh::config::Config, String> {
        // Parse JSON string directly into Config using serde
        // If content is empty or "{}", this will create a default config
        if self.content.trim().is_empty() || self.content.trim() == "{}" {
            return Ok(zenoh::config::Config::default());
        }

        serde_json::from_str(&self.content)
            .map_err(|e| format!("Invalid JSON config: {}", e))
    }

    /// Create from a zenoh::Config by serializing to JSON
    pub fn from_config(config: &zenoh::config::Config) -> Result<Self, String> {
        let json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        Ok(Self { content: json })
    }
}

impl Default for ZenohConfigEdit {
    fn default() -> Self {
        Self {
            content: "{}".to_string(),
        }
    }
}

/// Validated Zenoh configuration JSON.
/// This is a newtype wrapper that guarantees the JSON is valid for zenoh::Config.
/// It can ONLY be created through validation.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/",
    type = "Record<string, any>"
)]
pub struct ZenohConfigJson {
    /// Validated JSON representation of zenoh::Config
    #[serde(flatten)]
    config_json: JsonValue,
}

impl TryFrom<ZenohConfigJson> for zenoh::config::Config {
    type Error = String;

    fn try_from(value: ZenohConfigJson) -> Result<Self, Self::Error> {
        serde_json::from_value(value.config_json)
            .map_err(|e| format!("Failed to deserialize zenoh config: {}", e))
    }
}

impl ZenohConfigJson {
    /// Create from JSON with validation
    pub fn from_json(json: JsonValue) -> Result<Self, String> {
        // Verify it's valid for zenoh::Config
        let _config: zenoh::config::Config = serde_json::from_value(json.clone())
            .map_err(|e| format!("Invalid zenoh config JSON: {}", e))?;

        Ok(Self { config_json: json })
    }

    /// Get a reference to the underlying JSON
    pub fn as_json(&self) -> &JsonValue {
        &self.config_json
    }

    /// Get the websocket port from the config JSON
    pub fn get_websocket_port(&self) -> Option<u16> {
        self.config_json
            .get("plugins")
            .and_then(|p| p.get("remote_api"))
            .and_then(|ra| ra.get("websocket_port"))
            .and_then(|wp| wp.as_u64())
            .map(|port| port as u16)
    }
}
