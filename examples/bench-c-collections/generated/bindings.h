#ifndef bench_H
#define bench_H
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct { uint64_t hi; uint64_t lo; } bench_int128;
 typedef struct { const uint8_t *data; size_t len; } bench_buffer;
void bench_free_buffer(bench_buffer b);
void bench_free_error(char *error);
/* Binding target identity; use the matching Rust library. */
#define bench_TARGET "x86_64-unknown-linux-gnu"
#ifdef __cplusplus
static_assert(sizeof(void*) * 8 == 64, "prebindgen: header pointer width differs; regenerate for this target");
#else
_Static_assert(sizeof(void*) * 8 == 64, "prebindgen: header pointer width differs; regenerate for this target");
#endif
/* Field paths below start at a source parameter or return and list their ABI values.
 * Optional groups below name their discriminator. Rules apply only in active
 * selector arms and present enclosing groups. Absent payload wires are ignored. */
typedef struct bench_resource bench_resource;
void bench_drop_resource(bench_resource *ptr);
typedef struct { const uint8_t *data; size_t len; } bench_buffer_view;
/* Exclusive borrowed bytes; never free or consume. Keep the owner alive, and do not access it or overlapping storage until the view's last use. */
typedef struct { uint8_t *data; size_t len; } bench_buffer_view_mut;
/* Non-Result calls return their first value directly; additional values use output pointers.
 * Binding failures require explicit .panic() and abort. Destructors return void. */
uint64_t bench_control(uintptr_t p0_w1);
typedef struct {
    bench_resource * v0;
} bench_make_handles_ret_w0_row;
typedef struct { bench_make_handles_ret_w0_row *data; size_t len; } bench_make_handles_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void bench_make_handles_ret_w0_rows_free(bench_make_handles_ret_w0_rows rows);
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
bench_make_handles_ret_w0_rows bench_make_handles(uintptr_t p0_w1);
typedef struct {
    bench_resource * v0;
} bench_make_many_ret_w4_row;
typedef struct { bench_make_many_ret_w4_row *data; size_t len; } bench_make_many_ret_w4_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void bench_make_many_ret_w4_rows_free(bench_make_many_ret_w4_rows rows);
typedef struct {
    bench_make_many_ret_w4_rows v0;
} bench_make_many_ret_w0_row;
typedef struct { bench_make_many_ret_w0_row *data; size_t len; } bench_make_many_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void bench_make_many_ret_w0_rows_free(bench_make_many_ret_w0_rows rows);
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return[]: ret_w4 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
bench_make_many_ret_w0_rows bench_make_many(uintptr_t p0_w1);
typedef struct {
    bench_resource * v0;
} bench_make_nested_ret_w4_row;
typedef struct { bench_make_nested_ret_w4_row *data; size_t len; } bench_make_nested_ret_w4_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void bench_make_nested_ret_w4_rows_free(bench_make_nested_ret_w4_rows rows);
typedef struct {
    bench_make_nested_ret_w4_rows v0;
} bench_make_nested_ret_w0_row;
typedef struct { bench_make_nested_ret_w0_row *data; size_t len; } bench_make_nested_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void bench_make_nested_ret_w0_rows_free(bench_make_nested_ret_w0_rows rows);
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return[]: ret_w4 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
bench_make_nested_ret_w0_rows bench_make_nested(uintptr_t p0_w1);
typedef struct {
    uint64_t v0;
} bench_make_numbers_ret_w0_row;
typedef struct { bench_make_numbers_ret_w0_row *data; size_t len; } bench_make_numbers_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void bench_make_numbers_ret_w0_rows_free(bench_make_numbers_ret_w0_rows rows);
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
bench_make_numbers_ret_w0_rows bench_make_numbers(uintptr_t p0_w1);
typedef struct {
    bench_resource * v0;
    uint64_t v1;
    bench_resource * v2;
} bench_make_records_ret_w0_row;
typedef struct { bench_make_records_ret_w0_row *data; size_t len; } bench_make_records_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void bench_make_records_ret_w0_rows_free(bench_make_records_ret_w0_rows rows);
/* Collection return: ret_w0 carries rows of 3 ABI fields; element-owned fields are released separately from row storage. */
/* Field return[].first: row.v0. */
/* Field return[].tag: row.v1. */
/* Field return[].second: row.v2. */
bench_make_records_ret_w0_rows bench_make_records(uintptr_t p0_w1);
uint64_t bench_resource_value(const bench_resource * p0_w1);
#ifdef __cplusplus
}
#endif
#endif
