use logform::Format;
use std::any::Any;
use winston_transport::Transport;

pub struct ConsoleTransportOptions {
    level: Option<String>,
    format: Option<Format>,
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
        let options = options.unwrap_or_else(|| ConsoleTransportOptions {
            level: None,
            format: None,
        });

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
        self.options.level.as_ref()
    }

    fn get_format(&self) -> Option<&Format> {
        self.options.format.as_ref()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub struct ConsoleTransportBuilder {
    level: Option<String>,
    format: Option<Format>,
    //console_warn_levels: Option<Vec<String>>,
    //stderr_levels: Option<Vec<String>>,
    //debug_stdout: Option<bool>,
    // eol: Option<String>,
}

impl ConsoleTransportBuilder {
    pub fn new() -> Self {
        Self {
            level: None,
            format: None,
            //console_warn_levels: None,
            // stderr_levels: None,
            // debug_stdout: None,
            // eol: None,
        }
    }

    pub fn level<T: Into<String>>(mut self, level: T) -> Self {
        self.level = Some(level.into());
        self
    }

    pub fn format(mut self, format: Format) -> Self {
        self.format = Some(format);
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
            level: self.level,
            format: self.format,
            //console_warn_levels: self.console_warn_levels,
            //stderr_levels: self.stderr_levels,
            //debug_stdout: self.debug_stdout,
            //eol: self.eol,
        };
        ConsoleTransport::new(Some(options))
    }
}
