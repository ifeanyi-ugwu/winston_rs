//use std::collections::HashMap;
use logform::{Format, LogInfo};
use std::any::Any;
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::sync::Mutex;
use winston_transport::{LogQuery, Queryable, Transport};

pub struct FileTransportOptions {
    pub level: Option<String>,
    pub format: Option<Format>,
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

    pub fn builder() -> FileTransportBuilder {
        FileTransportBuilder::new()
    }

    /*unused
    pub fn flush(&self) -> std::io::Result<()> {
           let mut file = self.file.lock().unwrap();
           file.flush()
       }
    */
}

impl FileTransport {
    fn parse_log_entry(&self, line: &str) -> Option<LogInfo> {
        let parsed: serde_json::Value = serde_json::from_str(line).ok()?;
        // println!("Parsed log entry: {:?}", parsed); // Debug print

        let level = parsed["level"].as_str()?;
        let message = parsed["message"].as_str()?;
        let meta = parsed
            .as_object()?
            .iter()
            //.map(|(k, v)| (k.clone(), v.clone()))
            .filter_map(|(k, v)| {
                if k != "level" && k != "message" {
                    Some((k.clone(), v.clone()))
                } else {
                    None
                }
            })
            .collect::<HashMap<_, _>>(); // Collect all metadata

        Some(LogInfo {
            level: level.to_string(),
            message: message.to_string(),
            meta,
        })
    }
}

impl Transport for FileTransport {
    /*fn log(&self, message: &str, _level: &str) {
        let mut file = self.file.lock().unwrap();

        writeln!(file, "{}", message).unwrap();
    } */

    fn log(&self, info: LogInfo) {
        let mut file = self.file.lock().unwrap();
        if let Err(e) = writeln!(file, "{}", info.message) {
            eprintln!("Failed to write to log file: {}", e);
        }
        if let Err(e) = file.flush() {
            eprintln!("Failed to flush log file: {}", e);
        }
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

    fn as_queryable(&self) -> Option<&dyn Queryable> {
        Some(self)
    }
}

impl Queryable for FileTransport {
    fn query(&self, query: &LogQuery) -> Result<Vec<LogInfo>, String> {
        let file = File::open(self.options.filename.as_ref().unwrap())
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        let reader = BufReader::new(file);

        let mut results = Vec::new();
        let mut line_count = 0;

        // Determine the start and limit values
        let start = query.start.unwrap_or(0);
        let limit = query.limit.unwrap_or(usize::MAX);

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read line: {}", e))?;
            if let Some(entry) = self.parse_log_entry(&line) {
                //println!("parsed entry {:?}", entry);
                if query.matches(&entry) {
                    // Skip lines until the start position
                    if line_count >= start {
                        results.push(entry);
                    }
                    line_count += 1;

                    // Stop reading if the limit is reached
                    if results.len() >= limit {
                        break;
                    }
                }
            }
        }

        // Apply sorting to the results
        query.sort(&mut results);
        //println!("results: {:?}", results);
        Ok(results)
    }
}

pub struct FileTransportBuilder {
    level: Option<String>,
    format: Option<Format>,
    filename: Option<String>,
}

impl FileTransportBuilder {
    pub fn new() -> Self {
        Self {
            level: None,
            format: None,
            filename: None,
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

    pub fn filename<T: Into<String>>(mut self, filename: T) -> Self {
        self.filename = Some(filename.into());
        self
    }

    pub fn build(self) -> FileTransport {
        let options = FileTransportOptions {
            level: self.level,
            format: self.format,
            filename: self.filename,
            // Set other fields as needed
        };
        FileTransport::new(options)
    }
}
