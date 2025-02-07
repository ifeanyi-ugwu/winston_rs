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
