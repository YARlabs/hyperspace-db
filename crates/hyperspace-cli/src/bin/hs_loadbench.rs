//! `hs_loadbench`: closed-loop concurrent load benchmark for HyperspaceDB (gRPC).
//!
//! Measures what a single-client harness cannot: throughput under concurrency, tail latency
//! (p50 / p95 / p99 / p99.9) and recall@k against exact ground truth, for unary `Search`
//! and batched `SearchBatch`. Each client is a tokio task with its own HTTP/2 connection
//! (`--connections` to share fewer), so the client side never becomes the bottleneck the
//! way Python clients do.
//!
//! ```text
//! # spawn a server on a temp data dir, ingest 20k×1024 clustered vectors, sweep clients
//! cargo run --release -p hyperspace-cli --bin hs_loadbench -- \
//!     --server-bin target/release/hyperspace-server --quant medium \
//!     --n 20000 --dim 1024 --clients 1,4,16,32 --duration 8
//!
//! # against a running server (collection created and filled by the tool)
//! hs_loadbench --addr http://127.0.0.1:50051 --api-key KEY --n 100000 --batch 32
//! ```
//!
//! Server-side knobs (rerank, ef, M, ...) are passed to a spawned server with
//! `--env HS_RERANK_ENABLED=1 --env HS_RERANK_OVERSAMPLE=4`. `--json out.json` writes the
//! result rows for scripts / CI gates (`--min-recall`, `--min-qps` fail the run).

use hyperspace_proto::hyperspace::database_client::DatabaseClient;
use hyperspace_proto::hyperspace::{
    BatchInsertRequest, BatchSearchRequest, CollectionSchema, CollectionStatsRequest,
    CreateCollectionRequest, SearchRequest, VectorComponent, VectorData,
};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use std::collections::HashSet;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tonic::metadata::MetadataValue;
use tonic::transport::Channel;
use tonic::Request;

#[derive(Clone, Debug)]
struct Opts {
    addr: String,
    api_key: Option<String>,
    server_bin: Option<String>,
    server_env: Vec<(String, String)>,
    port: u16,
    collection: String,
    quant: String,
    metric: String,
    n: usize,
    dim: usize,
    queries: usize,
    clusters: usize,
    clients: Vec<usize>,
    connections: usize,
    duration: f64,
    warmup: f64,
    batch: usize,
    top_k: u32,
    ef: Option<u32>,
    compact: bool,
    skip_ingest: bool,
    rebuild: bool,
    /// Sent as `x-expected-vectors` on create (shard count); `None` = `n`, `Some(0)` = omit.
    expected_vectors: Option<u64>,
    json: Option<String>,
    min_recall: Option<f64>,
    min_qps: Option<f64>,
    label: String,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            addr: "http://127.0.0.1:50051".into(),
            api_key: std::env::var("HYPERSPACE_API_KEY").ok(),
            server_bin: None,
            server_env: Vec::new(),
            port: 52151,
            collection: "loadbench".into(),
            quant: "medium".into(),
            metric: "cosine".into(),
            n: 20_000,
            dim: 1024,
            queries: 500,
            clusters: 50,
            clients: vec![1, 4, 16, 32],
            connections: 0,
            duration: 6.0,
            warmup: 1.0,
            batch: 0,
            top_k: 10,
            ef: None,
            compact: true,
            skip_ingest: false,
            rebuild: false,
            expected_vectors: None,
            json: None,
            min_recall: None,
            min_qps: None,
            label: String::new(),
        }
    }
}

const USAGE: &str = "hs_loadbench [--addr URL | --server-bin PATH [--port P] [--env K=V]...]
  [--api-key KEY] [--collection NAME] [--quant medium|none|...] [--metric cosine|l2]
  [--n N] [--dim D] [--queries Q] [--clusters C] [--clients 1,4,16]
  [--connections K (0 = one per client)] [--duration SECS] [--warmup SECS]
  [--batch B (0 = unary Search)] [--top-k K] [--ef EF] [--f64-wire] [--skip-ingest] [--rebuild]
  [--expected-vectors N (default --n; 0 = no size hint)]
  [--label TEXT] [--json PATH] [--min-recall R] [--min-qps Q]";

fn parse_args() -> Result<Opts, String> {
    let mut o = Opts::default();
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        let mut val = || it.next().ok_or_else(|| format!("{a} needs a value"));
        match a.as_str() {
            "--addr" => o.addr = val()?,
            "--api-key" => o.api_key = Some(val()?),
            "--server-bin" => o.server_bin = Some(val()?),
            "--env" => {
                let kv = val()?;
                let (k, v) = kv.split_once('=').ok_or("--env expects K=V")?;
                o.server_env.push((k.to_string(), v.to_string()));
            }
            "--port" => o.port = val()?.parse().map_err(|e| format!("--port: {e}"))?,
            "--collection" => o.collection = val()?,
            "--quant" => o.quant = val()?,
            "--metric" => o.metric = val()?,
            "--n" => o.n = val()?.parse().map_err(|e| format!("--n: {e}"))?,
            "--dim" => o.dim = val()?.parse().map_err(|e| format!("--dim: {e}"))?,
            "--queries" => o.queries = val()?.parse().map_err(|e| format!("--queries: {e}"))?,
            "--clusters" => o.clusters = val()?.parse().map_err(|e| format!("--clusters: {e}"))?,
            "--clients" => {
                o.clients = val()?
                    .split(',')
                    .map(|c| c.trim().parse::<usize>())
                    .collect::<Result<_, _>>()
                    .map_err(|e| format!("--clients: {e}"))?;
            }
            "--connections" => {
                o.connections = val()?.parse().map_err(|e| format!("--connections: {e}"))?;
            }
            "--duration" => o.duration = val()?.parse().map_err(|e| format!("--duration: {e}"))?,
            "--warmup" => o.warmup = val()?.parse().map_err(|e| format!("--warmup: {e}"))?,
            "--batch" => o.batch = val()?.parse().map_err(|e| format!("--batch: {e}"))?,
            "--top-k" => o.top_k = val()?.parse().map_err(|e| format!("--top-k: {e}"))?,
            "--ef" => o.ef = Some(val()?.parse().map_err(|e| format!("--ef: {e}"))?),
            "--f64-wire" => o.compact = false,
            "--skip-ingest" => o.skip_ingest = true,
            "--rebuild" => o.rebuild = true,
            "--expected-vectors" => {
                o.expected_vectors = Some(
                    val()?
                        .parse()
                        .map_err(|e| format!("--expected-vectors: {e}"))?,
                );
            }
            "--label" => o.label = val()?,
            "--json" => o.json = Some(val()?),
            "--min-recall" => {
                o.min_recall = Some(val()?.parse().map_err(|e| format!("--min-recall: {e}"))?);
            }
            "--min-qps" => o.min_qps = Some(val()?.parse().map_err(|e| format!("--min-qps: {e}"))?),
            "-h" | "--help" => return Err(USAGE.to_string()),
            other => return Err(format!("unknown argument {other}\n{USAGE}")),
        }
    }
    if o.clients.is_empty() || o.clients.contains(&0) {
        return Err("--clients must be positive".into());
    }
    if o.server_bin.is_some() {
        o.addr = format!("http://127.0.0.1:{}", o.port);
        o.api_key.get_or_insert_with(|| "hs_loadbench".to_string());
    }
    Ok(o)
}

/// Standard normal via Box–Muller (keeps the tool free of extra dependencies).
fn normal(rng: &mut StdRng) -> f32 {
    let u1: f64 = rng.gen_range(f64::EPSILON..1.0);
    let u2: f64 = rng.gen();
    ((-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()) as f32
}

fn normalize(v: &mut [f32]) {
    let n = v.iter().map(|x| x * x).sum::<f32>().sqrt().max(1e-12);
    v.iter_mut().for_each(|x| *x /= n);
}

fn centroids(dim: usize, clusters: usize, rng: &mut StdRng) -> Vec<Vec<f32>> {
    (0..clusters.max(1))
        .map(|_| {
            let mut c: Vec<f32> = (0..dim).map(|_| normal(rng)).collect();
            normalize(&mut c);
            c
        })
        .collect()
}

/// Clustered unit vectors (semantic-manifold-like): `n` points drawn around shared
/// centroids, deterministic per seed. `stream` separates data (0) from queries (1).
fn clustered(n: usize, dim: usize, clusters: usize, seed: u64, stream: u64) -> Vec<Vec<f32>> {
    let cents = centroids(dim, clusters, &mut StdRng::seed_from_u64(seed));
    let spread = 1.5 / (dim as f32).sqrt();
    let mut rng = StdRng::seed_from_u64(seed ^ (stream + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
    (0..n)
        .map(|_| {
            let c = &cents[rng.gen_range(0..cents.len())];
            let mut v: Vec<f32> = c.iter().map(|&x| x + spread * normal(&mut rng)).collect();
            normalize(&mut v);
            v
        })
        .collect()
}

/// Exact top-k by squared L2 (rank-equivalent to cosine on unit vectors), multi-threaded.
fn ground_truth(data: &[Vec<f32>], qs: &[Vec<f32>], k: usize) -> Vec<HashSet<u32>> {
    let threads = std::thread::available_parallelism().map_or(4, |n| n.get());
    let chunk = qs.len().div_ceil(threads).max(1);
    let mut out = vec![HashSet::new(); qs.len()];
    std::thread::scope(|s| {
        for (qc, oc) in qs.chunks(chunk).zip(out.chunks_mut(chunk)) {
            s.spawn(move || {
                for (q, o) in qc.iter().zip(oc.iter_mut()) {
                    let mut best: Vec<(f32, u32)> = data
                        .iter()
                        .enumerate()
                        .map(|(i, v)| {
                            let d: f32 = v.iter().zip(q).map(|(a, b)| (a - b) * (a - b)).sum();
                            (d, i as u32)
                        })
                        .collect();
                    let nth = k.min(best.len() - 1);
                    best.select_nth_unstable_by(nth, |a, b| a.0.total_cmp(&b.0));
                    best.truncate(k);
                    *o = best.into_iter().map(|x| x.1).collect();
                }
            });
        }
    });
    out
}

/// Adds `x-api-key` to every request (cloneable, so connections can be shared).
#[derive(Clone)]
struct ApiKey(Option<MetadataValue<tonic::metadata::Ascii>>);

impl tonic::service::Interceptor for ApiKey {
    fn call(&mut self, mut r: Request<()>) -> Result<Request<()>, tonic::Status> {
        if let Some(k) = &self.0 {
            r.metadata_mut().insert("x-api-key", k.clone());
        }
        Ok(r)
    }
}

type Client = DatabaseClient<tonic::codegen::InterceptedService<Channel, ApiKey>>;

async fn connect(o: &Opts) -> Result<Client, Box<dyn std::error::Error>> {
    let ch = Channel::from_shared(o.addr.clone())?
        .tcp_nodelay(true)
        .connect_timeout(Duration::from_secs(10))
        .connect()
        .await?;
    let key = ApiKey(o.api_key.as_deref().and_then(|k| k.parse().ok()));
    Ok(DatabaseClient::with_interceptor(ch, key)
        .max_decoding_message_size(256 << 20)
        .max_encoding_message_size(256 << 20))
}

fn search_req(o: &Opts, q: &[f32]) -> SearchRequest {
    SearchRequest {
        collection: o.collection.clone(),
        vector: if o.compact {
            Vec::new()
        } else {
            q.iter().map(|&x| f64::from(x)).collect()
        },
        vector_f32: if o.compact { q.to_vec() } else { Vec::new() },
        top_k: o.top_k,
        ef_search: o.ef,
        ..Default::default()
    }
}

struct Row {
    clients: usize,
    qps: f64,
    p50: f64,
    p95: f64,
    p99: f64,
    p999: f64,
    mean: f64,
    recall: f64,
    errors: u64,
    server_cpu_ms: Option<f64>,
}

fn pct(sorted: &[u64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let i = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[i] as f64 / 1e6
}

fn server_rss_mb(pid: u32) -> Option<f64> {
    let out = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "rss="])
        .output()
        .ok()?;
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<f64>()
        .ok()
        .map(|kb| kb / 1024.0)
}

fn server_cpu_secs(pid: u32) -> Option<f64> {
    let out = Command::new("ps")
        .args(["-p", &pid.to_string(), "-o", "cputime="])
        .output()
        .ok()?;
    let s = String::from_utf8_lossy(&out.stdout)
        .trim()
        .replace('-', ":");
    let mut acc = 0.0;
    for part in s.split(':') {
        acc = acc * 60.0 + part.parse::<f64>().ok()?;
    }
    Some(acc)
}

#[allow(clippy::too_many_lines)]
async fn run_level(
    o: &Arc<Opts>,
    clients: &[Client],
    c: usize,
    qs: &Arc<Vec<Vec<f32>>>,
    gt: &Arc<Vec<HashSet<u32>>>,
    server_pid: Option<u32>,
) -> Row {
    let per_req = o.batch.max(1);
    let run = |secs: f64, record: bool| {
        let mut tasks = Vec::with_capacity(c);
        for t in 0..c {
            let mut cl = clients[t % clients.len()].clone();
            let (o, qs, gt) = (Arc::clone(o), Arc::clone(qs), Arc::clone(gt));
            tasks.push(tokio::spawn(async move {
                let mut lat = Vec::new();
                let (mut hits, mut total, mut errors) = (0u64, 0u64, 0u64);
                let mut qi = t * 7919 % qs.len();
                let deadline = Instant::now() + Duration::from_secs_f64(secs);
                while Instant::now() < deadline {
                    let idx: Vec<usize> = (0..per_req).map(|j| (qi + j) % qs.len()).collect();
                    qi = (qi + per_req) % qs.len();
                    let t0 = Instant::now();
                    let results: Result<Vec<Vec<u32>>, tonic::Status> = if o.batch == 0 {
                        cl.search(search_req(&o, &qs[idx[0]]))
                            .await
                            .map(|r| vec![r.into_inner().results.iter().map(|x| x.id).collect()])
                    } else {
                        let req = BatchSearchRequest {
                            searches: idx.iter().map(|&i| search_req(&o, &qs[i])).collect(),
                        };
                        cl.search_batch(req).await.map(|r| {
                            r.into_inner()
                                .responses
                                .into_iter()
                                .map(|x| x.results.iter().map(|y| y.id).collect())
                                .collect()
                        })
                    };
                    let el = t0.elapsed().as_nanos() as u64;
                    match results {
                        Ok(res) if record => {
                            lat.push(el);
                            for (i, ids) in idx.iter().zip(res) {
                                let g = &gt[*i];
                                hits += ids.iter().filter(|id| g.contains(id)).count() as u64;
                                total += g.len().min(o.top_k as usize) as u64;
                            }
                        }
                        Ok(_) => {}
                        Err(_) => errors += 1,
                    }
                }
                (lat, hits, total, errors)
            }));
        }
        tasks
    };
    if o.warmup > 0.0 {
        for t in run(o.warmup, false) {
            let _ = t.await;
        }
    }
    let cpu0 = server_pid.and_then(server_cpu_secs);
    let start = Instant::now();
    let tasks = run(o.duration, true);
    let mut lat = Vec::new();
    let (mut hits, mut total, mut errors) = (0u64, 0u64, 0u64);
    for t in tasks {
        if let Ok((l, h, tt, e)) = t.await {
            lat.extend(l);
            hits += h;
            total += tt;
            errors += e;
        }
    }
    let wall = start.elapsed().as_secs_f64();
    let cpu1 = server_pid.and_then(server_cpu_secs);
    lat.sort_unstable();
    let queries = (lat.len() * per_req) as f64;
    Row {
        clients: c,
        qps: queries / wall,
        p50: pct(&lat, 0.50),
        p95: pct(&lat, 0.95),
        p99: pct(&lat, 0.99),
        p999: pct(&lat, 0.999),
        mean: lat.iter().sum::<u64>() as f64 / lat.len().max(1) as f64 / 1e6,
        recall: if total == 0 {
            0.0
        } else {
            hits as f64 / total as f64
        },
        errors,
        server_cpu_ms: match (cpu0, cpu1) {
            (Some(a), Some(b)) if queries > 0.0 => Some((b - a) * 1e3 / queries),
            _ => None,
        },
    }
}

struct ServerGuard(Option<Child>, Option<std::path::PathBuf>);

impl Drop for ServerGuard {
    fn drop(&mut self) {
        if let Some(mut c) = self.0.take() {
            let _ = c.kill();
            let _ = c.wait();
        }
        if let Some(d) = self.1.take() {
            let _ = std::fs::remove_dir_all(d);
        }
    }
}

fn spawn_server(o: &Opts) -> Result<ServerGuard, Box<dyn std::error::Error>> {
    let Some(bin) = &o.server_bin else {
        return Ok(ServerGuard(None, None));
    };
    let dir = std::env::temp_dir().join(format!("hs_loadbench_{}_{}", std::process::id(), o.port));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir)?;
    let mut cmd = Command::new(bin);
    cmd.env("HS_DATA_DIR", &dir)
        .env("HS_PORT", o.port.to_string())
        .env("HS_HTTP_PORT", (o.port - 1).to_string())
        .env("HS_QUANTIZATION_LEVEL", &o.quant)
        .env("HS_V4_ENGINE", "1")
        .env("RUST_LOG", "warn")
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(k) = &o.api_key {
        cmd.env("HYPERSPACE_API_KEY", k);
    }
    for (k, v) in &o.server_env {
        cmd.env(k, v);
    }
    let child = cmd.spawn()?;
    let guard = ServerGuard(Some(child), Some(dir));
    let deadline = Instant::now() + Duration::from_secs(30);
    while std::net::TcpStream::connect(("127.0.0.1", o.port)).is_err() {
        if Instant::now() > deadline {
            return Err("server did not open its port within 30 s".into());
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    Ok(guard)
}

/// Waits until the collection's indexing queue stays empty for 1.5 s: background work the
/// server queues after ingest or a rebuild (online calibration, ef auto-tuning) is done.
async fn wait_idle(admin: &mut Client, collection: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut idle_since: Option<Instant> = None;
    loop {
        let s = admin
            .get_collection_stats(CollectionStatsRequest {
                name: collection.to_string(),
            })
            .await?
            .into_inner();
        if s.indexing_queue == 0 {
            let since = *idle_since.get_or_insert_with(Instant::now);
            if since.elapsed() >= Duration::from_millis(1500) {
                return Ok(());
            }
        } else {
            idle_since = None;
        }
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let o = match parse_args() {
        Ok(o) => Arc::new(o),
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    let guard = spawn_server(&o)?;
    let server_pid = guard.0.as_ref().map(Child::id);
    let mut admin = connect(&o).await?;

    let t = Instant::now();
    let data = clustered(o.n, o.dim, o.clusters, 42, 0);
    let qs = clustered(o.queries, o.dim, o.clusters, 42, 1);
    println!(
        "data: {} × {}D ({} clusters), {} queries generated in {:.1}s",
        o.n,
        o.dim,
        o.clusters,
        qs.len(),
        t.elapsed().as_secs_f64()
    );

    if !o.skip_ingest {
        let schema = CollectionSchema {
            components: vec![VectorComponent {
                name: "default".into(),
                metric: o.metric.clone(),
                full_dimension: o.dim as u32,
                weight: 1.0,
            }],
            cascade_pipeline: vec![],
            encrypted: Some(false),
        };
        let mut req = Request::new(CreateCollectionRequest {
            name: o.collection.clone(),
            schema: Some(schema),
        });
        req.metadata_mut()
            .insert("x-quantization-level", o.quant.parse()?);
        let expected = o.expected_vectors.unwrap_or(o.n as u64);
        if expected > 0 {
            req.metadata_mut()
                .insert("x-expected-vectors", expected.to_string().parse()?);
        }
        admin.create_collection(req).await?;
        let t = Instant::now();
        for (b, chunk) in data.chunks(1000).enumerate() {
            let vectors = chunk
                .iter()
                .enumerate()
                .map(|(i, v)| VectorData {
                    id: (b * 1000 + i) as u32,
                    vector_f32: v.clone(),
                    ..Default::default()
                })
                .collect();
            admin
                .batch_insert(BatchInsertRequest {
                    collection: o.collection.clone(),
                    vectors,
                    ..Default::default()
                })
                .await?;
        }
        let sent = t.elapsed().as_secs_f64();
        loop {
            let s = admin
                .get_collection_stats(CollectionStatsRequest {
                    name: o.collection.clone(),
                })
                .await?
                .into_inner();
            if s.indexing_queue == 0 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        let indexed = t.elapsed().as_secs_f64();
        wait_idle(&mut admin, &o.collection).await?;
        println!(
            "ingest: {:.0} vec/s sent, {:.0} vec/s indexed ({indexed:.1}s, then idle after {:.1}s)",
            o.n as f64 / sent,
            o.n as f64 / indexed,
            t.elapsed().as_secs_f64()
        );
    }

    if o.rebuild {
        // RebuildIndex: wall time and the server's peak RSS while it runs.
        let rss0 = server_pid.and_then(server_rss_mb);
        let peak = Arc::new(std::sync::Mutex::new(rss0.unwrap_or(0.0)));
        let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
        let sampler = server_pid.map(|pid| {
            let (peak, stop) = (Arc::clone(&peak), Arc::clone(&stop));
            std::thread::spawn(move || {
                while !stop.load(std::sync::atomic::Ordering::Relaxed) {
                    if let Some(r) = server_rss_mb(pid) {
                        let mut p = peak.lock().unwrap();
                        *p = p.max(r);
                    }
                    std::thread::sleep(Duration::from_millis(50));
                }
            })
        });
        let t = Instant::now();
        admin
            .rebuild_index(hyperspace_proto::hyperspace::RebuildIndexRequest {
                name: o.collection.clone(),
                filter_query: None,
            })
            .await?;
        let secs = t.elapsed().as_secs_f64();
        wait_idle(&mut admin, &o.collection).await?;
        stop.store(true, std::sync::atomic::Ordering::Relaxed);
        if let Some(h) = sampler {
            let _ = h.join();
        }
        let after = server_pid.and_then(server_rss_mb);
        let peak = *peak.lock().unwrap();
        match (rss0, after) {
            (Some(a), Some(b)) => println!(
                "rebuild: {secs:.1}s, server RSS {a:.0} MB -> peak {peak:.0} MB (+{:.0}) -> {b:.0} MB",
                peak - a
            ),
            _ => println!("rebuild: {secs:.1}s"),
        }
    }

    let t = Instant::now();
    let gt = Arc::new(ground_truth(&data, &qs, o.top_k as usize));
    drop(data);
    println!("ground truth: {:.1}s", t.elapsed().as_secs_f64());
    let qs = Arc::new(qs);

    let max_c = *o.clients.iter().max().unwrap_or(&1);
    let conns = if o.connections == 0 {
        max_c
    } else {
        o.connections.min(max_c)
    };
    let mut clients = Vec::with_capacity(conns);
    for _ in 0..conns {
        clients.push(connect(&o).await?);
    }

    let mode = if o.batch == 0 {
        "Search".to_string()
    } else {
        format!("SearchBatch×{}", o.batch)
    };
    println!(
        "\n{} quant={} metric={} {mode} top_k={} ef={} wire={}",
        o.label,
        o.quant,
        o.metric,
        o.top_k,
        o.ef.map_or("server".into(), |e| e.to_string()),
        if o.compact { "f32" } else { "f64" }
    );
    println!(
        "{:>7} {:>9} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>10} {:>6}",
        "clients", "QPS", "mean ms", "p50", "p95", "p99", "p99.9", "recall", "cpu ms/q", "errors"
    );
    let mut rows = Vec::new();
    for &c in &o.clients {
        let r = run_level(&o, &clients, c, &qs, &gt, server_pid).await;
        println!(
            "{:>7} {:>9.0} {:>8.3} {:>8.3} {:>8.3} {:>8.3} {:>8.3} {:>7.2}% {:>10} {:>6}",
            r.clients,
            r.qps,
            r.mean,
            r.p50,
            r.p95,
            r.p99,
            r.p999,
            r.recall * 100.0,
            r.server_cpu_ms.map_or("-".into(), |v| format!("{v:.3}")),
            r.errors
        );
        rows.push(r);
    }

    if let Some(path) = &o.json {
        let body: Vec<String> = rows
            .iter()
            .map(|r| {
                format!(
                    "{{\"label\":{:?},\"quant\":{:?},\"mode\":{:?},\"n\":{},\"dim\":{},\"clients\":{},\"qps\":{:.1},\"mean_ms\":{:.4},\"p50_ms\":{:.4},\"p95_ms\":{:.4},\"p99_ms\":{:.4},\"p999_ms\":{:.4},\"recall\":{:.5},\"server_cpu_ms_per_query\":{},\"errors\":{}}}",
                    o.label, o.quant, mode, o.n, o.dim, r.clients, r.qps, r.mean, r.p50, r.p95, r.p99, r.p999, r.recall,
                    r.server_cpu_ms.map_or("null".into(), |v| format!("{v:.4}")), r.errors
                )
            })
            .collect();
        std::fs::write(path, format!("[{}]\n", body.join(",\n")))?;
    }

    let mut failed = false;
    if let Some(min) = o.min_recall {
        if let Some(r) = rows.iter().find(|r| r.recall < min) {
            eprintln!(
                "FAIL: recall {:.4} < {min} at {} clients",
                r.recall, r.clients
            );
            failed = true;
        }
    }
    if let Some(min) = o.min_qps {
        let best = rows.iter().map(|r| r.qps).fold(0.0, f64::max);
        if best < min {
            eprintln!("FAIL: best QPS {best:.0} < {min}");
            failed = true;
        }
    }
    if rows.iter().any(|r| r.errors > 0) {
        eprintln!("FAIL: request errors");
        failed = true;
    }
    drop(guard);
    if failed {
        std::process::exit(1);
    }
    Ok(())
}
