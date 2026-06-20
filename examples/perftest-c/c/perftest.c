/*
 * C micro-benchmark over the prebindgen-generated `perftest` C ABI.
 *
 * Mirrors `perftest-flat/examples/perftest.rs`: it runs the same three benchmarks
 * (`storage_put`, `storage_get`, `storage_callback`) through the generated C
 * bindings, operating on an opaque `storage_t *` handle (`storage_new` /
 * `storage_drop`). Because `Payload` is declared `.repr_c_struct`, it crosses the C
 * ABI by direct reinterpret (zero-copy): `storage_put(s, &p)` hands Rust the C
 * struct's memory as a `const Payload &`, and the callback receives a
 * `const payload_t *` borrow. The `label` string is an opaque `string_t *` (built by
 * `string_new`, freed by `string_drop`).
 *
 * Compare the printed ns/op against the Rust runner to see the cost of crossing the
 * (zero-copy) C boundary vs calling the Rust functions natively.
 */
#include <stdint.h>
#include <stdio.h>
#include <time.h>

#include "perftest.h"

#define N 50000000ULL

static double now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1.0e9 + (double)ts.tv_nsec;
}

static void report(const char *name, uint64_t n, double elapsed_ns) {
    double ns_per_op = elapsed_ns / (double)n;
    double mops = (double)n / (elapsed_ns / 1.0e9) / 1.0e6;
    printf("%-10s %8.2f ns/op   %8.1f Mops/s\n", name, ns_per_op, mops);
}

/* Callback: a pure observer of the borrowed payload. */
static void on_payload(const struct payload_t *pl, void *ctx) {
    *(uint64_t *)ctx += pl->id;
}

int main(void) {
    struct payload_t p;
    p.id = 42;
    p.seq = 7;
    p.value = 3.5;
    p.flag = true;
    p.label = string_new("hello, payload");

    struct storage_t *s = storage_new();
    storage_put(s, &p); /* seed the storage */

    printf("perftest-c (generated C ABI), N = %llu iterations per op\n\n",
           (unsigned long long)N);

    uint64_t sink = 0;
    double t0, t1;

    t0 = now_ns();
    for (uint64_t i = 0; i < N; i++) {
        storage_put(s, &p);
    }
    t1 = now_ns();
    report("put", N, t1 - t0);

    t0 = now_ns();
    for (uint64_t i = 0; i < N; i++) {
        struct payload_t g = storage_get(s);
        sink += g.id;
        payload_drop(&g); /* frees the cloned `string_t *` each iteration */
    }
    t1 = now_ns();
    report("get", N, t1 - t0);

    struct closure_payload_t closure;
    closure.context = &sink;
    closure.call = on_payload;
    closure.drop = NULL;
    t0 = now_ns();
    for (uint64_t i = 0; i < N; i++) {
        storage_callback(s, closure);
    }
    t1 = now_ns();
    report("callback", N, t1 - t0);

    storage_drop(s);
    string_drop(p.label);

    printf("\n(sink = %llu)\n", (unsigned long long)sink);
    return 0;
}
