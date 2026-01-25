use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperspace_core::vector::HyperVector;
use rand::Rng;

const DIM: usize = 128; // Multiple of 8

fn criterion_benchmark(c: &mut Criterion) {
    let mut rng = rand::thread_rng();

    // Generate two random vectors inside the unit ball
    let v1_data: [f64; DIM] =
        core::array::from_fn(|_| rng.gen_range(-0.5..0.5) / (DIM as f64).sqrt());
    let v2_data: [f64; DIM] =
        core::array::from_fn(|_| rng.gen_range(-0.5..0.5) / (DIM as f64).sqrt());

    let v1 = HyperVector::<DIM>::new(v1_data).unwrap();
    let v2 = HyperVector::<DIM>::new(v2_data).unwrap();

    let mut group = c.benchmark_group("poincare_distance");

    // Bench SIMD implementation
    group.bench_function("simd_f64x8", |b| {
        b.iter(|| black_box(v1.poincare_distance_sq(black_box(&v2))))
    });

    // Bench Naive implementation for comparison
    group.bench_function("scalar_naive", |b| {
        b.iter(|| {
            // Naive calculation inline to avoid adding code to the crate just for testing
            let mut sum_sq = 0.0;
            for i in 0..DIM {
                let diff = v1.coords[i] - v2.coords[i];
                sum_sq += diff * diff;
            }
            let delta = sum_sq * v1.alpha * v2.alpha;
            black_box(1.0 + 2.0 * delta)
        })
    });

    group.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
