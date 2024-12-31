use super::block::Block;
use super::size_classes::SizeClass;
use core::cell::UnsafeCell;
use std::thread_local;

thread_local! {
    static LOCAL_CACHE: UnsafeCell<ThreadLocalCache> = UnsafeCell::new(ThreadLocalCache::new());
}

struct ThreadLocalCache {
    free_lists: [*mut Block; 32],
    alloc_count: usize,
}

impl ThreadLocalCache {
    const fn new() -> Self {
        Self {
            free_lists: [core::ptr::null_mut(); 32],
            alloc_count: 0,
        }
    }
}

pub struct ThreadCache {
    _private: (),
}

impl ThreadCache {
    pub const fn new() -> Self {
        Self { _private: () }
    }

    pub fn allocate(&self, size: usize) -> Option<*mut u8> {
        LOCAL_CACHE.with(|cache| {
            let cache = unsafe { &mut *cache.get() };
            let size_class = SizeClass::from_size(size);
            let index = size_class.index();

            if cache.free_lists[index].is_null() {
                return None;
            }

            let block = cache.free_lists[index];
            cache.free_lists[index] = unsafe { (*block).next };
            cache.alloc_count += 1;

            Some(block as *mut u8)
        })
    }

    pub fn deallocate(&self, ptr: *mut u8, size: usize) -> bool {
        LOCAL_CACHE.with(|cache| {
            let cache = unsafe { &mut *cache.get() };
            let size_class = SizeClass::from_size(size);
            let index = size_class.index();

            let block = ptr as *mut Block;
            unsafe {
                (*block).next = cache.free_lists[index];
            }
            cache.free_lists[index] = block;
            true
        })
    }
}
