mod common;

use std::{fs, sync::Arc};

use winston::{log, transports, Logger};
use winston_transport::Transport;

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

    // Verify the content of the first log file
    let log1_content = fs::read_to_string("test_log1.log").expect("Failed to read test_log1.log");
    assert!(log1_content.contains("Message before removing transport"));

    // Verify the content of the second log file
    let log2_content = fs::read_to_string("test_log2.log").expect("Failed to read test_log2.log");
    assert!(log2_content.contains("Message before removing transport"));

    // Remove the first transport
    assert!(logger.remove_transport(Arc::clone(&file_transport1)));

    // Verify that trying to remove the same transport again returns false
    assert!(!logger.remove_transport(Arc::clone(&file_transport1)));

    // Log another message - should only go to second transport
    log!(logger, info, "Message after removing transport");

    // Flush the logger again
    logger.flush().unwrap();

    // Verify that the first log file did not record the second message
    let log1_content = fs::read_to_string("test_log1.log").expect("Failed to read test_log1.log");
    assert!(!log1_content.contains("Message after removing transport"));

    // Verify that the second log file recorded the second message
    let log2_content = fs::read_to_string("test_log2.log").expect("Failed to read test_log2.log");
    assert!(log2_content.contains("Message after removing transport"));

    // Clean up test files
    common::delete_file_if_exists("test_log1.log");
    common::delete_file_if_exists("test_log2.log");
}
