use crate::{logger_levels::LoggerLevels, logger_transport::LoggerTransport};
use logform::{json, Format, LogInfo};
use std::{collections::HashMap, fmt, sync::Arc};
use winston_transport::Transport;

// Wrapper type for Transport to implement Debug
pub struct DebugTransport(pub Arc<dyn Transport<LogInfo>>);

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

#[derive(Clone)]
pub struct LoggerOptions {
    pub levels: Option<LoggerLevels>,
    pub format: Option<Arc<dyn Format<Input = LogInfo> + Send + Sync>>,
    pub level: Option<String>,
    pub transports: Option<Vec<LoggerTransport<LogInfo>>>,
    pub channel_capacity: Option<usize>,
    pub backpressure_strategy: Option<BackpressureStrategy>,
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
    pub fn level<T: Into<String>>(mut self, level: T) -> Self {
        self.level = Some(level.into());
        self
    }

    /// Sets the log format for the logger.
    ///
    /// # Arguments
    ///
    /// * `format` - The log format to be used.
    pub fn format<F>(mut self, format: F) -> Self
    where
        F: Format<Input = LogInfo> + Send + Sync + 'static,
    {
        self.format = Some(Arc::new(format));
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
    pub fn add_transport(mut self, transport: Arc<dyn Transport<LogInfo> + Send + Sync>) -> Self {
        self.transports
            .get_or_insert_with(Vec::new)
            .push(LoggerTransport::new(transport));
        self
    }

    pub fn transports(
        mut self,
        transports: Vec<Arc<dyn Transport<LogInfo> + Send + Sync>>,
    ) -> Self {
        self.transports = Some(
            transports
                .into_iter()
                .map(|t| LoggerTransport::new(t))
                .collect(),
        );
        self
    }

    /// Sets custom logging levels for the logger.
    ///
    /// # Arguments
    ///
    /// * `levels` - A `HashMap` where the key is the level name and the value is its severity.
    pub fn levels(mut self, levels: HashMap<String, u8>) -> Self {
        self.levels = Some(LoggerLevels::new(levels));
        self
    }

    /// Sets the channel capacity for the logger.
    ///
    /// # Arguments
    ///
    /// * `capacity` - An `usize` that defines the capacity of the channel.
    pub fn channel_capacity(mut self, capacity: usize) -> Self {
        self.channel_capacity = Some(capacity);
        self
    }

    /// Sets the backpressure strategy for the logger.
    ///
    /// # Arguments
    ///
    /// * `strategy` - The backpressure strategy to apply when the channel is full.
    pub fn backpressure_strategy(mut self, strategy: BackpressureStrategy) -> Self {
        self.backpressure_strategy = Some(strategy);
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
    /// - A channel capacity of 1024.
    /// - A backpressure strategy set to `BackpressureStrategy::Block`, meaning the logger will block on overflow until space is available.

    fn default() -> Self {
        LoggerOptions {
            levels: Some(LoggerLevels::default()),
            level: Some("info".to_string()),
            transports: Some(Vec::new()),
            format: Some(Arc::new(json())),
            channel_capacity: Some(1024),
            backpressure_strategy: Some(BackpressureStrategy::Block),
        }
    }
}

impl std::fmt::Debug for LoggerOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LoggerOptions")
            .field("levels", &self.levels)
            .field("level", &self.level)
            .field("transports", &self.transports)
            .field("channel_capacity", &self.channel_capacity)
            .field("backpressure_strategy", &self.backpressure_strategy)
            // For the format field, just print a placeholder because it can't be debugged:
            .field("format", &"<Format trait object>")
            .finish()
    }
}

#[derive(Clone, Debug)]
pub enum BackpressureStrategy {
    DropOldest,
    Block,
    DropCurrent,
}
