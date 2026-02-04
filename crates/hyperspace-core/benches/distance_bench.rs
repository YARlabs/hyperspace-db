use hyperspace_core::vector::HyperVector;

#[test]
fn bench_distance_speed() {
    let a = HyperVector::<1024>::new([0.001; 1024]).unwrap();
    let b = HyperVector::<1024>::new([0.002; 1024]).unwrap();

    let start = std::time::Instant::now();
    let iterations = 1_000_000;

    // "Warming up" the CPU cache
    let mut black_box = 0.0;

    for _ in 0..iterations {
        black_box += a.poincare_distance_sq(&b);
    }

    let duration = start.elapsed();
    println!(
        "⏱️ 1M distances took: {:?} (Check sum: {})",
        duration, black_box
    );
}
