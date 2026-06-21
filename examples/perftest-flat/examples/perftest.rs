//! Native micro-benchmark of the `perftest-flat` functions (no FFI).
//!
//! Mirrors `perftest-c/c/perftest.c` (generated C ABI) and
//! `perftest-kotlin/.../Bench.kt` (generated JNI) — compare the numbers to see the
//! cost of crossing the (zero-copy) C boundary / the JNI boundary vs calling the
//! Rust functions directly. All three emit the same `BEGIN_PERFTEST … END_PERFTEST`
//! block; `examples/perftest-bench.sh` builds, runs, and tabulates them.
//!
//! Run with: `cargo run --release -p perftest-flat --example perftest`
//! Iteration count: `PERFTEST_N=1000000 cargo run --release …` (default 5_000_000).

use std::hint::black_box;
use std::time::Instant;

use perftest_flat::{storage_callback, storage_get, storage_new, storage_put_by_take, Payload};

/// Iterations per measured variant. Overridable via `PERFTEST_N` so the shared
/// benchmark harness can run all three languages at one `N` (and a fast smoke).
fn iterations() -> u64 {
    std::env::var("PERFTEST_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5_000_000)
}

/// Time `body` for `n` iterations and print one normalized result row:
/// `<op> <variant> <ns_per_op> <mops>`.
fn bench(op: &str, variant: &str, n: u64, mut body: impl FnMut()) {
    let start = Instant::now();
    for _ in 0..n {
        body();
    }
    let elapsed = start.elapsed();
    let ns_per_op = elapsed.as_nanos() as f64 / n as f64;
    let mops = n as f64 / elapsed.as_secs_f64() / 1.0e6;
    println!("{op:<10} {variant:<16} {ns_per_op:>9.2} {mops:>9.1}");
}

fn main() {
    let n = iterations();
    let seed = Payload {
        id: 42,
        seq: 7,
        value: 3.5,
        flag: true,
        label: Some(Box::new("hello, payload".to_string())),
    };
    let mut storage = storage_new();
    storage_put_by_take(&mut storage, seed.clone());

    let mut sink: i64 = 0;

    println!("BEGIN_PERFTEST lang=rust n={n}");

    bench("put", "native", n, || {
        // `storage_put_by_take` consumes its argument by value, so provide a fresh
        // owned payload each call (the clone re-allocates the `label`, mirroring the C
        // benchmark's per-iter `string_new`).
        storage_put_by_take(&mut storage, black_box(seed.clone()));
    });

    bench("get", "native", n, || {
        let g = storage_get(&storage);
        sink = sink.wrapping_add(g.id);
        black_box(&g);
    });

    bench("callback", "native", n, || {
        // The callback bound is `Fn(&Payload) + 'static`, so it can't capture `sink`
        // by reference; touch the borrowed payload through `black_box` (parity with
        // C's "observe the payload" callback — the point is the dispatch cost).
        storage_callback(&storage, |p| {
            black_box(p.id);
        });
    });

    println!("END_PERFTEST");

    // Keep `sink` observable so the benchmarks aren't optimized away.
    println!("(sink = {sink})");
}
