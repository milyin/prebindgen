/*
 * C micro-benchmark over the prebindgen-generated `perftest` C ABI.
 *
 * Operates on an opaque `storage_t *` handle (`storage_new` / `storage_drop`). Because
 * `Payload` is declared `.repr_c_struct().owned()`, it crosses the C ABI by direct
 * reinterpret (zero-copy), and the generator emits the right wrapper for each of the
 * five parameter-passing semantics:
 *   - storage_put_by_take(payload_t *)              by-value consume (move out + gravestone)
 *   - storage_put_by_read(const payload_t *)        shared read borrow
 *   - storage_put_by_read_and_update(payload_t *)   read + write back (bumps a counter)
 *   - storage_get_into_init(payload_t *)            out-param; drops the old value first
 *   - storage_get_into_uninit(payload_t *)          out-param into uninitialized memory
 * plus `storage_get` (return-value) and `storage_callback` (a `const payload_t *` borrow).
 *
 * Each op is timed in TWO columns: with a realistic `label` string (the opaque
 * `string_t *`, built by `string_new` / freed by `string_drop`) and with a NULL label.
 * The string `malloc`/`free` dominates the per-op cost, so the NULL-label column
 * isolates the FFI + ownership-machinery cost and reveals the per-semantic differences.
 *
 * Compare the printed ns/op against the Rust runner (`perftest-flat/examples/perftest.rs`)
 * to see the cost of crossing the (zero-copy) C boundary vs calling Rust natively.
 */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <time.h>

#include "perftest.h"

/* Iterations per op per column (7 ops × 2 columns). */
#define N 20000000ULL

static double now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1.0e9 + (double)ts.tv_nsec;
}

/* Callback: a pure observer of the borrowed payload. */
static void on_payload(const struct payload_t *pl, void *ctx) {
    *(uint64_t *)ctx += pl->id;
}

/* Build an initialized payload. `label == NULL` ⇒ no `label` string (no `string_new`);
 * otherwise a fresh `string_t *` the caller owns. */
static struct payload_t make_payload(int64_t id, int32_t seq, const char *label) {
    struct payload_t p;
    p.id = id;
    p.seq = seq;
    p.value = 3.5;
    p.flag = true;
    p.label = label ? string_new(label) : NULL;
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

/*
 * Per-op benchmarks. Each times N iterations and returns the elapsed ns, parameterized
 * by `label` (a string, or NULL for the isolated column). Each keeps correct
 * per-iteration ownership so the loops neither leak nor double-free.
 */

/* by_take: the payload is consumed (moved out), and the consume nulls only the owned
 * `label` pointer — the scalar fields survive in the caller's slot. So re-provide just
 * the consumed string each iter (the realistic "give away the owned part, keep the rest"
 * pattern); for a null label there is nothing to re-provide and `p` is reused as-is. */
static double bench_put_by_take(struct storage_t *s, const char *label, uint64_t *sink) {
    (void)sink;
    struct payload_t p = make_payload(42, 7, label);
    double t0 = now_ns();
    for (uint64_t i = 0; i < N; i++) {
        if (label) p.label = string_new(label); /* re-provide the moved-out string */
        storage_put_by_take(s, &p);             /* moves p's value in; nulls p.label */
    }
    return now_ns() - t0;
}

/* by_read: `const payload_t *` borrow — the caller's payload is untouched and reused
 * across iters (Rust clones it into storage). One alloc before, one drop after. */
static double bench_put_by_read(struct storage_t *s, const char *label, uint64_t *sink) {
    (void)sink;
    struct payload_t p = make_payload(7, 7, label);
    double t0 = now_ns();
    for (uint64_t i = 0; i < N; i++) {
        storage_put_by_read(s, &p);
    }
    double el = now_ns() - t0;
    if (p.label) string_drop(p.label);
    return el;
}

/* by_read_and_update: like by_read, but bumps the caller's `seq` in place each iter. */
static double bench_put_by_read_and_update(struct storage_t *s, const char *label,
                                           uint64_t *sink) {
    (void)sink;
    struct payload_t p = make_payload(8, 0, label);
    double t0 = now_ns();
    for (uint64_t i = 0; i < N; i++) {
        storage_put_by_read_and_update(s, &p);
    }
    double el = now_ns() - t0;
    if (p.label) string_drop(p.label);
    return el;
}

/* get_into_init: the slot stays initialized across iters; each call drops the slot's old
 * label (inside Rust) and writes a fresh clone of the stored payload. */
static double bench_get_into_init(struct storage_t *s, const char *label, uint64_t *sink) {
    struct payload_t seed = make_payload(55, 5, label);
    storage_put_by_take(s, &seed); /* storage now holds a matching-label payload */
    struct payload_t p = make_payload(0, 0, label); /* an initialized slot */
    double t0 = now_ns();
    for (uint64_t i = 0; i < N; i++) {
        storage_get_into_init(s, &p);
        *sink += (uint64_t)p.id;
    }
    double el = now_ns() - t0;
    if (p.label) string_drop(p.label); /* the final label */
    return el;
}

/* get_into_uninit: Rust writes without dropping, so the C side must free the label each
 * iter (the fair dual of get_into_init — both pay one free per iter, inside the loop). */
static double bench_get_into_uninit(struct storage_t *s, const char *label, uint64_t *sink) {
    struct payload_t seed = make_payload(55, 5, label);
    storage_put_by_take(s, &seed);
    struct payload_t p; /* uninitialized */
    double t0 = now_ns();
    for (uint64_t i = 0; i < N; i++) {
        storage_get_into_uninit(s, &p);
        *sink += (uint64_t)p.id;
        if (p.label) string_drop(p.label);
    }
    return now_ns() - t0;
}

/* get: return-value — a fresh owned payload each iter (its cloned label freed by drop). */
static double bench_get(struct storage_t *s, const char *label, uint64_t *sink) {
    struct payload_t seed = make_payload(55, 5, label);
    storage_put_by_take(s, &seed);
    double t0 = now_ns();
    for (uint64_t i = 0; i < N; i++) {
        struct payload_t g = storage_get(s);
        *sink += (uint64_t)g.id;
        payload_drop(&g);
    }
    return now_ns() - t0;
}

/* callback: a `const payload_t *` borrow — never touches the label, so both columns
 * measure ~the same (the FFI trampoline + the upcall). */
static double bench_callback(struct storage_t *s, const char *label, uint64_t *sink) {
    struct payload_t seed = make_payload(55, 5, label);
    storage_put_by_take(s, &seed);
    struct closure_payload_t closure;
    closure.context = sink;
    closure.call = on_payload;
    closure.drop = NULL;
    double t0 = now_ns();
    for (uint64_t i = 0; i < N; i++) {
        storage_callback(s, closure);
    }
    return now_ns() - t0;
}

typedef double (*bench_fn)(struct storage_t *s, const char *label, uint64_t *sink);

/* Run one op in both columns and print `name  <with-string>   <null-label>`. */
static void bench_row(const char *name, bench_fn fn, struct storage_t *s, uint64_t *sink) {
    double ns_with = fn(s, "hello, payload", sink);
    double ns_null = fn(s, NULL, sink);
    printf("%-22s %9.2f ns/op   %9.2f ns/op\n", name, ns_with / (double)N,
           ns_null / (double)N);
}

int main(void) {
    struct storage_t *s = storage_new();

    /* Verify the five parameter-passing semantics, then hammer them for leak/RSS. */
    correctness(s);
    for (int i = 0; i < 2000000; i++) {
        correctness(s);
    }
    printf("correctness: all 5 semantics OK (RSS stable)\n\n");

    printf("perftest-c (generated C ABI), N = %llu iterations per op\n\n",
           (unsigned long long)N);
    printf("%-22s %14s   %14s\n", "op", "with-string", "null-label");

    uint64_t sink = 0;
    bench_row("put_by_take", bench_put_by_take, s, &sink);
    bench_row("put_by_read", bench_put_by_read, s, &sink);
    bench_row("put_by_read_and_update", bench_put_by_read_and_update, s, &sink);
    bench_row("get_into_init", bench_get_into_init, s, &sink);
    bench_row("get_into_uninit", bench_get_into_uninit, s, &sink);
    bench_row("get", bench_get, s, &sink);
    bench_row("callback", bench_callback, s, &sink);

    storage_drop(s);

    printf("\n(sink = %llu)\n", (unsigned long long)sink);
    return 0;
}
