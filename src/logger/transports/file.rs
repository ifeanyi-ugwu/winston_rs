use super::{Transport, TransportStreamOptions};
//use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write};
use std::sync::Mutex;

pub struct FileTransportOptions {
    pub base: Option<TransportStreamOptions>,
    pub filename: Option<String>,
    /*
    unused yet
    pub dirname: Option<String>,
    pub options: Option<HashMap<String, String>>,
    pub maxsize: Option<u64>,
    pub stream: Option<Box<dyn Write + Send + Sync>>,
    pub rotation_format: Option<Box<dyn Fn() -> String + Send + Sync>>,
    pub zipped_archive: Option<bool>,
    pub max_files: Option<u64>,
    pub eol: Option<String>,
    pub tailable: Option<bool>,
    pub lazy: Option<bool>,
     */
}

pub struct FileTransport {
    file: Mutex<BufWriter<File>>,
    options: FileTransportOptions,
}

impl FileTransport {
    pub fn new(options: FileTransportOptions) -> Self {
        let file_path = options
            .filename
            .clone()
            .expect("File path is required for FileTransport");
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .expect("Failed to open log file");
        let writer = BufWriter::new(file);

        FileTransport {
            file: Mutex::new(writer),
            options,
        }
    }

    pub fn builder() -> FileTransportOptionsBuilder {
        FileTransportOptionsBuilder::new()
    }

    /*unused
    pub fn flush(&self) -> std::io::Result<()> {
           let mut file = self.file.lock().unwrap();
           file.flush()
       }
    */
}

impl Transport for FileTransport {
    /*fn log(&self, message: &str, _level: &str) {
        let mut file = self.file.lock().unwrap();

        writeln!(file, "{}", message).unwrap();
    } */

    fn log(&self, message: &str, _level: &str) {
        let mut file = self.file.lock().unwrap();
        if let Err(e) = writeln!(file, "{}", message) {
            eprintln!("Failed to write to log file: {}", e);
        }
        if let Err(e) = file.flush() {
            eprintln!("Failed to flush log file: {}", e);
        }
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

pub struct FileTransportOptionsBuilder {
    base: Option<TransportStreamOptions>,
    filename: Option<String>,
}

impl FileTransportOptionsBuilder {
    pub fn new() -> Self {
        Self {
            base: None,
            filename: None,
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

    pub fn filename<T: Into<String>>(mut self, filename: T) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn build(self) -> FileTransport {
        let options = FileTransportOptions {
            base: self.base,
            filename: self.filename,
            // Set other fields as needed
        };
        FileTransport::new(options)
    }
}
