use logform::LogInfo;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use winston::{
    create_level_macros, create_log_methods, log, meta, LogQuery, Logger, LoggerOptions,
};
use winston_transport::Transport;

#[derive(Clone)]
struct MockTransport {
    logged_messages: Arc<Mutex<Vec<LogInfo>>>,
}

impl MockTransport {
    fn new() -> Self {
        MockTransport {
            logged_messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_logged_messages(&self) -> Vec<LogInfo> {
        self.logged_messages.lock().unwrap().clone()
    }
}

impl Transport for MockTransport {
    fn log(&self, info: LogInfo) {
        self.logged_messages.lock().unwrap().push(info);
    }

    fn query(&self, _options: &LogQuery) -> Result<Vec<LogInfo>, String> {
        Ok(self.get_logged_messages())
    }
}

fn setup_logger() -> (Logger, Arc<Mutex<Vec<LogInfo>>>) {
    let mock_transport = Arc::new(MockTransport::new());
    let logged_messages = mock_transport.logged_messages.clone();
    let logger = Logger::builder()
        .transports(vec![mock_transport])
        .level("trace")
        .build();
    (logger, logged_messages)
}

#[test]
fn test_log_macro_no_logger() {
    /*winston::init(
        Logger::builder()
            .add_transport(MockTransport::new())
            .build(),
    );*/
    winston::configure(Some(
        LoggerOptions::new().add_transport(MockTransport::new()),
    ));
    log!(info, "Simple message");
    let _ = winston::flush();
    let query = LogQuery::new();
    let logs = winston::query(&query).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].level, "info");
    //assert_eq!(logs[0].message, "Simple message");
    assert!(logs[0].meta.is_empty());
    winston::close();
}

#[test]
fn test_log_macro_no_logger_with_metadata() {
    /*winston::init(
        Logger::builder()
            .add_transport(MockTransport::new())
            .build(),
    );*/
    winston::configure(Some(
        LoggerOptions::new().add_transport(MockTransport::new()),
    ));
    log!(warn, "Message with metadata", key1 = "value1", key2 = 123);
    let _ = winston::flush();
    let query = LogQuery::new();
    let logs = winston::query(&query).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].level, "warn");
    //assert_eq!(logs[0].message, "Message with metadata");
    assert_eq!(
        logs[0].meta.get("key1").and_then(Value::as_str),
        Some("value1")
    );
    assert_eq!(logs[0].meta.get("key2").and_then(Value::as_i64), Some(123));
}

#[test]
fn test_log_macro_with_logger() {
    let (logger, _) = setup_logger();
    log!(logger, error, "Message to specific logger");
    logger.flush().unwrap();
    let query = LogQuery::new();
    let logs = logger.query(&query).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].level, "error");
    //assert_eq!(logs[0].message, "Message to specific logger");
    assert!(logs[0].meta.is_empty());
}

#[test]
fn test_log_macro_with_logger_and_metadata() {
    let (logger, _) = setup_logger();
    log!(logger, debug, "Detailed info", detail = "extra", count = 42);
    let _ = logger.flush();
    let query = LogQuery::new();
    let logs = logger.query(&query).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].level, "debug");
    //assert_eq!(logs[0].message, "Detailed info");
    assert_eq!(
        logs[0].meta.get("detail").and_then(Value::as_str),
        Some("extra")
    );
    assert_eq!(logs[0].meta.get("count").and_then(Value::as_i64), Some(42));
}

#[test]
fn test_log_macro_with_logger_and_meta_macro() {
    let (logger, _) = setup_logger();
    log!(
        logger,
        trace,
        "Using meta! macro",
        meta!(user_id = 101, session_id = "abc-123")
    );
    logger.flush().unwrap();
    let query = LogQuery::new();
    let logs = logger.query(&query).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].level, "trace");
    //assert_eq!(logs[0].message, "Using meta! macro");
    assert_eq!(
        logs[0].meta.get("user_id").and_then(Value::as_i64),
        Some(101)
    );
    assert_eq!(
        logs[0].meta.get("session_id").and_then(Value::as_str),
        Some("abc-123")
    );
}

#[test]
fn test_log_macro_no_logger_with_meta_macro() {
    /*winston::init(
        Logger::builder()
            .add_transport(MockTransport::new())
            .build(),
    );*/
    winston::configure(Some(
        LoggerOptions::new().add_transport(MockTransport::new()),
    ));
    log!(
        info,
        "Using meta! without logger",
        meta!(status = "ok", duration = 150)
    );
    let _ = winston::flush();
    let query = LogQuery::new();
    let logs = winston::query(&query).unwrap();
    assert_eq!(logs.len(), 1);
    assert_eq!(logs[0].level, "info");
    //assert_eq!(logs[0].message, "Using meta! without logger");
    assert_eq!(
        logs[0].meta.get("status").and_then(Value::as_str),
        Some("ok")
    );
    assert_eq!(
        logs[0].meta.get("duration").and_then(Value::as_i64),
        Some(150)
    );
}

#[test]
fn test_meta_macro() {
    let metadata = meta!(id = 5, name = "test", active = true);
    assert_eq!(metadata.len(), 3);
    assert_eq!(metadata[0].0, "id");
    assert_eq!(metadata[0].1.as_i64(), Some(5));
    assert_eq!(metadata[1].0, "name");
    assert_eq!(metadata[1].1.as_str(), Some("test"));
    assert_eq!(metadata[2].0, "active");
    assert_eq!(metadata[2].1.as_bool(), Some(true));
}

#[test]
fn test_create_log_methods_trait_and_impl() {
    let (logger, _) = setup_logger();
    create_log_methods!(info, warn, error);
    logger.info("Informational message", None);
    logger.warn("Warning!", Some(vec![("type", "security".into())]));
    logger.error("Something went wrong", None);

    logger.flush().unwrap();
    let query = LogQuery::new();
    let logs = logger.query(&query).unwrap();
    assert_eq!(logs.len(), 3);

    assert_eq!(logs[0].level, "info");
    //assert_eq!(logs[0].message, "Informational message");
    assert!(logs[0].meta.is_empty());

    assert_eq!(logs[1].level, "warn");
    //assert_eq!(logs[1].message, "Warning!");
    assert_eq!(
        logs[1].meta.get("type").and_then(Value::as_str),
        Some("security")
    );

    assert_eq!(logs[2].level, "error");
    //assert_eq!(logs[2].message, "Something went wrong");
    assert!(logs[2].meta.is_empty());
}

#[test]
fn test_create_level_macros_with_logger() {
    let (logger, _) = setup_logger();
    create_level_macros!(info, warn);
    info!(logger, "This is an info level message");
    warn!(logger, "A warning occurred", vec![("code", 404)]);

    logger.flush().unwrap();
    let query = LogQuery::new();
    let logs = logger.query(&query).unwrap();
    assert_eq!(logs.len(), 2);

    assert_eq!(logs[0].level, "info");
    //assert_eq!(logs[0].message, "This is an info level message");
    assert!(logs[0].meta.is_empty());

    assert_eq!(logs[1].level, "warn");
    //assert_eq!(logs[1].message, "A warning occurred");
    assert_eq!(
        logs[1].meta.get("code").and_then(Value::as_f64),
        Some(404.0)
    );
}

#[test]
fn test_create_level_macros_no_logger() {
    /*winston::init(
        Logger::builder()
            .level("debug")
            .add_transport(MockTransport::new())
            .build(),
    );*/
    winston::configure(Some(
        LoggerOptions::new()
            .level("debug")
            .add_transport(MockTransport::new()),
    ));
    create_level_macros!(debug, error);
    error!("An error has happened");
    debug!("Debugging information", vec![("module", "parser")]);

    let _ = winston::flush();
    let query = LogQuery::new();
    let logs = winston::query(&query).unwrap();
    assert_eq!(logs.len(), 2);

    assert_eq!(logs[0].level, "error");
    //assert_eq!(logs[0].message, "An error has happened");
    assert!(logs[0].meta.is_empty());

    assert_eq!(logs[1].level, "debug");
    //assert_eq!(logs[1].message, "Debugging information");
    assert_eq!(
        logs[1].meta.get("module").and_then(Value::as_str),
        Some("parser")
    );
}
