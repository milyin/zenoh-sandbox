use std::sync::Arc;

use chrono::{DateTime, Utc};
use parking_lot::RwLock as ParkingLotRwLock;
use serde::{Deserialize, Serialize};
use tracing_subscriber::Layer;

// ============================================================================
// Constants
// ============================================================================

/// Number of log entries per page
pub const LOG_PAGE_SIZE: usize = 100;

/// Maximum number of log entries to keep
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
    pub level: String,
    /// The target module/component that produced the log
    pub target: String,
    /// The log message
    pub message: String,
}

// ============================================================================
// Log Storage
// ============================================================================

/// Stores logs from all runtimes in a circular buffer
pub struct LogStorage {
    /// Maximum number of log entries to keep
    max_entries: usize,
    /// Log entries (most recent first)
    logs: Arc<ParkingLotRwLock<Vec<LogEntry>>>,
}

impl LogStorage {
    pub fn new(max_entries: usize) -> Self {
        Self {
            max_entries,
            logs: Arc::new(ParkingLotRwLock::new(Vec::new())),
        }
    }

    /// Add a log entry
    pub fn add_log(&self, entry: LogEntry) {
        let mut logs = self.logs.write();

        // Insert at the beginning (most recent first)
        logs.insert(0, entry);

        // Keep only max_entries
        if logs.len() > self.max_entries {
            logs.truncate(self.max_entries);
        }
    }

    /// Get a page of logs
    /// Page 0 returns the most recent logs
    pub fn get_page(&self, page: usize) -> Vec<LogEntry> {
        let logs = self.logs.read();
        let start = page * LOG_PAGE_SIZE;
        let end = ((page + 1) * LOG_PAGE_SIZE).min(logs.len());

        if start >= logs.len() {
            return Vec::new();
        }

        logs[start..end].to_vec()
    }

    /// Get a reference to the internal logs for the custom layer
    pub fn logs_ref(&self) -> Arc<ParkingLotRwLock<Vec<LogEntry>>> {
        self.logs.clone()
    }
}

impl Default for LogStorage {
    fn default() -> Self {
        Self::new(MAX_LOG_ENTRIES)
    }
}

// ============================================================================
// Log Capture Layer
// ============================================================================

/// Custom tracing layer that captures logs into LogStorage
pub struct LogCaptureLayer {
    logs: Arc<ParkingLotRwLock<Vec<LogEntry>>>,
    max_entries: usize,
}

impl LogCaptureLayer {
    pub fn new(logs: Arc<ParkingLotRwLock<Vec<LogEntry>>>, max_entries: usize) -> Self {
        Self { logs, max_entries }
    }
}

impl<S> Layer<S> for LogCaptureLayer
where
    S: tracing::Subscriber,
{
    fn on_event(
        &self,
        event: &tracing::Event<'_>,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        // Extract log level
        let level = event.metadata().level().to_string();

        // Extract target
        let target = event.metadata().target().to_string();

        // Extract message using a visitor
        let mut message = String::new();
        let mut visitor = MessageVisitor(&mut message);
        event.record(&mut visitor);

        // Create log entry
        let entry = LogEntry {
            timestamp: Utc::now(),
            level,
            target,
            message,
        };

        // Store in logs
        let mut logs = self.logs.write();
        logs.insert(0, entry);

        // Keep only max_entries
        if logs.len() > self.max_entries {
            logs.truncate(self.max_entries);
        }
    }
}

/// Visitor to extract message from tracing event
struct MessageVisitor<'a>(&'a mut String);

impl<'a> tracing::field::Visit for MessageVisitor<'a> {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            *self.0 = format!("{:?}", value);
            // Remove surrounding quotes if present
            if self.0.starts_with('"') && self.0.ends_with('"') {
                *self.0 = self.0[1..self.0.len() - 1].to_string();
            }
        }
    }
}
