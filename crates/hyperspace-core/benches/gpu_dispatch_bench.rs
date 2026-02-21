use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperspace_core::gpu::{
    batch_cosine_distance_cpu, batch_distance_auto, batch_l2_distance_cpu,
    batch_poincare_distance_cpu, GpuMetric,
};
use rand::Rng;

const DIM: usize = 768;
const BATCH: usize = 512;

fn gen_batch_vectors() -> (Vec<Vec<f64>>, Vec<f64>) {
    let mut rng = rand::thread_rng();
    let mut vectors = Vec::with_capacity(BATCH);
    for _ in 0..BATCH {
        let mut v = Vec::with_capacity(DIM);
        for _ in 0..DIM {
            v.push(rng.gen_range(-0.5..0.5));
        }
        vectors.push(v);
    }

    let mut query = Vec::with_capacity(DIM);
    for _ in 0..DIM {
        query.push(rng.gen_range(-0.5..0.5));
    }
    (vectors, query)
}

fn bench_metric(
    c: &mut Criterion,
    name: &str,
    metric: GpuMetric,
    cpu_fn: fn(&[&[f64]], &[f64]) -> Vec<f64>,
) {
    let (vectors, query) = gen_batch_vectors();
    let refs: Vec<&[f64]> = vectors.iter().map(Vec::as_slice).collect();
    let mut group = c.benchmark_group(name);

    group.bench_function("cpu_reference", |b| {
        b.iter(|| black_box(cpu_fn(black_box(&refs), black_box(&query))))
    });

    group.bench_function("auto_dispatch", |b| {
        b.iter(|| {
            let (dist, backend) = batch_distance_auto(metric, black_box(&refs), black_box(&query));
            black_box((dist, backend));
        })
    });

    group.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    // Safety: benchmark process sets env before worker threads start.
    unsafe {
        std::env::set_var("HS_GPU_BATCH_ENABLED", "true");
        std::env::set_var("HS_GPU_L2_ENABLED", "true");
        std::env::set_var("HS_GPU_COSINE_ENABLED", "true");
        std::env::set_var("HS_GPU_POINCARE_ENABLED", "true");
    }

    bench_metric(c, "batch_l2_dispatch", GpuMetric::L2, batch_l2_distance_cpu);
    bench_metric(
        c,
        "batch_cosine_dispatch",
        GpuMetric::Cosine,
        batch_cosine_distance_cpu,
    );
    bench_metric(
        c,
        "batch_poincare_dispatch",
        GpuMetric::Poincare,
        batch_poincare_distance_cpu,
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
