use std::{cell::RefCell, collections::HashMap, sync::Arc};

use chrono::{DateTime, Utc};
use parking_lot::RwLock as ParkingLotRwLock;
use serde::{Deserialize, Serialize};
use tracing_subscriber::Layer;
use zenoh::session::ZenohId;

// ============================================================================
// Constants
// ============================================================================

/// Number of log entries per page
pub const LOG_PAGE_SIZE: usize = 100;

/// Maximum number of log entries to keep per runtime
const MAX_LOG_ENTRIES: usize = 10_000;

// ============================================================================
// Runtime Context (Thread-Local)
// ============================================================================

thread_local! {
    /// Thread-local storage for the current runtime context
    static RUNTIME_CONTEXT: RefCell<Option<ZenohId>> = const { RefCell::new(None) };
}

/// Set the current runtime context for this thread
pub fn set_runtime_context(zid: ZenohId) {
    RUNTIME_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = Some(zid);
    });
}

/// Clear the current runtime context for this thread
pub fn clear_runtime_context() {
    RUNTIME_CONTEXT.with(|ctx| {
        *ctx.borrow_mut() = None;
    });
}

/// Get the current runtime context for this thread
pub fn get_runtime_context() -> Option<ZenohId> {
    RUNTIME_CONTEXT.with(|ctx| *ctx.borrow())
}

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

/// Stores logs from all runtimes, separated by ZenohId
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
        let runtime_logs = logs.entry(zid).or_insert_with(Vec::new);

        // Insert at the beginning (most recent first)
        runtime_logs.insert(0, entry);

        // Keep only max_entries
        if runtime_logs.len() > self.max_entries {
            runtime_logs.truncate(self.max_entries);
        }
    }

    /// Get a page of logs for a specific runtime
    /// Page 0 returns the most recent logs
    pub fn get_page(&self, zid: &ZenohId, page: usize) -> Vec<LogEntry> {
        let logs = self.logs.read();
        if let Some(runtime_logs) = logs.get(zid) {
            let start = page * LOG_PAGE_SIZE;
            let end = ((page + 1) * LOG_PAGE_SIZE).min(runtime_logs.len());

            if start >= runtime_logs.len() {
                return Vec::new();
            }

            runtime_logs[start..end].to_vec()
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

// ============================================================================
// Log Capture Layer
// ============================================================================

/// Custom tracing layer that captures logs into LogStorage
pub struct LogCaptureLayer {
    logs: Arc<ParkingLotRwLock<HashMap<ZenohId, Vec<LogEntry>>>>,
    max_entries: usize,
}

impl LogCaptureLayer {
    pub fn new(
        logs: Arc<ParkingLotRwLock<HashMap<ZenohId, Vec<LogEntry>>>>,
        max_entries: usize,
    ) -> Self {
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
        // Get the current runtime context
        let zid = match get_runtime_context() {
            Some(zid) => zid,
            None => return, // No context, skip logging
        };

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

        // Store in logs for this runtime
        let mut logs = self.logs.write();
        let runtime_logs = logs.entry(zid).or_insert_with(Vec::new);
        runtime_logs.insert(0, entry);

        // Keep only max_entries
        if runtime_logs.len() > self.max_entries {
            runtime_logs.truncate(self.max_entries);
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
