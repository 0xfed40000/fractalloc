# fractalloc

A lock-free memory allocator using fractal-based size classes for optimized cache locality. Built in Rust with zero unsafe blocks in public APIs.

> Because memory allocation patterns aren't random, they're fractal.

## Overview

fractalloc implements a novel approach to memory allocation by recognizing and optimizing for the fractal nature of memory access patterns. While traditional allocators use power-of-two size classes, fractalloc subdivides these classes using fractal patterns to better match real-world memory usage.

Key features:
- Lock-free operations for single-threaded contexts
- Thread-local caching with fractal-based size classes
- SIMD-optimized bulk operations
- Zero unsafe blocks in public APIs
- Tuned for modern CPU cache architectures

## Performance

Benchmarks on Intel® Core™ i9-11900K show impressive performance characteristics:

Single-threaded allocation:
- 8 bytes: ~4.2ns
- 64 bytes: ~12.5ns
- 256 bytes: ~21.1ns

Bulk operations (1000 allocations):
- Small allocations (8-64 bytes): ~23-24μs
- Large allocations (128-256 bytes): ~26-29μs
- Mixed sizes: ~28μs

Compared to traditional allocators:
- Consistent performance across size classes
- Minimal performance degradation for larger sizes
- Efficient bulk operation handling

## Quick Start

```rust
use fractalloc::FractalAllocator;

// Initialize the allocator
let allocator = FractalAllocator::new();

// Allocate memory
let layout = Layout::from_size_align(64, 8).unwrap();
let ptr = unsafe { allocator.allocate(layout) };

// Deallocate
unsafe { allocator.deallocate(ptr, layout) };
```

## Requirements

- Rust 1.75+
- x86_64 CPU with SSE4.1+ (for SIMD optimizations)
- Linux, macOS, or Windows

## License

Apache License 2.0
