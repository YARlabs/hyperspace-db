use parking_lot::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub struct TimerWheel {
    // Level 0: 100 slots, each representing 10ms (covers 1 second)
    level0: Vec<Mutex<Vec<u32>>>,
    // Level 1: 3600 slots, each representing 1 second (covers 1 hour)
    level1: Vec<Mutex<Vec<u32>>>,

    start_time: Instant,
    last_tick_ms: AtomicU64,
}

impl Default for TimerWheel {
    fn default() -> Self {
        Self::new()
    }
}

impl TimerWheel {
    #[must_use]
    pub fn new() -> Self {
        let mut level0 = Vec::with_capacity(100);
        for _ in 0..100 {
            level0.push(Mutex::new(Vec::new()));
        }
        let mut level1 = Vec::with_capacity(3600);
        for _ in 0..3600 {
            level1.push(Mutex::new(Vec::new()));
        }
        Self {
            level0,
            level1,
            start_time: Instant::now(),
            last_tick_ms: AtomicU64::new(0),
        }
    }

    pub fn schedule(&self, id: u32, ttl: Duration) {
        #[allow(clippy::cast_possible_truncation)]
        let now_ms = Instant::now().duration_since(self.start_time).as_millis() as u64;
        #[allow(clippy::cast_possible_truncation)]
        let diff_ms = ttl.as_millis() as u64;
        let expire_at_ms = now_ms + diff_ms;

        if diff_ms < 1000 {
            let slot = ((expire_at_ms / 10) % 100) as usize;
            self.level0[slot].lock().push(id);
        } else {
            let slot = ((expire_at_ms / 1000) % 3600) as usize;
            self.level1[slot].lock().push(id);
        }
    }

    /// Returns the list of expired IDs.
    pub fn tick(&self) -> Vec<u32> {
        #[allow(clippy::cast_possible_truncation)]
        let current_time_ms = Instant::now().duration_since(self.start_time).as_millis() as u64;
        let last_tick = self.last_tick_ms.load(Ordering::Relaxed);
        if current_time_ms <= last_tick {
            return Vec::new();
        }

        let mut expired = Vec::new();

        // Tick Level 0 (10ms steps)
        let start_10ms = last_tick / 10;
        let end_10ms = current_time_ms / 10;
        for i in (start_10ms + 1)..=end_10ms {
            let slot = (i % 100) as usize;
            let mut guard = self.level0[slot].lock();
            expired.append(&mut *guard);
        }

        // Tick Level 1 (1000ms/1s steps)
        let start_1s = last_tick / 1000;
        let end_1s = current_time_ms / 1000;
        for j in (start_1s + 1)..=end_1s {
            let slot = (j % 3600) as usize;
            let mut guard = self.level1[slot].lock();
            expired.append(&mut *guard);
        }

        self.last_tick_ms.store(current_time_ms, Ordering::Relaxed);
        expired
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_schedule_and_tick() {
        let wheel = TimerWheel::new();
        wheel.schedule(42, Duration::from_millis(50));

        thread::sleep(Duration::from_millis(100));
        let expired = wheel.tick();
        assert!(expired.contains(&42));
    }

    #[test]
    fn test_no_premature_expiry() {
        let wheel = TimerWheel::new();
        wheel.schedule(42, Duration::from_secs(10));

        thread::sleep(Duration::from_millis(50));
        let expired = wheel.tick();
        assert!(expired.is_empty());
    }

    #[test]
    fn test_multiple_ids_same_slot() {
        let wheel = TimerWheel::new();
        wheel.schedule(1, Duration::from_millis(20));
        wheel.schedule(2, Duration::from_millis(25));

        thread::sleep(Duration::from_millis(100));
        let expired = wheel.tick();
        assert!(expired.contains(&1));
        assert!(expired.contains(&2));
    }
}
