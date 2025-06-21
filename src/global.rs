use crate::{Logger, LoggerOptions};
use lazy_static::lazy_static;
use logform::LogInfo;
use std::sync::Arc;
use winston_transport::Transport;

lazy_static! {
    static ref GLOBAL_LOGGER: Logger = Logger::new(None);
}

pub fn log(entry: LogInfo) {
    GLOBAL_LOGGER.log(entry);
}

pub fn configure(options: Option<LoggerOptions>) {
    GLOBAL_LOGGER.configure(options);
}

pub fn close() {
    GLOBAL_LOGGER.close();
}

pub fn flush() -> Result<(), String> {
    GLOBAL_LOGGER.flush()
}

pub fn query(options: &winston_transport::LogQuery) -> Result<Vec<LogInfo>, String> {
    GLOBAL_LOGGER.query(options)
}

pub fn add_transport(transport: Arc<dyn Transport>) -> bool {
    GLOBAL_LOGGER.add_transport(transport)
}

pub fn remove_transport(transport: Arc<dyn Transport>) -> bool {
    GLOBAL_LOGGER.remove_transport(transport)
}
