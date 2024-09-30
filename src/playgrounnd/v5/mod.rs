use async_trait::async_trait;
use chrono::Utc;
use futures::channel::mpsc::{self, Receiver, Sender};
use futures::{Sink, SinkExt, Stream, StreamExt};
use std::collections::HashMap;
use std::fmt;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

// LogInfo struct representing log entries
#[derive(Clone, Debug)]
pub struct LogInfo {
    pub level: String,
    pub message: String,
    pub timestamp: chrono::DateTime<Utc>,
}

impl LogInfo {
    pub fn new(level: &str, message: &str) -> Self {
        LogInfo {
            level: level.to_string(),
            message: message.to_string(),
            timestamp: Utc::now(),
        }
    }
}

// Logger that manages log entries as a stream
pub struct Logger {
    sender: Sender<LogInfo>,
    transports: Arc<Vec<Arc<dyn LogSink + Send + Sync>>>,
}

impl Logger {
    pub fn new(buffer_size: usize) -> Self {
        let (sender, receiver) = mpsc::channel(buffer_size);
        let transports: Arc<Vec<Arc<dyn LogSink + Send + Sync>>> = Arc::new(Vec::new());

        // Spawn a background task to process log entries
        tokio::spawn({
            let transports = transports.clone();
            async move {
                let mut log_stream = LogStream { receiver };

                // Process each log entry
                while let Some(log_entry) = log_stream.next().await {
                    for transport in transports.iter() {
                        if let Err(e) = transport.log(&log_entry).await {
                            eprintln!("Failed to log: {}", e);
                        }
                    }
                }
            }
        });

        Logger { sender, transports }
    }

    // Simple user-facing log function
    pub fn log(&self, level: &str, message: &str) {
        let log_info = LogInfo::new(level, message);
        let _ = self.sender.clone().try_send(log_info);
    }

    // Add a new transport sink
    pub fn add_transport<T>(&mut self, transport: T)
    where
        T: LogSink + Send + Sync + 'static,
    {
        let mut transports = Arc::get_mut(&mut self.transports).unwrap();
        transports.push(Arc::new(transport));
    }
}

// Trait for log sinks
#[async_trait]
pub trait LogSink: Send + Sync {
    async fn log(&self, info: &LogInfo) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

// Stream for processing log entries
struct LogStream {
    receiver: Receiver<LogInfo>,
}

impl Stream for LogStream {
    type Item = LogInfo;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Pin::new(&mut self.get_mut().receiver).poll_next(cx)
    }
}

// Console transport implementing the LogSink trait
pub struct ConsoleTransport;

#[async_trait]
impl LogSink for ConsoleTransport {
    async fn log(&self, info: &LogInfo) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("[{}] {}: {}", info.timestamp, info.level, info.message);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{self, Duration};

    #[tokio::test]
    async fn test_logger() {
        let mut logger = Logger::new(10);

        // Add a console transport
        logger.add_transport(ConsoleTransport);

        // Log some messages
        logger.log("info", "This is an info message");
        logger.log("error", "This is an error message");
        logger.log("debug", "This is a debug message");

        // Wait to allow logs to be processed
        time::sleep(Duration::from_secs(1)).await;
    }
}
