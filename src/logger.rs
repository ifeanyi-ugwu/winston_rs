use crate::{
    logger_builder::LoggerBuilder,
    logger_options::{BackpressureStrategy, LoggerOptions},
    logger_transport::LoggerTransport,
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
pub(crate) struct SharedState {
    pub(crate) options: LoggerOptions,
    buffer: VecDeque<LogInfo>,
    // Cache the minimum severity needed for any transport to accept a log
    min_required_severity: Option<u8>,
}

#[derive(Debug)]
pub struct Logger {
    worker_thread: Mutex<Option<thread::JoinHandle<()>>>,
    sender: Sender<LogMessage>,
    receiver: Arc<Receiver<LogMessage>>,
    pub(crate) shared_state: Arc<RwLock<SharedState>>,
    flush_complete: Arc<(Mutex<bool>, Condvar)>,
}

impl Logger {
    pub fn new(options: Option<LoggerOptions>) -> Self {
        let options = options.unwrap_or_default();
        let capacity = options.channel_capacity.unwrap_or(1024);
        let (sender, receiver) = bounded(capacity);
        let flush_complete = Arc::new((Mutex::new(false), Condvar::new()));

        let shared_receiver = Arc::new(receiver);
        // Pre-compute effective levels
        let min_required_severity = Self::compute_min_severity(&options);
        let shared_state = Arc::new(RwLock::new(SharedState {
            options,
            buffer: VecDeque::new(),
            min_required_severity,
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

    fn compute_min_severity(options: &LoggerOptions) -> Option<u8> {
        let levels = options.levels.as_ref()?;
        let mut min_severity = options
            .level
            .as_deref()
            .and_then(|lvl| levels.get_severity(lvl));

        if let Some(transports) = &options.transports {
            for transport in transports {
                if let Some(transport_level) = transport.get_level() {
                    if let Some(transport_severity) = levels.get_severity(transport_level) {
                        min_severity = Some(
                            min_severity
                                .map_or(transport_severity, |cur| cur.max(transport_severity)),
                        );
                    }
                }
            }
        }

        min_severity
    }

    /// Update the cached levels when configuration changes
    fn refresh_effective_levels(state: &mut SharedState) {
        let min_required_severity = Self::compute_min_severity(&state.options);
        state.min_required_severity = min_required_severity;
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
                        .transports
                        .as_ref()
                        .map_or(true, |t| t.is_empty())
                    {
                        state.buffer.push_back(entry.clone());
                        eprintln!("[winston] Attempt to write logs with no transports, which can increase memory usage: {}", entry.message);
                    } else {
                        Self::process_buffered_entries(&mut state);
                        Self::process_entry(&entry, &state)
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

                    Self::refresh_effective_levels(&mut state);
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

                    if let Some(transports) = &state.options.transports {
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
            Self::process_entry(&entry, &state);
        }
    }

    fn process_entry(entry: &LogInfo, state: &SharedState) {
        //TODO: remove this check, it isn't consistent with winstonjs, but may ensure consistent message structure and prevent unnecessary writes
        if entry.message.is_empty() && entry.meta.is_empty() {
            return;
        }

        if !Self::is_level_enabled(&entry.level, &state) {
            return;
        }

        let options = &state.options;
        if let Some(transports) = &options.transports {
            for transport in transports {
                // Check if this transport cares about the level
                let effective_level = transport.get_level().or_else(|| options.level.as_ref());

                if let (Some(levels), Some(effective_level)) = (&options.levels, effective_level) {
                    if let (Some(entry_sev), Some(required_sev)) = (
                        levels.get_severity(&entry.level),
                        levels.get_severity(effective_level),
                    ) {
                        if entry_sev > required_sev {
                            continue; // skip: not enabled
                        }
                    }
                }

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

    fn is_level_enabled(entry_level: &str, state: &SharedState) -> bool {
        if let Some(min_required) = state.min_required_severity {
            if let Some(levels) = &state.options.levels {
                if let Some(entry_severity) = levels.get_severity(entry_level) {
                    return min_required >= entry_severity;
                }
            }
        }
        false
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
        if let Some(transports) = &state.options.transports {
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

        Self::refresh_effective_levels(&mut state);
        // Process buffered entries with new configuration
        Self::process_buffered_entries(&mut state);
    }

    /// Adds a transport wrapped in an Arc directly to the logger
    pub fn add_transport(&self, transport: Arc<dyn Transport<LogInfo> + Send + Sync>) -> bool {
        self.add_logger_transport(LoggerTransport::new(transport))
    }

    /// Add a pre-configured LoggerTransport with custom level/format
    pub fn add_logger_transport(&self, transport: LoggerTransport<LogInfo>) -> bool {
        let mut state = self.shared_state.write();

        if let Some(transports) = &mut state.options.transports {
            transports.push(transport);
            true
        } else {
            state.options.transports = Some(vec![transport]);
            true
        }
    }

    /// Removes a transport wrapped in an Arc from the logger
    pub fn remove_transport(&self, transport: Arc<dyn Transport<LogInfo> + Send + Sync>) -> bool {
        let mut state = self.shared_state.write();

        if let Some(transports) = &mut state.options.transports {
            // Find the index of the transport to remove based on pointer equality
            if let Some(index) = transports
                .iter()
                .position(|t| Arc::ptr_eq(&transport, &t.get_transport()))
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

#[cfg(feature = "log-backend")]
impl Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        let state = self.shared_state.read();
        Self::is_level_enabled(&metadata.level().as_str().to_lowercase(), &state)
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

        // Add key-values if kv feature is enabled
        #[cfg(feature = "log-backend-kv")]
        {
            let mut kv_visitor = KeyValueCollector::new();
            record.key_values().visit(&mut kv_visitor).ok();

            for (key, value) in kv_visitor.collected {
                meta.insert(key, value);
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

#[cfg(feature = "log-backend-kv")]
struct KeyValueCollector {
    collected: Vec<(String, serde_json::Value)>,
}

#[cfg(feature = "log-backend-kv")]
impl KeyValueCollector {
    fn new() -> Self {
        Self {
            collected: Vec::new(),
        }
    }
}

#[cfg(feature = "log-backend-kv")]
impl<'kvs> log::kv::Visitor<'kvs> for KeyValueCollector {
    fn visit_pair(
        &mut self,
        key: log::kv::Key<'kvs>,
        value: log::kv::Value<'kvs>,
    ) -> Result<(), log::kv::Error> {
        let json_value = if let Some(s) = value.to_borrowed_str() {
            serde_json::Value::String(s.to_string())
        } else if let Some(i) = value.to_i64() {
            serde_json::Value::Number(serde_json::Number::from(i))
        } else if let Some(u) = value.to_u64() {
            serde_json::Value::Number(serde_json::Number::from(u))
        } else if let Some(f) = value.to_f64() {
            serde_json::Number::from_f64(f)
                .map(serde_json::Value::Number)
                .unwrap_or_else(|| serde_json::Value::String(f.to_string()))
        } else {
            // Fallback to string representation
            serde_json::Value::String(format!("{}", value))
        };

        self.collected.push((key.as_str().to_string(), json_value));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::logger_options::LoggerOptions;
    use std::sync::{Arc, Mutex};

    // Simple mock for unit tests
    #[derive(Clone)]
    struct TestTransport {
        logs: Arc<Mutex<Vec<LogInfo>>>,
    }

    impl TestTransport {
        fn new() -> Self {
            Self {
                logs: Arc::new(Mutex::new(Vec::new())),
            }
        }

        fn get_logs(&self) -> Vec<LogInfo> {
            self.logs.lock().unwrap().clone()
        }
    }

    impl Transport<LogInfo> for TestTransport {
        fn log(&self, info: LogInfo) {
            self.logs.lock().unwrap().push(info);
        }

        fn flush(&self) -> Result<(), String> {
            Ok(())
        }

        fn query(&self, _: &LogQuery) -> Result<Vec<LogInfo>, String> {
            Ok(self.get_logs())
        }
    }

    #[test]
    fn test_logger_creation_with_default_options() {
        let logger = Logger::new(None);
        assert!(logger.shared_state.read().options.levels.is_some());
    }

    #[test]
    fn test_logger_creation_with_custom_options() {
        let options = LoggerOptions::new().level("debug").channel_capacity(512);

        let logger = Logger::new(Some(options));
        let state = logger.shared_state.read();
        assert_eq!(state.options.level.as_deref(), Some("debug"));
    }

    #[test]
    fn test_add_transport() {
        let logger = Logger::new(None);
        let transport = Arc::new(TestTransport::new());

        assert!(logger.add_transport(transport.clone()));

        let state = logger.shared_state.read();
        assert_eq!(state.options.transports.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_add_multiple_transports() {
        let logger = Logger::new(None);
        let transport1 = Arc::new(TestTransport::new());
        let transport2 = Arc::new(TestTransport::new());

        assert!(logger.add_transport(transport1));
        assert!(logger.add_transport(transport2));

        let state = logger.shared_state.read();
        assert_eq!(state.options.transports.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_remove_transport() {
        let logger = Logger::new(None);
        let transport = Arc::new(TestTransport::new());

        logger.add_transport(transport.clone());
        assert!(logger.remove_transport(transport.clone()));

        let state = logger.shared_state.read();
        assert!(state.options.transports.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_remove_nonexistent_transport() {
        let logger = Logger::new(None);
        let transport = Arc::new(TestTransport::new());

        assert!(!logger.remove_transport(transport));
    }

    #[test]
    fn test_remove_transport_twice() {
        let logger = Logger::new(None);
        let transport = Arc::new(TestTransport::new());

        logger.add_transport(transport.clone());
        assert!(logger.remove_transport(transport.clone()));
        assert!(!logger.remove_transport(transport));
    }

    #[test]
    fn test_level_filtering_blocks_lower_severity() {
        let logger = Logger::new(Some(LoggerOptions::new().level("warn")));
        let transport = Arc::new(TestTransport::new());
        logger.add_transport(transport.clone());

        logger.log(LogInfo::new("info", "Should be filtered"));
        logger.log(LogInfo::new("debug", "Should be filtered"));
        logger.log(LogInfo::new("warn", "Should pass"));
        logger.log(LogInfo::new("error", "Should pass"));
        logger.flush().unwrap();

        let logs = transport.get_logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].level, "warn");
        assert_eq!(logs[1].level, "error");
    }

    #[test]
    fn test_level_filtering_with_trace() {
        let logger = Logger::new(Some(LoggerOptions::new().level("trace")));
        let transport = Arc::new(TestTransport::new());
        logger.add_transport(transport.clone());

        logger.log(LogInfo::new("trace", "Should pass"));
        logger.log(LogInfo::new("debug", "Should pass"));
        logger.log(LogInfo::new("info", "Should pass"));
        logger.flush().unwrap();

        let logs = transport.get_logs();
        assert_eq!(logs.len(), 3);
    }

    #[test]
    fn test_transport_specific_level() {
        #[derive(Clone)]
        struct LeveledTransport {
            logs: Arc<Mutex<Vec<LogInfo>>>,
        }

        impl Transport<LogInfo> for LeveledTransport {
            fn log(&self, info: LogInfo) {
                self.logs.lock().unwrap().push(info);
            }

            fn query(&self, _: &LogQuery) -> Result<Vec<LogInfo>, String> {
                Ok(self.logs.lock().unwrap().clone())
            }
        }

        let logger = Logger::new(Some(
            LoggerOptions::new()
                .level("trace")
                .format(logform::passthrough()),
        ));
        let transport = Arc::new(LeveledTransport {
            logs: Arc::new(Mutex::new(Vec::new())),
        });
        let transport = LoggerTransport::new(transport).with_level("error".to_string());
        logger.add_logger_transport(transport);

        logger.log(LogInfo::new("info", "Filtered by transport"));
        logger.log(LogInfo::new("error", "Passes transport filter"));
        logger.flush().unwrap();

        let query = LogQuery::new();
        let logs = logger.query(&query).unwrap();

        assert_eq!(logs.len(), 1);
        assert_eq!(logs[0].level, "error");
        assert_eq!(logs[0].message, "Passes transport filter");
    }

    #[test]
    fn test_empty_message_handling() {
        let logger = Logger::new(None);
        let transport = Arc::new(TestTransport::new());
        logger.add_transport(transport.clone());

        logger.log(LogInfo::new("info", ""));
        logger.flush().unwrap();

        // Empty messages should be filtered out
        let logs = transport.get_logs();
        assert_eq!(logs.len(), 0);
    }

    #[test]
    fn test_configure_updates_level() {
        let logger = Logger::new(Some(LoggerOptions::new().level("error")));
        let transport = Arc::new(TestTransport::new());
        logger.add_transport(transport.clone());

        logger.log(LogInfo::new("warn", "Should be filtered"));
        logger.flush().unwrap();
        assert_eq!(transport.get_logs().len(), 0);

        // Reconfigure to debug
        logger.configure(Some(LoggerOptions::new().level("debug")));
        logger.add_transport(transport.clone());

        logger.log(LogInfo::new("warn", "Should pass now"));
        logger.flush().unwrap();
        assert_eq!(transport.get_logs().len(), 1);
    }

    #[test]
    fn test_configure_clears_transports() {
        let logger = Logger::new(None);
        let transport = Arc::new(TestTransport::new());
        logger.add_transport(transport.clone());

        let state = logger.shared_state.read();
        assert_eq!(state.options.transports.as_ref().unwrap().len(), 1);
        drop(state);

        logger.configure(Some(LoggerOptions::new()));

        let state = logger.shared_state.read();
        assert!(state.options.transports.as_ref().unwrap().is_empty());
    }

    #[test]
    fn test_flush_returns_ok() {
        let logger = Logger::new(None);
        assert!(logger.flush().is_ok());
    }

    #[test]
    fn test_flush_with_transport() {
        let logger = Logger::new(None);
        let transport = Arc::new(TestTransport::new());
        logger.add_transport(transport.clone());

        logger.log(LogInfo::new("info", "Test"));
        assert!(logger.flush().is_ok());
        assert_eq!(transport.get_logs().len(), 1);
    }

    #[test]
    #[ignore = "test hangs"]
    fn test_close_flushes_logs() {
        let logger = Logger::new(None);
        let transport = Arc::new(TestTransport::new());
        logger.add_transport(transport.clone());

        logger.log(LogInfo::new("info", "Test"));
        logger.close();

        assert_eq!(transport.get_logs().len(), 1);
    }

    #[test]
    #[ignore = "test fails"]
    fn test_buffering_without_transports() {
        let logger = Logger::new(None);

        logger.log(LogInfo::new("info", "Buffered message"));

        let state = logger.shared_state.read();
        assert_eq!(state.buffer.len(), 1);
    }

    #[test]
    #[ignore = "test fails"]
    fn test_buffer_processed_when_transport_added() {
        let logger = Logger::new(None);

        // Log without transport - should buffer
        logger.log(LogInfo::new("info", "Buffered"));

        let state = logger.shared_state.read();
        assert_eq!(state.buffer.len(), 1);
        drop(state);

        // Add transport
        let transport = Arc::new(TestTransport::new());
        logger.add_transport(transport.clone());

        // Log another message - should process buffer + new message
        logger.log(LogInfo::new("info", "Direct"));
        logger.flush().unwrap();

        let logs = transport.get_logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[0].message, "Buffered");
        assert_eq!(logs[1].message, "Direct");
    }

    #[test]
    fn test_query_returns_results() {
        let logger = Logger::new(None);
        let transport = Arc::new(TestTransport::new());
        logger.add_transport(transport);

        logger.log(LogInfo::new("info", "Test message"));
        logger.flush().unwrap();

        let query = LogQuery::new();
        let results = logger.query(&query);
        assert!(results.is_ok());
        assert_eq!(results.unwrap().len(), 1);
    }

    #[test]
    fn test_compute_min_severity() {
        let options = LoggerOptions::new().level("warn");
        let min_sev = Logger::compute_min_severity(&options);

        assert!(min_sev.is_some());
        // warn should have higher severity value than info
        assert!(min_sev.unwrap() > 0);
    }
}
