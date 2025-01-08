#[macro_export]
macro_rules! log {
    ($level:ident, $message:expr $(, $key:ident = $value:expr)* $(,)?) => {{
        let entry = $crate::format::LogInfo::new(stringify!($level), $message)
            $(.with_meta(stringify!($key), $value))*;
        $crate::log(entry);
    }};
    ($logger:expr, $level:ident, $message:expr $(, $key:ident = $value:expr)* $(,)?) => {{
        let entry = $crate::format::LogInfo::new(stringify!($level), $message)
            $(.with_meta(stringify!($key), $value))*;
        $logger.log(entry);
    }};
}

#[macro_export]
macro_rules! create_log_methods {
    ($($level:ident),*) => {
        impl Logger {
            $(
                pub fn $level(&self, message: &str) {
                    let log_entry = LogInfo::new(stringify!($level), message);
                    self.log(log_entry);
                }
            )*
        }
    };
}
