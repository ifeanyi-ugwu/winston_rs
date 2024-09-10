// Convenience macros for global logging
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log($crate::format::LogInfo::new("info", &format!($($arg)*)));
    }
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::log($crate::format::LogInfo::new("warn", &format!($($arg)*)));
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::log($crate::format::LogInfo::new("error", &format!($($arg)*)));
    }
}

// ... Add more macros for other log levels
