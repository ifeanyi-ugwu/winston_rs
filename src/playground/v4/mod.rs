use async_trait::async_trait;
use chrono::Utc;
use futures::channel::mpsc::{channel, Receiver, Sender};
use futures::StreamExt;
use std::collections::HashMap;
use std::fmt;
use std::io::{BufWriter, Write};
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::runtime::Runtime;

// LogInfo struct to represent log entries
#[derive(Clone, Debug)]
pub struct LogInfo {
    pub level: String,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub metadata: HashMap<String, String>,
}

impl LogInfo {
    pub fn new(level: &str, message: &str) -> Self {
        LogInfo {
            level: level.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        }
    }
}

// LoggerOptions struct for configuration
#[derive(Clone)]
pub struct LoggerOptions {
    pub level: String,
    pub levels: HashMap<String, usize>,
}

impl Default for LoggerOptions {
    fn default() -> Self {
        let mut levels = HashMap::new();
        levels.insert("error".to_string(), 0);
        levels.insert("warn".to_string(), 1);
        levels.insert("info".to_string(), 2);
        levels.insert("debug".to_string(), 3);
        levels.insert("trace".to_string(), 4);

        LoggerOptions {
            level: "info".to_string(),
            levels,
        }
    }
}

// Transport trait for log transports
#[async_trait]
pub trait Transport: Send + Sync {
    fn name(&self) -> &str;
    async fn log(&self, info: &LogInfo) -> Result<(), Box<dyn std::error::Error>>;
}

// Logger struct
pub struct Logger {
    options: LoggerOptions,
    sender: Option<Sender<LogInfo>>,
    transports: Arc<Mutex<Vec<Arc<dyn Transport>>>>,
    handle: Option<thread::JoinHandle<()>>, // Background worker thread handle
}

impl Logger {
    pub fn new(options: LoggerOptions) -> Self {
        let (sender, receiver) = channel(100);
        let transports = Arc::new(Mutex::new(Vec::new()));

        // Clone the state to move into the background thread
        let transports_clone = transports.clone();

        // Spawn a dedicated thread for logging
        let handle = thread::spawn(move || {
            let runtime = Runtime::new().unwrap();
            runtime.block_on(async move {
                Logger::process_logs(receiver, transports_clone).await;
            });
        });

        Logger {
            options,
            sender: Some(sender),
            transports,
            handle: Some(handle),
        }
    }

    pub fn add_transport(&self, transport: Arc<dyn Transport>) {
        let mut transports = self.transports.lock().unwrap();
        transports.push(transport);
    }

    pub fn log(&self, level: &str, message: &str) {
        if let Some(sender) = &self.sender {
            let log_info = LogInfo::new(level, message);
            let _ = sender.clone().try_send(log_info);
        }
    }

    pub fn shutdown(&mut self) {
        // Drop the sender to signal the receiver to stop
        self.sender = None;
        if let Some(handle) = self.handle.take() {
            handle.join().expect("Logger thread panicked");
        }
    }

    async fn process_logs(
        mut receiver: Receiver<LogInfo>,
        transports: Arc<Mutex<Vec<Arc<dyn Transport>>>>,
    ) {
        while let Some(log_info) = receiver.next().await {
            let transports = transports.lock().unwrap();
            for transport in transports.iter() {
                let _ = transport.log(&log_info).await;
            }
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        self.shutdown(); // Ensure all logs are flushed before dropping the logger
    }
}

// Example console transport
pub struct ConsoleTransport;

#[async_trait]
impl Transport for ConsoleTransport {
    fn name(&self) -> &str {
        "console"
    }

    async fn log(&self, info: &LogInfo) -> Result<(), Box<dyn std::error::Error>> {
        println!("[{}] {}: {}", info.timestamp, info.level, info.message);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    #[test]
    fn test_logger() {
        let mut logger = Logger::new(LoggerOptions::default());

        // Add a console transport
        logger.add_transport(Arc::new(ConsoleTransport));

        // Log messages
        logger.log("info", "This is an info message");
        logger.log("error", "This is an error message");
        logger.log("debug", "This is a debug message");

        // Shutdown the logger to flush logs
        // logger.shutdown(); // Ensure this is called to cleanly exit
    }
}
