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

/// Compute the difference between two JSON values.
/// Returns a JSON object containing only fields that differ from base.
/// Deleted fields are represented as null.
pub fn json_diff(base: &JsonValue, modified: &JsonValue) -> JsonValue {
    use serde_json::Map;

    match (base, modified) {
        // Both are objects - recursively compare fields
        (JsonValue::Object(base_obj), JsonValue::Object(modified_obj)) => {
            let mut diff = Map::new();

            // Check all fields in modified
            for (key, modified_value) in modified_obj {
                if let Some(base_value) = base_obj.get(key) {
                    // Field exists in both - compare values
                    if base_value != modified_value {
                        // Values differ - compute nested diff
                        let nested_diff = json_diff(base_value, modified_value);
                        // Only include if there's an actual difference
                        if !nested_diff.is_null()
                            && !(nested_diff.is_object()
                                && nested_diff.as_object().unwrap().is_empty())
                        {
                            diff.insert(key.clone(), nested_diff);
                        }
                    }
                } else {
                    // Field is new in modified - include it
                    diff.insert(key.clone(), modified_value.clone());
                }
            }

            // Check for deleted fields (in base but not in modified)
            for key in base_obj.keys() {
                if !modified_obj.contains_key(key) {
                    diff.insert(key.clone(), JsonValue::Null);
                }
            }

            JsonValue::Object(diff)
        }
        // Both are arrays - compare element by element
        (JsonValue::Array(base_arr), JsonValue::Array(modified_arr)) => {
            if base_arr == modified_arr {
                JsonValue::Null
            } else {
                modified.clone()
            }
        }
        // Different types or primitive values
        _ => {
            if base == modified {
                JsonValue::Null
            } else {
                modified.clone()
            }
        }
    }
}
