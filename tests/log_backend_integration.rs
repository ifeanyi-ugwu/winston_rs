#![cfg(all(test, feature = "log-backend"))]

mod common;
use common::MemoryTransport;

use std::sync::Arc;
use winston::{Logger, LoggerOptions};

#[test]
fn test_log_backend_integration() {
    let memory_transport = Arc::new(MemoryTransport::new());

    let options = LoggerOptions::default();
    let logger = Logger::new(Some(options));
    logger.add_transport(memory_transport.clone());
    logger
        .init_as_global()
        .expect("Failed to init global logger");

    // Send some logs
    log::info!("Hello from integration test");
    log::warn!("Something might be wrong");
    log::error!("Something went wrong!");

    // Flush to make sure everything is processed
    log::logger().flush();

    // Assert logs were captured
    let logs = memory_transport.logs.lock().unwrap();
    assert!(logs
        .iter()
        .any(|log| log.message.contains("Hello from integration test")));
    assert!(logs
        .iter()
        .any(|log| log.message.contains("Something might be wrong")));
    assert!(logs
        .iter()
        .any(|log| log.message.contains("Something went wrong!")));
}
