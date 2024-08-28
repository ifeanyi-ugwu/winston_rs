pub mod create_logger;
mod custom_levels;
mod default_levels;
pub mod log_entry;
mod logger_builder;
mod logger_options;
pub mod transports;

use custom_levels::CustomLevels;
use lazy_static::lazy_static;
use log_entry::{convert_log_entry, LogEntry};
use logform::{json, Format};
use logger_builder::LoggerBuilder;
pub use logger_options::LoggerOptions;
use std::sync::{Arc, Mutex};
use transports::Transport;

pub struct Logger {
    levels: CustomLevels,
    format: Format,
    level: String,
    transports: Vec<Arc<dyn Transport + Send + Sync>>,
}

lazy_static! {
    static ref DEFAULT_LOGGER: Mutex<Logger> = Mutex::new(Logger::new(None));
}

impl Logger {
    pub fn new(options: Option<LoggerOptions>) -> Self {
        let options = options.unwrap_or_default();
        let levels = CustomLevels::new(options.levels.unwrap_or_default());
        let level = options.level.unwrap_or_default();
        let transports = options.transports.unwrap_or_default();
        let format = options.format.unwrap_or_else(|| json());

        let logger = Logger {
            levels,
            format,
            level,
            transports,
        };

        logger
    }

    pub fn is_level_enabled(&self, level: &str) -> bool {
        let given_level_value = self.get_level_severity(level);
        if given_level_value.is_none() {
            return false;
        }

        let configured_level_value = self.get_level_severity(&self.level);
        if configured_level_value.is_none() {
            return false;
        }

        if self.transports.is_empty() {
            return configured_level_value.unwrap() >= given_level_value.unwrap();
        }

        self.transports.iter().any(|transport| {
            let transport_level_value = transport
                .get_level()
                .and_then(|transport_level| self.get_level_severity(transport_level))
                .unwrap_or(configured_level_value.unwrap());
            transport_level_value >= given_level_value.unwrap()
        })
    }

    fn get_level_severity(&self, level: &str) -> Option<u8> {
        self.levels.get_severity(level)
    }

    fn format_message(
        &self,
        entry: &LogEntry,
        transport_format: Option<&Format>,
    ) -> Option<String> {
        let converted_entry = convert_log_entry(entry);

        // Apply the transport-specific format if provided
        let formatted_entry = if let Some(format) = transport_format {
            format.transform(converted_entry.clone(), None)
        } else {
            // Otherwise, use the default logger format
            self.format.transform(converted_entry.clone(), None)
        };

        formatted_entry.map(|entry| entry.message)
    }

    pub fn log(&self, entry: LogEntry) {
        if entry.message.is_empty() && entry.meta.is_empty() {
            return;
        }

        if !self.is_level_enabled(&entry.level) {
            return;
        }

        for transport in &self.transports {
            if let Some(formatted_message) = self.format_message(&entry, transport.get_format()) {
                transport.log(&formatted_message, &entry.level);
            }
        }
    }

    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::new()
    }

    pub fn configure(&mut self, options: LoggerOptions) {
        if let Some(levels) = options.levels {
            self.levels = CustomLevels::new(levels);
        }
        if let Some(format) = options.format {
            self.format = format;
        }
        if let Some(level) = options.level {
            self.level = level;
        }
        if let Some(transports) = options.transports {
            self.transports = transports;
        }
    }

    pub fn default() -> &'static Mutex<Logger> {
        &DEFAULT_LOGGER
    }
}

macro_rules! create_log_methods {
    ($($level:ident),*) => {
        impl Logger {
            $(
                pub fn $level(&self, message: &str) {
                    let log_entry = LogEntry::builder(stringify!($level), message).build();
                    self.log(log_entry);
                }
            )*
        }
    };
}

create_log_methods!(info, warn, error, debug, trace);

// Global logging functions
pub fn log(level: &str, message: &str) {
    DEFAULT_LOGGER
        .lock()
        .unwrap()
        .log(LogEntry::builder(level, message).build());
}

pub fn configure(options: LoggerOptions) {
    DEFAULT_LOGGER.lock().unwrap().configure(options);
}

// Convenience macros for global logging
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log("info", &format!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::log("warn", &format!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::log("error", &format!($($arg)*));
    }
}

// ... Add more macros for other log levels
