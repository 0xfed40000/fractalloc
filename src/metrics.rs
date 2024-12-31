use core::sync::atomic::{AtomicUsize, Ordering};

#[allow(dead_code)]
pub struct AllocatorMetrics {
    total_allocations: AtomicUsize,
    total_deallocations: AtomicUsize,
    bytes_allocated: AtomicUsize,
    allocation_failures: AtomicUsize,
    cache_hits: AtomicUsize,
    cache_misses: AtomicUsize,
}

#[allow(dead_code)]
impl AllocatorMetrics {
    pub const fn new() -> Self {
        Self {
            total_allocations: AtomicUsize::new(0),
            total_deallocations: AtomicUsize::new(0),
            bytes_allocated: AtomicUsize::new(0),
            allocation_failures: AtomicUsize::new(0),
            cache_hits: AtomicUsize::new(0),
            cache_misses: AtomicUsize::new(0),
        }
    }

    pub fn record_allocation(&self, size: usize) {
        self.total_allocations.fetch_add(1, Ordering::Relaxed);
        self.bytes_allocated.fetch_add(size, Ordering::Relaxed);
    }

    pub fn record_deallocation(&self, size: usize) {
        self.total_deallocations.fetch_add(1, Ordering::Relaxed);
        self.bytes_allocated.fetch_sub(size, Ordering::Relaxed);
    }

    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> AllocationStats {
        AllocationStats {
            total_allocations: self.total_allocations.load(Ordering::Relaxed),
            total_deallocations: self.total_deallocations.load(Ordering::Relaxed),
            bytes_allocated: self.bytes_allocated.load(Ordering::Relaxed),
            allocation_failures: self.allocation_failures.load(Ordering::Relaxed),
            cache_hits: self.cache_hits.load(Ordering::Relaxed),
            cache_misses: self.cache_misses.load(Ordering::Relaxed),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct AllocationStats {
    pub total_allocations: usize,
    pub total_deallocations: usize,
    pub bytes_allocated: usize,
    pub allocation_failures: usize,
    pub cache_hits: usize,
    pub cache_misses: usize,
}

#[allow(dead_code)]
impl AllocationStats {
    pub fn cache_hit_rate(&self) -> f64 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            self.cache_hits as f64 / total as f64
        }
    }
}
