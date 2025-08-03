use criterion::{black_box, criterion_group, criterion_main, Criterion};
use logform::LogInfo;
//use std::collections::HashMap;
//use std::sync::Arc;

mod oncelock_logger {
    use std::sync::OnceLock;
    use winston::Logger;

    static GLOBAL_LOGGER: OnceLock<Logger> = OnceLock::new();

    pub fn init(logger: Logger) {
        GLOBAL_LOGGER
            .set(logger)
            .expect("Logger already initialized");
    }

    fn global_logger() -> &'static Logger {
        GLOBAL_LOGGER.get_or_init(|| Logger::default())
    }

    pub fn log(entry: logform::LogInfo) {
        global_logger().log(entry);
    }

    pub fn configure(new_options: Option<winston::LoggerOptions>) {
        global_logger().configure(new_options)
    }

    pub fn flush() -> Result<(), String> {
        global_logger().flush()
    }
}

mod lazy_static_logger {
    use lazy_static::lazy_static;
    use logform::LogInfo;
    use winston::{Logger, LoggerOptions};

    lazy_static! {
        static ref GLOBAL_LOGGER: Logger = Logger::new(None);
    }

    pub fn log(entry: LogInfo) {
        GLOBAL_LOGGER.log(entry);
    }

    pub fn configure(options: Option<LoggerOptions>) {
        GLOBAL_LOGGER.configure(options);
    }

    pub fn flush() -> Result<(), String> {
        GLOBAL_LOGGER.flush()
    }
}

fn create_test_log_entry() -> LogInfo {
    //let mut entry = LogInfo::new();
    //entry.set_level("info".to_string());
    //entry.set_message("Test log message for benchmarking".to_string());
    //entry.set_timestamp(std::time::SystemTime::now());

    // Add some metadata
    //let mut meta = HashMap::new();
    //meta.insert("service".to_string(), "benchmark".into());
    //meta.insert("request_id".to_string(), "12345".into());
    //entry.set_meta(meta);

    let entry = LogInfo::new("info", "Test log message for benchmarking")
        .with_meta("service", "benchmark")
        .with_meta("request_id", "12345");

    entry
}

fn benchmark_oncelock_logging(c: &mut Criterion) {
    // Initialize the OnceLock logger
    oncelock_logger::init(winston::Logger::default());

    let log_entry = create_test_log_entry();

    c.bench_function("oncelock_single_log", |b| {
        b.iter(|| {
            oncelock_logger::log(black_box(log_entry.clone()));
        })
    });

    c.bench_function("oncelock_multiple_logs", |b| {
        b.iter(|| {
            for _ in 0..100 {
                oncelock_logger::log(black_box(log_entry.clone()));
            }
        })
    });

    c.bench_function("oncelock_configure", |b| {
        b.iter(|| {
            oncelock_logger::configure(black_box(None));
        })
    });

    c.bench_function("oncelock_flush", |b| {
        b.iter(|| {
            let _ = oncelock_logger::flush();
        })
    });
}

fn benchmark_lazy_static_logging(c: &mut Criterion) {
    let log_entry = create_test_log_entry();

    c.bench_function("lazy_static_single_log", |b| {
        b.iter(|| {
            lazy_static_logger::log(black_box(log_entry.clone()));
        })
    });

    c.bench_function("lazy_static_multiple_logs", |b| {
        b.iter(|| {
            for _ in 0..100 {
                lazy_static_logger::log(black_box(log_entry.clone()));
            }
        })
    });

    c.bench_function("lazy_static_configure", |b| {
        b.iter(|| {
            lazy_static_logger::configure(black_box(None));
        })
    });

    c.bench_function("lazy_static_flush", |b| {
        b.iter(|| {
            let _ = lazy_static_logger::flush();
        })
    });
}

fn benchmark_initialization_overhead(c: &mut Criterion) {
    c.bench_function("oncelock_first_access", |b| {
        b.iter(|| {
            // This simulates the first access overhead
            // Note: In real usage, you'd need to reset the OnceLock between iterations
            // but that's not possible, so this is more of a theoretical benchmark
            black_box(&oncelock_logger::flush());
        })
    });

    c.bench_function("lazy_static_first_access", |b| {
        b.iter(|| {
            // lazy_static initializes on first access
            black_box(&lazy_static_logger::flush());
        })
    });
}

fn benchmark_concurrent_access(c: &mut Criterion) {
    use std::sync::Arc;
    use std::thread;

    let log_entry = Arc::new(create_test_log_entry());

    c.bench_function("oncelock_concurrent_logging", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let entry = Arc::clone(&log_entry);
                    thread::spawn(move || {
                        for _ in 0..25 {
                            oncelock_logger::log((*entry).clone());
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });

    c.bench_function("lazy_static_concurrent_logging", |b| {
        b.iter(|| {
            let handles: Vec<_> = (0..4)
                .map(|_| {
                    let entry = Arc::clone(&log_entry);
                    thread::spawn(move || {
                        for _ in 0..25 {
                            lazy_static_logger::log((*entry).clone());
                        }
                    })
                })
                .collect();

            for handle in handles {
                handle.join().unwrap();
            }
        })
    });
}

criterion_group!(
    benches,
    benchmark_oncelock_logging,
    benchmark_lazy_static_logging,
    benchmark_initialization_overhead,
    benchmark_concurrent_access
);
criterion_main!(benches);
