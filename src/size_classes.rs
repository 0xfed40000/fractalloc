pub struct SizeClass {
    index: usize,
    size: usize,
}

impl SizeClass {
    pub const fn new(index: usize) -> Self {
        let index = if index >= 31 { 31 } else { index };
        let base = 8usize << (index / 4);
        let fractal_factor = match index % 4 {
            0 => 1,
            1 => 2,
            2 => 3,
            _ => 4,
        };

        Self {
            index,
            size: base * fractal_factor,
        }
    }

    pub const fn from_size(size: usize) -> Self {
        let mut index = 0;
        let mut current_size = 8;

        while current_size < size && index < 31 {
            index += 1;
            current_size = Self::new(index).size;
        }

        Self::new(index)
    }

    pub const fn from_index(index: usize) -> Self {
        Self::new(index)
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    pub const fn index(&self) -> usize {
        self.index
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_classes() {
        let sizes = [
            (8, 0),
            (16, 1),
            (24, 2),
            (32, 3),
            (64, 4),
        ];

        for (size, expected_index) in sizes {
            let size_class = SizeClass::from_size(size);
            assert_eq!(size_class.index(), expected_index);
            assert!(size_class.size() >= size);
        }
    }
}
