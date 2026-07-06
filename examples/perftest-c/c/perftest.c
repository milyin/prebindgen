/*
 * C micro-benchmark over the prebindgen-generated `perftest` C ABI.
 *
 * Operates on an opaque `storage_t *` handle (`storage_new` / `storage_drop`). Because
 * `Payload` is declared `.repr_c_struct()` (owned-ness inferred from its `label` field),
 * it crosses the C ABI by direct reinterpret (zero-copy), and the generator emits the
 * right wrapper for each of the five parameter-passing semantics:
 *   - storage_put_by_take(payload_t *)              by-value consume (move out + gravestone)
 *   - storage_put_by_read(const payload_t *)        shared read borrow
 *   - storage_put_by_read_and_update(payload_t *)   read + write back (bumps a counter)
 *   - storage_get_into_init(payload_t *)            out-param; drops the old value first
 *   - storage_get_into_uninit(payload_t *)          out-param into uninitialized memory
 * plus `storage_get` (return-value) and `storage_callback` (a `const payload_t *` borrow).
 *
 * Each op is timed in TWO variants: `.str` (a realistic `label` string, the opaque
 * `string_t *` built by `string_new` / freed by `string_drop`) and `.null` (NULL label).
 * The string `malloc`/`free` dominates the per-op cost, so the `.null` variant isolates
 * the FFI + ownership-machinery cost.
 *
 * Output is the shared normalized block (see `examples/perftest-bench.sh`):
 *   BEGIN_PERFTEST lang=c n=<N>
 *   <op> <variant> <ns_per_op> <mops>
 *   END_PERFTEST
 * `N` is overridable via the `PERFTEST_N` env var (default 5_000_000). Compare against the
 * Rust runner (native) and the Kotlin runner (JNI) for the same operations.
 */
#include <assert.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>

#include "perftest.h"

/* Iterations per measured variant; set from PERFTEST_N in main (default 5_000_000). */
static uint64_t g_n = 5000000ULL;
/* Batch size for the vector ops; set from PERFTEST_VEC_N in main (default 16). */
static uint64_t g_vec_n = 16ULL;

/* Vector ops process `g_vec_n` elements per call, so they run `g_n / g_vec_n`
 * iterations — keeping total element work ≈ the single-op `g_n` (bounded wall-time)
 * while the reported ns is PER CALL (the whole-batch crossing cost). */
static uint64_t vec_iters(void) {
    uint64_t it = g_n / (g_vec_n ? g_vec_n : 1);
    return it ? it : 1;
}

static double now_ns(void) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    return (double)ts.tv_sec * 1.0e9 + (double)ts.tv_nsec;
}

/* Callback: a pure observer of the borrowed payload. */
static void on_payload(const struct payload_t *pl, void *ctx) {
    *(uint64_t *)ctx += pl->id;
}

/* Whole-batch callback: the slice is delivered BY REFERENCE (`const payload_t *` +
 * length), so iterate it in place — no per-element copy. */
static void on_payload_vec(const struct payload_t *arr, uintptr_t n, void *ctx) {
    for (uintptr_t i = 0; i < n; i++) *(uint64_t *)ctx += arr[i].id;
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

/* Build a C-owned batch of `n` payloads (each with the given `label`). The block is
 * libc-`malloc`'d, so it is freed with libc `free` (via `free_batch`) — distinct from
 * `perftest_free`, which releases blocks malloc'd inside the Rust cdylib. */
static struct payload_t *make_batch(uint64_t n, const char *label) {
    struct payload_t *b = malloc((size_t)n * sizeof *b);
    for (uint64_t i = 0; i < n; i++) b[i] = make_payload((int64_t)i, (int32_t)i, label);
    return b;
}

static void free_batch(struct payload_t *b, uint64_t n) {
    for (uint64_t i = 0; i < n; i++)
        if (b[i].label) string_drop(b[i].label);
    free(b);
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
     * writing the stored payload. Returns true (present) and the caller drops the NEW
     * label. */
    struct payload_t pi = make_payload(0, 0, "old");
    assert(storage_get_into_init(s, &pi)); /* present → wrote */
    assert(pi.id == 55);                   /* now holds the stored payload */
    string_drop(pi.label);

    /* get_into_uninit: the slot is uninitialized; Rust writes without dropping it.
     * Returns true (present) and initializes the slot. */
    struct payload_t pun; /* uninitialized */
    assert(storage_get_into_uninit(s, &pun));
    assert(pun.id == 55);
    string_drop(pun.label);

    /* Array API. storage_put_slice takes a `const payload_t *` slice read BY CLONE
     * (zero-copy reinterpret of the block to `&[Payload]`), so the caller's input
     * array — including its `label` strings — is untouched and still C-owned.
     * storage_get_vec returns a malloc'd `(payload_t *, size_t)` batch; the C side
     * frees each element's `label` with `payload_drop`, then the block with `perftest_free`. */
    struct payload_t batch[3] = {
        make_payload(10, 1, "a"),
        make_payload(20, 2, NULL),
        make_payload(30, 3, "ccc"),
    };
    storage_put_slice(s, batch, 3);
    assert(batch[0].label != NULL && batch[1].label == NULL && batch[2].label != NULL);

    /* get_vec → `bool f(.., payload_t **out, size_t *out_len)`: true + a malloc'd batch
     * (None ⇒ false). The C side frees each element's label then the block. */
    struct payload_t *out = NULL;
    uintptr_t out_len = 0;
    assert(storage_get_vec(s, &out, &out_len));
    assert(out_len == 3);
    assert(out[0].id == 10 && out[1].id == 20 && out[2].id == 30);
    assert(out[0].seq == 1 && out[1].seq == 2 && out[2].seq == 3);
    assert(out[0].label != NULL && string_len(out[0].label) == 1); /* "a" */
    assert(out[1].label == NULL);
    assert(out[2].label != NULL && string_len(out[2].label) == 3); /* "ccc" */
    for (uintptr_t i = 0; i < out_len; i++) payload_drop(&out[i]); /* free each label */
    perftest_free(out);                                            /* free the block */

    /* Whole-batch callback: ONE fire, the slice delivered by reference; sum the ids
     * over the 3 stored payloads (10 + 20 + 30 == 60). */
    uint64_t cb_sum = 0;
    struct closure_payload_vec_t vclosure = {.context = &cb_sum, .call = on_payload_vec, .drop = NULL};
    struct payload_vec_handler_t *vhandler = payload_vec_handler_new(vclosure);
    storage_callback_vec(s, vhandler);
    assert(cb_sum == 60);
    payload_vec_handler_drop(vhandler);

    /* The single-payload `storage_get` → `bool f(.., payload_t *out)`: the first
     * element of the stored batch. */
    struct payload_t first;
    assert(storage_get(s, &first));
    assert(first.id == 10);
    payload_drop(&first);
    string_drop(batch[0].label); /* the input labels are still C-owned (by-clone read) */
    string_drop(batch[2].label);

    /* Empty contract: clearing with an empty slice ⇒ every get reports absence. */
    storage_put_slice(s, NULL, 0); /* clear */
    struct payload_t none_slot;
    assert(!storage_get(s, &none_slot));
    struct payload_t *none_arr = NULL;
    uintptr_t none_len = 7;
    assert(!storage_get_vec(s, &none_arr, &none_len));
    struct payload_t init_slot = make_payload(99, 9, "keep");
    assert(!storage_get_into_init(s, &init_slot)); /* left untouched */
    assert(init_slot.id == 99);
    string_drop(init_slot.label);
}

/*
 * Per-op benchmarks. Each times `g_n` iterations and returns the elapsed ns, parameterized
 * by `label` (a string, or NULL for the isolated variant). Each keeps correct
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
    for (uint64_t i = 0; i < g_n; i++) {
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
    for (uint64_t i = 0; i < g_n; i++) {
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
    for (uint64_t i = 0; i < g_n; i++) {
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
    for (uint64_t i = 0; i < g_n; i++) {
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
    for (uint64_t i = 0; i < g_n; i++) {
        storage_get_into_uninit(s, &p);
        *sink += (uint64_t)p.id;
        if (p.label) string_drop(p.label);
    }
    return now_ns() - t0;
}

/* get: `bool storage_get(.., payload_t *out)` — true + a fresh owned payload each iter
 * (its cloned label freed by drop). Seeded, so it is always present. */
static double bench_get(struct storage_t *s, const char *label, uint64_t *sink) {
    struct payload_t seed = make_payload(55, 5, label);
    storage_put_by_take(s, &seed);
    double t0 = now_ns();
    for (uint64_t i = 0; i < g_n; i++) {
        struct payload_t g;
        if (storage_get(s, &g)) {
            *sink += (uint64_t)g.id;
            payload_drop(&g);
        }
    }
    return now_ns() - t0;
}

/* callback: a `const payload_t *` borrow — never touches the label, so both variants
 * measure ~the same (the FFI trampoline dispatch). The callback is prepared ONCE into a
 * reusable `payload_handler_t *` (the "declare the subscriber once" step), then the loop
 * measures `storage_callback` itself firing it — matching the Rust/Kotlin benches. */
static double bench_callback(struct storage_t *s, const char *label, uint64_t *sink) {
    struct payload_t seed = make_payload(55, 5, label);
    storage_put_by_take(s, &seed);
    struct closure_payload_t closure;
    closure.context = sink;
    closure.call = on_payload;
    closure.drop = NULL;
    struct payload_handler_t *handler = payload_handler_new(closure);
    double t0 = now_ns();
    for (uint64_t i = 0; i < g_n; i++) {
        storage_callback(s, handler);
    }
    double dt = now_ns() - t0;
    payload_handler_drop(handler);
    return dt;
}

/* ── Vector ops: store/get/callback a whole batch of `g_vec_n` payloads per call ──
 * Each runs `vec_iters()` calls and returns the total ns; `emit_vec` reports ns PER
 * CALL. Compare against `g_vec_n ×` the single-op number to see FFI-overhead
 * amortization. */

/* put_vec: `storage_put_slice` reads the batch BY CLONE, so the input array (and its
 * labels) is reused across iters — one batch alloc before, one free after. */
static double bench_put_vec(struct storage_t *s, const char *label, uint64_t *sink) {
    (void)sink;
    uint64_t n = g_vec_n, it = vec_iters();
    struct payload_t *batch = make_batch(n, label);
    double t0 = now_ns();
    for (uint64_t i = 0; i < it; i++) {
        storage_put_slice(s, batch, n);
    }
    double el = now_ns() - t0;
    free_batch(batch, n);
    return el;
}

/* get_vec: each call returns a fresh malloc'd batch (cloned labels); free per iter. */
static double bench_get_vec(struct storage_t *s, const char *label, uint64_t *sink) {
    uint64_t n = g_vec_n, it = vec_iters();
    struct payload_t *seed = make_batch(n, label);
    storage_put_slice(s, seed, n);
    free_batch(seed, n); /* storage cloned it */
    double t0 = now_ns();
    for (uint64_t i = 0; i < it; i++) {
        struct payload_t *arr = NULL;
        uintptr_t m = 0;
        if (storage_get_vec(s, &arr, &m)) {
            for (uintptr_t j = 0; j < m; j++) {
                *sink += (uint64_t)arr[j].id;
                payload_drop(&arr[j]);
            }
            perftest_free(arr);
        }
    }
    return now_ns() - t0;
}

/* callback_vec: the prepared handler is fired once per iter with the whole batch
 * delivered BY REFERENCE (no per-element copy) — `on_payload_vec` sums in place. */
static double bench_callback_vec(struct storage_t *s, const char *label, uint64_t *sink) {
    uint64_t n = g_vec_n, it = vec_iters();
    struct payload_t *seed = make_batch(n, label);
    storage_put_slice(s, seed, n);
    free_batch(seed, n);
    struct closure_payload_vec_t closure = {.context = sink, .call = on_payload_vec, .drop = NULL};
    struct payload_vec_handler_t *handler = payload_vec_handler_new(closure);
    double t0 = now_ns();
    for (uint64_t i = 0; i < it; i++) {
        storage_callback_vec(s, handler);
    }
    double dt = now_ns() - t0;
    payload_vec_handler_drop(handler);
    return dt;
}

typedef double (*bench_fn)(struct storage_t *s, const char *label, uint64_t *sink);

/* Run one op in both variants and print two normalized rows:
 * `<op> <variant>.str <ns> <mops>` and `<op> <variant>.null <ns> <mops>`. */
static void emit(const char *op, const char *variant, bench_fn fn, struct storage_t *s,
                 uint64_t *sink) {
    double ns_str = fn(s, "hello, payload", sink) / (double)g_n;
    double ns_null = fn(s, NULL, sink) / (double)g_n;
    char vbuf[32];
    snprintf(vbuf, sizeof vbuf, "%s.str", variant);
    printf("%-10s %-16s %9.2f %9.1f\n", op, vbuf, ns_str, 1000.0 / ns_str);
    snprintf(vbuf, sizeof vbuf, "%s.null", variant);
    printf("%-10s %-16s %9.2f %9.1f\n", op, vbuf, ns_null, 1000.0 / ns_null);
}

/* Like `emit`, but for a vector op: normalize by the vector iteration count
 * (`vec_iters()`), so the reported ns is PER CALL (each processing `g_vec_n` payloads). */
static void emit_vec(const char *op, const char *variant, bench_fn fn, struct storage_t *s,
                     uint64_t *sink) {
    double it = (double)vec_iters();
    double ns_str = fn(s, "hello, payload", sink) / it;
    double ns_null = fn(s, NULL, sink) / it;
    char vbuf[32];
    snprintf(vbuf, sizeof vbuf, "%s.str", variant);
    printf("%-12s %-16s %9.2f %9.1f\n", op, vbuf, ns_str, 1000.0 / ns_str);
    snprintf(vbuf, sizeof vbuf, "%s.null", variant);
    printf("%-12s %-16s %9.2f %9.1f\n", op, vbuf, ns_null, 1000.0 / ns_null);
}

int main(void) {
    const char *env = getenv("PERFTEST_N");
    if (env) {
        unsigned long long v = strtoull(env, NULL, 10);
        if (v) g_n = (uint64_t)v;
    }
    const char *venv = getenv("PERFTEST_VEC_N");
    if (venv) {
        unsigned long long v = strtoull(venv, NULL, 10);
        if (v) g_vec_n = (uint64_t)v;
    }

    struct storage_t *s = storage_new();

    /* Verify the five parameter-passing semantics, then hammer them for leak/RSS.
     * Diagnostics go to stderr so stdout stays the clean parseable block. */
    correctness(s);
    uint64_t corr = g_n < 1000000ULL ? g_n : 1000000ULL;
    for (uint64_t i = 0; i < corr; i++) {
        correctness(s);
    }
    fprintf(stderr, "correctness: all semantics OK (RSS stable)\n");

    uint64_t sink = 0;
    printf("BEGIN_PERFTEST lang=c n=%llu\n", (unsigned long long)g_n);
    emit("put", "by_take", bench_put_by_take, s, &sink);
    emit("put", "by_read", bench_put_by_read, s, &sink);
    emit("put", "by_read_upd", bench_put_by_read_and_update, s, &sink);
    emit("get", "return", bench_get, s, &sink);
    emit("get", "into_init", bench_get_into_init, s, &sink);
    emit("get", "into_uninit", bench_get_into_uninit, s, &sink);
    emit("callback", "-", bench_callback, s, &sink);
    /* Vector ops: a whole batch of g_vec_n payloads per call (ns reported PER CALL). */
    emit_vec("put_vec", "batch", bench_put_vec, s, &sink);
    emit_vec("get_vec", "batch", bench_get_vec, s, &sink);
    emit_vec("callback_vec", "batch", bench_callback_vec, s, &sink);
    printf("END_PERFTEST\n");

    storage_drop(s);

    printf("(sink = %llu)\n", (unsigned long long)sink);
    return 0;
}
