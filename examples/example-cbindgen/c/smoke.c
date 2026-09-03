/*
 * C smoke test over the prebindgen-generated `example_flat` C ABI, focused on
 * the **tagged union** (`Shape`, declared `.tagged_union()`).
 *
 * A data-carrying Rust enum crosses by value as a `#[repr(C)]` enum with payload
 * variants, which cbindgen renders as a tag + `union`:
 *
 *   typedef struct shape_t {
 *     shape_t_Tag tag;
 *     union { struct { double circle; }; Rect_Body rect; Labeled_Body labeled; };
 *   } shape_t;
 *
 * What this exercises:
 *   - constructing and reading EACH arm — unit, single-payload tuple,
 *     multi-field named, and the owning tuple arm (`char *` + a C enum);
 *   - the union in every position: returned, taken by value as a parameter,
 *     and carried through a `drawing_t` data-struct field;
 *   - `shape_drop` freeing the ACTIVE arm, and being a no-op the second time
 *     (the freed slot is nulled) and on a non-owning arm;
 *   - a tag NO variant has. A C caller can always write one, so the binding
 *     validates it before a Rust enum exists: `shape_try_area` reports it
 *     through its `char **e`, and `shape_drop` ignores it. Constructing that
 *     value is the point of the check — it is not a Rust `shape_t` at all;
 *   - the same rule one level down, on a `bool` payload (`note_t`'s `Flagged`
 *     arm): a byte outside `{0,1}` is normalised, not materialised;
 *   - and one level below THAT, on a `bool` field of a `data_struct` payload
 *     (`caption_t`'s `emphatic`) — which is what makes the invariant
 *     transitive rather than true only of the tag and the direct payloads.
 *
 * Exits non-zero on the first failed check.
 *
 * This file is also the repo's ownership contract in executable form — every
 * arm C receives is C's to release — so it is run under ASan/LSan/UBSan by
 * `examples/smoke-asan.sh` in CI. A missing `*_drop` or `example_free` here is
 * a failure, not a silent PASS.
 */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* The committed header is per VARIANT — target architecture and feature set —
 * like the Rust file `lib.rs` `include!`s, because `#[prebindgen]` cfg handling
 * makes the generated surface differ by both. The default-feature name is derived
 * here; a build that enables features passes `-DEXAMPLE_FLAT_HEADER=...` (see
 * CMakeLists.txt), since only the builder knows which features it asked for. */
#if !defined(EXAMPLE_FLAT_HEADER)
#if defined(__x86_64__) || defined(_M_X64)
#define EXAMPLE_FLAT_HEADER "example_flat_x86_64.h"
#elif defined(__aarch64__) || defined(_M_ARM64)
#define EXAMPLE_FLAT_HEADER "example_flat_aarch64.h"
#else
#error "no committed example_flat header for this target architecture"
#endif
#endif
#include EXAMPLE_FLAT_HEADER

static int failures = 0;

#define CHECK(cond)                                                            \
    do {                                                                       \
        if (!(cond)) {                                                         \
            fprintf(stderr, "FAIL %s:%d: %s\n", __FILE__, __LINE__, #cond);     \
            failures++;                                                        \
        }                                                                      \
    } while (0)

/* Every arm is constructible and reads back as itself. */
static void test_each_arm(void) {
    shape_t empty = shape_new_empty();
    CHECK(empty.tag == Empty);
    CHECK(shape_area(empty) == 0.0);

    shape_t circle = shape_new_circle(2.0);
    CHECK(circle.tag == Circle);
    CHECK(circle.circle == 2.0);
    CHECK(fabs(shape_area(circle) - 3.14159265358979323846 * 4.0) < 1e-9);

    shape_t rect = shape_new_rect(3.0, 4.0);
    CHECK(rect.tag == Rect);
    CHECK(rect.rect.width == 3.0);
    CHECK(rect.rect.height == 4.0);
    CHECK(shape_area(rect) == 12.0);

    shape_t labeled = shape_new_labeled("hexagon", Mul);
    CHECK(labeled.tag == Labeled);
    CHECK(strcmp(labeled.labeled._0, "hexagon") == 0);
    CHECK(labeled.labeled._1 == Mul);
    /* The label reads back across the boundary; `shape_get_label` consumes the
     * union by value and returns a fresh `char *` block. */
    char *label = shape_get_label(labeled);
    CHECK(strcmp(label, "hexagon") == 0);
    example_free(label);
    shape_drop(&labeled);

    /* A non-`Labeled` arm reads back as the empty string. */
    char *none = shape_get_label(shape_new_circle(1.0));
    CHECK(strcmp(none, "") == 0);
    example_free(none);
}

/* The union as a data-struct field: out through `drawing_new`, back in through
 * `drawing_get_shape`. The struct has no destructor, so the owning field is
 * released individually — exactly the existing data-struct contract.
 *
 * OWNERSHIP: passing a union by value does NOT transfer its payload. The callee
 * copies the string contents out, so every union C holds — the argument it
 * passed and each one it got back — is its own to drop. Each `shape_new_labeled`
 * below is therefore paired with a `shape_drop`. */
static void test_struct_field(void) {
    shape_t inner = shape_new_labeled("triangle", Add);
    drawing_t d = drawing_new(7, inner);
    CHECK(d.id == 7);
    CHECK(d.shape.tag == Labeled);
    CHECK(strcmp(d.shape.labeled._0, "triangle") == 0);

    /* `d.shape` is a fresh block, not `inner`'s: the argument survives the call
     * and is still the caller's to release. */
    CHECK(d.shape.labeled._0 != inner.labeled._0);

    shape_t back = drawing_get_shape(d);
    CHECK(back.tag == Labeled);
    CHECK(strcmp(back.labeled._0, "triangle") == 0);
    CHECK(back.labeled._1 == Add);
    shape_drop(&back);
    shape_drop(&d.shape);
    shape_drop(&inner);

    /* A plain-data arm rides through the field just the same, and owns nothing. */
    drawing_t plain = drawing_new(1, shape_new_rect(2.0, 5.0));
    shape_t plain_back = drawing_get_shape(plain);
    CHECK(plain_back.tag == Rect);
    CHECK(shape_area(plain_back) == 10.0);
}

/* `shape_drop` frees the active arm, nulls the freed slot, and tolerates being
 * called again, on a non-owning arm, and on NULL. */
static void test_drop(void) {
    shape_t owning = shape_new_labeled("to-free", Sub);
    CHECK(owning.labeled._0 != NULL);
    shape_drop(&owning);
    CHECK(owning.labeled._0 == NULL);
    shape_drop(&owning); /* second drop: the nulled slot makes it a no-op */

    shape_t plain = shape_new_rect(1.0, 1.0);
    shape_drop(&plain); /* non-owning arm: nothing to free */
    CHECK(shape_area(plain) == 1.0);

    shape_drop(NULL);
}

/* A tag no variant declares — what a buggy or hostile C caller produces. It is
 * written through the raw bytes because no valid `shape_t` has this value. */
static shape_t with_raw_tag(int tag) {
    shape_t s = shape_new_rect(1.0, 1.0);
    memcpy(&s, &tag, sizeof tag);
    return s;
}

/* The fallible route: a rejected tag reaches the error channel, and the call
 * has no other effect. */
static void test_invalid_tag(void) {
    double out = -1.0;
    char *e = NULL;

    /* A valid arm first, so the comparison is against a working call. */
    CHECK(shape_try_area(shape_new_rect(3.0, 4.0), &out, &e));
    CHECK(e == NULL);
    CHECK(out == 12.0);

    /* 99 selects no variant. */
    out = -1.0;
    CHECK(!shape_try_area(with_raw_tag(99), &out, &e));
    CHECK(e != NULL);
    CHECK(strstr(e, "invalid tag 99") != NULL);
    CHECK(out == -1.0);
    example_free(e);
    e = NULL;

    /* Negative too: the tag is read as a signed int, not truncated into a
     * variant. */
    CHECK(!shape_try_area(with_raw_tag(-1), &out, &e));
    CHECK(e != NULL);
    CHECK(strstr(e, "invalid tag -1") != NULL);
    example_free(e);
    e = NULL;

    /* The domain error still comes through the same channel, unchanged. The
     * argument is by value, so its label block is still ours to release. */
    shape_t labeled = shape_new_labeled("hexagon", Add);
    CHECK(!shape_try_area(labeled, &out, &e));
    CHECK(e != NULL);
    CHECK(strstr(e, "no area") != NULL);
    example_free(e);
    shape_drop(&labeled);

    /* `shape_drop` is the other C entry point into these bytes: it checks the
     * tag too, and an out-of-range one is simply nothing to release. */
    shape_t bad = with_raw_tag(99);
    shape_drop(&bad);
}

/* Payload kinds the zero-copy mirror policy could not express (#158 part 2):
 * a nested `data_struct` crossing BY VALUE, and a converted leaf whose wire is
 * its conversion's destination rather than its own layout. Both directions,
 * plus the union's drop reaching THROUGH the struct payload to free the
 * `char *` it owns. */
static void test_converter_derived_payloads(void) {
    /* Unit arm. */
    note_t silent = note_new_silent();
    CHECK(silent.tag == Silent);
    CHECK(note_value(silent) == 0);

    /* Converted leaf: `Millis` crosses as the `uint64_t` its conversion
     * produces, not as its own struct layout. */
    note_t after = note_new_after(1500);
    CHECK(after.tag == After);
    CHECK(after.after == 1500);
    CHECK(note_value(after) == 1500);

    /* Nested data_struct by value: the whole record rides in the union arm. */
    note_t titled = note_new_titled(7, "chapter one", true);
    CHECK(titled.tag == Titled);
    CHECK(titled.titled.id == 7);
    CHECK(strcmp(titled.titled.text, "chapter one") == 0);
    CHECK(titled.titled.emphatic == true);
    /* …and crosses back IN, rebuilt through the payload's own converter. */
    CHECK(note_value(titled) == 7);
    CHECK(note_emphatic(titled) == true);

    /* The union's drop reaches through the struct payload and frees its
     * owning field, nulling the slot so a second drop is a no-op. */
    note_drop(&titled);
    CHECK(titled.titled.text == NULL);
    note_drop(&titled);

    /* Non-owning arms tolerate the drop and stay readable. */
    note_drop(&after);
    CHECK(note_value(after) == 1500);
    note_drop(NULL);
}

/* A `bool` payload. `bool` is the one scalar whose domain is restricted, and C
 * can put any byte in the slot — `memcpy` from a `char`, a union, a struct read
 * off the wire. Rust may not HOLD such a byte in a `bool`, so the payload
 * crosses behind `MaybeUninit` (spelled `bool` in the header, unchanged) and the
 * byte is normalised C-style on the way in: nonzero is true. */
static void test_out_of_domain_bool_payload(void) {
    note_t on = note_new_flagged(true);
    CHECK(on.tag == Flagged);
    CHECK(note_value(on) == 1);

    note_t off = note_new_flagged(false);
    CHECK(off.tag == Flagged);
    CHECK(note_value(off) == 0);

    /* A byte no Rust `bool` may hold, written the way a C caller can. */
    unsigned char junk = 2;
    memcpy(&off.flagged, &junk, sizeof junk);
    CHECK(note_value(off) == 1);

    junk = 0xff;
    memcpy(&off.flagged, &junk, sizeof junk);
    CHECK(note_value(off) == 1);

    /* A bool owns nothing: the drop is a no-op on this arm. */
    note_drop(&off);
}

/* The same rule one level DOWN, on a `bool` field of a `data_struct` payload
 * (#170 instance 2). This is what makes the union's "no invalid value is ever
 * materialised" invariant transitive: the nested record is rebuilt through
 * per-field wires, so each restricted-validity field has to be normalised on
 * its own, not just the tag and the direct payloads. */
static void test_out_of_domain_bool_data_struct_field(void) {
    static const unsigned char bytes[] = {0, 1, 2, 0xff};
    static const bool expected[] = {false, true, true, true};

    for (size_t i = 0; i < sizeof bytes / sizeof bytes[0]; i++) {
        note_t titled = note_new_titled(7, "chapter one", false);
        memcpy(&titled.titled.emphatic, &bytes[i], 1);

        CHECK(note_emphatic(titled) == expected[i]);
        /* The rest of the record still decodes normally around it. */
        CHECK(note_value(titled) == 7);

        note_drop(&titled);
    }
}

/* A union nested inside a struct payload. `note_t`'s `Sketched` arm carries a
 * `drawing_t` BY VALUE, whose own `shape` field is another union with an owning
 * arm — a `char *` two levels down that nothing else can reach: a union arm is
 * not a top-level struct field the C caller releases by hand. `note_drop` has to
 * reach through the record and call the nested union's own typed drop. */
static void test_nested_union_payload(void) {
    note_t sketched = note_new_sketched(3, "outline");
    CHECK(sketched.tag == Sketched);
    CHECK(sketched.sketched.id == 3);
    CHECK(sketched.sketched.shape.tag == Labeled);
    CHECK(strcmp(sketched.sketched.shape.labeled._0, "outline") == 0);

    /* …and crosses back IN, rebuilt through both levels of converter. */
    CHECK(note_value(sketched) == 3);

    /* The outer drop reaches the inner union's active arm and nulls it, so a
     * second drop of either is a no-op. */
    note_drop(&sketched);
    CHECK(sketched.sketched.shape.labeled._0 == NULL);
    note_drop(&sketched);
    shape_drop(&sketched.sketched.shape);
}

/* #189's alias preflight, under the leak detector that makes it meaningful.
 * `calculator_merge(x, x)` would hand one allocation to two `Box::from_raw`
 * calls — a double free ASan reports immediately. The generated wrapper
 * compares the pointers before either conversion runs, so the call is rejected
 * and the handle is untouched: still live, still ours to drop exactly once. */
static void test_alias_preflight(void) {
    char *e = NULL;
    calculator_t *c = calculator_new();

    CHECK(calculator_merge(c, c, &e) == NULL);
    CHECK(e != NULL);
    CHECK(strstr(e, "aliasing arguments") != NULL);
    example_free(e);
    e = NULL;

    /* Nothing was consumed — the handle still works and is dropped once. */
    CHECK(calculator_get_value(c) == 0.0);

    /* The mixed consume/borrow shape is rejected by the same rule. */
    double out = -1.0;
    CHECK(!calculator_absorb(c, c, &out, &e));
    CHECK(e != NULL);
    CHECK(out == -1.0);
    example_free(e);
    e = NULL;

    /* Two DISTINCT handles still work — the preflight adds no false positive.
     * Both are consumed by the merge; the result is the only thing to drop. */
    calculator_t *merged = calculator_merge(c, calculator_new(), &e);
    CHECK(merged != NULL);
    CHECK(e == NULL);
    calculator_drop(merged);
}

/* A composite in ARGUMENT position: `impl Fn(Option<f64>)`.
 *
 * A composite has no converter of its own — `Option`/`Vec`/`Cow` resolve to a
 * marker whose destination is `()` — and the real ABI is the shape it lowers
 * to. The callback-argument path used to call that marker as if it were a
 * converter, and a marker takes no arguments, so this shape emitted a binding
 * that did not build (#428).
 *
 * `Option<f64>` has no spare bit pattern, so the shape is a `bool` beside the
 * value and the C `call` takes both. When the flag is false the value is
 * unspecified — the same contract a `Result`'s out-param carries — so this
 * reads it only in the present case. */
struct maybe_calls {
    int fired;
    bool present[2];
    double value;
};

static void on_maybe_value(bool present, double value, void *ctx) {
    struct maybe_calls *calls = (struct maybe_calls *)ctx;
    if (calls->fired < 2) {
        calls->present[calls->fired] = present;
        if (present) {
            calls->value = value;
        }
    }
    calls->fired++;
}

/* The same shape over an enum whose discriminants skip zero — the case
 * `Option<double>` cannot detect.
 *
 * A declared `enum_type`'s wire is the Rust enum itself, so filling the absent
 * slot with any fabricated value builds an invalid one. The slot is left
 * unwritten instead, and C reads it only when the flag says to. */
struct maybe_grades {
    int fired;
    bool present[2];
    enum grade_t value;
};

static void on_maybe_grade(bool present, enum grade_t value, void *ctx) {
    struct maybe_grades *calls = (struct maybe_grades *)ctx;
    if (calls->fired < 2) {
        calls->present[calls->fired] = present;
        if (present) {
            calls->value = value;
        }
    }
    calls->fired++;
}

static void test_optional_enum_callback_arg(void) {
    char *e = NULL;
    calculator_t *c = calculator_new();
    double applied = -1.0;
    CHECK(calculator_apply(c, Add, 20.0, &applied, &e));
    CHECK(e == NULL);

    struct maybe_grades calls = {0, {false, false}, Low};
    struct closure_maybe_grade_t closure = {
        .context = &calls, .call = on_maybe_grade, .drop = NULL};
    calculator_grade_or_none(c, closure);

    CHECK(calls.fired == 2);
    CHECK(calls.present[0]);
    CHECK(calls.value == High);
    CHECK(!calls.present[1]);

    calculator_drop(c);
}

/* A composite whose lowering ALLOCATES, delivered to a closure that cannot
 * receive it.
 *
 * `Vec<f64>` crosses as a malloc'd `(double *, size_t)` the C side owns. A
 * closure struct whose `call` is NULL receives nothing, so converting the
 * argument at all would hand that block to nobody — a leak on every
 * invocation, and one only a leak detector can see (#428 review). This section
 * is why `smoke-asan.sh` runs under LSan: with the encode outside the call
 * guard, the block below is reported and the run fails.
 *
 * The live case is beside it, so the same fixture shows the array still
 * arrives when there IS a callback — a guard that skipped the call as well
 * would pass a leak check and be useless. */
static void on_history_batch(double *values, uintptr_t len, void *ctx) {
    double *sum = (double *)ctx;
    for (uintptr_t i = 0; i < len; i++) {
        *sum += values[i];
    }
    example_free(values); /* the array is C's to free */
}

static void test_allocating_callback_arg(void) {
    char *e = NULL;
    calculator_t *c = calculator_new();
    double applied = -1.0;
    CHECK(calculator_apply(c, Add, 4.0, &applied, &e));
    CHECK(calculator_apply(c, Add, 6.0, &applied, &e));
    CHECK(e == NULL);

    /* Live: the batch arrives and is freed by the receiver. */
    double sum = 0.0;
    struct closure_history_batch_t live = {
        .context = &sum, .call = on_history_batch, .drop = NULL};
    calculator_history_batch(c, live);
    CHECK(sum == 14.0); /* 4 + 10 */

    /* No `call`: nothing may be allocated, because nothing could free it. */
    struct closure_history_batch_t silent = {
        .context = NULL, .call = NULL, .drop = NULL};
    calculator_history_batch(c, silent);

    calculator_drop(c);
}

static void test_optional_callback_arg(void) {
    char *e = NULL;
    calculator_t *c = calculator_new();
    double applied = -1.0;
    CHECK(calculator_apply(c, Add, 7.0, &applied, &e));
    CHECK(applied == 7.0);
    CHECK(e == NULL);

    struct maybe_calls calls = {0, {false, false}, -1.0};
    struct closure_maybe_value_t closure = {
        .context = &calls, .call = on_maybe_value, .drop = NULL};
    calculator_last_or_none(c, closure);

    /* Fires twice from one call: the recorded value, then `None`. */
    CHECK(calls.fired == 2);
    CHECK(calls.present[0]);
    CHECK(calls.value == 7.0);
    CHECK(!calls.present[1]);

    calculator_drop(c);
}

static void test_handle_lifecycle(void) {
    char *e = NULL;
    double out = -1.0;
    calculator_t *c = calculator_new();

    CHECK(calculator_get_value(c) == 0.0);
    CHECK(calculator_get_count(c) == 0);

    CHECK(calculator_apply(c, Add, 2.0, &out, &e));
    CHECK(e == NULL);
    CHECK(out == 2.0);
    CHECK(calculator_apply(c, Mul, 3.0, &out, &e));
    CHECK(out == 6.0);

    /* The mutation stuck on the Rust side of the handle, not on a copy. */
    CHECK(calculator_get_value(c) == 6.0);
    CHECK(calculator_get_count(c) == 2);
    CHECK(calculator_is(c, 6.0));
    CHECK(!calculator_is(c, 2.0));

    /* The clone carries the state over and then diverges. */
    calculator_t *clone = calculator_new_clone(c);
    CHECK(calculator_get_value(clone) == 6.0);
    CHECK(calculator_get_count(clone) == 2);
    CHECK(calculator_apply(clone, Sub, 1.0, &out, &e));
    CHECK(out == 5.0);
    CHECK(calculator_get_value(clone) == 5.0);
    CHECK(calculator_get_value(c) == 6.0);

    calculator_drop(clone);
    calculator_drop(c);
}

/* The `char **e` channel on a `Result`-returning function. Three sources of
 * error reach it — a rejected input string, a domain error, and a discriminant
 * `operation_t` has no variant for — and on the success path neither the error
 * nor the out-param is touched by the other's convention. */
static void test_error_channel(void) {
    char *e = NULL;
    double out = -1.0;

    /* Ok arm: the handle comes back, `e` stays NULL. */
    calculator_t *parsed = calculator_new_from_str("42.5", &e);
    CHECK(parsed != NULL);
    CHECK(e == NULL);
    CHECK(calculator_get_value(parsed) == 42.5);
    CHECK(calculator_get_count(parsed) == 1);

    /* Err arm: NULL handle, message through `e`, and it is ours to free. */
    calculator_t *bad = calculator_new_from_str("not a number", &e);
    CHECK(bad == NULL);
    CHECK(e != NULL);
    CHECK(strstr(e, "parse error") != NULL);
    example_free(e);
    e = NULL;

    /* A domain error leaves the out-param and the handle alone. */
    CHECK(!calculator_apply(parsed, Div, 0.0, &out, &e));
    CHECK(e != NULL);
    CHECK(strstr(e, "division by zero") != NULL);
    CHECK(out == -1.0);
    CHECK(calculator_get_value(parsed) == 42.5);
    CHECK(calculator_get_count(parsed) == 1);
    example_free(e);
    e = NULL;

    /* A C caller can pass an `int` no `operation_t` enumerator names. Like the
     * union tag, it is validated before a Rust `Operation` exists, and reported
     * through the same channel rather than materialised. */
    enum operation_t op;
    int raw = 77;
    memcpy(&op, &raw, sizeof raw);
    CHECK(!calculator_apply(parsed, op, 1.0, &out, &e));
    CHECK(e != NULL);
    CHECK(strstr(e, "invalid discriminant 77") != NULL);
    CHECK(out == -1.0);
    CHECK(calculator_get_value(parsed) == 42.5);
    example_free(e);

    calculator_drop(parsed);
}

/* The two owned returns. Both are plain malloc'd blocks released by the single
 * `example_free` the binding declares — the handles keep their typed `*_drop`,
 * these do not. An empty `Vec` is the null/zero pair, not a block. */
static void test_owned_returns(void) {
    char *e = NULL;
    double out = 0.0;
    calculator_t *c = calculator_new();

    /* A `String` return crosses as a NUL-terminated `char *`. */
    char *empty_repr = calculator_to_string(c);
    CHECK(strcmp(empty_repr, "Calculator(0)") == 0);
    example_free(empty_repr);

    /* No history yet: the array is (NULL, 0), with nothing to free. */
    size_t len = 99;
    double *none = calculator_get_history(c, &len);
    CHECK(none == NULL);
    CHECK(len == 0);

    CHECK(calculator_apply(c, Add, 2.0, &out, &e));
    CHECK(calculator_apply(c, Mul, 3.0, &out, &e));
    CHECK(e == NULL);

    char *repr = calculator_to_string(c);
    CHECK(strcmp(repr, "Calculator(6)") == 0);
    example_free(repr);

    /* `Vec<f64>` crosses as a pointer + a length out-param, in order. */
    double *history = calculator_get_history(c, &len);
    CHECK(len == 2);
    CHECK(history != NULL);
    CHECK(history[0] == 2.0);
    CHECK(history[1] == 6.0);
    example_free(history);

    calculator_drop(c);
}

/* The closure struct C fills in and Rust calls back through. `context` is C's
 * allocation: the generated wrapper owns the closure for the duration and
 * promises to call `drop` exactly once when it releases it. Both halves are
 * checked — the count here, and the block itself by LeakSanitizer, which is
 * what makes a missing `drop` a failure rather than a silent PASS. */
#define CB_CTX_MAGIC 0xC0FFEEu

static double g_seen[8];
static size_t g_seen_n;
static int g_drops;

static void cb_call(double v, void *ctx) {
    CHECK(ctx != NULL && *(uint32_t *)ctx == CB_CTX_MAGIC);
    if (g_seen_n < sizeof g_seen / sizeof g_seen[0]) {
        g_seen[g_seen_n] = v;
    }
    g_seen_n++;
}

static void cb_drop(void *ctx) {
    CHECK(ctx != NULL && *(uint32_t *)ctx == CB_CTX_MAGIC);
    free(ctx);
    g_drops++;
}

static void test_callback(void) {
    char *e = NULL;
    double out = 0.0;
    calculator_t *c = calculator_new();
    CHECK(calculator_apply(c, Add, 2.0, &out, &e));
    CHECK(calculator_apply(c, Mul, 3.0, &out, &e));
    CHECK(e == NULL);

    uint32_t *ctx = malloc(sizeof *ctx);
    *ctx = CB_CTX_MAGIC;
    closure_value_t f = {ctx, cb_call, cb_drop};

    calculator_for_each(c, f);

    /* One upcall per recorded value, in application order, and the context
     * released once the call returned. */
    CHECK(g_seen_n == 2);
    CHECK(g_seen[0] == 2.0);
    CHECK(g_seen[1] == 6.0);
    CHECK(g_drops == 1);

    calculator_drop(c);
}

/* Plain values: a `data_struct` and a fieldless enum crossing both ways. `Foo`'s
 * field SET is target- and feature-dependent (only `id` is universal), so the
 * checks are on `id` and on the round trip, not on a fixed layout. `caption_t`
 * is the data struct with an owned `char *` field — no destructor is generated
 * for a data struct, so the field is released by hand. */
static void test_value_types(void) {
    foo_t f = foo_new(9);
    CHECK(f.id == 9);
    CHECK(foo_get_id(f) == 9);

    /* The discriminants differ per target arch, so compare against the
     * enumerator the header defines for THIS build rather than a literal. */
    enum inside_foo_t d = inside_foo_default();
    CHECK(d == DouddleDee);
    CHECK(inside_foo_value(d) == (int32_t)DouddleDee);
    CHECK(inside_foo_value(DouddleDum) == (int32_t)DouddleDum);

    caption_t cap = caption_new(3, "caption", true);
    CHECK(cap.id == 3);
    CHECK(strcmp(cap.text, "caption") == 0);
    CHECK(cap.emphatic == true);
    example_free(cap.text);

    caption_t plain = caption_new(4, "", false);
    CHECK(strcmp(plain.text, "") == 0);
    CHECK(plain.emphatic == false);
    example_free(plain.text);
}

int main(void) {
    test_each_arm();
    test_struct_field();
    test_drop();
    test_invalid_tag();
    test_converter_derived_payloads();
    test_out_of_domain_bool_payload();
    test_out_of_domain_bool_data_struct_field();
    test_nested_union_payload();
    test_alias_preflight();
    test_handle_lifecycle();
    test_error_channel();
    test_owned_returns();
    test_callback();
    test_value_types();
    test_optional_callback_arg();
    test_optional_enum_callback_arg();
    test_allocating_callback_arg();

    if (failures != 0) {
        fprintf(stderr, "FAILED - %d check(s)\n", failures);
        return 1;
    }
    printf("PASS - tagged union: every arm, every position, drop, invalid tag, "
           "converter-derived payloads, out-of-domain bool payload and "
           "data-struct field, nested union payload, alias preflight, "
           "optional callback argument (scalar and zero-less enum), "
           "allocating callback argument with and without a call; "
           "handle lifecycle, error channel, owned returns, closure struct, "
           "by-value structs and enums\n");
    return 0;
}
