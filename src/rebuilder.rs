use crate::{
    logger_options::{DebugTransport, LoggerOptions},
    Logger,
};
use logform::Format;
use std::{collections::HashMap, sync::Arc};
use winston_transport::Transport;

pub struct LoggerRebuilder<'a> {
    logger: &'a Logger,
    new_options: LoggerOptions,
}

impl<'a> LoggerRebuilder<'a> {
    pub fn new(logger: &'a Logger) -> Self {
        LoggerRebuilder {
            logger,
            new_options: logger.shared_state.read().options.clone(),
        }
    }

    pub fn level<T: Into<String>>(mut self, level: T) -> Self {
        self.new_options.level = Some(level.into());
        self
    }

    pub fn format(mut self, format: Format) -> Self {
        self.new_options.format = Some(format);
        self
    }

    pub fn add_transport<T: Transport + 'static>(mut self, transport: T) -> Self {
        let transport = Arc::new(transport);
        self.new_options
            .transports
            .get_or_insert_with(Vec::new)
            .push(DebugTransport(transport));
        self
    }

    pub fn transports(mut self, transports: Vec<Arc<dyn Transport>>) -> Self {
        self.new_options.transports = Some(transports.into_iter().map(DebugTransport).collect());
        self
    }

    pub fn levels(mut self, levels: HashMap<String, u8>) -> Self {
        self.new_options.levels = Some(crate::logger_levels::LoggerLevels::new(levels));
        self
    }

    pub fn rebuild(self) {
        self.logger.configure(Some(self.new_options));
    }
}
