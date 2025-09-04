use crate::{
    logger_builder::LoggerBuilder,
    logger_options::{BackpressureStrategy, DebugTransport, LoggerOptions},
};
use crossbeam_channel::{bounded, Receiver, Sender, TrySendError};
use logform::LogInfo;
use parking_lot::RwLock;
use std::{
    collections::VecDeque,
    sync::{Arc, Condvar, Mutex},
    thread,
};
use winston_transport::{LogQuery, Transport};

#[derive(Debug)]
pub enum LogMessage {
    Entry(LogInfo),
    Configure(LoggerOptions),
    Shutdown,
    Flush,
}

#[derive(Debug)]
struct SharedState {
    options: LoggerOptions,
    buffer: VecDeque<LogInfo>,
}

#[derive(Debug)]
pub struct Logger {
    worker_thread: Mutex<Option<thread::JoinHandle<()>>>,
    sender: Sender<LogMessage>,
    receiver: Arc<Receiver<LogMessage>>,
    shared_state: Arc<RwLock<SharedState>>,
    flush_complete: Arc<(Mutex<bool>, Condvar)>,
}

impl Logger {
    pub fn new(options: Option<LoggerOptions>) -> Self {
        let options = options.unwrap_or_default();
        let capacity = options.channel_capacity.unwrap_or(1024);
        let (sender, receiver) = bounded(capacity);
        let flush_complete = Arc::new((Mutex::new(false), Condvar::new()));

        let shared_receiver = Arc::new(receiver);
        let shared_state = Arc::new(RwLock::new(SharedState {
            options,
            buffer: VecDeque::new(),
        }));

        let worker_receiver = Arc::clone(&shared_receiver);
        let worker_shared_state = Arc::clone(&shared_state);
        let worker_flush_complete = Arc::clone(&flush_complete);

        // Spawn a worker thread to handle logging
        let worker_thread = thread::spawn(move || {
            //println!("Worker thread starting..."); // Debug print
            Self::worker_loop(worker_receiver, worker_shared_state, worker_flush_complete);
            //println!("Worker thread finished."); // Debug print
        });

        Logger {
            worker_thread: Mutex::new(Some(worker_thread)),
            sender,
            shared_state,
            receiver: shared_receiver,
            flush_complete,
        }
    }

    fn worker_loop(
        receiver: Arc<Receiver<LogMessage>>,
        shared_state: Arc<RwLock<SharedState>>,
        flush_complete: Arc<(Mutex<bool>, Condvar)>,
    ) {
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
                LogMessage::Flush => {
                    let mut state = shared_state.write();
                    Self::process_buffered_entries(&mut state);

                    if let Some(transports) = state.options.get_transports() {
                        for transport in transports {
                            let _ = transport.flush();
                        }
                    }

                    // Signal completion
                    let (lock, cvar) = &*flush_complete;
                    let mut completed = lock.lock().unwrap();
                    *completed = true;
                    cvar.notify_one();
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
        //TODO: remove this check, it isn't consistent with winstonjs, but may ensure consistent message structure and prevent unnecessary writes
        if entry.message.is_empty() && entry.meta.is_empty() {
            return;
        }

        if !Self::is_level_enabled(&entry.level, options) {
            return;
        }

        if let Some(transports) = options.get_transports() {
            for transport in transports {
                let formatted_message = match (transport.get_format(), &options.format) {
                    (Some(tf), Some(_lf)) => tf.transform(entry.clone()),
                    (Some(tf), None) => tf.transform(entry.clone()),
                    (None, Some(lf)) => lf.transform(entry.clone()),
                    (None, None) => Some(entry.clone()),
                };
                if let Some(msg) = formatted_message {
                    transport.log(msg);
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

    pub fn query(&self, options: &LogQuery) -> Result<Vec<LogInfo>, String> {
        let state = self.shared_state.read();
        let mut results = Vec::new();

        // First, query the buffered entries
        results.extend(
            state
                .buffer
                .iter()
                .filter(|entry| options.matches(entry))
                .cloned(),
        );

        // Then, query each transport
        if let Some(transports) = state.options.get_transports() {
            for transport in transports {
                match transport.query(options) {
                    Ok(mut logs) => results.append(&mut logs),
                    Err(e) => return Err(format!("Query failed: {}", e)),
                }
            }
        }

        Ok(results)
    }

    pub fn log(&self, entry: LogInfo) {
        match self.sender.try_send(LogMessage::Entry(entry)) {
            Ok(_) => {}
            Err(TrySendError::Full(LogMessage::Entry(entry))) => {
                self.handle_full_channel(entry);
            }
            Err(TrySendError::Full(LogMessage::Configure(config))) => {
                eprintln!("[winston] Channel is full, forcing config update.");
                let _ = self.sender.send(LogMessage::Configure(config));
            }
            Err(TrySendError::Full(LogMessage::Shutdown)) => {
                eprintln!("[winston] Channel is full, forcing shutdown.");
                let _ = self.sender.send(LogMessage::Shutdown);
            }
            Err(TrySendError::Full(LogMessage::Flush)) => {
                eprintln!("[winston] Channel is full, forcing flush.");
                let _ = self.sender.send(LogMessage::Flush);
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
        if let Err(e) = self.sender.try_send(LogMessage::Entry(entry)) {
            eprintln!(
                "[winston] Failed to log after dropping oldest. Dropping current message: {:?}",
                e.into_inner()
            );
        }
    }

    pub fn close(&self) {
        if let Err(e) = self.flush() {
            eprintln!("Error flushing logs: {}", e);
        }

        let _ = self.sender.send(LogMessage::Shutdown);

        if let Ok(mut thread_handle) = self.worker_thread.lock() {
            if let Some(handle) = thread_handle.take() {
                if let Err(e) = handle.join() {
                    eprintln!("Error joining worker thread: {:?}", e);
                }
            }
        } else {
            eprintln!("Error acquiring lock on worker thread handle during close.");
        }
    }

    // though the flush method on transports is synchronous and can be called directly when this is called,
    // processing it in the background worker ensures that messages sitting in the pipeline is processed
    // before each transport's flush method is called
    pub fn flush(&self) -> Result<(), String> {
        let (lock, cvar) = &*self.flush_complete;
        let mut completed = lock.lock().unwrap();
        *completed = false;

        self.sender
            .send(LogMessage::Flush)
            .map_err(|e| e.to_string())?;

        while !*completed {
            completed = cvar.wait(completed).unwrap();
        }

        Ok(())
    }

    pub fn builder() -> LoggerBuilder {
        LoggerBuilder::new()
    }

    /// Updates the logger configuration with new options, following this fallback chain:
    /// new options -> existing options -> defaults. Always clears existing transports
    /// and processes buffered entries after updating.
    ///
    /// Note: The backpressure strategy and channel capacity are not reconfigured, as they are only used during logger creation.
    ///
    /// # Arguments
    /// * `new_options` - Optional new configuration. If `None`, the existing configuration is retained.
    pub fn configure(&self, new_options: Option<LoggerOptions>) {
        let mut state = self.shared_state.write();
        let default_options = LoggerOptions::default();

        if let Some(t) = state.options.transports.as_mut() {
            t.clear();
        }

        if let Some(options) = new_options {
            state.options.format = options
                .format
                .or_else(|| state.options.format.take().or(default_options.format));

            state.options.levels = options
                .levels
                .or_else(|| state.options.levels.take().or(default_options.levels));

            state.options.level = options
                .level
                .or_else(|| state.options.level.take().or(default_options.level));

            // Add all transports we have been provided
            if let Some(transports) = options.transports {
                state.options.transports = Some(transports);
            }
        }

        // Process buffered entries with new configuration
        Self::process_buffered_entries(&mut state);
    }

    /// Adds a transport wrapped in an Arc directly to the logger
    pub fn add_transport(&self, transport: Arc<dyn Transport>) -> bool {
        let mut state = self.shared_state.write();
        if let Some(transports) = &mut state.options.transports {
            transports.push(DebugTransport(transport));
            true
        } else {
            state.options.transports = Some(vec![DebugTransport(transport)]);
            true
        }
    }

    /// Removes a transport wrapped in an Arc from the logger
    pub fn remove_transport(&self, transport: Arc<dyn Transport>) -> bool {
        let mut state = self.shared_state.write();

        if let Some(transports) = &mut state.options.transports {
            // Find the index of the transport to remove based on pointer equality
            if let Some(index) = transports
                .iter()
                .position(|t| Arc::ptr_eq(&transport, &t.0))
            {
                transports.remove(index);
                true
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        //println!("Dropping Logger!"); // Debug print
        self.close();
        // println!("Logger dropped"); // Debug print
    }
}

impl Default for Logger {
    fn default() -> Self {
        Logger::new(None)
    }
}

#[cfg(feature = "log-backend")]
use log::{Log, Metadata, Record};
use std::sync::OnceLock;

#[cfg(feature = "log-backend")]
static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

#[cfg(feature = "log-backend")]
impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let state = self.shared_state.read();
        Self::is_level_enabled(&metadata.level().as_str().to_lowercase(), &state.options)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        // Convert log::Record to LogInfo
        let mut meta = std::collections::HashMap::new();
        // Add timestamp
        meta.insert(
            "timestamp".to_string(),
            serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
        );
        // Add target (module path)
        meta.insert(
            "target".to_string(),
            serde_json::Value::String(record.target().to_string()),
        );
        // Add file location if available
        if let Some(file) = record.file() {
            meta.insert(
                "file".to_string(),
                serde_json::Value::String(file.to_string()),
            );
        }
        // Add line number if available
        if let Some(line) = record.line() {
            meta.insert(
                "line".to_string(),
                serde_json::Value::Number(serde_json::Number::from(line)),
            );
        }
        // Add module path if different from target
        if let Some(module_path) = record.module_path() {
            if module_path != record.target() {
                meta.insert(
                    "module_path".to_string(),
                    serde_json::Value::String(module_path.to_string()),
                );
            }
        }

        let log_info = LogInfo {
            level: record.level().as_str().to_lowercase(),
            message: record.args().to_string(),
            meta,
        };

        self.log(log_info);
    }

    fn flush(&self) {
        let _ = self.flush();
    }
}

#[cfg(feature = "log-backend")]
impl Logger {
    /// Initialize this logger as the global logger for the `log` crate
    pub fn init_as_global(self) -> Result<(), log::SetLoggerError> {
        let logger = GLOBAL_LOGGER.get_or_init(|| self);

        log::set_logger(logger)?;
        log::set_max_level(log::LevelFilter::Trace);
        Ok(())
    }

    /// Create a logger with default options and set it as the global logger
    pub fn init_default_global() -> Result<(), log::SetLoggerError> {
        let logger = Logger::new(None);
        logger.init_as_global()
    }
}

#[cfg(all(test, feature = "log-backend"))]
mod tests {
    use super::*;
    use crate::{logger_options::LoggerOptions, transports};
    use std::sync::Arc;

    #[test]
    fn test_log_backend_integration() {
        // Create a logger with console transport
        let mut options = LoggerOptions::default();
        options.transports = Some(vec![crate::logger_options::DebugTransport(Arc::new(
            transports::stdout(),
        ))]);

        let logger = Logger::new(Some(options));

        // Initialize as global logger
        logger
            .init_as_global()
            .expect("Failed to initialize global logger");

        // Test logging through the log crate
        log::info!("This is an info message from the log crate");
        log::warn!("This is a warning message");
        log::error!("This is an error message");

        // Flush to ensure all messages are processed
        log::logger().flush();

        // The test passes if no panics occur and messages appear in console
        // In a real test, you might want to capture the output and verify it
    }
}
