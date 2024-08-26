pub mod create_logger;
mod custom_levels;
mod default_levels;
pub mod log_entry;
mod logger_builder;
pub mod transports;

use custom_levels::CustomLevels;
use log_entry::{convert_log_entry, LogEntry};
use logform::{json, BoxedLogFormat as LogFormat};
use logger_builder::LoggerBuilder;
use std::{collections::HashMap, sync::Arc};
use transports::{
    console::{ConsoleTransport, ConsoleTransportOptions},
    Transport, TransportStreamOptions,
};

pub struct Logger {
    levels: CustomLevels,
    format: LogFormat,
    level: String,
    transports: Vec<Arc<dyn Transport + Send + Sync>>,
}

pub struct LoggerOptions {
    pub levels: Option<HashMap<String, u8>>,
    pub format: Option<LogFormat>,
    pub level: Option<String>,
    pub transports: Option<Vec<Arc<dyn Transport + Send + Sync>>>,
}

impl Default for LoggerOptions {
    fn default() -> Self {
        let console_options = ConsoleTransportOptions {
            base: Some(TransportStreamOptions {
                level: Some("info".to_string()),
                format: None,
            }),
        };

        let console_transport = Arc::new(ConsoleTransport::new(Some(console_options)));

        LoggerOptions {
            levels: Some(default_levels::default_levels()),
            level: Some("info".to_string()),
            transports: Some(vec![console_transport]),
            format: Some(json()),
        }
    }
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

    fn format_message(&self, entry: &LogEntry, transport_format: Option<&LogFormat>) -> String {
        let mut converted_entry = convert_log_entry(entry);

        // Apply the transport-specific format if provided
        if let Some(format) = transport_format {
            format.transform(&mut converted_entry);
        } else {
            // Otherwise, use the default logger format
            self.format.transform(&mut converted_entry);
        }

        converted_entry.message
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
