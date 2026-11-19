//! Flat, contiguous Layer-0 adjacency arena for the live (mutable) HNSW graph.
//!
//! Layer 0 holds every node and is where >95% of search time is spent. Instead of one
//! `RwLock<SmallVec<..>>` per node (pointer chase, ~150 B, frequent heap spills), all
//! layer-0 neighbor lists live in fixed-stride slots of a few large `AtomicU32` segments:
//!
//! ```text
//! slot(id) = [ seq | count | n_0 | n_1 | ... | n_{cap-1} ]      (stride = cap + 2 words)
//! ```
//!
//! * Readers use a seqlock (lock-free, retry on concurrent write).
//! * Writers serialize per slot by CAS-ing `seq` from even to odd.
//! * Segments grow geometrically (`BASE << k` slots) so tiny indexes (e.g. cold chunks)
//!   stay small, while lookups remain O(1) via `leading_zeros`.

use arc_swap::ArcSwap;
use parking_lot::Mutex;
use smallvec::SmallVec;
use std::sync::atomic::{fence, AtomicU32, Ordering};
use std::sync::Arc;

/// Slots in the first segment; segment `k` holds `BASE << k` slots.
const BASE_SHIFT: u32 = 10;
const BASE: usize = 1 << BASE_SHIFT;

type Segment = Arc<[AtomicU32]>;

/// Scratch buffer type for neighbor snapshots.
pub(crate) type Links = SmallVec<[u32; 64]>;

#[inline(always)]
fn locate(id: u32) -> (usize, usize) {
    // Cumulative capacity before segment k: BASE * (2^k - 1).
    let x = (id as usize >> BASE_SHIFT) + 1;
    let seg = (usize::BITS - 1 - x.leading_zeros()) as usize;
    let start = BASE * ((1usize << seg) - 1);
    (seg, id as usize - start)
}

pub(crate) struct Layer0Arena {
    cap: usize,
    stride: usize,
    segments: ArcSwap<Vec<Segment>>,
    grow: Mutex<()>,
}

impl std::fmt::Debug for Layer0Arena {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Layer0Arena")
            .field("cap", &self.cap)
            .field("segments", &self.segments.load().len())
            .finish_non_exhaustive()
    }
}

/// A query-scoped view holding one `ArcSwap` guard for the whole traversal.
pub(crate) struct Layer0View<'a> {
    arena: &'a Layer0Arena,
    segs: arc_swap::Guard<Arc<Vec<Segment>>>,
}

impl Layer0Arena {
    /// Creates an arena whose slots hold up to `cap` neighbors.
    pub(crate) fn new(cap: usize) -> Self {
        let cap = cap.max(1);
        Self {
            cap,
            stride: cap + 2,
            segments: ArcSwap::from_pointee(Vec::new()),
            grow: Mutex::new(()),
        }
    }

    /// Ensures a slot exists for `id` (call before publishing the node).
    pub(crate) fn ensure(&self, id: u32) {
        let (seg, _) = locate(id);
        if seg < self.segments.load().len() {
            return;
        }
        let _g = self.grow.lock();
        let current = self.segments.load();
        if seg < current.len() {
            return;
        }
        let mut next: Vec<Segment> = (**current).clone();
        while next.len() <= seg {
            let slots = BASE << next.len();
            let words: Segment = (0..slots * self.stride)
                .map(|_| AtomicU32::new(0))
                .collect();
            next.push(words);
        }
        self.segments.store(Arc::new(next));
    }

    pub(crate) fn view(&self) -> Layer0View<'_> {
        Layer0View {
            arena: self,
            segs: self.segments.load(),
        }
    }

    /// Lock-free seqlock snapshot of `id`'s neighbors into `out`.
    pub(crate) fn read_into(&self, id: u32, out: &mut Links) {
        self.view().read_into(id, out);
    }

    /// Runs `f` on the neighbor list of `id` under the slot's writer lock and stores the
    /// result (truncated to capacity). Returns the stored length.
    pub(crate) fn update(&self, id: u32, f: impl FnOnce(&mut Links)) -> usize {
        self.ensure(id);
        let segs = self.segments.load();
        let (seg, off) = locate(id);
        let slot = &segs[seg][off * self.stride..(off + 1) * self.stride];
        let seq = &slot[0];

        // Acquire the per-slot writer lock: even -> odd.
        let mut s = seq.load(Ordering::Relaxed);
        loop {
            if s & 1 == 0 {
                match seq.compare_exchange_weak(s, s + 1, Ordering::Acquire, Ordering::Relaxed) {
                    Ok(_) => break,
                    Err(cur) => s = cur,
                }
            } else {
                std::hint::spin_loop();
                s = seq.load(Ordering::Relaxed);
            }
        }
        fence(Ordering::Release);

        let count = (slot[1].load(Ordering::Relaxed) as usize).min(self.cap);
        let mut buf: Links = slot[2..2 + count]
            .iter()
            .map(|w| w.load(Ordering::Relaxed))
            .collect();
        f(&mut buf);
        let n = buf.len().min(self.cap);
        for (w, &v) in slot[2..2 + n].iter().zip(buf.iter()) {
            w.store(v, Ordering::Relaxed);
        }
        slot[1].store(n as u32, Ordering::Relaxed);

        // Release the lock: odd -> next even.
        seq.store(s + 2, Ordering::Release);
        n
    }

    /// Replaces the neighbor list of `id`.
    pub(crate) fn set(&self, id: u32, links: &[u32]) -> usize {
        self.update(id, |buf| {
            buf.clear();
            buf.extend_from_slice(links);
        })
    }
}

impl Layer0View<'_> {
    /// Lock-free seqlock snapshot of `id`'s neighbors into `out` (empty if no slot).
    #[inline]
    pub(crate) fn read_into(&self, id: u32, out: &mut Links) {
        out.clear();
        let (seg, off) = locate(id);
        let Some(words) = self.segs.get(seg) else {
            return;
        };
        let stride = self.arena.stride;
        let slot = &words[off * stride..(off + 1) * stride];
        loop {
            let s1 = slot[0].load(Ordering::Acquire);
            if s1 & 1 != 0 {
                std::hint::spin_loop();
                continue;
            }
            let count = (slot[1].load(Ordering::Relaxed) as usize).min(self.arena.cap);
            out.clear();
            out.extend(slot[2..2 + count].iter().map(|w| w.load(Ordering::Relaxed)));
            fence(Ordering::Acquire);
            if slot[0].load(Ordering::Relaxed) == s1 {
                return;
            }
        }
    }

    /// Address of `id`'s slot for prefetching (null if absent).
    #[inline(always)]
    pub(crate) fn slot_ptr(&self, id: u32) -> *const u8 {
        let (seg, off) = locate(id);
        self.segs.get(seg).map_or(std::ptr::null(), |w| {
            w.as_ptr()
                .wrapping_add(off * self.arena.stride)
                .cast::<u8>()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locate_is_contiguous_across_segments() {
        let mut expected = (0usize, 0usize);
        for id in 0..(BASE * 7 + 3) as u32 {
            let got = locate(id);
            assert_eq!(got, expected, "id={id}");
            let seg_len = BASE << expected.0;
            expected.1 += 1;
            if expected.1 == seg_len {
                expected = (expected.0 + 1, 0);
            }
        }
    }

    #[test]
    fn update_read_roundtrip_and_truncation() {
        let arena = Layer0Arena::new(4);
        let mut out = Links::new();
        arena.read_into(5000, &mut out);
        assert!(out.is_empty());
        assert_eq!(arena.set(5000, &[1, 2, 3]), 3);
        arena.read_into(5000, &mut out);
        assert_eq!(&out[..], &[1, 2, 3]);
        assert_eq!(arena.update(5000, |b| b.extend_from_slice(&[4, 5, 6])), 4);
        arena.read_into(5000, &mut out);
        assert_eq!(&out[..], &[1, 2, 3, 4]);
    }

    #[test]
    fn concurrent_writers_do_not_lose_updates() {
        let arena = Arc::new(Layer0Arena::new(64));
        arena.ensure(0);
        let handles: Vec<_> = (0..8u32)
            .map(|t| {
                let a = Arc::clone(&arena);
                std::thread::spawn(move || {
                    for i in 0..8u32 {
                        a.update(0, |b| b.push(t * 8 + i));
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        let mut out = Links::new();
        arena.read_into(0, &mut out);
        out.sort_unstable();
        assert_eq!(out.len(), 64);
        assert!(out.iter().copied().eq(0..64));
    }
}
