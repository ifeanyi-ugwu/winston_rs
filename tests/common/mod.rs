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
    fn log(&self, message: &str, level: &str) {
        let delay = self.delay;
        let message = message.to_string();
        let level = level.to_string();

        // Directly delay in the current thread (synchronous for testing)
        thread::sleep(delay);
        println!("Delayed log: {} - {}", level, message);
    }

    fn get_level(&self) -> Option<&String> {
        None
    }

    fn get_format(&self) -> Option<&crate::format::Format> {
        None
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
