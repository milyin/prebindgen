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
