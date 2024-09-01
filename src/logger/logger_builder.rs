use super::{transports::Transport, Logger, LoggerOptions};
use logform::Format;
use std::sync::Arc;

pub struct LoggerBuilder {
    options: LoggerOptions,
}

impl LoggerBuilder {
    pub fn new() -> Self {
        LoggerBuilder {
            options: LoggerOptions::default(),
        }
    }

    pub fn level<T: Into<String>>(mut self, level: T) -> Self {
        self.options.level = Some(level.into());
        self
    }

    pub fn format(mut self, format: Format) -> Self {
        self.options.format = Some(format);
        self
    }

    pub fn add_transport<T: Transport + Send + Sync + 'static>(mut self, transport: T) -> Self {
        self.options
            .transports
            .get_or_insert_with(Vec::new)
            .push(Arc::new(transport));
        self
    }

    pub fn build(self) -> Logger {
        Logger::new(Some(self.options))
    }
}
