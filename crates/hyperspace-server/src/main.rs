#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

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
        .unwrap_or_else(|| cpu_cores.clamp(2, 8));

    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(threads)
        .enable_all()
        .thread_name("hyperspace-core")
        .build()?;

    rt.block_on(hyperspace_server::main_entry())
}
