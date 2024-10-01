use futures::sink::Sink;
use futures::task::{Context, Poll};
use std::io::{BufWriter, Write};
use std::pin::Pin;
use std::sync::{Arc, Mutex};

pub struct Transport {
    writer: Arc<Mutex<Box<dyn Write + Send>>>,
}

impl Transport {
    pub fn new<W: Write + Send + 'static>(writer: W) -> Self {
        Transport {
            writer: Arc::new(Mutex::new(Box::new(BufWriter::new(writer)))),
        }
    }
}

impl Sink<String> for Transport {
    type Error = std::io::Error;

    fn poll_ready(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn start_send(mut self: Pin<&mut Self>, item: String) -> Result<(), Self::Error> {
        let mut writer = self.writer.lock().unwrap();
        writeln!(writer, "{}", item)?;
        Ok(())
    }

    fn poll_flush(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        let mut writer = self.writer.lock().unwrap();
        writer.flush()?;
        Poll::Ready(Ok(()))
    }

    fn poll_close(
        mut self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Result<(), Self::Error>> {
        self.poll_flush(_cx)
    }
}
