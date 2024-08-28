use std::{thread, time::Duration};

use winston::transports::Transport;

pub struct DelayedTransport {
    delay: Duration,
}

impl DelayedTransport {
    pub fn new(delay: Duration) -> Self {
        DelayedTransport { delay }
    }
}

impl Transport for DelayedTransport {
    fn log(&self, _message: &str, _level: &str) {
        // Simulate a delay
        thread::sleep(self.delay);
        println!("DelayedTransport logging: {:?}", _message);
    }

    fn get_level(&self) -> Option<&String> {
        None
    }

    fn get_format(&self) -> Option<&crate::format::Format> {
        None
    }
}
