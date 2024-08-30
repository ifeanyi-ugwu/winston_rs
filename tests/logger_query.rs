use chrono::{TimeZone, Utc};
use winston::{format, transports, LogQuery, Logger, Order};

#[test]
fn test_logging_and_querying() {
    // Setup logger with file transport
    let logger = Logger::builder()
        .add_transport(transports::File::builder().filename("test_log.log").build())
        .format(format::combine(vec![format::timestamp(), format::json()]))
        .build();

    // Log some messages
    logger.info("Test message 1");
    logger.error("Test error message");
    logger.warn("Test warning");

    // Define query to retrieve logs
    let query = LogQuery::new()
        .order(Order::Descending)
        .from(Utc.with_ymd_and_hms(2024, 8, 28, 0, 0, 0).unwrap())
        .until(Utc.with_ymd_and_hms(2024, 8, 29, 23, 59, 59).unwrap())
        .levels(vec!["error"])
        .limit(10)
        .search_term("t")
        .fields(vec!["message"]);

    // Execute the query
    let results = logger.query(&query);
    assert!(results.is_ok());

    let logs = results.unwrap();

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
}
