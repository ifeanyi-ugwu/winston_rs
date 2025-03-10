use std::sync::{Arc, Mutex};
use winston::format::LogInfo;
use winston::{log, Logger};
use winston_transport::Transport;

#[derive(Clone)]
struct MockTransport {
    logs: Arc<Mutex<Vec<String>>>,
}

impl MockTransport {
    fn new() -> Self {
        MockTransport {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    fn get_logs(&self) -> Vec<String> {
        self.logs.lock().unwrap().clone()
    }
}

impl Transport for MockTransport {
    fn log(&self, info: LogInfo) {
        let mut logs = self.logs.lock().unwrap();
        logs.push(format!("{} - {}", info.level, info.message));
        //println!("Logging: {} - {}", info.level, info.message);
    }

    fn get_level(&self) -> Option<&String> {
        None
    }

    fn get_format(&self) -> Option<&winston::format::Format> {
        None
    }
}

#[test]
fn test_add_and_remove_transport() {
    let mock_transport1 = Arc::new(MockTransport::new());
    let mock_transport2 = Arc::new(MockTransport::new());

    let logger = Logger::builder().level("info").build();

    assert!(logger.add_transport(Arc::clone(&mock_transport1) as Arc<dyn Transport>));
    assert!(logger.add_transport(Arc::clone(&mock_transport2) as Arc<dyn Transport>));

    log!(logger, info, "Message before removing transport");

    logger.flush().unwrap();

    assert!(mock_transport1
        .get_logs()
        .iter()
        .any(|log| log.contains("Message before removing transport")));
    assert!(mock_transport2
        .get_logs()
        .iter()
        .any(|log| log.contains("Message before removing transport")));

    assert!(logger.remove_transport(Arc::clone(&mock_transport1) as Arc<dyn Transport>));
    assert!(!logger.remove_transport(Arc::clone(&mock_transport1) as Arc<dyn Transport>));

    log!(logger, info, "Message after removing transport");

    logger.flush().unwrap();

    assert!(!mock_transport1
        .get_logs()
        .iter()
        .any(|log| log.contains("Message after removing transport")));
    assert!(mock_transport2
        .get_logs()
        .iter()
        .any(|log| log.contains("Message after removing transport")));
}
