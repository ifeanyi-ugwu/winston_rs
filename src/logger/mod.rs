pub mod create_logger;
mod custom_levels;
mod default_levels;
pub mod log_entry;
pub mod log_query;
mod logger_builder;
mod logger_options;
pub mod transports;

use crossbeam_channel::{unbounded, Receiver, Sender};
use custom_levels::CustomLevels;
use lazy_static::lazy_static;
use log_entry::{convert_log_entry, LogEntry};
pub use log_query::LogQuery;
use logform::{json, Format};
use logger_builder::LoggerBuilder;
pub use logger_options::LoggerOptions;
use std::collections::HashMap;
use std::ops::Deref;
use std::sync::{Arc, Mutex, RwLock};
use std::{clone, thread};
use transports::Transport;

#[derive(Debug)]
pub enum LogMessage {
    Entry(LogEntry),
    Configure(LoggerOptions),
    Shutdown,
}

struct SharedState {
    options: LoggerOptions,
    buffer: Vec<LogEntry>,
}

pub struct Logger {
    //levels: CustomLevels,
    //format: Format,
    //level: String,
    // transports: Vec<Arc<dyn Transport + Send + Sync>>,
    worker_thread: Option<thread::JoinHandle<()>>,
    sender: Sender<LogMessage>,
    //buffer: Arc<Mutex<Vec<LogEntry>>>, // Buffer to store log messages
    shared_state: Arc<RwLock<SharedState>>,
}

impl Logger {
    pub fn new(options: Option<LoggerOptions>) -> Self {
        /*let options = options.unwrap_or_default();
        let options_for_worker = Arc::new(Mutex::new(options.clone()));
        let levels = CustomLevels::new(options.levels.unwrap_or_default());
        let level = options.level.unwrap_or_default();
        let transports = options.get_transports().unwrap_or_default();
        let format = options.get_format().unwrap();
        //.unwrap_or_else(|| logger_options::DebugFormat(json()));

        let (sender, receiver) = unbounded();
        let buffer = Arc::new(Mutex::new(Vec::new()));

        let transports_clone = transports.clone();
        let levels_clone = levels.clone();
        let level_clone = level.clone();

        let options_clone = Arc::clone(&options_for_worker);
        let buffer_clone = Arc::clone(&buffer);
        let format_clone = format.clone();
        let shared_state = Arc::new(RwLock::new(SharedState {
            options,
            buffer: Vec::new(),
        }));*/
        let options = options.unwrap_or_default();
        let (sender, receiver) = unbounded();
        let shared_state = Arc::new(RwLock::new(SharedState {
            options,
            buffer: Vec::new(),
        }));

        let worker_shared_state = Arc::clone(&shared_state);

        // Spawn a worker thread to handle logging
        let worker_thread = thread::spawn(move || {
            println!("Worker thread starting...");
            Self::worker_loop(
                receiver,
                // transports_clone,
                //buffer_clone,
                //  format_clone,
                //  levels_clone,
                //  level_clone,
                // options_clone,
                worker_shared_state,
            );
            println!("Worker thread finished.");
        });

        Logger {
            //levels,
            // format,
            //level,
            //transports,
            //log_sender: sender,
            worker_thread: Some(worker_thread),
            sender,
            //buffer,
            shared_state,
        }
    }

    /*fn process_logs(
        receiver: Receiver<LogMessage>,
        transports: Vec<Arc<dyn Transport + Send + Sync>>,
        buffer: Arc<Mutex<Vec<LogEntry>>>,
        default_format: Format,
        levels: CustomLevels,
        level: String,
        options: Arc<Mutex<LoggerOptions>>,
    ) {
        /*for message in receiver {
            match message {
                LogMessage::Entry(entry) => {
                    // Add entry to buffer
                    let mut buf = buffer.lock().unwrap();
                    buf.push(entry);
                    while let Some(entry) = buf.pop() {
                        Self::process_entry(&entry, &transports, &default_format, &levels, &level);
                    }
                }
                //LogMessage::Shutdown => break,
                LogMessage::Shutdown => {
                    // Process remaining buffer before shutdown
                    let mut buf = buffer.lock().unwrap();
                    while let Some(entry) = buf.pop() {
                        Self::process_entry(&entry, &transports, &default_format, &levels, &level);
                        println!("Processed log entry during shutdown: {:?}", entry);
                    }
                    break;
                }
            }
        }*/
        loop {
            match receiver.recv().unwrap() {
                LogMessage::Entry(entry) => {
                    let mut buf = buffer.lock().unwrap();
                    buf.push(entry);
                    Self::process_buffered_entries(
                        // &transports,
                        &mut buf,
                        // &default_format,
                        //&levels,
                        // &level,
                        &options,
                    );
                }
                LogMessage::Shutdown => {
                    let mut buf = buffer.lock().unwrap();
                    Self::process_buffered_entries(
                        // &transports,
                        &mut buf,
                        //&default_format,
                        //&levels,
                        //&level,
                        &options,
                    );
                    break;
                }
            }
        }

        println!("Logger worker thread shutting down...");
    }*/

    fn worker_loop(receiver: Receiver<LogMessage>, shared_state: Arc<RwLock<SharedState>>) {
        for message in receiver {
            match message {
                LogMessage::Entry(entry) => {
                    let mut state = shared_state.write().unwrap();
                    if state
                        .options
                        .get_transports()
                        .map_or(true, |t| t.is_empty())
                    {
                        state.buffer.push(entry.clone());
                        eprintln!("[winston] Attempt to write logs with no transports, which can increase memory usage: {}", entry.message);
                    } else {
                        Self::process_entry(&entry, &state.options);
                        Self::process_buffered_entries(&mut state);
                    }
                }
                LogMessage::Configure(new_options) => {
                    let mut state = shared_state.write().unwrap();
                    //state.options = new_options;
                    // Update only the provided options
                    if let Some(level) = new_options.level {
                        state.options.level = Some(level);
                    }
                    if let Some(levels) = new_options.levels {
                        state.options.levels = Some(levels);
                    }
                    if let Some(transports) = new_options.transports {
                        state.options.transports = Some(transports);
                    }
                    if let Some(format) = new_options.format {
                        state.options.format = Some(format);
                    }
                    // Add any other options that need to be configurable

                    // Process buffered entries with new configuration
                    Self::process_buffered_entries(&mut state);
                }
                //LogMessage::Shutdown => break,
                LogMessage::Shutdown => {
                    let mut state = shared_state.write().unwrap();
                    Self::process_buffered_entries(&mut state);
                    break;
                }
            }
        }
    }
    /*fn process_buffered_entries(
        //transports: &[Arc<dyn Transport + Send + Sync>],
        buffer: &mut Vec<LogEntry>,
        // default_format: &Format,
        //levels: &CustomLevels,
        //level: &String,
        options: &Arc<Mutex<LoggerOptions>>,
    ) {
        let options = options.lock().unwrap();
        let levels = CustomLevels::new(options.levels.clone().unwrap_or_default());
        let format = options.format.clone().unwrap_or_else(|| json());
        let level = options.level.clone().unwrap_or_default();
        let transports = options.transports.clone().unwrap_or_default();

        while let Some(entry) = buffer.pop() {
            Self::process_entry(&entry, &transports, &format, &levels, &level);
        }
    }*/

    fn process_buffered_entries(state: &mut SharedState) {
        while let Some(entry) = state.buffer.pop() {
            Self::process_entry(&entry, &state.options);
        }
    }

    /*fn process_entry(
        entry: &LogEntry,
        transports: &[Arc<dyn Transport + Send + Sync>],
        default_format: &Format,
        levels: &CustomLevels,
        level: &String,
    ) {
        if !Self::is_level_enabled_static(&levels, &level, &entry.level) {
            return;
        }

        if entry.message.is_empty() && entry.meta.is_empty() {
            return;
        }

        if transports.is_empty() {
            eprintln!("[winston] Attempt to write logs with no transports, which can increase memory usage: {}",entry.message);
            return;
        }

        for transport in transports {
            if let Some(formatted_message) =
                Self::format_message_static(entry, transport.get_format(), default_format)
            {
                transport.log(&formatted_message, &entry.level);
            } else {
                println!("Did not format message");
            }
        }
        println!("Processed log entry: {:?}", entry);
    }*/
    fn process_entry(entry: &LogEntry, options: &LoggerOptions) {
        let levels = CustomLevels::new(options.levels.clone().unwrap_or_default());
        let level = options.level.as_deref().unwrap_or("info");
        let format = options.get_format().cloned().unwrap_or_else(|| json());

        if !Self::is_level_enabled(&levels, level, &entry.level) {
            return;
        }

        //TODO: remove this check, it isn't consistent with winstonjs, but may ensure consistent message structure and prevent unnecessary writes
        if entry.message.is_empty() && entry.meta.is_empty() {
            return;
        }

        if let Some(transports) = options.get_transports() {
            for transport in transports {
                if let Some(formatted_message) =
                    Self::format_message(entry, transport.get_format(), &format)
                {
                    transport.log(&formatted_message, &entry.level);
                }
            }
        }
    }

    fn is_level_enabled(levels: &CustomLevels, configured_level: &str, entry_level: &str) -> bool {
        let given_level_value = levels.get_severity(entry_level);
        let configured_level_value = levels.get_severity(configured_level);

        match (given_level_value, configured_level_value) {
            (Some(given), Some(configured)) => configured >= given,
            _ => false,
        }
    }

    fn format_message(
        entry: &LogEntry,
        transport_format: Option<&Format>,
        default_format: &Format,
    ) -> Option<String> {
        let converted_entry = convert_log_entry(entry);
        let format_to_use = transport_format.unwrap_or(default_format);
        format_to_use
            .transform(converted_entry, None)
            .map(|entry| entry.message)
    }

    pub fn query(&self, options: &LogQuery) -> Result<Vec<LogEntry>, String> {
        let state = self.shared_state.read().unwrap();
        let mut results = Vec::new();

        // First, query the buffered entries
        for entry in &state.buffer {
            if options.matches(entry) {
                results.push(entry.clone());
            }
        }

        // Then, query each transport
        if let Some(transports) = state.options.get_transports() {
            for transport in transports {
                if let Some(queryable_transport) = transport.as_queryable() {
                    match queryable_transport.query(options) {
                        Ok(mut logs) => results.append(&mut logs),
                        Err(e) => return Err(format!("Query failed: {}", e)),
                    }
                }
            }
        }

        Ok(results)
    }

    /*fn is_level_enabled_static(
        levels: &CustomLevels,
        configured_level: &str,
        entry_level: &str,
    ) -> bool {
        let given_level_value = levels.get_severity(entry_level);
        let configured_level_value = levels.get_severity(configured_level);

        if given_level_value.is_none() || configured_level_value.is_none() {
            return false;
        }

        configured_level_value.unwrap() >= given_level_value.unwrap()
    }

    fn format_message_static(
        entry: &LogEntry,
        format: Option<&Format>,
        default_format: &Format,
    ) -> Option<String> {
        let converted_entry = convert_log_entry(entry);

        // Apply the provided format if available, otherwise use the default format
        let format_to_use = format.unwrap_or(default_format);
        format_to_use
            .transform(converted_entry, None)
            .map(|entry| entry.message)
    }*/

    /*pub fn is_level_enabled(&self, level: &str) -> bool {
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
    }*/

    /*pub fn log(&self, entry: LogEntry) {
        self.log_async(entry);
    }*/

    pub fn log(&self, entry: LogEntry) {
        let _ = self.sender.send(LogMessage::Entry(entry));
    }

    /*pub fn log_sync(&self, entry: LogEntry) {
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
    */
    /*pub fn log_async(&self, entry: LogEntry) {
        println!("Sending log message: {:?}", entry);
        //let _ = self.sender.send(LogMessage::Entry(entry));
        let result = self.sender.send(LogMessage::Entry(entry));
        if result.is_err() {
            println!("Failed to send log message!");
        }
    }*/

    pub fn close(&mut self) {
        let _ = self.sender.send(LogMessage::Shutdown); // Send shutdown signal
        if let Some(thread) = self.worker_thread.take() {
            //thread.join().unwrap();
            if let Err(e) = thread.join() {
                eprintln!("Error joining worker thread: {:?}", e);
            }
        }
    }

    /// Gracefully shuts down the logger by:
    ///
    /// 1. **Sending a Shutdown Signal:**
    ///    Sends a `Shutdown` message to the internal worker thread to indicate that no more log entries should be processed. This ensures that the worker thread stops accepting new log messages.
    ///
    /// 2. **Processing Remaining Entries:**
    ///    The worker thread processes any remaining log entries in the buffer before terminating. This step is crucial to avoid losing log messages that were enqueued before the shutdown signal was sent.
    ///
    /// 3. **Joining the Worker Thread:**
    ///    Waits for the worker thread to complete its processing and exit. This ensures that all buffered log entries are handled and that the thread is cleanly terminated.
    ///
    /// **Rationale:**
    /// - **Message Integrity:** Guarantees that all log messages in the buffer are processed, preventing data loss.
    /// - **Resource Management:** Helps in releasing resources like memory and thread handles, preventing leaks and ensuring clean termination of the logger.
    /// - **Thread Safety:** Ensures that the worker thread completes its task before the logger is fully dropped, avoiding potential issues with incomplete processing.
    ///
    /// **Note:**
    /// - In the context of global loggers initialized with `lazy_static!`, the `Drop` implementation might not be guaranteed to run if the global logger is not explicitly closed before the application exits. This can lead to unprocessed log entries if the application terminates abruptly. Hence, the `shutdown` method is crucial for ensuring that all log messages are properly handled.
    pub fn shutdown() {
        // Call close method which will send shutdown signal and join the worker thread
        let mut logger = DEFAULT_LOGGER.lock().unwrap();
        logger.close();
    }

    /*fn format_message(
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
    }*/

    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::new()
    }

    pub fn configure(&self, new_options: Option<LoggerOptions>) {
        if new_options.is_none() {
            return;
        } else {
            let _ = self
                .sender
                .send(LogMessage::Configure(new_options.unwrap()));
        }
    }

    /*pub fn configure(&mut self, options: Option<LoggerOptions>) {
        // Reset to original defaults
        /*let default_options = LoggerOptions::default();
        self.level = default_options.level.unwrap_or_default();
        self.levels = CustomLevels::new(default_options.levels.unwrap_or_default());
        self.transports = default_options.transports.unwrap_or_default();
        self.format = default_options.format.unwrap_or_else(|| json());*/
        /* let mut opts = self.options.lock().unwrap();
        if let Some(new_opts) = options.clone() {
            if let Some(level) = new_opts.level {
                opts.level = Some(level);
            }
            if let Some(levels) = new_opts.levels {
                opts.levels = Some(levels);
            }
            if let Some(transports) = new_opts.transports {
                opts.transports = Some(transports);
            }
            if let Some(format) = new_opts.format {
                opts.format = Some(format);
            }
        }*/
        // Apply new options if provided
        if let Some(opts) = options {
            if let Some(level) = opts.level {
                self.level = level;
            }
            if let Some(levels) = opts.levels {
                self.levels = CustomLevels::new(levels);
            }
            if let Some(transports) = opts.transports {
                self.transports = transports;
            }
            if let Some(format) = opts.format {
                self.format = format;
            }
        }
        *self = Logger::new(options)
    }*/

    /*pub fn query(&self, options: &LogQuery) -> Result<Vec<LogEntry>, String> {
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
    }*/

    /*pub fn default() -> &'static Mutex<Logger> {
        &DEFAULT_LOGGER
    }*/
}

impl Drop for Logger {
    fn drop(&mut self) {
        println!("Dropping Logger!");
        self.close();
        println!("Logger dropped");
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
/*lazy_static! {
    static ref DEFAULT_LOGGER: Logger = Logger::new(None);
}*/

// Global logging functions
pub fn log(level: &str, message: &str) {
    //init_logger();
    DEFAULT_LOGGER
        .lock()
        .unwrap()
        .log(LogEntry::builder(level, message).build());
}

pub fn configure(options: Option<LoggerOptions>) {
    DEFAULT_LOGGER.lock().unwrap().configure(options);
    // DEFAULT_LOGGER.configure(options);
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
