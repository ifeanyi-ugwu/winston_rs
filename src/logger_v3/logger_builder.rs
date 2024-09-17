use super::{logger_options::DebugFormat, transports::transport::Transport, Logger, LoggerOptions};
use logform::Format;

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
        self.options.format = Some(DebugFormat(format));
        self
    }

    pub fn transports(mut self, transports: Vec<Transport>) -> Self {
        self.options.transports = transports;
        self
    }

    pub fn build(self) -> Logger {
        Logger::new(Some(self.options))
    }
}
