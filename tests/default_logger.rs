mod common;

use common::DelayedTransport;
use std::time::{Duration, Instant};
use winston::{
    configure,
    format::{self, LogInfo},
    log_error, log_info, log_warn,
    transports::Console,
    Logger, LoggerOptions,
};

#[test]
fn test_default_logger() {
    log_info!("This use the default configuration");
    log_info!("This is the second that uses the default configuration to check queue order");

    configure(Some(
        LoggerOptions::new()
            .level("debug")
            .add_transport(Console::new(None))
            .format(format::combine(vec![format::timestamp(), format::json()])),
    ));

    log_info!("This will use the new configuration");
    Logger::shutdown();
}

#[test]
fn test_default_logger_macros() {
    log_info!("This is an info message");
    log_warn!("This is a warning");

    let error = "an error";
    log_error!("This is an error: {}", error);
    Logger::shutdown();
}

#[test]
fn test_configure_on_custom_logger() {
    let logger = Logger::new(None);

    logger.info("This is a message from the custom logger");

    logger.configure(Some(
        LoggerOptions::new()
            .add_transport(Console::new(None))
            .format(format::simple())
            .level("debug"),
    ));

    logger.info("This is a message from the custom logger");
}

#[test]
fn test_logger_non_blocking() {
    const NUM_MESSAGES: usize = 100;
    const PROCESS_DELAY: Duration = Duration::from_millis(100);

    let delayed_transport = DelayedTransport::new(PROCESS_DELAY);

    let logger = Logger::builder()
        .add_transport(Console::new(None))
        .add_transport(delayed_transport)
        .format(format::pretty_print().with_option("colorize", "true"))
        .build();

    // Measure time to enqueue all messages
    let enqueue_start = Instant::now();

    for i in 0..NUM_MESSAGES {
        let log_entry = LogInfo::new("info", &format!("Test message {}", i));
        logger.log(log_entry);
    }

    let actual_enqueue_duration = enqueue_start.elapsed();

    // Calculate theoretical synchronous enqueue time
    let theoretical_sync_duration = PROCESS_DELAY * NUM_MESSAGES as u32;

    println!("Number of messages: {}", NUM_MESSAGES);
    println!("Delay per message: {:?}", PROCESS_DELAY);
    println!("Actual enqueue duration: {:?}", actual_enqueue_duration);
    println!(
        "Theoretical synchronous duration: {:?}",
        theoretical_sync_duration
    );

    // Assertions
    assert!(
        actual_enqueue_duration < theoretical_sync_duration,
        "Actual enqueue time ({:?}) should be less than theoretical synchronous time ({:?})",
        actual_enqueue_duration,
        theoretical_sync_duration
    );

    // Check that actual enqueuing was significantly faster than theoretical synchronous time
    assert!(
        actual_enqueue_duration * 5 < theoretical_sync_duration,
        "Actual enqueue time ({:?}) should be at least 5 times faster than theoretical synchronous time ({:?})",
        actual_enqueue_duration,
        theoretical_sync_duration
    );
}
