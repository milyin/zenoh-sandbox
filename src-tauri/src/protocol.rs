use serde::{Deserialize, Serialize};
use zenoh::config::Config;

use crate::logs::LogEntry;

// ============================================================================
// Messages between main process and runtime process
// ============================================================================

/// Messages sent from main process to runtime process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MainToRuntime {
    /// Start the runtime with the given zenoh::Config
    Start(Config),
    /// Stop the runtime gracefully
    Stop,
    /// Request the current Zenoh configuration
    GetConfig,
}

/// Messages sent from runtime process to main process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeToMain {
    /// Runtime started successfully with this ZenohId
    Started(String),
    /// Runtime failed to start
    StartError(String),
    /// A log entry from the runtime
    Log(LogEntry),
    /// Runtime stopped
    Stopped,
    /// Response with the current Zenoh configuration
    Config(Config),
}
