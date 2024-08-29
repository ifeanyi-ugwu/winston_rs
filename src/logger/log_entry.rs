use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    pub meta: HashMap<String, Value>,
}

impl LogEntry {
    pub fn builder(level: &str, message: &str) -> LogEntryBuilder {
        LogEntryBuilder::new(level, message)
    }

    pub fn flush() -> Self {
        LogEntry {
            level: "FLUSH".to_string(),
            message: String::new(),
            meta: HashMap::new(),
        }
    }

    pub fn is_flush(&self) -> bool {
        self.level == "FLUSH"
    }
}

pub struct LogEntryBuilder {
    level: String,
    message: String,
    meta: HashMap<String, Value>,
}

impl LogEntryBuilder {
    fn new(level: &str, message: &str) -> Self {
        LogEntryBuilder {
            level: level.to_string(),
            message: message.to_string(),
            meta: HashMap::new(),
        }
    }

    pub fn option(mut self, key: &str, value: Value) -> Self {
        self.meta.insert(key.to_string(), value);
        self
    }

    pub fn build(self) -> LogEntry {
        LogEntry {
            level: self.level,
            message: self.message,
            meta: self.meta,
        }
    }
}

pub fn convert_log_entry(entry: &LogEntry) -> logform::LogInfo {
    logform::LogInfo {
        level: entry.level.clone(),
        message: entry.message.clone(),
        meta: entry.meta.clone(),
    }
}
