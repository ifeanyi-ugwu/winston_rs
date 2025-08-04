mod global;
mod log_macros;
mod logger;
mod logger_builder;
mod logger_levels;
mod logger_options;
pub mod transports;

pub use global::{add_transport, close, configure, flush, log, query, remove_transport};
pub use logform as format;
pub use logger::Logger;
pub use logger_options::{BackpressureStrategy, LoggerOptions};
pub use winston_transport::LogQuery;
