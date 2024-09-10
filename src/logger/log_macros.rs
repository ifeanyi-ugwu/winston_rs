// Convenience macros for global logging
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log(LogInfo::new("info", &format!($($arg)*)));
    }
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::log(LogInfo::new("info", &format!($($arg)*)));
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
$crate::log(LogInfo::new("info", &format!($($arg)*)));
    }
}

// ... Add more macros for other log levels
