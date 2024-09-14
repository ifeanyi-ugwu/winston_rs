//use std::io::Write;
use std::io::{BufWriter, Write};
//use std::sync::{Arc, Mutex};
use std::sync::Arc;
use tokio::sync::mpsc::{self, Receiver, Sender};
use tokio::sync::Mutex;
use tokio::task;

// Define a global runtime, initialized on first use
use std::sync::Once;
use tokio::runtime::{Builder, Runtime};

static INIT: Once = Once::new();
static mut RUNTIME: Option<Runtime> = None;

pub fn initialize_runtime() {
    INIT.call_once(|| {
        let rt = Builder::new_multi_thread()
            //.worker_threads(4)
            .enable_all()
            .build()
            .expect("Failed to create runtime");
        unsafe {
            RUNTIME = Some(rt);
        }
    });
}

pub fn get_runtime() -> &'static Runtime {
    unsafe { RUNTIME.as_ref().expect("Runtime not initialized") }
}

#[derive(Debug)]
pub struct Transport<W: Write + Send + 'static> {
    writer: Arc<Mutex<BufWriter<W>>>,
    tx: Option<Sender<String>>,
    handle: Option<tokio::task::JoinHandle<()>>,
}

impl<W: Write + Send + 'static> Transport<W> {
    pub fn new(writer: W) -> Self {
        let (tx, mut rx) = mpsc::channel(1024);
        let writer = Arc::new(Mutex::new(BufWriter::new(writer)));

        // Spawn a background task to process log entries
        let writer_clone = Arc::clone(&writer);
        let handle = get_runtime().spawn(async move {
            while let Some(message) = rx.recv().await {
                let mut writer = writer_clone.lock().await;
                // writeln!(writer, "{}", message).unwrap();
                //writer.flush().unwrap();
                if writeln!(writer, "{}", message).is_ok() {
                    let _ = writer.flush(); // Flush after each log to avoid delays
                }
            }
        });

        Transport {
            writer,
            tx: Some(tx),
            handle: Some(handle),
        }
    }

    // Synchronous API for the user, but internally handles the log asynchronously
    /*pub fn log(&self, message: String) {
        // Send the log message to the background task
        //let _ = self.tx.try_send(message);
        if let Err(e) = self.tx.try_send(message) {
            eprintln!("Failed to send log message: {:?}", e); // Handle log failure
        }
    }*/
    pub fn log(&self, message: String) {
        if let Some(ref tx) = self.tx {
            if let Err(e) = tx.try_send(message) {
                eprintln!("Failed to send log message: {:?}", e); // Handle log failure
            }
        } else {
            eprintln!("Logger is not available");
        }
    }
}

//drop is unfinished, it causes the program not to exit
//this is not the reason for abandonment, but because before implementing drop,
//it was slower and used comparatively high memory(given it was not yet processing all the messages, and not yet applying the formatting)
//than the other implementations
impl<W: Write + Send + 'static> Drop for Transport<W> {
    fn drop(&mut self) {
        // Close the channel
        // self.tx.closed();
        if let Some(tx) = self.tx.take() {
            // Take the handle out of the Option
            if let Some(handle) = self.handle.take() {
                // Block on the handle in a blocking task
                get_runtime().block_on(async {
                    // Await the closing of the channel
                    //self.tx.closed().await;
                    tx.closed().await;

                    if let Err(e) = handle.await {
                        eprintln!("Error joining logger task: {:?}", e);
                    }
                });
            }
        }
    }
}

// File transport
use std::fs::File;
//use std::io::{BufWriter, Write};

pub fn create_file_transport(file_path: &str) -> Transport<File> {
    let file = File::create(file_path).expect("Failed to create log file");
    Transport::new(file)
}

// Console transport
use std::io::{self};

pub fn create_console_transport() -> Transport<io::Stdout> {
    Transport::new(io::stdout())
}
