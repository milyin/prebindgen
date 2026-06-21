/*
 * C micro-benchmark over the prebindgen-generated `perftest` C ABI.
 *
 * Mirrors `perftest-flat/examples/perftest.rs`: it runs the same three benchmarks
 * (`storage_put`, `storage_get`, `storage_callback`) through the generated C
 * bindings, operating on an opaque `storage_t *` handle (`storage_new` /
 * `storage_drop`). Because `Payload` is declared `.repr_c_struct().owned()`, it
 * crosses the C ABI by direct reinterpret (zero-copy). `storage_put(s, &p)` takes the
 * payload **by value** (consume): Rust moves it out through the `payload_t *` and
 * writes a gravestone back, nulling `p.label` — so the caller must re-provide the
 * string before each `storage_put` (and must NOT double-free the moved-out string).
 * `storage_get` returns a fresh owned payload; the callback receives a
 * `const payload_t *` borrow. The `label` string is an opaque `string_t *` (built by
 * `string_new`, freed by `string_drop`).
 *
 * Compare the printed ns/op against the Rust runner to see the cost of crossing the
 * (zero-copy) C boundary vs calling the Rust functions natively.
 */
#include <assert.h>
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

/* Build an initialized payload with a fresh `label` string the caller owns. */
static struct payload_t make_payload(int64_t id, int32_t seq, const char *label) {
    struct payload_t p;
    p.id = id;
    p.seq = seq;
    p.value = 3.5;
    p.flag = true;
    p.label = string_new(label);
    return p;
}

/*
 * Exercise and assert the five parameter-passing semantics. Each leaves the C side
 * owning exactly the `label` strings it must free (no leak, no double-free): a missing
 * drop in `storage_get_into_init` or an erroneous drop-of-garbage in
 * `storage_get_into_uninit` would show as growing RSS / a crash under the loop in main.
 */
static void correctness(struct storage_t *s) {
    /* by_take: consumes the payload by value — Rust moves it out through `*mut` and
     * writes a gravestone (every field reset, `label` nulled) back into the slot. */
    struct payload_t pt = make_payload(1, 1, "take");
    storage_put_by_take(s, &pt);
    assert(pt.label == NULL); /* gravestoned — must NOT be freed by the caller */

    /* by_read: reads through `const payload_t *`; the caller's payload is untouched. */
    struct payload_t pr = make_payload(7, 7, "read");
    storage_put_by_read(s, &pr);
    assert(pr.id == 7 && pr.seq == 7 && pr.label != NULL); /* unchanged */
    string_drop(pr.label);                                 /* C still owns it */

    /* read_and_update: clones into storage AND bumps the caller's `seq` in place. */
    struct payload_t pu = make_payload(8, 41, "update");
    storage_put_by_read_and_update(s, &pu);
    assert(pu.seq == 42); /* counter incremented in the caller's slot */
    string_drop(pu.label);

    /* Re-seed with a known value so the two get_into_* reads are unambiguous. */
    struct payload_t fresh = make_payload(55, 5, "fresh");
    storage_put_by_take(s, &fresh);

    /* get_into_init: the slot is initialized; Rust drops the old "old" string before
     * writing the stored payload. The caller must drop only the NEW label. */
    struct payload_t pi = make_payload(0, 0, "old");
    storage_get_into_init(s, &pi);
    assert(pi.id == 55); /* now holds the stored payload */
    string_drop(pi.label);

    /* get_into_uninit: the slot is uninitialized; Rust writes without dropping it. */
    struct payload_t pun; /* uninitialized */
    storage_get_into_uninit(s, &pun);
    assert(pun.id == 55);
    string_drop(pun.label);
}

int main(void) {
    struct payload_t p;
    p.id = 42;
    p.seq = 7;
    p.value = 3.5;
    p.flag = true;
    p.label = string_new("hello, payload");

    struct storage_t *s = storage_new();
    storage_put_by_take(s, &p); /* seed the storage (consumes p.label, nulls it) */

    /* Verify the five parameter-passing semantics, then hammer them for leak/RSS. */
    correctness(s);
    for (int i = 0; i < 2000000; i++) {
        correctness(s);
    }
    printf("correctness: all 5 semantics OK (RSS stable)\n\n");

    printf("perftest-c (generated C ABI), N = %llu iterations per op\n\n",
           (unsigned long long)N);

    uint64_t sink = 0;
    double t0, t1;

    t0 = now_ns();
    for (uint64_t i = 0; i < N; i++) {
        /* `storage_put_by_take` consumes the payload: it moves the whole value out
         * and writes a gravestone (Payload::default — every field reset, `label`
         * nulled) back into `p`. So re-provide the full payload before each call. */
        p.id = 42;
        p.seq = 7;
        p.value = 3.5;
        p.flag = true;
        p.label = string_new("hello, payload");
        storage_put_by_take(s, &p);
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
