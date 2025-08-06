use client::{init_profiler, profile};

use rayon::prelude::*;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::thread;
use std::time::Duration;

// Test functions that simulate different workloads
#[profile]
fn cpu_intensive_work(n: u64) -> u64 {
    let mut sum = 0;
    for i in 0..n {
        sum += i * i;
        sum %= 1000_000_007;
        // Simulate some work
        if i % 10000 == 0 {
            thread::sleep(Duration::from_micros(1));
        }
    }
    sum
}

#[profile]
fn io_simulation(duration_ms: u64) {
    thread::sleep(Duration::from_millis(duration_ms));
}

#[profile]
fn memory_work(size: usize) -> Vec<i32> {
    let mut vec = Vec::with_capacity(size);
    for i in 0..size {
        vec.push(i as i32);
    }
    // Simulate some processing
    vec.iter_mut().for_each(|x| *x *= 2);
    vec
}

fn main() {
    init_profiler();
    println!("=== Multi-threaded Profiling Test with Rayon ===\n");

    // Test 1: Parallel computation
    println!("1. Parallel CPU-intensive work:");
    let numbers: Vec<u64> = (1..=8).map(|i| i * 50000).collect();

    let results: Vec<u64> = numbers.par_iter().map(|&n| cpu_intensive_work(n)).collect();

    println!("Results: {:?}\n", results);

    // Test 2: Parallel I/O simulation
    println!("2. Parallel I/O simulation:");
    let io_times: Vec<u64> = vec![100, 200, 150, 80, 300, 120];

    io_times
        .par_iter()
        .for_each(|&duration| io_simulation(duration));

    println!();

    // Test 3: Mixed workload with thread pool
    println!("3. Mixed workload:");
    let counter = Arc::new(AtomicU64::new(0));

    (0..10).into_par_iter().for_each(|i| {
        // Some CPU work
        let _result = cpu_intensive_work(10000);

        // Some memory work
        let _vec = memory_work(1000);

        // Some I/O simulation
        io_simulation(50);

        // Update shared counter
        counter.fetch_add(1, Ordering::Relaxed);
    });

    println!("Counter final value: {}\n", counter.load(Ordering::Relaxed));

    // Test 4: Nested parallel operations
    /*
    println!("4. Nested parallel operations:");
    let data: Vec<Vec<i32>> = (0..4)
        .map(|i| (0..1000).map(|j| i * 1000 + j).collect())
        .collect();

    let processed: Vec<i32> = data
        .par_iter()
        .map(|chunk| {
            chunk
                .par_iter()
                .map(|&x| {
                    // Simulate some computation
                    let mut result = x;
                    for _ in 0..100 {
                        result = result.wrapping_mul(17).wrapping_add(42);
                    }
                    result
                })
                .sum()
        })
        .collect();

    println!("Processed chunk sums: {:?}\n", processed);
    */

    // Test 5: Custom thread pool
    println!("5. Custom thread pool size test:");
    let custom_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(3)
        .build()
        .unwrap();

    custom_pool.install(|| {
        (0..6).into_par_iter().for_each(|i| {
            cpu_intensive_work(25000);
        });
    });

    println!("\n=== Test completed ===");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profiler_in_parallel() {
        let results: Vec<u64> = (1..=4)
            .into_par_iter()
            .map(|i| cpu_intensive_work(i * 1000))
            .collect();

        assert_eq!(results.len(), 4);
    }

    #[test]
    fn test_memory_work_parallel() {
        let sizes = vec![100, 200, 300, 400];
        let results: Vec<Vec<i32>> = sizes.par_iter().map(|&size| memory_work(size)).collect();

        assert_eq!(results.len(), 4);
        assert_eq!(results[0].len(), 100);
        assert_eq!(results[3].len(), 400);
    }
}
