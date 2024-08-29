use super::{custom_levels::CustomLevels, log_entry::convert_log_entry, transports::Transport};
use crate::LogEntry;
use crossbeam_channel::Receiver as CBReceiver;
use logform::Format;
use std::sync::Arc;

pub struct LoggerWorker {
    pub levels: CustomLevels,
    pub format: Format,
    pub level: String,
    pub transports: Vec<Arc<dyn Transport + Send + Sync>>,
    pub log_receiver: CBReceiver<LogEntry>,
}

impl LoggerWorker {
    pub fn run(&self) {
        while let Ok(entry) = self.log_receiver.recv() {
            if entry.is_flush() {
                // Process any remaining entries
                while let Ok(entry) = self.log_receiver.try_recv() {
                    self.process_log_entry(entry);
                }
                break;
            }
            if self.is_level_enabled(&entry.level) {
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
