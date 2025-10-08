use std::sync::Arc;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use logform::LogInfo;
use winston::Logger;

fn benchmark_logging(c: &mut Criterion) {
    let logger = Logger::builder()
        .add_transport(Arc::new(winston::transports::stdout()))
        .build();

    c.bench_function("log_message", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                logger.log(black_box(LogInfo::new("info", "benchmark message")));
            }
        })
    });

    // Generate a unique filename using the current system time
    let filename = format!(
        "test_log_{}.log",
        std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    );
    let file_transport = winston::transports::File::builder()
        .filename(&filename)
        .build();

    let logger = Logger::builder()
        //.add_transport(Arc::new(file_transport)) //TODO: uncomment when the file transport is updated
        .build();

    c.bench_function("log_message_to_file", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                logger.log(black_box(LogInfo::new("info", "benchmark message")));
            }
        })
    });

    // Delete the log file after the benchmark
    std::fs::remove_file(&filename).expect("Failed to delete log file");
}

criterion_group!(benches, benchmark_logging);
criterion_main!(benches);
