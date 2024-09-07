use logform::LogInfo;
use std::{thread, time::Duration};
use winston_transport::Transport;

pub struct DelayedTransport {
    delay: Duration,
}

impl DelayedTransport {
    pub fn new(delay: Duration) -> Self {
        DelayedTransport { delay }
    }
}

impl Transport for DelayedTransport {
    fn log(&self, info: LogInfo) {
        let delay = self.delay;
        let message = info.message;
        let level = info.level;

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

pub fn generate_random_filename() -> String {
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    format!("test_log_{}.log", timestamp)
}
