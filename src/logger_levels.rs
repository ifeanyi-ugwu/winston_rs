use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct LoggerLevels {
    levels: HashMap<String, u8>,
}

impl LoggerLevels {
    pub fn new<K: Into<String>>(levels: impl IntoIterator<Item = (K, u8)>) -> Self {
        LoggerLevels {
            levels: levels.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        }
    }

    pub fn get_severity(&self, key: &str) -> Option<u8> {
        self.levels.get(key).copied()
    }
}

impl Default for LoggerLevels {
    fn default() -> Self {
        LoggerLevels::new([
            ("error", 0),
            ("warn", 1),
            ("info", 2),
            ("debug", 3),
            ("trace", 4),
        ])
    }
}

impl From<LoggerLevels> for HashMap<String, u8> {
    fn from(logger_levels: LoggerLevels) -> Self {
        logger_levels.levels
    }
}
