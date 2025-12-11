use std::{collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use parking_lot::RwLock as ParkingLotRwLock;
use serde::{Deserialize, Serialize};
use zenoh::session::ZenohId;

use crate::ts::log::LogEntryLevel;

// ============================================================================
// Constants
// ============================================================================

/// Number of log entries per page
pub const LOG_PAGE_SIZE: usize = 100;

/// Maximum number of log entries to keep per runtime
const MAX_LOG_ENTRIES: usize = 10_000;

// ============================================================================
// Log Entry Structure
// ============================================================================

/// A single log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    /// Timestamp of the log entry
    pub timestamp: DateTime<Utc>,
    /// Log level (e.g., "INFO", "DEBUG", "ERROR")
    pub level: LogEntryLevel,
    /// The target module/component that produced the log
    pub target: String,
    /// The log message
    pub message: String,
}

// ============================================================================
// Log Storage
// ============================================================================

/// Stores logs from all runtimes, separated by ZenohId
#[derive(Clone)]
pub struct LogStorage {
    /// Maximum number of log entries to keep per runtime
    max_entries: usize,
    /// Map of ZenohId to log entries (most recent first)
    logs: Arc<ParkingLotRwLock<HashMap<ZenohId, Vec<LogEntry>>>>,
}

impl LogStorage {
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            logs: Arc::new(ParkingLotRwLock::new(HashMap::new())),
        }
    }

    /// Add a log entry for a specific runtime
    pub fn add_log(&self, zid: ZenohId, entry: LogEntry) {
        let mut logs = self.logs.write();
        let runtime_logs = logs.entry(zid).or_default();

        // Insert at the beginning (most recent first)
        runtime_logs.insert(0, entry);

        // Keep only max_entries
        if runtime_logs.len() > self.max_entries {
            runtime_logs.truncate(self.max_entries);
        }
    }

    /// Get a page of logs for a specific runtime
    /// Page 0 returns the most recent logs
    pub fn get_page(&self, zid: &ZenohId, level: Option<LogEntryLevel>, page: usize) -> Vec<LogEntry> {
        let logs = self.logs.read();
        if let Some(runtime_logs) = logs.get(zid) {
            let filtered_logs: Vec<LogEntry> = runtime_logs
                .iter().filter(|&entry| {
                    if let Some(ref lvl) = level {
                        &entry.level >= lvl
                    } else {
                        true
                    }
                }).cloned()
                .collect();

            let start = page * LOG_PAGE_SIZE;
            let end = ((page + 1) * LOG_PAGE_SIZE).min(filtered_logs.len());

            if start >= filtered_logs.len() {
                return Vec::new();
            }

            filtered_logs[start..end].to_vec()
        } else {
            Vec::new()
        }
    }

    /// Clear logs for a specific runtime
    pub fn clear_logs(&self, zid: &ZenohId) {
        let mut logs = self.logs.write();
        logs.remove(zid);
    }

    /// Get a reference to the internal logs for the custom layer
    pub fn logs_ref(&self) -> Arc<ParkingLotRwLock<HashMap<ZenohId, Vec<LogEntry>>>> {
        self.logs.clone()
    }
}

impl Default for LogStorage {
    fn default() -> Self {
        Self::new(MAX_LOG_ENTRIES)
    }
}

