use super::custom_levels::CustomLevels;
use logform::{json, Format};
use std::{collections::HashMap, fmt, sync::Arc};
use winston_transport::Transport;

// We'll use a wrapper type for Format to implement Debug
#[derive(Clone)]
pub struct DebugFormat(pub Format);

impl fmt::Debug for DebugFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Format")
    }
}

// We'll use a wrapper type for Transport to implement Debug
pub struct DebugTransport(pub Arc<dyn Transport + Send + Sync>);

impl fmt::Debug for DebugTransport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Transport")
    }
}

impl Clone for DebugTransport {
    fn clone(&self) -> Self {
        DebugTransport(Arc::clone(&self.0))
    }
}

#[derive(Debug, Clone)]
pub struct LoggerOptions {
    pub levels: Option<CustomLevels>,
    pub format: Option<DebugFormat>,
    pub level: Option<String>,
    pub transports: Option<Vec<DebugTransport>>,
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
        self.format = Some(DebugFormat(format));
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
        self.transports
            .as_mut()
            .unwrap()
            .push(DebugTransport(Arc::new(transport)));
        self
    }

    /// Sets custom logging levels for the logger.
    ///
    /// # Arguments
    ///
    /// * `levels` - A `HashMap` where the key is the level name and the value is its severity.
    pub fn levels(mut self, levels: HashMap<String, u8>) -> Self {
        self.levels = Some(CustomLevels::new(levels));
        self
    }

    // Helper method to get the actual Format
    pub fn get_format(&self) -> Option<&Format> {
        self.format.as_ref().map(|df| &df.0)
    }

    // Helper method to get the actual Transports
    pub fn get_transports(&self) -> Option<Vec<Arc<dyn Transport + Send + Sync>>> {
        self.transports
            .as_ref()
            .map(|ts| ts.iter().map(|dt| Arc::clone(&dt.0)).collect())
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
            levels: Some(CustomLevels::default()),
            level: Some("info".to_string()),
            transports: Some(Vec::new()),
            format: Some(DebugFormat(json())),
        }
    }
}
