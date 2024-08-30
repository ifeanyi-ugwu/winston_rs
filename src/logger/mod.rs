pub mod create_logger;
mod custom_levels;
mod default_levels;
pub mod log_entry;
pub mod log_query;
mod logger_builder;
mod logger_options;
mod logger_worker;
pub mod transports;

use crossbeam_channel::{bounded, Sender as CBSender};
use custom_levels::CustomLevels;
use lazy_static::lazy_static;
use log_entry::LogEntry;
pub use log_query::LogQuery;
use logform::{json, Format};
use logger_builder::LoggerBuilder;
pub use logger_options::LoggerOptions;
use logger_worker::LoggerWorker;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use transports::Transport;

pub struct Logger {
    levels: CustomLevels,
    format: Format,
    level: String,
    transports: Vec<Arc<dyn Transport + Send + Sync>>,
    log_sender: CBSender<LogEntry>,
    worker_thread: Option<thread::JoinHandle<()>>,
}

impl Logger {
    pub fn new(options: Option<LoggerOptions>) -> Self {
        let options = options.unwrap_or_default();
        let levels = CustomLevels::new(options.levels.unwrap_or_default());
        let level = options.level.unwrap_or_default();
        let transports = options.transports.unwrap_or_default();
        let format = options.format.unwrap_or_else(|| json());

        let (sender, receiver) = bounded(1000);

        let max_batch_size = options.max_batch_size.unwrap_or(100);
        let flush_interval = options.flush_interval.unwrap_or(Duration::from_secs(1));

        let mut worker = LoggerWorker::new(
            levels.clone(),
            format.clone(),
            level.clone(),
            transports.clone(),
            receiver,
            max_batch_size,
            flush_interval,
        );

        let worker_thread = thread::spawn(move || worker.run());

        Logger {
            levels,
            format,
            level,
            transports,
            log_sender: sender,
            worker_thread: Some(worker_thread),
        }
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

    pub fn log(&self, entry: LogEntry) {
        // Send the log entry to the worker thread
        let _ = self.log_sender.send(entry);
    }

    pub fn flush(&self) {
        // Send a special "flush" message
        let _ = self.log_sender.send(LogEntry::flush());
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

    pub fn query(&self, options: &LogQuery) -> Result<Vec<LogEntry>, String> {
        let mut results = Vec::new();

        for transport in &self.transports {
            if let Some(queryable_transport) = transport.as_queryable() {
                match queryable_transport.query(options) {
                    Ok(mut logs) => results.append(&mut logs),
                    Err(e) => return Err(format!("Query failed: {}", e)),
                }
            }
        }

        Ok(results)
    }

    pub fn default() -> &'static Mutex<Logger> {
        &DEFAULT_LOGGER
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.flush();
        if let Some(thread) = self.worker_thread.take() {
            thread.join().unwrap();
        }
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

// Global logger implementation
lazy_static! {
    static ref DEFAULT_LOGGER: Mutex<Logger> = Mutex::new(Logger::new(None));
}

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
