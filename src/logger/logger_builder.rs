use super::{
    logger_options::{DebugFormat, DebugTransport},
    Logger, LoggerOptions,
};
use logform::Format;
use std::sync::Arc;
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

    /// Sets the logging level for the logger.
    pub fn level<T: Into<String>>(mut self, level: T) -> Self {
        self.options.level = Some(level.into());
        self
    }

    /// Sets the log format for the logger.
    pub fn format(mut self, format: Format) -> Self {
        self.options.format = Some(DebugFormat(format));
        self
    }

    /*pub fn add_transport<T: Transport + Send + Sync + 'static>(mut self, transport: T) -> Self {
        self.options
            .transports
            .get_or_insert_with(Vec::new)
            .push(Arc::new(transport));
        self
    }*/

    /// Adds a single transport to the existing list of transports.
    pub fn add_transport<T: Transport + Send + Sync + 'static>(mut self, transport: T) -> Self {
        if self.options.transports.is_none() {
            self.options.transports = Some(Vec::new());
        }
        self.options
            .transports
            .as_mut()
            .unwrap()
            .push(DebugTransport(Arc::new(transport)));
        self
    }

    pub fn build(self) -> Logger {
        Logger::new(Some(self.options))
    }
}
