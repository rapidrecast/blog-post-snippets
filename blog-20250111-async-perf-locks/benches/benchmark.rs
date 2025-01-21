use blog_20250111_async_perf_locks::async_read_single::AsyncReadSingleEngine;
use blog_20250111_async_perf_locks::async_write_single::AsyncWriteSingleEngine;
use blog_20250111_async_perf_locks::blocking_read_single::BlockingReadSingleEngine;
use blog_20250111_async_perf_locks::blocking_write_single::BlockingWriteSingleEngine;
use blog_20250111_async_perf_locks::std_channel_single::StdChannelSingleEngine;
use blog_20250111_async_perf_locks::{Engine, RequestHandler};
use criterion::{criterion_group, criterion_main, Bencher, Criterion};
use std::future::Future;
use tokio::runtime::Runtime;

pub fn benchmarks(_c: &mut Criterion) {
    let mut c = Criterion::default()
        .measurement_time(std::time::Duration::from_secs(60));
    let mut group = c.benchmark_group("Async perf workload approaches");
    let rt = Runtime::new().unwrap(); // Create a Tokio runtime for async execution

    group.bench_function("Async Read Single Benchmark", |b| {
        run_benchmark(b, &rt, AsyncReadSingleEngine::default());
    });

    group.bench_function("Async Write Single Benchmark", |b| {
        run_benchmark(b, &rt, AsyncWriteSingleEngine::default());
    });

    group.bench_function("Blocking Read Single Benchmark", |b| {
        run_benchmark(b, &rt, BlockingReadSingleEngine::default());
    });

    group.bench_function("Blocking Write Single Benchmark", |b| {
        run_benchmark(b, &rt, BlockingWriteSingleEngine::default());
    });

    group.bench_function("Std Channel Single Benchmark", |b| {
        run_benchmark(b, &rt, StdChannelSingleEngine::default());
    });

    group.finish()
}

pub fn run_benchmark<E: Engine>(b: &mut Bencher, rt: &Runtime, engine: E) {
    // Prepare the shared engine and handlers
    const scale: usize = 100_000;
    let handler1 = RequestHandler::<scale, _> {
        engine: engine.clone(),
    };
    let handler2 = RequestHandler::<scale, _> {
        engine: engine.clone(),
    };
    let handler3 = RequestHandler::<scale, _> {
        engine: engine.clone(),
    };

    // Benchmark the handlers running concurrently
    b.to_async(rt)
        .iter(|| async {
            tokio::join!(
                    handler1.simulate_workload(),
                    handler2.simulate_workload(),
                    handler3.simulate_workload()
                );
        });
}

// Criterion boilerplate
criterion_group!(benches, benchmarks);
criterion_main!(benches);
