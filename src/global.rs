use crate::Logger;
use std::sync::{Arc, OnceLock};
use winston_transport::Transport;

static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

fn global_logger() -> &'static Logger {
    GLOBAL_LOGGER.get_or_init(|| Logger::default())
}

pub fn log(entry: logform::LogInfo) {
    global_logger().log(entry);
}

pub fn configure(new_options: Option<crate::LoggerOptions>) {
    global_logger().configure(new_options)
}

pub fn flush() -> Result<(), String> {
    global_logger().flush()
}

pub fn close() {
    global_logger().close();
}

pub fn query(options: &winston_transport::LogQuery) -> Result<Vec<logform::LogInfo>, String> {
    global_logger().query(options)
}

pub fn add_transport(transport: Arc<dyn Transport>) -> bool {
    global_logger().add_transport(transport)
}

pub fn remove_transport(transport: Arc<dyn Transport>) -> bool {
    global_logger().remove_transport(transport)
}
