mod common;

use common::MockTransport;
use logform::LogInfo;
use std::sync::Arc;
use winston::Logger;

#[test]
fn test_global_configure() {
    if !winston::is_initialized() {
        winston::init(Logger::new(None));
    }

    let transport = Arc::new(MockTransport::new());

    winston::configure(Some(
        winston::LoggerOptions::new()
            .level("error")
            .add_transport(transport.clone()),
    ));

    winston::log(LogInfo::new("info", "Filtered"));
    winston::log(LogInfo::new("error", "Passes"));
    winston::flush().unwrap();

    assert_eq!(transport.log_count(), 1);
}
