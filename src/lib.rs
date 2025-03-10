mod global;
mod log_macros;
mod logger;
mod logger_builder;
mod logger_levels;
mod logger_options;
mod rebuilder;
pub mod transports;

pub use global::{close, configure, flush, log, rebuilder};
pub use logform as format;
pub use logger::Logger;
pub use logger_options::{BackpressureStrategy, LoggerOptions};
pub use winston_transport::LogQuery;
//pub use logger::*;
