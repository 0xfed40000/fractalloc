use core::sync::atomic::{AtomicUsize, Ordering};

#[allow(dead_code)]
pub struct AtomicBumpAllocator {
    current: AtomicUsize,
    end: usize,
}

#[allow(dead_code)]
impl AtomicBumpAllocator {
    pub const fn new(start: usize, size: usize) -> Self {
        Self {
            current: AtomicUsize::new(start),
            end: start + size,
        }
    }

    pub fn allocate(&self, size: usize, align: usize) -> Option<*mut u8> {
        let mut current = self.current.load(Ordering::Relaxed);

        loop {
            let aligned = (current + align - 1) & !(align - 1);

            if aligned + size > self.end {
                return None;
            }

            match self.current.compare_exchange_weak(
                current,
                aligned + size,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return Some(aligned as *mut u8),
                Err(new_current) => current = new_current,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atomic_bump() {
        let mut memory = [0u8; 1024];
        let start = memory.as_mut_ptr() as usize;
        let allocator = AtomicBumpAllocator::new(start, 1024);

        let ptr1 = allocator.allocate(16, 8).unwrap();
        assert_eq!(ptr1 as usize % 8, 0);

        let ptr2 = allocator.allocate(16, 8).unwrap();
        assert_eq!(ptr2 as usize % 8, 0);
        assert!(ptr2 as usize > ptr1 as usize + 16);
    }
}
