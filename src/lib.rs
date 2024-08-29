mod logger;

pub use logform as format;
pub use logger::create_logger::create_logger;
pub use logger::log_entry::LogEntry;
pub use logger::transports;
pub use logger::{
    configure, log,
    log_query::{LogQuery, Order},
    Logger, LoggerOptions,
};
//pub use logger::*;
