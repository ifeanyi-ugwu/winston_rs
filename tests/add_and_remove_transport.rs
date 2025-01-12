mod common;

use std::{fs, sync::Arc, thread, time::Duration};

use winston::{log, transports, Logger};
use winston_transport::Transport;

fn read_file_with_retry(filename: &str, max_retries: u32, delay_ms: u64) -> String {
    for attempt in 0..max_retries {
        match fs::read_to_string(filename) {
            Ok(content) => return content,
            Err(e) => {
                if attempt == max_retries - 1 {
                    panic!(
                        "Failed to read {} after {} attempts: {}",
                        filename, max_retries, e
                    );
                }
                thread::sleep(Duration::from_millis(delay_ms));
            }
        }
    }
    panic!("Failed to read {} after {} attempts", filename, max_retries);
}

#[test]
fn test_add_and_remove_transport() {
    // Create two file transports
    let file_transport1: Arc<dyn Transport + Send + Sync> = Arc::new(
        transports::File::builder()
            .filename("test_log1.log")
            .level("info")
            .build(),
    );

    let file_transport2: Arc<dyn Transport + Send + Sync> = Arc::new(
        transports::File::builder()
            .filename("test_log2.log")
            .level("info")
            .build(),
    );

    // Create a logger
    let logger = Logger::builder().level("info").build();

    // Add transports to the logger
    assert!(logger.add_transport(Arc::clone(&file_transport1)));
    assert!(logger.add_transport(Arc::clone(&file_transport2)));

    // Log a message with both transports
    log!(logger, info, "Message before removing transport");

    // Flush the logger to wait until it writes the messages to the files
    logger.flush().unwrap();

    // Add a small delay to ensure file operations are complete
    //thread::sleep(Duration::from_millis(50));

    // Verify the content of both log files with retry mechanism
    let log1_content = read_file_with_retry("test_log1.log", 5, 100);
    assert!(
        log1_content.contains("Message before removing transport"),
        "First message not found in test_log1.log. Content: {}",
        log1_content
    );

    let log2_content = read_file_with_retry("test_log2.log", 5, 100);
    assert!(
        log2_content.contains("Message before removing transport"),
        "First message not found in test_log2.log. Content: {}",
        log2_content
    );

    // Remove the first transport
    assert!(logger.remove_transport(Arc::clone(&file_transport1)));

    // Verify that trying to remove the same transport again returns false
    assert!(!logger.remove_transport(Arc::clone(&file_transport1)));

    // Log another message - should only go to second transport
    log!(logger, info, "Message after removing transport");

    // Flush the logger again
    logger.flush().unwrap();

    // Add a small delay to ensure file operations are complete
    thread::sleep(Duration::from_millis(50));

    // Verify the files with retry mechanism
    let log1_content = read_file_with_retry("test_log1.log", 5, 100);
    assert!(
        !log1_content.contains("Message after removing transport"),
        "Second message found in test_log1.log when it shouldn't be. Content: {}",
        log1_content
    );

    let log2_content = read_file_with_retry("test_log2.log", 5, 100);
    assert!(
        log2_content.contains("Message after removing transport"),
        "Second message not found in test_log2.log. Content: {}",
        log2_content
    );

    // Clean up test files
    common::delete_file_if_exists("test_log1.log");
    common::delete_file_if_exists("test_log2.log");
}
