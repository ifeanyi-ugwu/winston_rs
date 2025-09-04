use std::{thread, time::Duration};
use winston::format::LogInfo;
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
}

pub fn generate_random_filename() -> String {
    let timestamp = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    format!("test_log_{}.log", timestamp)
}

use std::fs;
use std::path::Path;

pub fn delete_file_if_exists(file_path: &str) {
    let path = Path::new(file_path);
    if path.exists() {
        fs::remove_file(path).unwrap_or_else(|err| {
            eprintln!("Failed to delete file {}: {}", file_path, err);
        });
    }
}

use std::sync::{Arc, Mutex};
use winston_transport::LogQuery;

#[derive(Debug, Default)]
pub struct MemoryTransport {
    pub logs: Arc<Mutex<Vec<LogInfo>>>,
}

impl MemoryTransport {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Transport for MemoryTransport {
    fn log(&self, info: LogInfo) {
        self.logs.lock().unwrap().push(info);
    }

    fn flush(&self) -> Result<(), String> {
        Ok(())
    }

    fn query(&self, _options: &LogQuery) -> Result<Vec<LogInfo>, String> {
        Ok(self.logs.lock().unwrap().clone())
    }
}
