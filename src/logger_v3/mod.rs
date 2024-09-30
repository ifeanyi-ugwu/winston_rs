//mod logger_builder;
mod logger_levels;
mod logger_options;
pub mod transports;

use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
//use lazy_static::lazy_static;
use logform::{json, Format, LogInfo};
//use logger_builder::LoggerBuilder;
pub use logger_options::LoggerOptions;
use parking_lot::RwLock;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use transports::transport::Transport;
//use winston_transport::LogQuery;
use scoped_threadpool::Pool;

#[derive(Debug)]
pub enum LogMessage {
    Entry(LogInfo),
    Configure(Option<LoggerOptions>),
    Shutdown,
}

#[derive(Debug)]
struct SharedState {
    options: LoggerOptions,
    buffer: VecDeque<LogInfo>,
}

pub struct Logger {
    //worker_thread: Option<thread::JoinHandle<()>>,
    //sender: Sender<LogMessage>,
    shared_state: Arc<RwLock<SharedState>>,
    transports: Arc<Mutex<Vec<Transport>>>,
    //sender: Sender<String>,
    sender: Option<Sender<LogInfo>>,
    worker_handle: Option<JoinHandle<()>>,
}

impl Logger {
    pub fn new(options: Option<LoggerOptions>) -> Self {
        let options = options.unwrap_or_default();
        //let (sender, receiver) = bounded(1024);
        let (sender, receiver) = unbounded();
        let shared_state = Arc::new(RwLock::new(SharedState {
            options,
            buffer: VecDeque::new(),
        }));

        // let worker_shared_state = Arc::clone(&shared_state);

        // Spawn a worker thread to handle logging
        /*let worker_thread = thread::spawn(move || {
            //println!("Worker thread starting..."); // Debug print
            Self::worker_loop(receiver, worker_shared_state);
            //println!("Worker thread finished."); // Debug print
        });*/

        let transports = Arc::new(Mutex::new(Vec::new()));

        // let worker_handle = Some(Self::start_worker_thread(receiver, Arc::clone(&transports)));

        let mut logger = Logger {
            // worker_thread: Some(worker_thread),
            //sender,
            shared_state,
            sender: Some(sender),
            transports,
            worker_handle: None,
        };

        let worker_handle = logger.start_worker_thread(receiver);
        logger.worker_handle = Some(worker_handle);
        logger
    }

    pub fn add_transport(&mut self, transport: Transport) {
        let mut transports = self.transports.lock().unwrap();
        transports.push(transport);
    }

    fn start_worker_thread(&self, receiver: Receiver<LogInfo>) -> JoinHandle<()> {
        let shared_state = Arc::clone(&self.shared_state);
        let transports = Arc::clone(&self.transports);

        let mut pool = Pool::new(1);

        // Spawn a background thread to listen for log messages
        thread::spawn(move || {
            for log_info in receiver {
                let state = shared_state.read();
                if !Self::is_level_enabled(&log_info.level, &state.options) {
                    continue;
                }

                let transports = transports.lock().unwrap();

                // Use the thread pool to execute the logging in parallel
                pool.scoped(|scoped| {
                    for transport in transports.iter() {
                        let entry_clone = log_info.clone();
                        scoped.execute(move || {
                            let formatted_message = Self::format_message(
                                &entry_clone,
                                transport.get_format(),
                                &logform::json(),
                            );
                            if let Some(formatted_message) = formatted_message {
                                transport.log(formatted_message.message); // Write logs in parallel
                            }
                        });
                    }
                });
            }
        })
    }

    fn format_message(
        entry: &LogInfo,
        transport_format: Option<&Format>,
        default_format: &Format,
    ) -> Option<LogInfo> {
        let format_to_use = transport_format.unwrap_or(default_format);
        format_to_use.transform(entry.clone(), None)
    }

    /*fn worker_loop(receiver: Receiver<LogMessage>, shared_state: Arc<RwLock<SharedState>>) {
            for message in receiver {
                match message {
                    LogMessage::Entry(entry) => {
                        let mut state = shared_state.write();
                        if state.options.transports.is_empty() {
                            state.buffer.push_back(entry.clone());
                            eprintln!("[winston] Attempt to write logs with no transports, which can increase memory usage: {}", entry.message);
                        } else {
                            Self::process_entry(&entry, &state.options)
                        }
                    }
                    LogMessage::Configure(new_options) => {
                        let mut state = shared_state.write();
                        Self::reconfigure(new_options, &mut state);
                        Self::process_buffered_entries(&mut state);
                    }
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
            let format = options.get_format().cloned().unwrap_or_else(|| json());

            if !Self::is_level_enabled(&entry.level, options) {
                return;
            }

            //TODO: remove this check, it isn't consistent with winstonjs, but may ensure consistent message structure and prevent unnecessary writes
            if entry.message.is_empty() && entry.meta.is_empty() {
                return;
            }

            for transport in &options.transports {
                if let Some(formatted_message) =
                    Self::format_message(entry, transport.get_format(), &format)
                {
                    transport.log(formatted_message.message);
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
            if !options.transports.is_empty() {
                // Return true if any transport's level is prioritized and matches the severity
                return options.transports.iter().any(|transport| {
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
        }

        /*pub fn query(&self, options: &LogQuery) -> Result<Vec<LogInfo>, String> {
                let state = self.shared_state.read();
                let mut results = Vec::new();

                // First, query the buffered entries
                for entry in &state.buffer {
                    if options.matches(entry) {
                        results.push(entry.clone());
                    }
                }

                // Then, query each transport
                for transport in state.options.transports {
                    if let Some(queryable_transport) = transport.as_queryable() {
                        match queryable_transport.query(options) {
                            Ok(mut logs) => results.append(&mut logs),
                            Err(e) => return Err(format!("Query failed: {}", e)),
                        }
                    }
                }

                Ok(results)
            }
        */

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
            println!("shutting down global logger");
            // Call close method which will send shutdown signal and join the worker thread
            // let mut logger = DEFAULT_LOGGER.lock().unwrap();
            //logger.close();
            let mut logger = DEFAULT_LOGGER.write();
            logger.close();
        }

        pub fn builder() -> LoggerBuilder {
            LoggerBuilder::new()
        }

        fn reconfigure(new_options: Option<LoggerOptions>, state: &mut SharedState) {
            state.options = new_options.unwrap_or_else(|| LoggerOptions::default());
        }
    */

    pub fn log(&self, entry: LogInfo) {
        if let Some(ref sender) = self.sender {
            let _ = sender.send(entry);
        }
    }

    fn is_level_enabled(entry_level: &str, options: &LoggerOptions) -> bool {
        let levels = options.levels.clone().unwrap_or_default();
        let global_level = options.level.as_deref().unwrap_or("info");

        let entry_level_value = match levels.get_severity(entry_level) {
            Some(value) => value,
            None => return false,
        };

        let global_level_value = match levels.get_severity(global_level) {
            Some(value) => value,
            None => return false,
        };

        // If no transports are defined, fall back to the global level comparison
        /*if !options.transports.is_empty() {
            // Return true if any transport's level is prioritized and matches the severity
            return options.transports.iter().any(|transport| {
                match transport
                    .get_level()
                    .and_then(|level| levels.get_severity(level))
                {
                    Some(transport_level_value) => transport_level_value >= entry_level_value,
                    None => global_level_value >= entry_level_value,
                }
            });
        }*/

        // Fallback to global level check if no transports
        global_level_value >= entry_level_value
    }

    pub fn close(&mut self) {
        // Take the sender out, which closes the channel
        self.sender.take(); // Dropping the sender closes the channel

        // Wait for the worker thread to finish processing
        if let Some(handle) = self.worker_handle.take() {
            if let Err(e) = handle.join() {
                eprintln!("Error joining worker thread: {:?}", e);
            }
        }
    }

    /*pub fn configure(&self, new_options: Option<LoggerOptions>) {
        let _ = self.sender.try_send(LogMessage::Configure(new_options));
    }*/

    /*pub fn default(
    ) -> parking_lot::lock_api::RwLockReadGuard<'static, parking_lot::RawRwLock, Logger> {
        DEFAULT_LOGGER.read()
    }*/
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.close();
    }
}

/*impl Drop for Logger {
    fn drop(&mut self) {
        //println!("Dropping Logger!"); // Debug print
        self.close();
        //println!("Logger dropped"); // Debug print
    }
}*/

macro_rules! create_log_methods {
    ($($level:ident),*) => {
        impl Logger {
            $(
                pub fn $level(&self, message: &str) {
                    let log_entry = LogInfo::new(stringify!($level), message);
                    self.log(log_entry);
                   // self.log(message.to_string());
                }
            )*
        }
    };
}

create_log_methods!(info, warn, error, debug, trace);
