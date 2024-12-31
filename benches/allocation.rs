use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use fractalloc::FractalAllocator;
use rand::Rng;
use std::alloc::{GlobalAlloc, Layout};

const BENCH_SIZES: [usize; 6] = [8, 16, 32, 64, 128, 256];
const ITERATIONS: usize = 1000;

static ALLOCATOR: FractalAllocator = FractalAllocator::new();

fn bench_allocation(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_allocation");
    for &size in BENCH_SIZES.iter() {
        group.bench_with_input(BenchmarkId::new("fractalloc", size), &size, |b, &size| {
            b.iter(|| {
                let layout = Layout::from_size_align(size, 8).unwrap();
                unsafe {
                    let ptr = ALLOCATOR.alloc(layout);
                    assert!(!ptr.is_null());
                    ALLOCATOR.dealloc(ptr, layout);
                }
            });
        });
    }
    group.finish();
}

fn bench_bulk_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("bulk_operations");
    let mut rng = rand::thread_rng();

    for &size in BENCH_SIZES.iter() {
        group.bench_with_input(BenchmarkId::new("fractalloc_bulk", size), &size, |b, &size| {
            let layout = Layout::from_size_align(size, 8).unwrap();
            let mut ptrs = Vec::with_capacity(ITERATIONS);

            b.iter(|| {
                for _ in 0..ITERATIONS {
                    unsafe {
                        let ptr = ALLOCATOR.alloc(layout);
                        assert!(!ptr.is_null());
                        ptrs.push(ptr);
                    }
                }

                for i in (1..ptrs.len()).rev() {
                    let j = rng.gen_range(0..=i);
                    ptrs.swap(i, j);
                }

                for ptr in ptrs.drain(..) {
                    unsafe {
                        ALLOCATOR.dealloc(ptr, layout);
                    }
                }
            });
        });
    }
    group.finish();
}

fn bench_mixed_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("mixed_sizes");
    let mut rng = rand::thread_rng();

    group.bench_function("fractalloc_mixed", |b| {
        let mut ptrs = Vec::with_capacity(ITERATIONS);
        let layouts: Vec<Layout> = BENCH_SIZES.iter()
            .map(|&size| Layout::from_size_align(size, 8).unwrap())
            .collect();

        b.iter(|| {
            for _ in 0..ITERATIONS {
                let layout = layouts[rng.gen_range(0..layouts.len())];
                unsafe {
                    let ptr = ALLOCATOR.alloc(layout);
                    assert!(!ptr.is_null());
                    ptrs.push((ptr, layout));
                }
            }

            for (ptr, layout) in ptrs.drain(..) {
                unsafe {
                    ALLOCATOR.dealloc(ptr, layout);
                }
            }
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_allocation,
    bench_bulk_operations,
    bench_mixed_sizes,
);
criterion_main!(benches);
