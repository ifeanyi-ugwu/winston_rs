use std::{fmt, sync::Arc};

use logform::{Format, LogInfo};
use winston_transport::{LogQuery, Transport};

#[derive(Clone)]
pub struct LoggerTransport<L> {
    transport: Arc<dyn Transport<L> + Send + Sync>,
    level: Option<String>,
    format: Option<Arc<dyn Format<Input = L> + Send + Sync>>,
}

impl<L> LoggerTransport<L> {
    pub fn new(transport: Arc<dyn Transport<L> + Send + Sync>) -> Self {
        Self {
            transport,
            level: None,
            format: None,
        }
    }

    pub fn with_level(mut self, level: String) -> Self {
        self.level = Some(level);
        self
    }

    pub fn with_format(mut self, format: Arc<dyn Format<Input = L> + Send + Sync>) -> Self {
        self.format = Some(format);
        self
    }

    pub fn get_level(&self) -> Option<&String> {
        self.level.as_ref()
    }
    pub fn get_format(&self) -> Option<Arc<dyn Format<Input = L> + Send + Sync>> {
        self.format.clone()
    }

    pub fn get_transport(&self) -> &Arc<dyn Transport<L> + Send + Sync> {
        &self.transport
    }
}

impl<L> fmt::Debug for LoggerTransport<L> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LoggerTransport")
            .field(
                "transport",
                &format!("Transport<{}>", std::any::type_name::<L>()),
            )
            .field("level", &self.level)
            .field("format", &self.format.as_ref().map(|_| "Format<...>"))
            .finish()
    }
}

impl From<Arc<dyn Transport<LogInfo> + Send + Sync>> for LoggerTransport<LogInfo> {
    fn from(transport: Arc<dyn Transport<LogInfo> + Send + Sync>) -> Self {
        LoggerTransport::new(transport)
    }
}
