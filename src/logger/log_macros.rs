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

#[macro_export]
macro_rules! log {
    ($level:ident, $message:expr $(, $key:ident = $value:expr)* $(,)?) => {{
        let entry = $crate::format::LogInfo::new(stringify!($level), $message)
            $(.add_meta(stringify!($key), $value))*;
        $crate::log(entry);
    }};
    ($logger:expr, $level:ident, $message:expr $(, $key:ident = $value:expr)* $(,)?) => {{
        let entry = $crate::format::LogInfo::new(stringify!($level), $message)
            $(.add_meta(stringify!($key), $value))*;
        $logger.log(entry);
    }};
}
