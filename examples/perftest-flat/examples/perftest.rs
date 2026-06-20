//! Native micro-benchmark of the three `perftest-flat` functions (no FFI).
//!
//! Mirrors `perftest-c/c/perftest.c`, which runs the same three benchmarks through
//! the generated C ABI — compare the numbers to see the cost of crossing the
//! (zero-copy) C boundary vs calling the Rust functions directly.
//!
//! Run with: `cargo run --release -p perftest-flat --example perftest`

use std::hint::black_box;
use std::time::Instant;

use perftest_flat::{payload_callback, payload_get, payload_put, Payload};

const N: u64 = 50_000_000;

fn bench(name: &str, n: u64, mut body: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..n {
        body();
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / n as f64;
    let mops = n as f64 / elapsed.as_secs_f64() / 1.0e6;
    println!("{name:<10} {ns_per_op:>8.2} ns/op   {mops:>8.1} Mops/s");
}

fn main() {
    let seed = Payload {
        id: 42,
        seq: 7,
        value: 3.5,
        flag: true,
        label: Some(Box::new("hello, payload".to_string())),
    };
    payload_put(&seed);

    println!("perftest-flat (native Rust), N = {N} iterations per op\n");

    let mut sink: u64 = 0;

    bench("put", N, || {
        payload_put(black_box(&seed));
    });

    bench("get", N, || {
        let g = payload_get();
        sink = sink.wrapping_add(g.id);
        black_box(&g);
    });

    bench("callback", N, || {
        payload_callback(move |p| {
            black_box(p);
        });
    });

    // Keep `sink` observable so `get` is not optimized away.
    println!("\n(sink = {sink})");
}
