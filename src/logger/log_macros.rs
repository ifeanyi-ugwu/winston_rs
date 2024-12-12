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
