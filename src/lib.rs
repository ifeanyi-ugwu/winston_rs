mod logger;

pub use logform as format;
pub use logger::create_logger::create_logger;
pub use logger::transports;
pub use logger::{configure, log, BackpressureStrategy, Logger, LoggerOptions};
pub use winston_transport::LogQuery;
//pub use logger::*;
