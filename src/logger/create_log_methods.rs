#[macro_export]
macro_rules! create_log_methods {
    ($logger:ident, $($level:ident),*) => {
        impl $logger {
            $(
                pub fn $level(&self, message: &str) {
                    self.log(stringify!($level), message);
                }
            )*
        }
    };
}
