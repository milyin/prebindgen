//! Native micro-benchmark of the `perftest-flat` functions (no FFI).
//!
//! Mirrors `perftest-c/c/perftest.c` (generated C ABI) and
//! `perftest-kotlin/.../Bench.kt` (generated JNI) — compare the numbers to see the
//! cost of crossing the (zero-copy) C boundary / the JNI boundary vs calling the
//! Rust functions directly. All three emit the same `BEGIN_PERFTEST … END_PERFTEST`
//! block; `examples/perftest-bench.sh` builds, runs, and tabulates them.
//!
//! Single-payload ops (`put`/`get`/`callback`) move one `Payload`; vector ops
//! (`put_vec`/`get_vec`/`callback_vec`) move a whole batch of `PERFTEST_VEC_N` payloads
//! per call (reported as ns per call, run `N / VEC_N` times).
//!
//! Run with: `cargo run --release -p perftest-flat --example perftest`
//! Iteration count: `PERFTEST_N=1000000 cargo run --release …` (default 5_000_000);
//! batch size: `PERFTEST_VEC_N=16` (default 16).

use std::hint::black_box;
use std::time::Instant;

use perftest_flat::{
    payload_handler_new, payload_vec_handler_new, storage_callback, storage_callback_vec,
    storage_get, storage_get_vec, storage_new, storage_put_by_take, storage_put_slice, Payload,
    Storage,
};

/// Iterations per measured single-op variant. Overridable via `PERFTEST_N` so the
/// shared benchmark harness can run all three languages at one `N` (and a fast smoke).
fn iterations() -> u64 {
    std::env::var("PERFTEST_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5_000_000)
}

/// Batch size for the vector ops (`PERFTEST_VEC_N`, default 16).
fn vec_n() -> usize {
    std::env::var("PERFTEST_VEC_N")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|&k| k > 0)
        .unwrap_or(16)
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
    println!("{op:<12} {variant:<16} {ns_per_op:>9.2} {mops:>9.1}");
}

/// Build a payload whose `label` is present (`Some`) or absent (`None`) — the two
/// string categories the benchmark compares.
fn make_seed(label: Option<&str>) -> Payload {
    Payload {
        id: 42,
        seq: 7,
        value: 3.5,
        flag: true,
        label: label.map(|s| Box::new(s.to_string())),
    }
}

/// Build a batch of `k` payloads (distinct ids), each carrying the given `label`.
fn make_batch(k: usize, label: Option<&str>) -> Vec<Payload> {
    (0..k)
        .map(|i| Payload {
            id: i as i64,
            seq: i as i32,
            value: i as f64,
            flag: i % 2 == 0,
            label: label.map(|s| Box::new(s.to_string())),
        })
        .collect()
}

/// Run put/get/callback for one string category (`str` = a heap `label`, `null` = no
/// `label`), emitting `<op> native.<cat>` rows. The `.null` rows isolate the FFI +
/// ownership cost; the `.str` rows add the `label` heap (de)allocation.
fn run_category(storage: &mut Storage, label: Option<&str>, cat: &str, n: u64, sink: &mut i64) {
    // Seed the storage so `get`/`callback` read a payload of this category.
    storage_put_by_take(storage, make_seed(label));

    bench("put", &format!("native.{cat}"), n, || {
        // `storage_put_by_take` consumes its argument by value, so provide a fresh
        // owned payload each call (a `.str` clone re-allocates the `label`, mirroring
        // the C benchmark's per-iter `string_new`; `.null` allocates nothing).
        storage_put_by_take(storage, black_box(make_seed(label)));
    });

    bench("get", &format!("native.{cat}"), n, || {
        let g = storage_get(storage).expect("seeded ⇒ present");
        *sink = sink.wrapping_add(g.id);
        black_box(&g);
    });

    // Callback prepared ONCE into a reusable handler (a real "declare the subscriber
    // once" step), then `storage_callback` fires it each iteration — so the loop
    // measures `storage_callback` itself, not callback creation. The bound is
    // `Fn(&Payload) + 'static`, so it can't capture `sink` by reference; touch the
    // borrowed payload through `black_box` (parity with C's "observe the payload"
    // callback — the point is the dispatch cost).
    let cb = payload_handler_new(|p| {
        black_box(p.id);
    });
    bench("callback", &format!("native.{cat}"), n, || {
        storage_callback(storage, &cb);
    });
}

/// Run the vector ops for one string category: a whole batch of `k` payloads per call,
/// `vec_iters` calls, reported as ns per call (compare to `k ×` the single-op number).
fn run_vec_category(
    storage: &mut Storage,
    label: Option<&str>,
    cat: &str,
    k: usize,
    vec_iters: u64,
    sink: &mut i64,
) {
    let batch = make_batch(k, label);

    bench("put_vec", &format!("batch.{cat}"), vec_iters, || {
        // `storage_put_slice` clones the slice, so the input batch is reused.
        storage_put_slice(storage, black_box(&batch));
    });

    storage_put_slice(storage, &batch); // seed for get/callback
    bench("get_vec", &format!("batch.{cat}"), vec_iters, || {
        let v = storage_get_vec(storage).expect("seeded ⇒ present");
        *sink = sink.wrapping_add(v.len() as i64);
        black_box(&v);
    });

    // Whole-batch callback prepared once; fired with the entire slice each iteration.
    let cb = payload_vec_handler_new(|payloads: &[Payload]| {
        black_box(payloads.len());
    });
    bench("callback_vec", &format!("batch.{cat}"), vec_iters, || {
        storage_callback_vec(storage, &cb);
    });
}

fn main() {
    let n = iterations();
    let k = vec_n();
    let vec_iters = (n / k as u64).max(1); // ≈ n elements of work, ns reported per call
    let mut storage = storage_new();
    let mut sink: i64 = 0;

    println!("BEGIN_PERFTEST lang=rust n={n}");
    run_category(&mut storage, Some("hello, payload"), "str", n, &mut sink);
    run_category(&mut storage, None, "null", n, &mut sink);
    run_vec_category(&mut storage, Some("hello, payload"), "str", k, vec_iters, &mut sink);
    run_vec_category(&mut storage, None, "null", k, vec_iters, &mut sink);
    println!("END_PERFTEST");

    // Keep `sink` observable so the benchmarks aren't optimized away.
    println!("(sink = {sink})");
}
