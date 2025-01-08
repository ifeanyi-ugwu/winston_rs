mod log_macros;
mod logger;
mod logger_builder;
mod logger_levels;
mod logger_options;
pub mod transports;

pub use logform as format;
pub use logger::{close, configure, flush, log, Logger};
pub use logger_options::{BackpressureStrategy, LoggerOptions};
pub use winston_transport::LogQuery;
//pub use logger::*;
