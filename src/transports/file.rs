//use std::collections::HashMap;
use super::proxy::Proxy;
use logform::{Format, LogInfo};
use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, Write};
use std::sync::Mutex;
use winston_transport::{LogQuery, Transport};

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
        //println!("Parsed log entry: {:?}", parsed); // Debug print

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
    }

    fn flush(&self) -> Result<(), String> {
        let mut file = self.file.lock().unwrap();
        //println!("Flushing file transport");

        file.flush()
            .map_err(|e| format!("Failed to flush file: {}", e))
    }

    fn get_level(&self) -> Option<&String> {
        self.options.level.as_ref()
    }

    fn get_format(&self) -> Option<&Format> {
        self.options.format.as_ref()
    }

    fn query(&self, query: &LogQuery) -> Result<Vec<LogInfo>, String> {
        let file = File::open(self.options.filename.as_ref().unwrap())
            .map_err(|e| format!("Failed to open log file: {}", e))?;
        let reader = BufReader::new(file);

        let mut results = Vec::new();

        // Determine the start and limit values
        let start = query.start.unwrap_or(0);
        let limit = query.limit.unwrap_or(usize::MAX);

        for (index, line) in reader.lines().enumerate() {
            let line = line.map_err(|e| format!("Failed to read line {}: {}", index, e))?;
            if let Some(entry) = self.parse_log_entry(&line) {
                if query.matches(&entry) {
                    // Skip lines until the start position
                    if index >= start {
                        results.push(entry);
                    }

                    // Stop reading if the limit is reached
                    if results.len() >= limit && limit != 0 {
                        break;
                    }
                }
            }
        }

        // Apply sorting to the results
        query.sort(&mut results);

        // Project fields if specified
        let results = if !query.fields.is_empty() {
            results
                .into_iter()
                .map(|entry| {
                    // Normalize fields to lowercase for case-insensitive matching
                    let normalized_fields: Vec<String> =
                        query.fields.iter().map(|f| f.to_lowercase()).collect();

                    LogInfo {
                        // Only include level if 'level' is in fields
                        level: if normalized_fields.contains(&"level".to_string()) {
                            entry.level
                        } else {
                            String::new()
                        },
                        // Only include message if 'message' is in fields
                        message: if normalized_fields.contains(&"message".to_string()) {
                            entry.message
                        } else {
                            String::new()
                        },
                        // Filter meta fields based on specified fields
                        meta: entry
                            .meta
                            .into_iter()
                            .filter(|(k, _)| normalized_fields.contains(&k.to_lowercase()))
                            .collect(),
                    }
                })
                .collect()
        } else {
            results
        };

        //println!("results: {:?}", results);
        Ok(results)
    }
}

impl Drop for FileTransport {
    fn drop(&mut self) {
        // Attempt to flush any remaining logs before dropping
        if let Ok(mut file) = self.file.lock() {
            if let Err(e) = file.flush() {
                eprintln!("Error flushing log file during drop: {}", e);
            }
        }
    }
}

impl Proxy for FileTransport {
    fn proxy(&self, target: &dyn Proxy) -> Result<usize, String> {
        let path = self
            .options
            .filename
            .as_ref()
            .ok_or("No file path provided")?;

        // First get a lock and flush any pending writes
        let mut file_guard = self
            .file
            .lock()
            .map_err(|_| "Failed to acquire file lock".to_string())?;

        file_guard
            .flush()
            .map_err(|e| format!("Failed to flush pending writes: {}", e))?;

        // Create a separate reader while holding the lock
        let file =
            File::open(path).map_err(|e| format!("Failed to open file for reading: {}", e))?;
        let reader = BufReader::new(file);

        let mut log_entries = Vec::new();

        for line in reader.lines() {
            let line = line.map_err(|e| format!("Failed to read log line: {}", e))?;
            if let Some(log) = self.parse_log_entry(&line) {
                log_entries.push(log);
            }
        }

        let log_count = log_entries.len();
        if log_count == 0 {
            return Ok(0);
        }

        // Send logs to target
        target.ingest(log_entries)?;

        // Truncate the file using the writer
        file_guard
            .get_mut()
            .set_len(0)
            .map_err(|e| format!("Failed to clear file: {}", e))?;

        // Seek back to start
        file_guard
            .rewind()
            .map_err(|e| format!("Failed to rewind after clear: {}", e))?;

        Ok(log_count)
    }

    fn ingest(&self, logs: Vec<LogInfo>) -> Result<(), String> {
        let path = self
            .options
            .filename
            .as_ref()
            .ok_or("No file path provided")?;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| format!("Failed to open file: {}", e))?;

        for log in logs {
            let formatted_log = match &self.options.format {
                Some(format) => format
                    .transform(log.clone(), None)
                    .ok_or_else(|| "Transform failed".to_string())?,
                None => log,
            };

            writeln!(file, "{}", formatted_log.message)
                .map_err(|e| format!("Failed to write log: {}", e))?;
        }

        Ok(())
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
