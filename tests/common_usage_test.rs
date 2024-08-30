use logform::{align, colorize, combine, json, simple, timestamp, Format};
use winston::{create_logger, transports, LogEntry, Logger};

#[test]
fn test_default_logger() {
    let default_logger = create_logger(None);
    default_logger.info("Testing default logger");
    // Add assertions or checks if needed

    let custom_logger = Logger::builder().format(Format::new(|_, _| None)).build();
    custom_logger.info("hi there")
}

#[test]
fn test_custom_logger() {
    let file_transport = transports::File::builder().filename("test_log.log").build();

    let custom_logger = Logger::builder()
        .level("debug")
        .format(combine(vec![
            colorize()
                .with_option(
                    "colors",
                    &serde_json::json!({"info": ["green"],"error":["red"]}).to_string(),
                )
                .with_option("all", "true"),
            align(),
            simple(),
        ]))
        .add_transport(transports::Console::new(None))
        .add_transport(file_transport)
        .build();

    custom_logger.info("Testing custom logger");
    let info = LogEntry::builder("info", "")
        .option("justaword", serde_json::json!("er"))
        .option("justAnObj", serde_json::json!({}))
        .build();
    custom_logger.log(info);
    let info = LogEntry::builder("info", "hi")
        .option("meta", serde_json::json!("s"))
        .build();
    custom_logger.log(info);
    custom_logger.error("nope");
    custom_logger.info("")
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
        .format(combine(vec![timestamp(), json()]))
        .build();

    logger_with_only_file_transport.info("Testing logger with only file transport")
    // Add assertions or checks if needed
}
