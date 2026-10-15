use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyperspace_core::graph::{CompactEdge, CsrGraph};
use roaring::RoaringBitmap;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

/// Generates a synthetic enterprise graph topology:
/// 25,000 entities, 250,000 edges (~10 edges/node average)
fn generate_enterprise_graph(
    num_nodes: usize,
    total_edges: usize,
) -> (CsrGraph, Vec<Vec<CompactEdge>>) {
    let mut adj = vec![Vec::new(); num_nodes];
    let mut edge_cnt = 0;

    // Power-law / scale-free distribution with high-degree hubs and regular nodes
    for (u, node_edges) in adj.iter_mut().enumerate().take(num_nodes) {
        let degree = if u < 50 {
            // Super-nodes / Core hubs
            200.min(total_edges - edge_cnt)
        } else if u < 1000 {
            // Sub-hubs
            30.min(total_edges - edge_cnt)
        } else {
            // Standard entities
            8.min(total_edges - edge_cnt)
        };

        for step in 1..=degree {
            let v = ((u * 37 + step * 101) % num_nodes) as u32;
            let rel = ((step % 4) + 1) as u16; // 1: OWNS, 2: DEPENDS_ON, 3: SUPERSEDES, 4: AMENDS
            node_edges.push(CompactEdge::new(v, rel, 1.0));
            edge_cnt += 1;
            if edge_cnt >= total_edges {
                break;
            }
        }
        if edge_cnt >= total_edges {
            break;
        }
    }

    let csr = CsrGraph::from_adjacency_list(&adj);
    (csr, adj)
}

/// Simulated external graph database server over Unix Domain Socket
struct MockExternalGraphServer {
    running: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl MockExternalGraphServer {
    fn spawn(server_stream: UnixStream, adj: Vec<Vec<CompactEdge>>) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let r_flag = running.clone();

        let handle = thread::spawn(move || {
            let mut stream = server_stream;
            let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(50)));
            let mut buf = [0u8; 1024];

            while r_flag.load(Ordering::Relaxed) {
                // Read query: seed node (u32) + hops (u8)
                match stream.read(&mut buf) {
                    Ok(0) => break,
                    Ok(n) => {
                        if n < 5 {
                            continue;
                        }
                        let seed = u32::from_le_bytes(buf[0..4].try_into().unwrap());
                        let max_hops = buf[4] as usize;

                        // Perform BFS in graph DB
                        let mut visited = Vec::new();
                        let mut frontier = vec![seed];
                        visited.push(seed);

                        for _ in 0..max_hops {
                            let mut next = Vec::new();
                            for &node in &frontier {
                                let idx = node as usize;
                                if idx < adj.len() {
                                    for edge in &adj[idx] {
                                        if !visited.contains(&edge.target) {
                                            visited.push(edge.target);
                                            next.push(edge.target);
                                            if next.len() > 64 {
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                            frontier = next;
                            if frontier.is_empty() {
                                break;
                            }
                        }

                        // Serialize response (JSON-like / binary payload)
                        let mut resp = Vec::with_capacity(4 + visited.len() * 4);
                        resp.extend_from_slice(&(visited.len() as u32).to_le_bytes());
                        for node in &visited {
                            resp.extend_from_slice(&node.to_le_bytes());
                        }

                        if stream.write_all(&resp).is_err() {
                            break;
                        }
                    }
                    Err(ref e)
                        if e.kind() == std::io::ErrorKind::WouldBlock
                            || e.kind() == std::io::ErrorKind::TimedOut =>
                    {
                        continue;
                    }
                    Err(_) => break,
                }
            }
        });

        Self {
            running,
            handle: Some(handle),
        }
    }
}

impl Drop for MockExternalGraphServer {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }
}

fn benchmark_graph_expansion(c: &mut Criterion) {
    const NUM_NODES: usize = 25_000;
    const NUM_EDGES: usize = 250_000;

    let (csr, adj) = generate_enterprise_graph(NUM_NODES, NUM_EDGES);

    // Setup Unix Domain Socket IPC pair
    let (mut client_sock, server_sock) =
        UnixStream::pair().expect("Failed to create UnixStream pair");
    let _server = MockExternalGraphServer::spawn(server_sock, adj);

    let test_seeds = [42u32, 105, 512, 1204, 8888];

    let mut group = c.benchmark_group("Graph_Expansion_Native_vs_Dual");

    // Benchmark 1: Native In-Engine CSR 2-hop BFS into RoaringBitmap
    group.bench_function("native_csr_2hop_reachability", |b| {
        let mut idx = 0;
        b.iter(|| {
            let seed = test_seeds[idx % test_seeds.len()];
            idx += 1;
            let bm: RoaringBitmap =
                csr.bfs_reachability(black_box(&[seed]), black_box(2), black_box(64), None);
            black_box(bm.len())
        });
    });

    // Benchmark 2: Simulated External Graph DB over Unix Domain Socket IPC (Dual-Query)
    group.bench_function("dual_db_ipc_2hop_query", |b| {
        let mut idx = 0;
        let mut resp_buf = vec![0u8; 8192];
        b.iter(|| {
            let seed = test_seeds[idx % test_seeds.len()];
            idx += 1;

            // Send query over UDS
            let mut req = [0u8; 5];
            req[0..4].copy_from_slice(&seed.to_le_bytes());
            req[4] = 2; // 2 hops
            client_sock.write_all(&req).expect("IPC write failed");

            // Read response
            let bytes_read = client_sock.read(&mut resp_buf).expect("IPC read failed");
            assert!(bytes_read >= 4);
            let count = u32::from_le_bytes(resp_buf[0..4].try_into().unwrap());
            black_box(count)
        });
    });

    // Benchmark 3: Pre-filter Bitmask scoring vs Post-filter Scoring
    group.bench_function("native_prefilter_bitmask_scoring", |b| {
        // Pre-filter: Reachability Bitmask generated first, only scoring candidate set
        let mask = csr.bfs_reachability(&[42], 2, 64, None);
        let dummy_scores: Vec<f32> = (0..NUM_NODES).map(|i| (i % 100) as f32 / 100.0).collect();

        b.iter(|| {
            let mut top_score = -1.0f32;
            let mut top_id = 0u32;
            for id in mask.iter() {
                let score = dummy_scores[id as usize];
                if score > top_score {
                    top_score = score;
                    top_id = id;
                }
            }
            black_box((top_id, top_score))
        });
    });

    group.bench_function("naive_postfilter_pruning_scoring", |b| {
        let mask = csr.bfs_reachability(&[42], 2, 64, None);
        let dummy_scores: Vec<f32> = (0..NUM_NODES).map(|i| (i % 100) as f32 / 100.0).collect();

        b.iter(|| {
            // Score all, then check if in mask
            let mut top_score = -1.0f32;
            let mut top_id = 0u32;
            for id in 0..NUM_NODES as u32 {
                let score = dummy_scores[id as usize];
                if mask.contains(id) && score > top_score {
                    top_score = score;
                    top_id = id;
                }
            }
            black_box((top_id, top_score))
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_graph_expansion);
criterion_main!(benches);
