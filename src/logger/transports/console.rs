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

    pub fn builder() -> ConsoleTransportBuilder {
        ConsoleTransportBuilder::new()
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

pub struct ConsoleTransportBuilder {
    base: Option<TransportStreamOptions>,
    //console_warn_levels: Option<Vec<String>>,
    //stderr_levels: Option<Vec<String>>,
    //debug_stdout: Option<bool>,
    // eol: Option<String>,
}

impl ConsoleTransportBuilder {
    pub fn new() -> Self {
        Self {
            base: None,
            //console_warn_levels: None,
            // stderr_levels: None,
            // debug_stdout: None,
            // eol: None,
        }
    }

    pub fn level<T: Into<String>>(mut self, level: T) -> Self {
        let level = level.into();
        self.base
            .get_or_insert_with(|| TransportStreamOptions {
                level: None,
                format: None,
            })
            .level = Some(level);
        self
    }

    pub fn format(mut self, format: String) -> Self {
        self.base
            .get_or_insert_with(|| TransportStreamOptions {
                level: None,
                format: None,
            })
            .format = Some(format);
        self
    }
    /* pub fn console_warn_levels(mut self, levels: Vec<String>) -> Self {
        self.console_warn_levels = Some(levels);
        self
    }

    pub fn stderr_levels(mut self, levels: Vec<String>) -> Self {
        self.stderr_levels = Some(levels);
        self
    }

    pub fn debug_stdout(mut self, debug: bool) -> Self {
        self.debug_stdout = Some(debug);
        self
    }

    pub fn eol(mut self, eol: String) -> Self {
        self.eol = Some(eol);
        self
    }
    */

    pub fn build(self) -> ConsoleTransport {
        let options = ConsoleTransportOptions {
            base: self.base,
            //console_warn_levels: self.console_warn_levels,
            //stderr_levels: self.stderr_levels,
            //debug_stdout: self.debug_stdout,
            //eol: self.eol,
        };
        ConsoleTransport::new(Some(options))
    }
}
