// #[global_allocator]
// static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let cpu_cores = std::thread::available_parallelism().map_or(4, std::num::NonZeroUsize::get);
    let threads = std::env::var("HS_WORKER_THREADS")
        .ok()
        .and_then(|v| {
            let trimmed = v.trim();
            if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("auto") || trimmed == "0" {
                None
            } else {
                trimmed.parse::<usize>().ok()
            }
        })
        .unwrap_or_else(|| cpu_cores.clamp(2, 128));

    let mut builder = tokio::runtime::Builder::new_multi_thread();
    builder
        .worker_threads(threads)
        .enable_all()
        .thread_name("hyperspace-core");

    // NUMA-aware placement: spread runtime threads evenly across sockets and keep each
    // thread (and its first-touch allocations) on a single node. No-op on 1-node hosts.
    if let Some(topo) = hyperspace_server::numa::topology_from_env() {
        println!(
            "🧭 [NUMA] {} nodes detected - pinning runtime threads per node (disable with HS_NUMA=0)",
            topo.node_count()
        );
        builder.on_thread_start(hyperspace_server::numa::thread_start_hook(topo));
    }

    let rt = builder.build()?;

    rt.block_on(hyperspace_server::main_entry())
}
