mod file;
mod proxy;

pub use file::FileTransport as File;
pub use proxy::{Proxy, ProxyTransport};
use std::io;
pub use winston_transport::WriterTransport;

// Convenience functions
pub fn stdout() -> WriterTransport<io::Stdout> {
    WriterTransport::new(io::stdout())
}

pub fn stderr() -> WriterTransport<io::Stderr> {
    WriterTransport::new(io::stderr())
}
