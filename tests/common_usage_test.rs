mod common;

use logform::chain;
use winston::{
    format::{align, colorize, json, simple, timestamp, Format, LogInfo},
    log, transports, Logger,
};

#[test]
fn test_default_logger() {
    let default_logger = Logger::default();
    log!(default_logger, info, "Testing default logger");
    // Add assertions or checks if needed

    struct NoOpFormat;
    impl Format for NoOpFormat {
        type Input = LogInfo;

        fn transform(&self, _input: Self::Input) -> Option<Self::Input> {
            None
        }
    }

    let custom_logger = Logger::builder().format(NoOpFormat).build();
    log!(custom_logger, info, "hi there")
}

#[test]
fn test_custom_logger() {
    let log_file_path = "test_log.log";
    let file_transport = transports::File::builder().filename(log_file_path).build();

    let custom_logger = Logger::builder()
        .level("debug")
        .format(chain!(
            colorize()
                .with_colors(vec![
                    ("info".to_string(), serde_json::json!(["green"])),
                    ("error".to_string(), serde_json::json!(["red"]))
                ])
                .with_all(true),
            align(),
            simple(),
        ))
        .add_transport(transports::stdout())
        .add_transport(file_transport)
        .build();

    log!(custom_logger, info, "Testing custom logger");
    let info = LogInfo::new("info", "")
        .with_meta("justaword", serde_json::json!("er"))
        .with_meta("justAnObj", serde_json::json!({}));
    custom_logger.log(info);
    let info = LogInfo::new("info", "hi").with_meta("meta", serde_json::json!("s"));
    custom_logger.log(info);
    log!(custom_logger, error, "nope");
    log!(custom_logger, info, "");
    // Add assertions or checks if needed

    // Clean up after test
    common::delete_file_if_exists(log_file_path);
}

#[test]
fn test_logger_with_only_file_transport() {
    let file_transport = transports::File::builder()
        .filename("test_log.log")
        .level("info")
        .build();

    let logger_with_only_file_transport = Logger::builder()
        .level("info")
        .add_transport(file_transport)
        .format(chain!(timestamp(), json()))
        .build();

    log!(
        logger_with_only_file_transport,
        info,
        "Testing logger with only file transport"
    )
    // Add assertions or checks if needed
}
