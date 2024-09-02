use super::{custom_levels::CustomLevels, log_entry::convert_log_entry, transports::Transport};
use crate::LogEntry;
use crossbeam_channel::Receiver as CBReceiver;
use logform::Format;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

pub struct LoggerWorker {
    pub levels: CustomLevels,
    pub format: Format,
    pub level: String,
    pub transports: Vec<Arc<dyn Transport + Send + Sync>>,
    pub log_receiver: CBReceiver<Option<LogEntry>>,
    shutdown_signal: Arc<AtomicBool>,
    worker_handle: Option<std::thread::JoinHandle<()>>,
}

impl LoggerWorker {
    pub fn new(
        levels: CustomLevels,
        format: Format,
        level: String,
        transports: Vec<Arc<dyn Transport + Send + Sync>>,
        log_receiver: CBReceiver<Option<LogEntry>>,
        shutdown_signal: Arc<AtomicBool>,
    ) -> Self {
        let shutdown_signal_clone = Arc::clone(&shutdown_signal);
        let levels_clone = levels.clone();
        let format_clone = format.clone();
        let level_clone = level.clone();
        let transports_clone = transports.clone();
        let log_rec_clone = log_receiver.clone();

        // Start the worker thread
        let worker_handle = thread::spawn(move || {
            let mut worker = LoggerWorker {
                levels: levels_clone,
                format: format_clone,
                level: level_clone,
                transports: transports_clone,
                log_receiver: log_rec_clone,
                shutdown_signal: shutdown_signal_clone,
                worker_handle: None,
            };

            worker.run();
        });

        LoggerWorker {
            levels,
            format,
            level,
            transports,
            log_receiver,
            shutdown_signal,
            worker_handle: Some(worker_handle),
        }
    }

    pub fn run(&mut self) {
        while !self.shutdown_signal.load(Ordering::Relaxed) {
            match self
                .log_receiver
                .recv_timeout(std::time::Duration::from_millis(100))
            {
                Ok(Some(entry)) => {
                    if self.is_level_enabled(&entry.level)
                        && (!entry.message.is_empty() || !entry.meta.is_empty())
                    {
                        self.process_log_entry(entry);
                    }
                }
                Ok(None) => break, // Shutdown signal received
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => continue,
                Err(_) => break, // Channel is disconnected
            }
        }

        // Process any remaining messages in the channel
        while let Ok(Some(entry)) = self.log_receiver.try_recv() {
            if self.is_level_enabled(&entry.level)
                && (!entry.message.is_empty() || !entry.meta.is_empty())
            {
                self.process_log_entry(entry);
            }
        }
    }

    fn process_log_entry(&self, entry: LogEntry) {
        for transport in &self.transports {
            if let Some(formatted_message) = self.format_message(&entry, transport.get_format()) {
                transport.log(&formatted_message, &entry.level);
            }
        }
    }

    fn format_message(
        &self,
        entry: &LogEntry,
        transport_format: Option<&Format>,
    ) -> Option<String> {
        let converted_entry = convert_log_entry(entry);

        // Apply the transport-specific format if provided
        let formatted_entry = if let Some(format) = transport_format {
            format.transform(converted_entry.clone(), None)
        } else {
            // Otherwise, use the default logger format
            self.format.transform(converted_entry.clone(), None)
        };

        formatted_entry.map(|entry| entry.message)
    }

    pub fn is_level_enabled(&self, level: &str) -> bool {
        let given_level_value = self.get_level_severity(level);
        if given_level_value.is_none() {
            return false;
        }

        let configured_level_value = self.get_level_severity(&self.level);
        if configured_level_value.is_none() {
            return false;
        }

        if self.transports.is_empty() {
            return configured_level_value.unwrap() >= given_level_value.unwrap();
        }

        self.transports.iter().any(|transport| {
            let transport_level_value = transport
                .get_level()
                .and_then(|transport_level| self.get_level_severity(transport_level))
                .unwrap_or(configured_level_value.unwrap());
            transport_level_value >= given_level_value.unwrap()
        })
    }

    fn get_level_severity(&self, level: &str) -> Option<u8> {
        self.levels.get_severity(level)
    }
}

impl Drop for LoggerWorker {
    fn drop(&mut self) {
        self.shutdown_signal.store(true, Ordering::Relaxed);

        if let Some(handle) = self.worker_handle.take() {
            handle.join().expect("Failed to join worker thread");
        }
    }
}
