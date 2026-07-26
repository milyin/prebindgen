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
 *     arm): a byte outside `{0,1}` is normalised, not materialised.
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
    note_t titled = note_new_titled(7, "chapter one");
    CHECK(titled.tag == Titled);
    CHECK(titled.titled.id == 7);
    CHECK(strcmp(titled.titled.text, "chapter one") == 0);
    /* …and crosses back IN, rebuilt through the payload's own converter. */
    CHECK(note_value(titled) == 7);

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

int main(void) {
    test_each_arm();
    test_struct_field();
    test_drop();
    test_invalid_tag();
    test_converter_derived_payloads();
    test_out_of_domain_bool_payload();
    test_nested_union_payload();

    if (failures != 0) {
        fprintf(stderr, "FAILED - %d check(s)\n", failures);
        return 1;
    }
    printf("PASS - tagged union: every arm, every position, drop, invalid tag, "
           "converter-derived payloads, out-of-domain bool payload, "
           "nested union payload\n");
    return 0;
}
