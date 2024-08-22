use super::{Transport, TransportStreamOptions};

pub struct ConsoleTransportOptions {
    pub base: Option<TransportStreamOptions>,
    /*
    unused yet
    pub console_warn_levels: Option<Vec<String>>,
    pub stderr_levels: Option<Vec<String>>,
    pub debug_stdout: Option<bool>,
    pub eol: Option<String>,
    */
}

pub struct ConsoleTransport {
    options: ConsoleTransportOptions,
}

impl ConsoleTransport {
    pub fn new(options: Option<ConsoleTransportOptions>) -> Self {
        let options = options.unwrap_or_else(|| ConsoleTransportOptions { base: None });

        ConsoleTransport { options }
    }
}

impl Transport for ConsoleTransport {
    fn log(&self, message: &str, _level: &str) {
        println!("{}", message);
    }

    fn get_level(&self) -> Option<&String> {
        self.options
            .base
            .as_ref()
            .and_then(|base| base.level.as_ref())
    }

    fn get_format(&self) -> Option<&String> {
        self.options
            .base
            .as_ref()
            .and_then(|base| base.format.as_ref())
    }
}
