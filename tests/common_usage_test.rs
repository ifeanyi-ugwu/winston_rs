use logform::{colorize_builder, combine, json, simple, timestamp};
use winston::{create_logger, transports, Logger};

#[test]
fn test_default_logger() {
    let default_logger = create_logger(None);
    default_logger.info("Testing default logger");
    // Add assertions or checks if needed
}

#[test]
fn test_custom_logger() {
    let file_transport = transports::File::builder().filename("test_log.txt").build();

    let custom_logger = Logger::builder()
        .level("debug")
        .format(combine(vec![
            colorize_builder().add_color("info", "red").build(),
            simple(),
        ]))
        .add_transport(transports::Console::new(None))
        .add_transport(file_transport)
        .build();

    custom_logger.info("Testing custom logger");
    // Add assertions or checks if needed
}

#[test]
fn test_logger_with_only_file_transport() {
    let file_transport = transports::File::builder()
        .filename("test_log.txt")
        .level("info")
        .build();

    let logger_with_only_file_transport = Logger::builder()
        .level("info")
        .add_transport(file_transport)
        .format(combine(vec![timestamp(None), json()]))
        .build();

    logger_with_only_file_transport.info("Testing logger with only file transport")
    // Add assertions or checks if needed
}
