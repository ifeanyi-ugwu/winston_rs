use super::{
    default_levels::default_levels,
    transports::{console::ConsoleTransportOptions, Console, Transport, TransportStreamOptions},
};
use logform::{json, Format};
use std::{collections::HashMap, sync::Arc};

pub struct LoggerOptions {
    pub levels: Option<HashMap<String, u8>>,
    pub format: Option<Format>,
    pub level: Option<String>,
    pub transports: Option<Vec<Arc<dyn Transport + Send + Sync>>>,
}

impl LoggerOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn level(mut self, level: &str) -> Self {
        self.level = Some(level.to_string());
        self
    }

    pub fn format(mut self, format: Format) -> Self {
        self.format = Some(format);
        self
    }

    pub fn transports<T: Transport + Send + Sync + 'static>(mut self, transports: Vec<T>) -> Self {
        // Initialize the vector if it doesn't exist, and then clear it to reset
        if self.transports.is_none() {
            self.transports = Some(Vec::new());
        } else {
            self.transports.as_mut().unwrap().clear();
        }

        // Wrap each transport in Arc and add to the internal transports vector
        for transport in transports {
            self.transports.as_mut().unwrap().push(Arc::new(transport));
        }

        self
    }

    pub fn add_transport<T: Transport + Send + Sync + 'static>(mut self, transport: T) -> Self {
        if self.transports.is_none() {
            self.transports = Some(Vec::new());
        }
        self.transports.as_mut().unwrap().push(Arc::new(transport));
        self
    }

    pub fn levels(mut self, levels: HashMap<String, u8>) -> Self {
        self.levels = Some(levels);
        self
    }
}

impl Default for LoggerOptions {
    fn default() -> Self {
        let console_options = ConsoleTransportOptions {
            base: Some(TransportStreamOptions {
                level: Some("info".to_string()),
                format: None,
            }),
        };

        let console_transport = Arc::new(Console::new(Some(console_options)));

        LoggerOptions {
            levels: Some(default_levels()),
            level: Some("info".to_string()),
            transports: Some(vec![console_transport]),
            format: Some(json()),
        }
    }
}
