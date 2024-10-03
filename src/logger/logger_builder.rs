use super::{Logger, LoggerOptions};
use logform::Format;
use std::collections::HashMap;
use winston_transport::Transport;

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
        self.options = self.options.level(level);
        self
    }

    pub fn format(mut self, format: Format) -> Self {
        self.options = self.options.format(format);
        self
    }

    pub fn add_transport<T: Transport + Send + Sync + 'static>(mut self, transport: T) -> Self {
        self.options = self.options.add_transport(transport);
        self
    }

    pub fn levels(mut self, levels: HashMap<String, u8>) -> Self {
        self.options = self.options.levels(levels);
        self
    }

    pub fn build(self) -> Logger {
        Logger::new(Some(self.options))
    }
}
