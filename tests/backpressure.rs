mod common;

use logform::LogInfo;
//use parking_lot::Mutex;
//use std::sync::Arc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use std::time::Instant;
use winston::*;
use winston_transport::Transport;

// A simple mock transport to capture log messages
#[derive(Debug, Default)]
struct MockTransport {
    pub logs: Arc<Mutex<Vec<String>>>,
    pub delay: Duration,
}

impl MockTransport {
    pub fn new(delay: Duration) -> Self {
        MockTransport {
            logs: Arc::new(Mutex::new(Vec::new())),
            delay,
        }
    }
}

impl Transport for MockTransport {
    fn log(&self, log: LogInfo) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(log.message.clone());

        thread::sleep(self.delay);
    }
}

fn get_message_from_json(json_str: &str) -> String {
    let v: serde_json::Value = serde_json::from_str(json_str).unwrap();
    v["message"].as_str().unwrap().to_string()
}

#[test]
fn test_backpressure_block_strategy() {
    let mock_transport = common::DelayedTransport::new(std::time::Duration::from_secs(1)); //MockTransport::default();
    let logger = Logger::builder()
        .channel_capacity(1)
        .backpressure_strategy(BackpressureStrategy::Block)
        .add_transport(mock_transport)
        .build();

    let logger_thread = thread::spawn(move || {
        // Measure time for the first log (shouldn't block)
        let start_time_1 = Instant::now();
        logger.log(LogInfo::new("info", "Message 1"));
        let duration_1 = start_time_1.elapsed();

        println!("duration 1 {:?}", duration_1);

        // Measure time for the second log (shouldn't block)
        let start_time_2 = Instant::now();
        logger.log(LogInfo::new("info", "Message 2"));
        let duration_2 = start_time_2.elapsed();
        println!("duration 2 {:?}", duration_2);

        // Measure time for the third log (this should block)
        let start_time_3 = Instant::now();
        logger.log(LogInfo::new("info", "Message 3"));
        let duration_3 = start_time_3.elapsed();
        println!("duration 3 {:?}", duration_3);

        // Return the log durations to the main test thread
        (duration_1, duration_2, duration_3)
    });

    // Join the logger thread to get the result
    let (duration_1, duration_2, duration_3) = logger_thread.join().unwrap();

    assert!(
        duration_3 > duration_1 && duration_3 > duration_2,
        "Logger should have blocked on the third message, but it didn't."
    );
}

#[test]
fn test_backpressure_drop_oldest_strategy() {
    let mock_transport = MockTransport::new(Duration::from_millis(50));
    let logs = Arc::clone(&mock_transport.logs);

    let logger = Logger::builder()
        .channel_capacity(2) // Capacity of 2 to easily test backpressure
        .backpressure_strategy(BackpressureStrategy::DropOldest)
        .add_transport(mock_transport)
        .build();

    for i in 1..=3 {
        logger.log(LogInfo::new("info", &format!("Message {}", i)));
    }

    // Allow time for processing
    thread::sleep(Duration::from_millis(150));

    let logs = logs.lock().unwrap();
    assert_eq!(
        logs.len(),
        2,
        "Only the two most recent messages should be logged"
    );
    assert_eq!(get_message_from_json(&logs[0]), "Message 2");
    assert_eq!(get_message_from_json(&logs[1]), "Message 3");
}

#[test]
fn test_backpressure_drop_current_strategy() {
    let mock_transport = MockTransport::new(Duration::from_millis(50));
    let logs = Arc::clone(&mock_transport.logs);

    let logger = Logger::builder()
        .channel_capacity(2)
        .backpressure_strategy(BackpressureStrategy::DropCurrent)
        .add_transport(mock_transport)
        .build();

    for i in 1..=3 {
        logger.log(LogInfo::new("info", &format!("Message {}", i)));
    }

    // Allow time for processing
    thread::sleep(Duration::from_millis(150));

    let logs = logs.lock().unwrap();
    assert_eq!(
        logs.len(),
        2,
        "Only the first two messages should be logged"
    );
    assert_eq!(get_message_from_json(&logs[0]), "Message 1");
    assert_eq!(get_message_from_json(&logs[1]), "Message 2");
}
