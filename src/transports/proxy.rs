use logform::LogInfo;
use std::{sync::Arc, thread, time::Duration};
use winston_transport::{LogQuery, Transport};

pub struct ProxyTransport {
    source_transport: Arc<dyn Transport>,
    target_transport: Arc<dyn Transport>,
}

impl ProxyTransport {
    pub fn new(
        primary_transport: Arc<dyn Transport>,
        secondary_transport: Arc<dyn Transport>,
        delegation_interval: Duration,
    ) -> Self {
        let primary_transport_clone = primary_transport.clone();
        let secondary_transport_clone = secondary_transport.clone();

        thread::spawn(move || loop {
            thread::sleep(delegation_interval);

            if let Ok(logs) = primary_transport_clone.query(&LogQuery::new()) {
                if !logs.is_empty() {
                    for log in logs {
                        secondary_transport_clone.log(log.clone());
                    }
                }
            }
        });

        Self {
            source_transport: primary_transport,
            target_transport: secondary_transport,
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
    use std::sync::{Arc, Mutex};
    use winston_transport::{LogQuery, Transport};

    struct MockTransport {
        logs: Arc<Mutex<Vec<LogInfo>>>,
    }

    impl MockTransport {
        fn new() -> Self {
            MockTransport {
                logs: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl Transport for MockTransport {
        fn log(&self, info: LogInfo) {
            let mut logs = self.logs.lock().unwrap();
            logs.push(info);
        }

        fn query(&self, _options: &LogQuery) -> Result<Vec<LogInfo>, String> {
            let logs = self.logs.lock().unwrap();
            Ok(logs.clone())
        }
    }

    #[test]
    fn test_logs_are_forwarded_to_target_transport() {
        let source_transport = Arc::new(MockTransport::new());
        let target_transport = Arc::new(MockTransport::new());

        let delegation_interval = Duration::from_secs(1);
        let proxy_transport = ProxyTransport::new(
            source_transport.clone(),
            target_transport.clone(),
            delegation_interval,
        );

        let log_info = LogInfo::new("info", "Test message");

        proxy_transport.log(log_info.clone());

        // Wait for the delegation to complete
        thread::sleep(delegation_interval * 2);

        // Verify that the secondary transport received the log
        let secondary_logs = target_transport.query(&LogQuery::new()).unwrap();
        assert_eq!(
            secondary_logs.len(),
            1,
            "Secondary transport should have received the delegated log"
        );
        assert_eq!(
            secondary_logs[0].message, log_info.message,
            "Log content is correct"
        );
    }
}
