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
/// This represents the subset of configuration that can be modified through the UI.
#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
#[ts(export, export_to = "../../src/types/generated/")]
pub struct ZenohConfigEdit {
    /// Zenoh mode: "peer", "router", or "client"
    pub mode: ZenohMode,
}

impl ZenohConfigEdit {
    /// Apply the editable fields to an existing zenoh::Config
    pub fn apply_to_config(
        &self,
        config: &mut zenoh::config::Config,
    ) -> Result<(), String> {
        let what_am_i: zenoh::config::WhatAmI = self.mode.clone().into();
        config.set_mode(Some(what_am_i))
            .map_err(|e| format!("Failed to set mode: {:?}", e))?;
        Ok(())
    }
}

impl From<&ZenohConfigEdit> for zenoh::config::Config {
    fn from(edit: &ZenohConfigEdit) -> Self {
        let mut config = zenoh::config::Config::default();
        edit.apply_to_config(&mut config).ok();
        config
    }
}

impl From<&zenoh::config::Config> for ZenohConfigEdit {
    fn from(config: &zenoh::config::Config) -> Self {
        let what_am_i = config.mode().unwrap_or(zenoh::config::WhatAmI::default());
        let mode: ZenohMode = what_am_i.into();
        ZenohConfigEdit { mode }
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

    /// Create a config from default with editable fields applied and auto-assigned port
    pub fn create_from_edit(edit: &ZenohConfigEdit, websocket_port: u16) -> Result<Self, String> {
        let mut config: zenoh::config::Config = edit.into();

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
