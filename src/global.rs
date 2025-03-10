use std::{collections::HashMap, sync::Arc};

use crate::{logger_options::DebugTransport, Logger, LoggerOptions};
use lazy_static::lazy_static;
use logform::LogInfo;
use parking_lot::RwLock;
use winston_transport::Transport;

lazy_static! {
    static ref DEFAULT_LOGGER: RwLock<Logger> = RwLock::new(Logger::new(None));
}

pub fn log(entry: LogInfo) {
    DEFAULT_LOGGER.read().log(entry);
}

pub fn configure(options: Option<LoggerOptions>) {
    DEFAULT_LOGGER.read().configure(options);
}

pub fn close() {
    DEFAULT_LOGGER.write().close();
}

pub fn flush() -> Result<(), String> {
    DEFAULT_LOGGER.read().flush()?;
    Ok(())
}

pub struct GlobalLoggerRebuilder {
    options: LoggerOptions,
}

impl GlobalLoggerRebuilder {
    pub fn new() -> Self {
        let current_options = DEFAULT_LOGGER.read().shared_state.read().options.clone();
        GlobalLoggerRebuilder {
            options: current_options,
        }
    }

    pub fn level<T: Into<String>>(mut self, level: T) -> Self {
        self.options.level = Some(level.into());
        self
    }

    pub fn format(mut self, format: logform::Format) -> Self {
        self.options.format = Some(format);
        self
    }

    pub fn add_transport<T: Transport + 'static>(mut self, transport: T) -> Self {
        let transport = Arc::new(transport);
        self.options
            .transports
            .get_or_insert_with(Vec::new)
            .push(DebugTransport(transport));
        self
    }

    pub fn transports(mut self, transports: Vec<Arc<dyn Transport>>) -> Self {
        self.options.transports = Some(transports.into_iter().map(DebugTransport).collect());
        self
    }

    pub fn levels(mut self, levels: HashMap<String, u8>) -> Self {
        self.options.levels = Some(crate::logger_levels::LoggerLevels::new(levels));
        self
    }

    pub fn rebuild(self) {
        configure(Some(self.options));
    }
}

pub fn rebuilder() -> GlobalLoggerRebuilder {
    GlobalLoggerRebuilder::new()
}
