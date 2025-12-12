use serde::{Deserialize, Serialize};
use tracing::Level;
use ts_rs::TS;

/// Zenoh mode enum for TypeScript
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS, Default, Eq, PartialEq, Hash)]
#[ts(export, export_to = "../../src/types/generated/")]
#[ts(repr(enum))]
#[repr(u8)]
pub enum LogEntryLevel {
    TRACE = 0,
    DEBUG = 1,
    #[default]
    INFO = 2,
    WARN = 3,
    ERROR = 4,
}

impl From<LogEntryLevel> for u8 {
    fn from(level: LogEntryLevel) -> Self {
        level as u8
    }
}

impl TryFrom<u8> for LogEntryLevel {
    type Error = String;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(LogEntryLevel::TRACE),
            1 => Ok(LogEntryLevel::DEBUG),
            2 => Ok(LogEntryLevel::INFO),
            3 => Ok(LogEntryLevel::WARN),
            4 => Ok(LogEntryLevel::ERROR),
            _ => Err(format!("Invalid LogEntryLevel: {}", value)),
        }
    }
}

impl From<Level> for LogEntryLevel {
    fn from(level: Level) -> Self {
        (&level).into()
    }
}

impl From<&Level> for LogEntryLevel {
    fn from(level: &Level) -> Self {
        match *level {
            Level::TRACE => LogEntryLevel::TRACE,
            Level::DEBUG => LogEntryLevel::DEBUG,
            Level::INFO => LogEntryLevel::INFO,
            Level::WARN => LogEntryLevel::WARN,
            Level::ERROR => LogEntryLevel::ERROR,
        }
    }
}

impl From<&LogEntryLevel> for Level {
    fn from(level: &LogEntryLevel) -> Self {
        match level {
            LogEntryLevel::TRACE => Level::TRACE,
            LogEntryLevel::DEBUG => Level::DEBUG,
            LogEntryLevel::INFO => Level::INFO,
            LogEntryLevel::WARN => Level::WARN,
            LogEntryLevel::ERROR => Level::ERROR,
        }
    }
}

impl From<&str> for LogEntryLevel {
    fn from(value: &str) -> Self {
        value.parse().unwrap_or(Level::INFO).into()
    }
}

impl PartialOrd for LogEntryLevel {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LogEntryLevel {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let this: Level = self.into();
        let other: Level = other.into();
        this.cmp(&other)
    }
}