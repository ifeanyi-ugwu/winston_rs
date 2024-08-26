use super::{transports::Transport, Logger, LoggerOptions};
use logform::Format;
use std::{collections::HashMap, sync::Arc};

pub struct LoggerBuilder {
    levels: Option<HashMap<String, u8>>,
    format: Option<Format>,
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

    pub fn add_level<T: Into<String>>(mut self, level: T, value: u8) -> Self {
        self.levels
            .get_or_insert_with(HashMap::new)
            .insert(level.into(), value);
        self
    }

    pub fn format(mut self, format: Format) -> Self {
        self.format = Some(format);
        self
    }

    pub fn level<T: Into<String>>(mut self, level: T) -> Self {
        self.level = Some(level.into());
        self
    }

    pub fn add_transport<T: Transport + Send + Sync + 'static>(mut self, transport: T) -> Self {
        self.transports
            .get_or_insert_with(Vec::new)
            .push(Arc::new(transport));
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
