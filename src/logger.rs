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

struct SharedState {
    options: LoggerOptions,
    buffer: VecDeque<LogInfo>,
}

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
                    (Some(tf), Some(_lf)) => tf.transform(entry.clone(), None),
                    (Some(tf), None) => tf.transform(entry.clone(), None),
                    (None, Some(lf)) => lf.transform(entry.clone(), None),
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
        for entry in &state.buffer {
            if options.matches(entry) {
                results.push(entry.clone());
            }
        }

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

    pub fn configure(&self, new_options: Option<LoggerOptions>) {
        let mut state = self.shared_state.write();

        // Clear existing transports
        if let Some(t) = state.options.transports.as_mut() {
            t.clear();
        }

        // Create a new default options instance
        let default_options = LoggerOptions::default();

        // Apply new options if provided
        if let Some(options) = new_options {
            // Format: use the new format if provided, otherwise use the existing format or default to JSON
            if let Some(format) = options.format {
                state.options.format = Some(format);
            } else if state.options.format.is_none() {
                state.options.format.clone_from(&default_options.format)
            }

            // Levels: use the new levels if provided, otherwise use the existing levels or default
            if let Some(levels) = options.levels {
                state.options.levels = Some(levels);
            } else if state.options.levels.is_none() {
                state.options.levels.clone_from(&default_options.levels)
            }

            // Level: use the new level if provided, otherwise use the existing level or default to "info"
            if let Some(level) = options.level {
                state.options.level = Some(level);
            } else if state.options.level.is_none() {
                state.options.level.clone_from(&default_options.level)
            }

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
