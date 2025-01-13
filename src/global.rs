use crate::{Logger, LoggerOptions};
use lazy_static::lazy_static;
use logform::LogInfo;
use parking_lot::RwLock;

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
