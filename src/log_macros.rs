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
macro_rules! meta {
    ($($key:ident = $value:expr),+ $(,)?) => {{
        vec![
            $(
                (stringify!($key), serde_json::to_value($value).unwrap())
            ),+
        ]
    }}
}

#[macro_export]
macro_rules! create_log_methods {
    ($($level:ident),*) => {
        pub trait LoggerMethods {
            $(
                fn $level(&self, message: &str, metadata: Option<Vec<(&'static str, serde_json::Value)>>);
            )*
        }

        impl LoggerMethods for $crate::Logger {
            $(
                fn $level(&self, message: &str, metadata: Option<Vec<(&'static str, serde_json::Value)>>) {
                    let mut entry = $crate::format::LogInfo::new(stringify!($level), message);
                    if let Some(meta) = metadata {
                        for (key, value) in meta {
                            entry = entry.with_meta(key, value);
                        }
                    }
                    self.log(entry);
                }
            )*
        }
    };
}

#[macro_export]
macro_rules! create_level_macros {
    ($($level:ident),*) => {
        $(
            macro_rules! $level {
                // First arm: Log without metadata
                ($logger:expr, $message:expr) => {
                    $crate::log!($logger, $level, $message);
                };

                // Second arm: Log with metadata
                ($logger:expr, $message:expr, $meta:expr) => {{
                    let mut entry = $crate::format::LogInfo::new(stringify!($level), $message);
                    for (key, value) in $meta {
                        entry = entry.with_meta(key, value);
                    }
                    $logger.log(entry);
                }};

                // Third arm: Log without metadata using the global logger
                ($message:expr) => {
                    $crate::log!($level, $message);
                };

                // Fourth arm: Log with metadata using the global logger
                // Modified to use a special marker to distinguish from the first arm
               (@global, $message:expr, $meta:expr) => {{
                    let mut entry = $crate::format::LogInfo::new(stringify!($level), $message);
                    for (key, value) in $meta {
                        entry = entry.with_meta(key, value);
                    }
                    $crate::log(entry);
                }};
            }
        )*
    };
}
