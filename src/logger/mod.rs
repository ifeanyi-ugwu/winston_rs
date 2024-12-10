mod log_macros;
mod logger_builder;
mod logger_levels;
mod logger_options;
pub mod transports;

use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use lazy_static::lazy_static;
use logform::{json, Format, LogInfo};
use logger_builder::LoggerBuilder;
pub use logger_options::BackpressureStrategy;
pub use logger_options::LoggerOptions;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::Arc;
use std::thread;
use winston_transport::LogQuery;

#[derive(Debug)]
pub enum LogMessage {
    Entry(LogInfo),
    Configure(LoggerOptions),
    Shutdown,
}

struct SharedState {
    options: LoggerOptions,
    buffer: VecDeque<LogInfo>,
}

pub struct Logger {
    worker_thread: Option<thread::JoinHandle<()>>,
    sender: Sender<LogMessage>,
    receiver: Arc<Receiver<LogMessage>>,
    shared_state: Arc<RwLock<SharedState>>,
}

impl Logger {
    pub fn new(options: Option<LoggerOptions>) -> Self {
        let options = options.unwrap_or_default();
        let capacity = options.channel_capacity.unwrap_or(1024);
        let (sender, receiver) = bounded(capacity);

        let shared_receiver = Arc::new(receiver);
        let shared_state = Arc::new(RwLock::new(SharedState {
            options,
            buffer: VecDeque::new(),
        }));

        let worker_receiver = Arc::clone(&shared_receiver);
        let worker_shared_state = Arc::clone(&shared_state);

        // Spawn a worker thread to handle logging
        let worker_thread = thread::spawn(move || {
            //println!("Worker thread starting..."); // Debug print
            Self::worker_loop(worker_receiver, worker_shared_state);
            //println!("Worker thread finished."); // Debug print
        });

        Logger {
            worker_thread: Some(worker_thread),
            sender,
            shared_state,
            receiver: shared_receiver,
        }
    }

    fn worker_loop(receiver: Arc<Receiver<LogMessage>>, shared_state: Arc<RwLock<SharedState>>) {
        for message in receiver.iter() {
            match message {
                LogMessage::Entry(entry) => {
                    let mut state = shared_state.write();
                    if state
                        .options
                        .get_transports()
                        .map_or(true, |t| t.is_empty())
                    {
                        state.buffer.push_back(entry.clone());
                        eprintln!("[winston] Attempt to write logs with no transports, which can increase memory usage: {}", entry.message);
                    } else {
                        Self::process_buffered_entries(&mut state);
                        Self::process_entry(&entry, &state.options)
                        //Self::process_entry(&entry, &state.options);
                        //Self::process_buffered_entries(&mut state);
                    }
                }
                LogMessage::Configure(new_options) => {
                    let mut state = shared_state.write();
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
                    let mut state = shared_state.write();
                    Self::process_buffered_entries(&mut state);
                    break;
                }
            }
        }
    }

    fn process_buffered_entries(state: &mut SharedState) {
        while let Some(entry) = state.buffer.pop_front() {
            Self::process_entry(&entry, &state.options);
        }
    }

    fn process_entry(entry: &LogInfo, options: &LoggerOptions) {
        let format = options.format.clone().unwrap_or_else(|| json());

        if !Self::is_level_enabled(&entry.level, options) {
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
                    transport.log(formatted_message);
                }
            }
        }
    }

    fn is_level_enabled(entry_level: &str, options: &LoggerOptions) -> bool {
        let levels = options.levels.clone().unwrap_or_default();
        let global_level = options.level.as_deref().unwrap_or("info");

        // Return false if we can't get severity for the entry level or global level
        let entry_level_value = match levels.get_severity(entry_level) {
            Some(value) => value,
            None => return false,
        };

        let global_level_value = match levels.get_severity(global_level) {
            Some(value) => value,
            None => return false,
        };

        // If no transports are defined, fall back to the global level comparison
        if let Some(transports) = options.get_transports() {
            // Return true if any transport's level is prioritized and matches the severity
            return transports.iter().any(|transport| {
                match transport
                    .get_level()
                    .and_then(|level| levels.get_severity(level))
                {
                    Some(transport_level_value) => transport_level_value >= entry_level_value,
                    None => global_level_value >= entry_level_value,
                }
            });
        }

        // Fallback to global level check if no transports
        global_level_value >= entry_level_value
    }

    fn format_message(
        entry: &LogInfo,
        transport_format: Option<&Format>,
        default_format: &Format,
    ) -> Option<LogInfo> {
        let format_to_use = transport_format.unwrap_or(default_format);
        format_to_use.transform(entry.clone(), None)
        //.map(|entry| entry.message)
    }

    pub fn query(&self, options: &LogQuery) -> Result<Vec<LogInfo>, String> {
        let state = self.shared_state.read();
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

    pub fn log(&self, entry: LogInfo) {
        match self.sender.try_send(LogMessage::Entry(entry.clone())) {
            Ok(_) => {}
            Err(TrySendError::Full(_)) => {
                self.handle_full_channel(entry);
            }
            Err(TrySendError::Disconnected(_)) => {
                eprintln!("[winston] Channel is disconnected. Unable to log message.");
            }
        }
    }

    pub fn logi(&self, entry: LogInfo) {
        let _ = self.sender.send(LogMessage::Entry(entry));
    }

    /// Handles backpressure strategies when the channel is full.
    fn handle_full_channel(&self, entry: LogInfo) {
        let strategy = {
            let state = self.shared_state.read();
            state
                .options
                .backpressure_strategy
                .clone()
                .unwrap_or(BackpressureStrategy::Block)
        };

        match strategy {
            BackpressureStrategy::DropOldest => {
                self.drop_oldest_and_retry(entry);
            }
            BackpressureStrategy::Block => {
                // Block until the channel has space
                let _ = self.sender.send(LogMessage::Entry(entry));
            }
            BackpressureStrategy::DropCurrent => {
                eprintln!(
                    "[winston] Dropping current log entry due to full channel: {}",
                    entry.message
                );
            }
        }
    }

    /// Drops the oldest log message from the channel and attempts to send the new one.
    fn drop_oldest_and_retry(&self, entry: LogInfo) {
        // Try to remove the oldest message from the channel using the shared receiver
        if let Ok(oldest) = self.receiver.try_recv() {
            eprintln!(
                "[winston] Dropped oldest log entry due to full channel: {:?}",
                oldest
            );
        }

        // Now try to send the new entry again
        if let Err(e) = self.sender.try_send(LogMessage::Entry(entry.clone())) {
            eprintln!(
                "[winston] Failed to log after dropping oldest. Dropping current message: {}",
                entry.message
            );
        }
    }

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
        // let mut logger = DEFAULT_LOGGER.lock().unwrap();
        //logger.close();
        let mut logger = DEFAULT_LOGGER.write();
        logger.close();
    }

    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::new()
    }

    pub fn configure(&self, new_options: Option<LoggerOptions>) {
        let mut state = self.shared_state.write();

        // Clear existing transports
        state.options.transports = Some(Vec::new());

        // Create a new default options instance
        let default_options = LoggerOptions::default();

        // Apply new options if provided
        if let Some(options) = new_options {
            // Format: use the new format if provided, otherwise use the existing format or default to JSON
            if let Some(format) = options.format {
                state.options.format = Some(format);
            } else if state.options.format.is_none() {
                state.options.format = default_options.format.clone();
            }

            // Levels: use the new levels if provided, otherwise use the existing levels or default
            if let Some(levels) = options.levels {
                state.options.levels = Some(levels);
            } else if state.options.levels.is_none() {
                state.options.levels = default_options.levels.clone();
            }

            // Level: use the new level if provided, otherwise use the existing level or default to "info"
            if let Some(level) = options.level {
                state.options.level = Some(level);
            } else if state.options.level.is_none() {
                state.options.level = default_options.level.clone();
            }

            // Add all transports we have been provided
            if let Some(transports) = options.transports {
                state.options.transports = Some(transports);
            }
        }

        // Process buffered entries with new configuration
        Self::process_buffered_entries(&mut state);
    }

    pub fn default(
    ) -> parking_lot::lock_api::RwLockReadGuard<'static, parking_lot::RawRwLock, Logger> {
        DEFAULT_LOGGER.read()
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        //println!("Dropping Logger!"); // Debug print
        self.close();
        // println!("Logger dropped"); // Debug print
    }
}

macro_rules! create_log_methods {
    ($($level:ident),*) => {
        impl Logger {
            $(
                pub fn $level(&self, message: &str) {
                    let log_entry = LogInfo::new(stringify!($level), message);
                    self.log(log_entry);
                }
            )*
        }
    };
}

create_log_methods!(info, warn, error, debug, trace);

// Global logger implementation
lazy_static! {
    static ref DEFAULT_LOGGER: RwLock<Logger> = RwLock::new(Logger::new(None));
}

// Global logging functions
pub fn log(entry: LogInfo) {
    //DEFAULT_LOGGER.lock().unwrap().log(entry);
    //let logger = DEFAULT_LOGGER.read();
    let logger = Logger::default();
    logger.log(entry);
}

pub fn configure(options: Option<LoggerOptions>) {
    //DEFAULT_LOGGER.lock().unwrap().configure(options);
    //let logger = DEFAULT_LOGGER.write();
    let logger = Logger::default();
    logger.configure(options);
}

#[macro_export]
macro_rules! log {
      ($level:ident, $message:expr $(, $key:expr => $value:expr)* $(,)?) => {{
        let entry = $crate::format::LogInfo::new(stringify!($level), $message)
            $(.add_meta($key, $value))*;
        $crate::log(entry);
    }};
     ($logger:expr, $level:ident, $message:expr $(, $key:expr => $value:expr)* $(,)?) => {{
        let entry = $crate::format::LogInfo::new(stringify!($level), $message)
            $(.add_meta($key, $value))*;
        $logger.log(entry);
    }};
}
