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
 *     value is the point of the check — it is not a Rust `shape_t` at all.
 *
 * Exits non-zero on the first failed check.
 */
#include <math.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

/* The committed header is per target architecture, like the Rust file `lib.rs`
 * `include!`s — `#[prebindgen]` cfg handling makes the generated surface differ
 * per target. */
#if defined(__x86_64__) || defined(_M_X64)
#include "example_flat_x86_64.h"
#elif defined(__aarch64__) || defined(_M_ARM64)
#include "example_flat_aarch64.h"
#else
#error "no committed example_flat header for this target architecture"
#endif

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

int main(void) {
    test_each_arm();
    test_struct_field();
    test_drop();
    test_invalid_tag();

    if (failures != 0) {
        fprintf(stderr, "FAILED - %d check(s)\n", failures);
        return 1;
    }
    printf("PASS - tagged union: every arm, every position, drop, invalid tag\n");
    return 0;
}
