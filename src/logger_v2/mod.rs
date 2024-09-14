pub mod transport;

use std::io::Write;
use std::sync::Arc;
use transport::Transport;

pub struct Logger<W: Write + Send + 'static> {
    transport: Arc<Transport<W>>,
}

impl<W: Write + Send + 'static> Logger<W> {
    pub fn new(transport: Transport<W>) -> Self {
        Logger {
            transport: Arc::new(transport),
        }
    }

    pub fn log(&self, message: &str) {
        let transport = Arc::clone(&self.transport);
        transport.log(message.to_string());
    }
}
