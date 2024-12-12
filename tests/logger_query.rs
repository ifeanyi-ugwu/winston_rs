mod common;

use winston::{format, log, transports, LogQuery, Logger};

#[test]
fn test_logging_and_querying() {
    let temp_path = common::generate_random_filename();
    // Setup logger with file transport
    let logger = Logger::builder()
        .add_transport(
            transports::File::builder()
                .filename(temp_path.clone())
                .build(),
        )
        .format(format::combine(vec![format::timestamp(), format::json()]))
        .build();

    // Log some messages
    log!(logger, info, "Test message 1");
    log!(logger, error, "Test error message");
    log!(logger, warn, "Test warning");

    // Sleep for a short duration to ensure logs are flushed to the file and the query will retrieve them
    std::thread::sleep(std::time::Duration::from_secs(1));

    // Define query to retrieve logs
    let query = LogQuery::new()
        //.order("desc")
        //.from("2 hours ago")
        //.until("now")
        .levels(vec!["error"]);
    //.limit(10)
    //.search_term("t")
    //.fields(vec!["message", "level"]);

    // Execute the query
    let results = logger.query(&query);
    assert!(results.is_ok());

    let logs = results.unwrap();

    assert!(!logs.is_empty(), "No logs were found. Logs: {:?}", logs);
    for (index, log) in logs.iter().enumerate() {
        println!("{:?}", log);
        assert_eq!(
            log.level, "error",
            "Expected error level at index {}",
            index
        );
        assert_eq!(
            log.message, "Test error message",
            "Expected test error message at index {}",
            index
        );
    }

    // Cleanup: remove the temporary file
    common::delete_file_if_exists(&temp_path)
}
