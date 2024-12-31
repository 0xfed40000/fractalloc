#[repr(C)]
pub struct Block {
    pub next: *mut Block,
    _data: [u8; 0],
}

#[allow(dead_code)]
impl Block {
    pub const fn new() -> Self {
        Self {
            next: core::ptr::null_mut(),
            _data: [],
        }
    }

    pub fn data_ptr(&mut self) -> *mut u8 {
        self as *mut _ as *mut u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_layout() {
        use core::mem::size_of;
        assert_eq!(size_of::<Block>(), size_of::<*mut Block>());
    }
}
