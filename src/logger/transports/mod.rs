pub mod console;
pub mod file;

use super::log_query::LogQuery;
use crate::LogEntry;
pub use console::ConsoleTransport as Console;
pub use file::FileTransport as File;
use logform::Format;
use std::any::Any;

pub trait Transport: Any + Send + Sync {
    fn log(&self, message: &str, level: &str);
    fn get_level(&self) -> Option<&String>;
    fn get_format(&self) -> Option<&Format>;
    fn as_any(&self) -> &dyn Any;
    fn as_queryable(&self) -> Option<&dyn Queryable> {
        None
    }
}

pub struct TransportStreamOptions {
    pub level: Option<String>,
    pub format: Option<Format>,
}

pub trait Queryable: Any + Send + Sync {
    fn query(&self, query: &LogQuery) -> Result<Vec<LogEntry>, String>;
}
