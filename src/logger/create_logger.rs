use super::{Logger, LoggerOptions};

pub fn create_logger(options: Option<LoggerOptions>) -> Logger {
    Logger::new(options)
}
