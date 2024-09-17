use super::{logger_levels::LoggerLevels, transports::transport::Transport};
use logform::{json, Format};
use std::{collections::HashMap, fmt};

pub struct DebugFormat(pub Format);

impl fmt::Debug for DebugFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Format")
    }
}

#[derive(Debug)]
pub struct LoggerOptions {
    pub levels: Option<LoggerLevels>,
    pub format: Option<DebugFormat>,
    pub level: Option<String>,
    pub transports: Vec<Transport>,
}

impl LoggerOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn level(mut self, level: &str) -> Self {
        self.level = Some(level.to_string());
        self
    }

    pub fn format(mut self, format: Format) -> Self {
        self.format = Some(DebugFormat(format));
        self
    }

    pub fn transports(mut self, transports: Vec<Transport>) -> Self {
        self.transports = transports;
        self
    }

    pub fn levels(mut self, levels: HashMap<String, u8>) -> Self {
        self.levels = Some(LoggerLevels::new(levels));
        self
    }

    pub fn get_format(&self) -> Option<&Format> {
        self.format.as_ref().map(|df| &df.0)
    }
}

impl Default for LoggerOptions {
    fn default() -> Self {
        LoggerOptions {
            levels: Some(LoggerLevels::default()),
            level: Some("info".to_string()),
            transports: Vec::new(),
            format: Some(DebugFormat(json())),
        }
    }
}
