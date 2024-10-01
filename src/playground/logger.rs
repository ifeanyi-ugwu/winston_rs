use futures::future::join_all;
use futures::sink::Sink;
use futures::task::{Context, Poll};
use std::collections::VecDeque;
use std::io::{BufWriter, Write};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::thread;

use futures::channel::mpsc;
use futures::{SinkExt, Stream, StreamExt};

use super::transport::Transport;

// The Logger holds a channel to receive logs and send them down a pipeline
pub struct Logger {
    sender: mpsc::Sender<String>,
    buffer: Arc<Mutex<VecDeque<String>>>,
    transports: Arc<Mutex<Vec<Transport>>>,
    handle: Option<thread::JoinHandle<()>>,
}

impl Logger {
    pub fn new(/*transports: Vec<Transport>*/) -> Self {
        let (sender, receiver) = mpsc::channel(100); // Buffer up to 100 messages
        let buffer = Arc::new(Mutex::new(VecDeque::new()));
        //let transports = Arc::new(Mutex::new(transports));
        let transports: Arc<Mutex<Vec<Transport>>> = Arc::new(Mutex::new(Vec::new()));

        // Spawn a worker thread to handle log processing
        let buffer_clone = Arc::clone(&buffer);
        let transports_clone = Arc::clone(&transports);
        let handle = std::thread::spawn(move || {
            let mut receiver: mpsc::Receiver<String> = receiver;
            let mut internal_buffer = buffer_clone.lock().unwrap();

            while let Some(log) = futures::executor::block_on(receiver.next()) {
                internal_buffer.push_back(log.clone());

                // Dispatch to each transport
                /*let mut transports = transports_clone.lock().unwrap();
                for transport in transports.iter_mut() {
                    let _ = futures::executor::block_on(transport.send(log.clone()));
                }*/
                let mut transports = transports_clone.lock().unwrap();
                // Collect all send operations into a single `Vec`
                let send_futures: Vec<_> = transports
                    .iter_mut()
                    .map(|transport| transport.send(log.clone())) // Create a future for each send
                    .collect();

                // Execute all sends concurrently
                let _ = futures::executor::block_on(join_all(send_futures));
            }
        });

        Logger {
            sender,
            buffer,
            transports,
            handle: Some(handle),
        }
    }

    // Log a message (non-blocking, does not require async)
    pub fn log(&self, message: &str) {
        let mut sender = self.sender.clone();
        let _ = futures::executor::block_on(sender.send(message.to_string()));
    }

    // Add a transport dynamically
    pub fn add_transport(&self, transport: Transport) {
        let mut transports = self.transports.lock().unwrap();
        transports.push(transport);
    }

    // Remove a transport (if needed)
    pub fn remove_transport(&self) -> Option<Transport> {
        let mut transports = self.transports.lock().unwrap();
        transports.pop()
    }

    fn transform_message(&mut self, message: String) -> String {
        // Example transformation: add a timestamp and upper-case the message
        let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S");
        format!("[{}] {}", timestamp, message.to_uppercase())
    }
}

impl Sink<String> for Logger {
    type Error = mpsc::SendError;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: String) -> Result<(), Self::Error> {
        let _ = self.sender.try_send(item); // Send message into channel buffer
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
}

impl Stream for Logger {
    type Item = String;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut buffer = self.buffer.lock().unwrap();

        // Emit messages from the internal buffer
        if let Some(log_message) = buffer.pop_front() {
            Poll::Ready(Some(log_message))
        } else {
            Poll::Pending
        }
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        // Close the sender to signal the receiver thread to exit
        self.sender.close_channel();

        // Wait for the background thread to finish processing logs
        if let Some(handle) = self.handle.take() {
            let _ = handle.join(); // Wait for the logging thread to complete
        }

        // Flush all transports
        /*let mut transports = self.transports.lock().unwrap();
        for transport in transports.iter_mut() {
            let _ = futures::executor::block_on(transport.flush());
        }*/
        // Flush all transports concurrently using `futures::future::join_all`
        let mut transports = self.transports.lock().unwrap();
        let flush_futures = transports.iter_mut().map(|transport| transport.flush());

        // Execute and block until all flush operations complete
        let _ = futures::executor::block_on(join_all(flush_futures));

        println!("Logger dropped and all transports flushed.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io;

    #[test]
    fn test_logger_with_dynamic_transports() {
        let file_transport = Transport::new(File::create("log.txt").unwrap());
        let console_transport = Transport::new(io::stdout());

        let another_file_transport = Transport::new(File::create("log2.txt").unwrap());

        // let logger = Logger::new(vec![file_transport, console_transport]);
        let logger = Logger::new();

        // Add transports dynamically
        logger.add_transport(file_transport);
        logger.add_transport(console_transport);

        logger.add_transport(another_file_transport);

        // Log some messages
        logger.log("This is a log message!");
        logger.log("Another log message");
    }
}
