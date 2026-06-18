use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperspace_cache::{CacheConfig, CacheMetricType, EvictionPolicy, VectorCache};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

fn bench_cache_ops(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let _guard = rt.enter();

    let config = CacheConfig {
        l1_capacity: 1000,
        eviction_policy: EvictionPolicy::Lru,
        ann_threshold: Some(10.0),
        ann_rebuild_batch: 50,
        collection_name: "bench_collection".to_string(),
        dimension: 128,
        metric: CacheMetricType::L2,
    };

    let cache = Arc::new(VectorCache::new(config));
    cache.start_background_tasks();

    // Seed some data
    let mut rng_val = 0.0;
    for i in 0..100 {
        let vec = vec![rng_val; 128];
        rng_val += 0.01;
        cache.insert(i, vec, HashMap::new(), None);
    }

    // Build/rebuild L2 ANN index synchronously so L2 search has data
    rt.block_on(async {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    });

    c.bench_function("cache_exact_hit", |b| {
        b.iter(|| {
            let res = cache.search(black_box(&[0.0; 128]), black_box(Some(5)), black_box(1));
            black_box(res);
        })
    });

    c.bench_function("cache_ann_search", |b| {
        b.iter(|| {
            let res = cache.search(black_box(&[0.5; 128]), black_box(None), black_box(5));
            black_box(res);
        })
    });

    c.bench_function("cache_insert", |b| {
        let mut id = 1000;
        b.iter(|| {
            id += 1;
            cache.insert(
                black_box(id),
                black_box(vec![0.1; 128]),
                black_box(HashMap::new()),
                black_box(None),
            );
            black_box(());
        })
    });
}

criterion_group!(benches, bench_cache_ops);
criterion_main!(benches);
