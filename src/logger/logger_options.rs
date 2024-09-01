use super::{default_levels::default_levels, transports::Transport};
use logform::{json, Format};
use std::{collections::HashMap, sync::Arc};

pub struct LoggerOptions {
    pub levels: Option<HashMap<String, u8>>,
    pub format: Option<Format>,
    pub level: Option<String>,
    pub transports: Option<Vec<Arc<dyn Transport + Send + Sync>>>,
}

impl LoggerOptions {
    /// Creates a new `LoggerOptions` instance with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the logging level for the logger.
    ///
    /// # Arguments
    ///
    /// * `level` - A string slice that represents the logging level.
    pub fn level(mut self, level: &str) -> Self {
        self.level = Some(level.to_string());
        self
    }

    /// Sets the log format for the logger.
    ///
    /// # Arguments
    ///
    /// * `format` - The log format to be used.
    pub fn format(mut self, format: Format) -> Self {
        self.format = Some(format);
        self
    }

    /// Replaces the existing transports with a new set of transports.
    ///
    /// This method clears any existing transports and replaces them with the
    /// provided vector of transports. Each transport is automatically wrapped
    /// in an `Arc` to ensure it is thread-safe.
    ///
    /// # Arguments
    ///
    /// * `transports` - A vector of transports that will replace the current transports.
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

    /// Adds a single transport to the existing list of transports.
    ///
    /// This method adds the provided transport to the existing list of transports,
    /// keeping any previously added transports intact. The transport is automatically
    /// wrapped in an `Arc` to ensure it is thread-safe.
    ///
    /// # Arguments
    ///
    /// * `transport` - A single transport to be added to the current list.
    pub fn add_transport<T: Transport + Send + Sync + 'static>(mut self, transport: T) -> Self {
        if self.transports.is_none() {
            self.transports = Some(Vec::new());
        }
        self.transports.as_mut().unwrap().push(Arc::new(transport));
        self
    }

    /// Sets custom logging levels for the logger.
    ///
    /// # Arguments
    ///
    /// * `levels` - A `HashMap` where the key is the level name and the value is its severity.
    pub fn levels(mut self, levels: HashMap<String, u8>) -> Self {
        self.levels = Some(levels);
        self
    }
}

impl Default for LoggerOptions {
    /// Provides the default configuration for `LoggerOptions`.
    ///
    /// The default configuration includes:
    /// - A default set of logging levels.
    /// - The logging level set to "info".
    /// - No default transports.
    /// - The JSON format for log entries.
    fn default() -> Self {
        LoggerOptions {
            levels: Some(default_levels()),
            level: Some("info".to_string()),
            transports: Some(Vec::new()),
            format: Some(json()),
        }
    }
}
