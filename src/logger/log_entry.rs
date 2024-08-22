use serde::Serialize;
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
pub struct LogEntry {
    pub level: String,
    pub message: String,
    options: HashMap<String, Value>,
}

impl LogEntry {
    pub fn builder(level: &str, message: &str) -> LogEntryBuilder {
        LogEntryBuilder::new(level, message)
    }
}

pub struct LogEntryBuilder {
    level: String,
    message: String,
    options: HashMap<String, Value>,
}

impl LogEntryBuilder {
    fn new(level: &str, message: &str) -> Self {
        LogEntryBuilder {
            level: level.to_string(),
            message: message.to_string(),
            options: HashMap::new(),
        }
    }

    pub fn option(mut self, key: &str, value: Value) -> Self {
        self.options.insert(key.to_string(), value);
        self
    }

    pub fn build(self) -> LogEntry {
        LogEntry {
            level: self.level,
            message: self.message,
            options: self.options,
        }
    }
}
