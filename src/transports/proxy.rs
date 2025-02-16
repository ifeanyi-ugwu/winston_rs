use logform::LogInfo;
use std::{sync::Arc, thread, time::Duration};
use winston_transport::{LogQuery, Transport};

pub trait Proxy: Transport {
    /// Proxies logs from this transport to another transport
    /// Returns the number of logs transferred
    fn proxy(&self, target: &dyn Proxy) -> Result<usize, String>;

    /// Ingest logs in batch
    fn ingest(&self, logs: Vec<LogInfo>) -> Result<(), String>;
}

pub struct ProxyTransport {
    source_transport: Arc<dyn Proxy>,
    target_transport: Arc<dyn Proxy>,
}

impl ProxyTransport {
    pub fn new(
        source_transport: Arc<dyn Proxy>,
        target_transport: Arc<dyn Proxy>,
        delegation_interval: Duration,
    ) -> Self {
        let source_transport_clone = source_transport.clone();
        let target_transport_clone = target_transport.clone();

        thread::spawn(move || loop {
            thread::sleep(delegation_interval);

            let _ = source_transport_clone.proxy(&*target_transport_clone);
        });

        Self {
            source_transport,
            target_transport,
        }
    }
}

impl Transport for ProxyTransport {
    fn log(&self, info: LogInfo) {
        self.source_transport.log(info);
    }

    fn query(&self, options: &LogQuery) -> Result<Vec<LogInfo>, String> {
        let mut logs = self.source_transport.query(options)?;
        logs.extend(self.target_transport.query(options)?);
        Ok(logs)
    }

    fn flush(&self) -> Result<(), String> {
        self.source_transport.flush()?;
        self.target_transport.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use logform::{json, timestamp};
    use std::sync::Mutex;

    struct MockTransport {
        logs: Arc<Mutex<Vec<LogInfo>>>,
    }

    impl MockTransport {
        fn new() -> Self {
            Self {
                logs: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl Proxy for MockTransport {
        fn proxy(&self, target: &dyn Proxy) -> Result<usize, String> {
            let mut logs = self.logs.lock().unwrap();
            let count = logs.len();

            target.ingest(logs.clone())?;

            // Clear source
            logs.clear();

            Ok(count)
        }

        fn ingest(&self, logs: Vec<LogInfo>) -> Result<(), String> {
            let mut storage = self.logs.lock().unwrap();
            storage.extend(logs);
            Ok(())
        }
    }

    impl Transport for MockTransport {
        fn log(&self, info: LogInfo) {
            let _ = self.ingest(vec![info]);
        }

        fn query(&self, _options: &LogQuery) -> Result<Vec<LogInfo>, String> {
            let logs = self.logs.lock().unwrap();
            Ok(logs.clone())
        }
    }

    #[test]
    fn test_proxy_transport_moves_logs_from_source_to_target() -> Result<(), String> {
        let source_transport = Arc::new(MockTransport::new());
        let target_transport = Arc::new(MockTransport::new());

        let log = LogInfo::new("test", "Test message");
        let log = timestamp().transform(log.clone(), None).unwrap();
        let log = json().transform(log.clone(), None).unwrap();

        let delegation_interval = Duration::from_secs(1);
        let proxy_transport = ProxyTransport::new(
            source_transport.clone(),
            target_transport.clone(),
            delegation_interval,
        );

        proxy_transport.log(log);

        // Wait for the delegation to complete
        thread::sleep(delegation_interval * 2);

        let source_logs_after = source_transport.query(&LogQuery::new())?;
        let target_logs_after = target_transport.query(&LogQuery::new())?;

        assert!(source_logs_after.is_empty(), "Source should be empty");
        assert_eq!(target_logs_after.len(), 1, "Target should have 1 log");

        Ok(())
    }
}
