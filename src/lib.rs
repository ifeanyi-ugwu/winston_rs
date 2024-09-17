mod logger;
pub mod logger_v2;
pub mod logger_v3;

pub use logform as format;
pub use logger::create_logger::create_logger;
pub use logger::transports;
pub use logger::{configure, log, Logger, LoggerOptions};
pub use winston_transport::LogQuery;
//pub use logger::*;
