use crate::{
    logger_options::{BackpressureStrategy, LoggerOptions},
    Logger,
};
use logform::Format;
use std::{collections::HashMap, sync::Arc};
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

    pub fn transports(mut self, transports: Vec<Arc<dyn Transport>>) -> Self {
        self.options = self.options.transports(transports);
        self
    }

    pub fn levels(mut self, levels: HashMap<String, u8>) -> Self {
        self.options = self.options.levels(levels);
        self
    }

    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.options.channel_capacity = Some(capacity);
        self
    }

    pub fn backpressure_strategy(mut self, strategy: BackpressureStrategy) -> Self {
        self.options.backpressure_strategy = Some(strategy);
        self
    }

    pub fn build(self) -> Logger {
        Logger::new(Some(self.options))
    }
}
