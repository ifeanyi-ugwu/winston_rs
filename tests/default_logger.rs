mod common;

use std::sync::Arc;

use winston::{
    configure, format, log_error, log_info, log_warn, transports::Console, Logger, LoggerOptions,
};

#[test]
fn test_default_logger() {
    log_info!("This use the default configuration");

    configure(
        LoggerOptions::new()
            .level("debug")
            .transports(vec![Console::new(None)])
            .format(format::combine(vec![format::timestamp(), format::json()])),
    );

    log_info!("This will use the new configuration");
}

#[test]
fn test_default_logger_macros() {
    log_info!("This is an info message");
    log_warn!("This is a warning");

    let error = "an error";
    log_error!("This is an error: {}", error);
}

#[test]
fn test_configure_on_custom_logger() {
    let mut logger = Logger::new(None);

    logger.info("This is a message from the custom logger");

    logger.configure(LoggerOptions {
        level: Some("debug".to_string()),
        transports: Some(vec![Arc::new(Console::new(None))]),
        format: Some(format::simple()),
        ..Default::default()
    });

    logger.info("This is a message from the custom logger");
}

use std::{
    thread,
    time::{Duration, Instant},
};

use crossbeam_channel::unbounded;

#[test]
fn test_logger_non_blocking() {
    // Create a channel to signal when the delayed transport has finished
    let (done_sender, done_receiver) = unbounded();

    let logger = Logger::builder()
        .add_transport(Console::new(None))
        .add_transport(common::DelayedTransport::new(Duration::from_millis(500), done_sender.clone())).format(format::pretty_print().with_option("colorize", "true"))
        .build();

    // Measure time taken for logging
    let start_time = Instant::now();

    // Log multiple messages
    for i in 0..10 {
        logger.info(&format!("Test message {}", i));
    }

    // Simulate a non-blocking task with a shorter duration
    let simulated_work_duration = Duration::from_millis(100);
    thread::sleep(simulated_work_duration); // Simulate some work

    let elapsed = start_time.elapsed();

    // Tolerance for expected execution time (adds some margin for variance in execution)
    let tolerance = Duration::from_millis(50);

    println!("Expected elapsed time: {:?}, Actual Elapsed time: {:?}",simulated_work_duration+tolerance, elapsed);

    // Assert that the elapsed time is within the expected range
    assert!(
        elapsed <= simulated_work_duration + tolerance,
        "Logging operation seems to block the caller thread! Expected elapsed time: {:?}, but got: {:?}",
        simulated_work_duration, 
        elapsed
    );

        // Wait for the delayed transport to finish
    done_receiver.recv().expect("Failed to receive completion signal");

    println!("Logging completed asynchronously after {:?}", start_time.elapsed());
}
