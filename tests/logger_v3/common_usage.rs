use std::{fs::File, io};

use logform::LogInfo;
//use logform::json;
use winston::logger_v3::{transports::transport::Transport, Logger};

#[test]
/*fn test_common_usage() {
    // Create individual transports
    let file_transport = Transport::new(std::fs::File::create("log.txt").unwrap(), None);
    let console_transport = Transport::new(std::io::stdout(), None);

    // Create a vector of transports
    let transports = vec![file_transport, console_transport];

    // Build the logger with all transports
    let logger = Logger::builder()
        .level("info")
        .format(json()) // Adjust as needed
        .transports(transports) // Set all transports at once
        .build();

    // Now you can use `logger` to log messages.

    logger.info("hi")
}*/
fn test_common_usage() {
    let file_transport = Transport::new(File::create("log.txt").unwrap(), None);
    let console_transport = Transport::new(io::stdout(), None);

    // Create logger and add transports
    let mut logger = Logger::new(None);
    logger.add_transport(file_transport);
    logger.add_transport(console_transport);

    // Log some messages
    //logger.log("Log message to file and console".to_string());
    // logger.log("Another log message".to_string());
    logger.log(LogInfo::new("info", "Log message to file and console"));
    logger.log(LogInfo::new("info", "Another log message"));
}

/*#[test]
fn test_logger_benchmarks() {
    use std::time::{Duration, Instant};

    fn benchmark_logger(logger: &Logger, message_count: usize) -> (Duration, usize) {
        let start = Instant::now();
        let mut successful_logs = 0;

        for i in 0..message_count {
            if logger.log(format!("Test message {}", i)).is_ok() {
                successful_logs += 1;
            }
        }

        let duration = start.elapsed();
        (duration, successful_logs)
    }

    // Usage
    let (duration, successful_logs) = benchmark_logger(&my_logger, 10000);
    println!("Logged {} messages in {:?}", successful_logs, duration);
    println!(
        "Throughput: {} messages/second",
        successful_logs as f64 / duration.as_secs_f64()
    );
}
*/
