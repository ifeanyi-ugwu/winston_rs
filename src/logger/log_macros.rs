// Convenience macros for global logging
#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        $crate::log("info", &format!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        $crate::log("warn", &format!($($arg)*));
    }
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        $crate::log("error", &format!($($arg)*));
    }
}

// ... Add more macros for other log levels
