#ifndef example_H
#define example_H
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct { uint64_t hi; uint64_t lo; } example_int128;
 typedef struct { const uint8_t *data; size_t len; } example_buffer;
void example_free_buffer(example_buffer b);
void example_free_error(char *error);
/* Trusted C ABI: arguments and callback results are not validated. Required pointers must be nonnull, correctly aligned, live and of the declared type; buffers and row ranges must be valid for their lengths. Strings must be UTF-8; characters, enum tags, option tags, selectors and fixed-array lengths must have valid declared representations. Owned values transfer exactly once. Shared/exclusive loans must satisfy Rust aliasing and lifetime rules, including interior pointers. Output pointers must be writable and disjoint from live inputs and each other. Row storage must remain live and unchanged throughout conversion and cleanup. Violating these preconditions is undefined behavior. Explicitly nullable values retain their documented meaning. */
/* Binding target identity; use the matching Rust library. */
#define example_TARGET "x86_64-unknown-linux-gnu"
#ifdef __cplusplus
static_assert(sizeof(void*) * 8 == 64, "prebindgen: header pointer width differs; regenerate for this target");
#else
_Static_assert(sizeof(void*) * 8 == 64, "prebindgen: header pointer width differs; regenerate for this target");
#endif
/* Field paths below start at a source parameter or return and list their ABI values.
 * Optional groups below name their discriminator. Rules apply only in active
 * selector arms and present enclosing groups. Absent payload wires are ignored. */
typedef struct example_calculator example_calculator;
void example_drop_calculator(example_calculator *ptr);
typedef struct example_local_counter example_local_counter;
void example_drop_local_counter(example_local_counter *ptr);
typedef struct example_caption example_caption;
void example_drop_caption(example_caption *ptr);
typedef struct example_foo example_foo;
void example_drop_foo(example_foo *ptr);
typedef struct { const uint8_t *data; size_t len; } example_buffer_view;
/* Exclusive borrowed bytes; never free or consume. Keep the owner alive, and do not access it or overlapping storage until the view's last use. */
typedef struct { uint8_t *data; size_t len; } example_buffer_view_mut;
/* Non-Result calls return their first value directly; additional values use output pointers.
 * Binding failures require explicit .panic() and abort. Destructors return void. */
example_calculator * example_calculator_new(void);
example_caption * example_caption_new(uint64_t p0_w1, example_buffer p1_w1, uint8_t p2_w1);
uint64_t example_foo_get_id(example_foo * p0_w1);
example_foo * example_foo_new(uint64_t p0_w1);
example_local_counter * example_local_counter_label(example_local_counter * p0_w1, example_buffer p1_w1);
example_local_counter * example_local_counter_new(uint64_t p0_w1);
uint64_t example_local_counter_value(const example_local_counter * p0_w1);
#ifdef __cplusplus
}
#endif
#endif
