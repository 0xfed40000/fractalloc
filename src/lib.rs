#![no_std]

#[cfg(feature = "std")]
extern crate std;

mod atomic;
mod block;
mod metrics;
mod size_classes;
mod thread_cache;

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicPtr, Ordering};

use block::Block;
#[cfg(unix)]
use libc::{mmap, MAP_ANONYMOUS, MAP_FAILED, MAP_PRIVATE, PROT_READ, PROT_WRITE};
use size_classes::SizeClass;
use thread_cache::ThreadCache;

#[cfg(windows)]
use windows_sys::Win32::System::Memory::{VirtualAlloc, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE};

#[cfg(target_arch = "x86_64")]
mod simd {
    #[cfg(target_feature = "sse2")]
    use core::arch::x86_64::*;

    #[allow(dead_code)]
    #[target_feature(enable = "sse2")]
    pub unsafe fn memset_simd(dst: *mut u8, val: u8, count: usize) {
        if count >= 16 {
            let val_vec = _mm_set1_epi8(val as i8);
            let mut ptr = dst as *mut __m128i;
            let end = dst.add(count & !15) as *mut __m128i;

            while ptr < end {
                _mm_store_si128(ptr, val_vec);
                ptr = ptr.add(1);
            }
        }

        let remainder = count & 15;
        if remainder > 0 {
            let mut ptr = dst.add(count & !15);
            let end = dst.add(count);
            while ptr < end {
                *ptr = val;
                ptr = ptr.add(1);
            }
        }
    }
}

pub struct FractalAllocator {
    free_lists: [AtomicPtr<Block>; 32],
    thread_cache: ThreadCache,
}

unsafe impl Send for FractalAllocator {}
unsafe impl Sync for FractalAllocator {}

impl FractalAllocator {
    pub const fn new() -> Self {
        const ATOMIC_NULL: AtomicPtr<Block> = AtomicPtr::new(core::ptr::null_mut());
        Self {
            free_lists: [ATOMIC_NULL; 32],
            thread_cache: ThreadCache::new(),
        }
    }

    fn size_class_index(size: usize) -> usize {
        SizeClass::from_size(size).index()
    }

    unsafe fn allocate_from_size_class(&self, size_class: usize) -> *mut u8 {
        let mut current = self.free_lists[size_class].load(Ordering::Acquire);

        while !current.is_null() {
            match self.free_lists[size_class].compare_exchange_weak(
                current,
                (*current).next,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => return current as *mut u8,
                Err(new_current) => current = new_current,
            }
        }

        self.allocate_new_page(size_class) as *mut u8
    }

    unsafe fn allocate_new_page(&self, size_class: usize) -> *mut Block {
        let page_size = 4096;
        let block_size = SizeClass::from_index(size_class).size();
        let blocks_per_page = page_size / block_size;

        let page = {
            #[cfg(unix)]
            {
                let p = mmap(
                    core::ptr::null_mut(),
                    page_size,
                    PROT_READ | PROT_WRITE,
                    MAP_PRIVATE | MAP_ANONYMOUS,
                    -1,
                    0,
                );
                if p == MAP_FAILED {
                    return core::ptr::null_mut();
                }
                p
            }
            #[cfg(windows)]
            {
                VirtualAlloc(
                    core::ptr::null_mut(),
                    page_size,
                    MEM_COMMIT | MEM_RESERVE,
                    PAGE_READWRITE,
                )
            }
        };

        let mut prev = core::ptr::null_mut();
        for i in 0..blocks_per_page {
            let block = page.add(i * block_size) as *mut Block;
            (*block).next = prev;
            prev = block;
        }

        prev
    }
}

unsafe impl GlobalAlloc for FractalAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if let Some(cached) = self.thread_cache.allocate(layout.size()) {
            return cached;
        }

        let size_class = Self::size_class_index(layout.size());
        self.allocate_from_size_class(size_class)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if self.thread_cache.deallocate(ptr, layout.size()) {
            return;
        }

        let block = ptr as *mut Block;
        let size_class = Self::size_class_index(layout.size());

        let mut current = self.free_lists[size_class].load(Ordering::Acquire);
        loop {
            (*block).next = current;
            match self.free_lists[size_class].compare_exchange_weak(
                current,
                block,
                Ordering::Release,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(new_current) => current = new_current,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_allocation() {
        let allocator = FractalAllocator::new();
        let layout = Layout::from_size_align(64, 8).unwrap();

        unsafe {
            let ptr = allocator.alloc(layout);
            assert!(!ptr.is_null());

            ptr.write(42);
            assert_eq!(ptr.read(), 42);

            allocator.dealloc(ptr, layout);
        }
    }
}
