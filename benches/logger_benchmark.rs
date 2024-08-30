use criterion::{black_box, criterion_group, criterion_main, Criterion};
use winston::{LogEntry, Logger};

fn benchmark_logging(c: &mut Criterion) {
    let logger = Logger::new(None);

    c.bench_function("log_message", |b| {
        b.iter(|| {
            for _ in 0..1000 {
                logger.log(black_box(LogEntry::new("info", "benchmark message")));
            }
        })
    });
}

criterion_group!(benches, benchmark_logging);
criterion_main!(benches);
