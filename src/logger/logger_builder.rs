use std::{collections::HashMap, sync::Arc};

use super::{transports::Transport, Logger, LoggerOptions};

pub struct LoggerBuilder {
    levels: Option<HashMap<String, u8>>,
    format: Option<String>,
    level: Option<String>,
    transports: Option<Vec<Arc<dyn Transport + Send + Sync>>>,
}

impl LoggerBuilder {
    pub fn new() -> Self {
        LoggerBuilder {
            levels: None,
            format: None,
            level: None,
            transports: None,
        }
    }

    pub fn levels(mut self, levels: HashMap<String, u8>) -> Self {
        self.levels = Some(levels);
        self
    }

    pub fn format(mut self, format: String) -> Self {
        self.format = Some(format);
        self
    }

    pub fn level(mut self, level: String) -> Self {
        self.level = Some(level);
        self
    }

    pub fn transports(mut self, transports: Vec<Arc<dyn Transport + Send + Sync>>) -> Self {
        self.transports = Some(transports);
        self
    }

    pub fn build(self) -> Logger {
        let options = LoggerOptions {
            levels: self.levels.or_else(|| LoggerOptions::default().levels),
            format: self.format.or_else(|| LoggerOptions::default().format),
            level: self.level.or_else(|| LoggerOptions::default().level),
            transports: self
                .transports
                .or_else(|| LoggerOptions::default().transports),
        };

        Logger::new(Some(options))
    }
}
