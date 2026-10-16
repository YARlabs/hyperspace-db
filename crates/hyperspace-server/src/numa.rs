//! NUMA-aware thread placement.
//!
//! On multi-socket machines (2-4 socket EPYC / Xeon) a worker thread that touches memory owned
//! by another socket pays an extra interconnect hop (UPI / Infinity Fabric) on every cache miss,
//! which is exactly the access pattern of HNSW graph traversal. The Linux first-touch policy allocates
//! pages on the node of the thread that first writes them, so keeping each runtime thread on a
//! single node (and spreading threads evenly across nodes) keeps the allocations made by that
//! thread - jemalloc thread caches, scratch buffers, visited sets, query vectors - node-local.
//!
//! The topology is read from `/sys/devices/system/node/node*/cpulist` and intersected with the
//! CPUs this process is actually allowed to run on (`sched_getaffinity`), so container cpusets
//! and `taskset` restrictions are respected.
//!
//! Behaviour is controlled by `HS_NUMA`:
//! * unset / `auto` - enabled automatically when 2+ NUMA nodes are visible,
//! * `0` / `off`    - disabled,
//! * `1` / `on`     - same as auto (there is nothing to do on single-node machines).
//!
//! On non-Linux platforms everything in this module is a no-op.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// CPUs grouped by NUMA node (only nodes with at least one usable CPU are kept).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumaTopology {
    nodes: Vec<Vec<usize>>,
}

impl NumaTopology {
    /// Builds a topology from explicit per-node CPU lists (empty nodes are dropped).
    pub fn from_nodes(nodes: Vec<Vec<usize>>) -> Self {
        Self {
            nodes: nodes.into_iter().filter(|n| !n.is_empty()).collect(),
        }
    }

    /// Detects the topology of the current machine, restricted to the allowed CPU set.
    pub fn detect() -> Self {
        #[cfg(target_os = "linux")]
        {
            linux::detect()
        }
        #[cfg(not(target_os = "linux"))]
        {
            Self { nodes: Vec::new() }
        }
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// `true` when there is more than one node, i.e. pinning can make a difference.
    pub fn is_numa(&self) -> bool {
        self.nodes.len() > 1
    }

    pub fn cpus_of_node(&self, node: usize) -> &[usize] {
        &self.nodes[node % self.nodes.len().max(1)]
    }

    /// Round-robin assignment of the `index`-th thread (or shard) to a node.
    pub fn node_for_index(&self, index: usize) -> usize {
        index % self.nodes.len().max(1)
    }
}

/// Parses a Linux `cpulist` string such as `0-3,8,10-11` into CPU ids.
pub fn parse_cpulist(s: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for part in s.trim().split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        match part.split_once('-') {
            Some((lo, hi)) => {
                if let (Ok(lo), Ok(hi)) = (lo.trim().parse::<usize>(), hi.trim().parse::<usize>()) {
                    out.extend(lo..=hi);
                }
            }
            None => {
                if let Ok(c) = part.parse::<usize>() {
                    out.push(c);
                }
            }
        }
    }
    out
}

/// Restricts the current thread to `cpus`. Returns `true` on success (always `false` on
/// platforms without thread-affinity support).
pub fn pin_current_thread(cpus: &[usize]) -> bool {
    #[cfg(target_os = "linux")]
    {
        linux::pin_current_thread(cpus)
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = cpus;
        false
    }
}

/// Returns the topology to use for pinning according to `HS_NUMA`, or `None` when NUMA
/// placement is disabled or pointless (single node / unsupported platform).
pub fn topology_from_env() -> Option<Arc<NumaTopology>> {
    let mode = std::env::var("HS_NUMA").unwrap_or_default();
    let mode = mode.trim().to_ascii_lowercase();
    if matches!(mode.as_str(), "0" | "off" | "false" | "no") {
        return None;
    }
    let topo = NumaTopology::detect();
    topo.is_numa().then(|| Arc::new(topo))
}

/// Creates a `tokio::runtime::Builder::on_thread_start` hook that pins every runtime thread
/// (workers and blocking-pool threads) to the CPUs of one node, assigning nodes round-robin so
/// the load is spread evenly across sockets. Threads stay free to migrate *within* their node.
pub fn thread_start_hook(topo: Arc<NumaTopology>) -> impl Fn() + Send + Sync + 'static {
    let counter = AtomicUsize::new(0);
    move || {
        let idx = counter.fetch_add(1, Ordering::Relaxed);
        let node = topo.node_for_index(idx);
        let _ = pin_current_thread(topo.cpus_of_node(node));
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::{parse_cpulist, NumaTopology};

    /// CPUs this process may run on (cgroup cpuset / taskset aware).
    fn allowed_cpus() -> Option<Vec<usize>> {
        // SAFETY: `cpu_set_t` is plain old data; sched_getaffinity only writes into it.
        unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            let size = std::mem::size_of::<libc::cpu_set_t>();
            if libc::sched_getaffinity(0, size, &raw mut set) != 0 {
                return None;
            }
            Some(
                (0..libc::CPU_SETSIZE as usize)
                    .filter(|&c| libc::CPU_ISSET(c, &set))
                    .collect(),
            )
        }
    }

    pub(super) fn detect() -> NumaTopology {
        let allowed = allowed_cpus();
        let mut nodes = Vec::new();
        for node in 0..1024usize {
            let path = format!("/sys/devices/system/node/node{node}/cpulist");
            let Ok(text) = std::fs::read_to_string(&path) else {
                // Nodes are numbered densely from 0; the first missing node ends the scan.
                break;
            };
            let mut cpus = parse_cpulist(&text);
            if let Some(allowed) = &allowed {
                cpus.retain(|c| allowed.contains(c));
            }
            nodes.push(cpus);
        }
        NumaTopology::from_nodes(nodes)
    }

    pub(super) fn pin_current_thread(cpus: &[usize]) -> bool {
        if cpus.is_empty() {
            return false;
        }
        // SAFETY: zeroed cpu_set_t is valid; CPU_SET is bounds-checked below.
        unsafe {
            let mut set: libc::cpu_set_t = std::mem::zeroed();
            let mut any = false;
            for &c in cpus {
                if c < libc::CPU_SETSIZE as usize {
                    libc::CPU_SET(c, &mut set);
                    any = true;
                }
            }
            let size = std::mem::size_of::<libc::cpu_set_t>();
            any && libc::sched_setaffinity(0, size, &raw const set) == 0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cpulists() {
        assert_eq!(parse_cpulist("0-3,8,10-11\n"), vec![0, 1, 2, 3, 8, 10, 11]);
        assert_eq!(parse_cpulist("5"), vec![5]);
        assert!(parse_cpulist("").is_empty());
        assert!(parse_cpulist("garbage,x-y").is_empty());
    }

    #[test]
    fn topology_round_robin_and_filtering() {
        let t = NumaTopology::from_nodes(vec![vec![0, 1], vec![], vec![2, 3]]);
        assert_eq!(t.node_count(), 2);
        assert!(t.is_numa());
        assert_eq!(t.node_for_index(0), 0);
        assert_eq!(t.node_for_index(1), 1);
        assert_eq!(t.node_for_index(2), 0);
        assert_eq!(t.cpus_of_node(1), &[2, 3]);
    }

    #[test]
    fn single_node_is_not_numa() {
        let t = NumaTopology::from_nodes(vec![vec![0, 1, 2, 3]]);
        assert!(!t.is_numa());
        assert_eq!(t.node_for_index(7), 0);
    }

    #[test]
    fn detect_never_panics() {
        let t = NumaTopology::detect();
        assert!(t.node_count() <= 1024);
    }
}
