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
