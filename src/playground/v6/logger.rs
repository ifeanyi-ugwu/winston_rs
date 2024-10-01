use futures::{Sink, SinkExt};
use std::io::{self, Write};
use tokio::fs::File;

use super::transport::Transport;

pub struct Logger<T: Sink<String> + Unpin> {
    sinks: Vec<T>,
}

impl<T: Sink<String, Error = std::io::Error> + Unpin> Logger<T> {
    pub fn new(sinks: Vec<T>) -> Self {
        Logger { sinks }
    }

    pub async fn log(&mut self, message: String) {
        for sink in &mut self.sinks {
            let _ = sink.send(message.clone()).await; // Clone the message for each transport
        }
    }
}

#[tokio::main]
async fn main() {
    // Create two transports: one for stdout and one for a file
    let console_transport = Transport::new(io::stdout(), None);
    let file_transport = Transport::new(File::create("log.txt").unwrap(), None);

    // Create a logger with both transports
    let mut logger = Logger::new(vec![console_transport, file_transport]);

    // Log a message to all transports
    logger.log("This is a test log message.".to_string()).await;
}
