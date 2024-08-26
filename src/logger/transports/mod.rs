pub mod console;
pub mod file;

pub use console::ConsoleTransport as Console;
pub use file::FileTransport as File;
use logform::Format;

use std::any::Any;

pub trait Transport: Any + Send + Sync {
    fn log(&self, message: &str, level: &str);
    fn get_level(&self) -> Option<&String>;
    fn get_format(&self) -> Option<&Format>;
}

pub struct TransportStreamOptions {
    pub level: Option<String>,
    pub format: Option<Format>,
}
