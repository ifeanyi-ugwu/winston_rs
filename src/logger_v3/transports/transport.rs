//use crossbeam_channel::{bounded, unbounded, Receiver, Sender};
use logform::Format;
use std::fmt;
use std::io::{BufWriter, Write};
use std::sync::{
    // mpsc::{channel, Sender},
    Arc,
    Mutex,
};
//use std::thread;

#[derive(Clone)]
pub struct TransportOptions {
    format: Option<Format>,
    level: Option<String>,
}

impl Default for TransportOptions {
    fn default() -> Self {
        TransportOptions {
            format: None,
            level: None,
        }
    }
}

impl fmt::Debug for TransportOptions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("TransportOptions")
            .field("format", &self.format.as_ref().map(|_| "Format instance")) // Placeholder, adjust as needed
            .field("level", &self.level)
            .finish()
    }
}

pub struct Transport {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
    //tx: Option<Sender<String>>,
    options: TransportOptions,
    //worker_handle: Option<thread::JoinHandle<()>>,
}

impl fmt::Debug for Transport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Custom implementation of Debug trait
        // We cannot directly print `writer` since it doesn't implement `Debug`.
        // So we only print the `tx` sender's state in this example.
        f.debug_struct("Transport")
            .field("tx", &"Sender") // Representing sender generically as it can't be debugged directly.
            .finish()
    }
}

impl Transport {
    pub fn new<W: Write + Send + 'static>(writer: W, options: Option<TransportOptions>) -> Self {
        //let (tx, rx) = channel();
        //let (tx, rx) = unbounded();
        let buffered_writer = BufWriter::new(writer);
        let writer = Arc::new(Mutex::new(
            Box::new(buffered_writer) as Box<dyn Write + Send>
        ));

        //*new thread per transport is not ideal and will not scale well, manage the thread in the central logger */
        // Spawn a background thread to process log entries
        //let writer_clone = Arc::clone(&writer);

        /*let worker_handle = thread::spawn(move || {
            for message in rx {
                if let Ok(mut writer) = writer_clone.lock() {
                    if writeln!(writer, "{}", message).is_ok() {
                        // let _ = writer.flush(); // Flush periodically or when necessary
                    }
                }
            }
        });*/

        Transport {
            writer,
            //tx: Some(tx),
            options: options.unwrap_or_default(),
            //worker_handle: Some(worker_handle),
        }
    }

    /*pub fn log(&self, message: String) {
        if let Some(tx) = &self.tx {
            if let Err(e) = tx.send(message) {
                eprintln!("Failed to send log message: {:?}", e);
            }
        }
    }*/

    pub fn log(&self, message: String) {
        if let Ok(mut writer) = self.writer.lock() {
            writeln!(writer, "{}", message).unwrap(); // Write the log message
            let _ = writer.flush(); // Flush to ensure the message is written
        }
    }

    pub fn get_format(&self) -> Option<&Format> {
        self.options.format.as_ref()
    }

    pub fn get_level(&self) -> Option<&String> {
        self.options.level.as_ref()
    }
}

/*impl Drop for Transport {
    fn drop(&mut self) {
        // Drop the sender to close the channel
        self.tx.take();

        // Ensure writer is properly flushed and closed
        //let _ = self.writer.lock().unwrap().flush();

        // Join the worker thread to ensure it has completed
        if let Some(handle) = self.worker_handle.take() {
            if let Err(e) = handle.join() {
                eprintln!("Failed to join worker thread: {:?}", e);
            }
        }
    }
}*/

// unused yet but
/*impl Write for Transport {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut writer = self.writer.lock().unwrap();
        writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        let mut writer = self.writer.lock().unwrap();
        writer.flush()
    }
}*/
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io;

    #[test]
    fn test_common_usage() {
        let file_transport = Transport::new(File::create("log.txt").unwrap(), None);
        let console_transport = Transport::new(io::stdout(), None);

        // Log a message
        file_transport.log("Log message to file".to_string());
        console_transport.log("Log message to console".to_string());
    }
}
