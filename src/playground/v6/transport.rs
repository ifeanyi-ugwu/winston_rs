use futures::{Sink, SinkExt};
use std::fmt;
use std::io::{BufWriter, Write};
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll};
use tokio::io::{AsyncWrite, AsyncWriteExt};

#[derive(Clone)]
pub struct TransportOptions {
    format: Option<String>,
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
            .field("format", &self.format)
            .field("level", &self.level)
            .finish()
    }
}

pub struct Transport<W: Write + Send + 'static> {
    writer: Arc<Mutex<BufWriter<W>>>, // Buffered Writer
    options: TransportOptions,
}

impl<W: Write + Send + 'static> Transport<W> {
    pub fn new(writer: W, options: Option<TransportOptions>) -> Self {
        let buffered_writer = BufWriter::new(writer);
        Transport {
            writer: Arc::new(Mutex::new(buffered_writer)),
            options: options.unwrap_or_default(),
        }
    }

    // Helper method for synchronous writes
    pub fn log_sync(&self, message: &str) {
        let mut writer = self.writer.lock().unwrap();
        writeln!(writer, "{}", message).unwrap();
    }
}

// Implement the Sink trait for the Transport
impl<W: Write + Send + 'static> Sink<String> for Transport<W> {
    type Error = std::io::Error;

    fn poll_ready(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(())) // Always ready to accept new messages
    }

    fn start_send(self: Pin<&mut Self>, item: String) -> Result<(), Self::Error> {
        let mut writer = self.writer.lock().unwrap();
        writeln!(writer, "{}", item)?; // Write the log message synchronously
        Ok(())
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        let mut writer = self.writer.lock().unwrap();
        writer.flush()?; // Flush the writer
        Poll::Ready(Ok(()))
    }

    fn poll_close(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(_cx) // Flush on close
    }
}
