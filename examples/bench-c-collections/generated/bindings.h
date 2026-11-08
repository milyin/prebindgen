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
/* Shared borrowed slice; never free or consume. Keep source storage alive; do not mutate it or obtain overlapping exclusive/mutable access until the view's last use. Empty views may have a non-null dangling data pointer; never dereference when len is zero. Callback argument views last only for that callback invocation. */
typedef struct { const int64_t *data; size_t len; } bench_i64_slice_view;
/* Binding target identity; use the matching Rust library. */
#define bench_TARGET "x86_64-unknown-linux-gnu"
#ifdef __cplusplus
static_assert(sizeof(void*) * 8 == 64, "prebindgen: header pointer width differs; regenerate for this target");
#else
_Static_assert(sizeof(void*) * 8 == 64, "prebindgen: header pointer width differs; regenerate for this target");
#endif
typedef int32_t bench_input_tag;
#define bench_input_tag_Low ((bench_input_tag)(-3))
#define bench_input_tag_Normal ((bench_input_tag)(0))
#define bench_input_tag_High ((bench_input_tag)(17))
/* Field paths below start at a source parameter or return and list their ABI values.
 * Optional groups below name their discriminator. Rules apply only in active
 * selector arms and present enclosing groups. Absent payload wires are ignored. */
typedef struct bench_borrowed_numbers bench_borrowed_numbers;
void bench_drop_borrowed_numbers(bench_borrowed_numbers *ptr);
typedef struct bench_resource bench_resource;
void bench_drop_resource(bench_resource *ptr);
typedef struct { const uint8_t *data; size_t len; } bench_buffer_view;
/* Exclusive borrowed bytes; never free or consume. Keep the owner alive, and do not access it or overlapping storage until the view's last use. */
typedef struct { uint8_t *data; size_t len; } bench_buffer_view_mut;
typedef struct {
#ifdef __cplusplus
    alignas(8)
#else
    _Alignas(8)
#endif
    unsigned char _storage[56];
} bench_native_packet;
/* Owning value: move, never duplicate ownership. Drop after the last use of its views; consumption replaces it with an empty value. */
void bench_drop_native_packet(bench_native_packet *value);
/* Rust signature: (& [i64])->(). The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    void (*run)(void *context, bench_i64_slice_view arg0);
    void *context;
    void (*drop)(void *context);
} bench_callback_3832b9cb035dc725;
typedef struct {
    bench_resource * v0;
} bench_callback_7fc0734290773f1a_callback_return_w1_row;
typedef struct { bench_callback_7fc0734290773f1a_callback_return_w1_row *data; size_t len; } bench_callback_7fc0734290773f1a_callback_return_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Rust signature: ()->Vec < c_collection_flat :: Resource >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    bench_callback_7fc0734290773f1a_callback_return_w1_rows (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const bench_callback_7fc0734290773f1a_callback_return_w1_rows *result);
} bench_callback_7fc0734290773f1a;
typedef struct {
    bench_resource * v0;
} bench_callback_bd272b159791e968_callback_return_w5_row;
typedef struct { bench_callback_bd272b159791e968_callback_return_w5_row *data; size_t len; } bench_callback_bd272b159791e968_callback_return_w5_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
typedef struct {
    bench_callback_bd272b159791e968_callback_return_w5_rows v0;
} bench_callback_bd272b159791e968_callback_return_w1_row;
typedef struct { bench_callback_bd272b159791e968_callback_return_w1_row *data; size_t len; } bench_callback_bd272b159791e968_callback_return_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Rust signature: ()->Vec < Vec < c_collection_flat :: Resource > >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    bench_callback_bd272b159791e968_callback_return_w1_rows (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const bench_callback_bd272b159791e968_callback_return_w1_rows *result);
} bench_callback_bd272b159791e968;
typedef struct {
    bench_resource * v0;
    uint64_t v1;
    bench_resource * v2;
} bench_callback_d4c2b6b7f94cdedf_result;
/* Rust signature: ()->c_collection_flat :: Record. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    bench_callback_d4c2b6b7f94cdedf_result (*run)(void *context);
    void *context;
    void (*drop)(void *context);
} bench_callback_d4c2b6b7f94cdedf;
typedef struct {
    bench_resource * v0;
    bench_native_packet v1;
    bench_resource * v2;
} bench_callback_f1755815cfab1b60_callback_return_w1_row;
typedef struct { bench_callback_f1755815cfab1b60_callback_return_w1_row *data; size_t len; } bench_callback_f1755815cfab1b60_callback_return_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Rust signature: ()->Vec < c_collection_flat :: NativeMixed >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    bench_callback_f1755815cfab1b60_callback_return_w1_rows (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const bench_callback_f1755815cfab1b60_callback_return_w1_rows *result);
} bench_callback_f1755815cfab1b60;
typedef struct {
    bench_native_packet v0;
} bench_callback_fc8e455d100e4602_callback_return_w1_row;
typedef struct { bench_callback_fc8e455d100e4602_callback_return_w1_row *data; size_t len; } bench_callback_fc8e455d100e4602_callback_return_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Rust signature: ()->Vec < c_collection_flat :: NativePacket >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    bench_callback_fc8e455d100e4602_callback_return_w1_rows (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const bench_callback_fc8e455d100e4602_callback_return_w1_rows *result);
} bench_callback_fc8e455d100e4602;
typedef bench_callback_3832b9cb035dc725 bench_borrowed_emit_callback;
typedef bench_callback_d4c2b6b7f94cdedf bench_callback_fixed_callback;
typedef bench_callback_d4c2b6b7f94cdedf_result bench_callback_fixed_callback_return;
typedef bench_callback_7fc0734290773f1a bench_callback_many_callback;
typedef bench_callback_7fc0734290773f1a_callback_return_w1_rows bench_callback_many_callback_return;
typedef bench_callback_7fc0734290773f1a_callback_return_w1_row bench_callback_many_callback_return_row;
typedef bench_callback_fc8e455d100e4602 bench_callback_native_callback;
typedef bench_callback_fc8e455d100e4602_callback_return_w1_rows bench_callback_native_callback_return;
typedef bench_callback_fc8e455d100e4602_callback_return_w1_row bench_callback_native_callback_return_row;
typedef bench_callback_f1755815cfab1b60 bench_callback_native_mixed_callback;
typedef bench_callback_f1755815cfab1b60_callback_return_w1_rows bench_callback_native_mixed_callback_return;
typedef bench_callback_f1755815cfab1b60_callback_return_w1_row bench_callback_native_mixed_callback_return_row;
typedef bench_callback_bd272b159791e968 bench_history_callback_callback;
typedef bench_callback_bd272b159791e968_callback_return_w1_rows bench_history_callback_callback_return;
typedef bench_callback_bd272b159791e968_callback_return_w1_row bench_history_callback_callback_return_row;
/* Non-Result calls return their first value directly; additional values use output pointers.
 * Binding failures require explicit .panic() and abort. Destructors return void. */
void bench_borrowed_emit(const bench_borrowed_numbers * p0_w1, bench_callback_3832b9cb035dc725 p1_callback, uintptr_t p2_w1);
bench_borrowed_numbers * bench_borrowed_numbers_new(uintptr_t p0_w1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
bench_i64_slice_view bench_borrowed_values(const bench_borrowed_numbers * p0_w1);
uint64_t bench_callback_fixed(bench_callback_d4c2b6b7f94cdedf p0_callback);
uint64_t bench_callback_many(bench_callback_7fc0734290773f1a p0_callback);
uint64_t bench_callback_native(bench_callback_fc8e455d100e4602 p0_callback);
uint64_t bench_callback_native_mixed(bench_callback_f1755815cfab1b60 p0_callback);
uint64_t bench_control(uintptr_t p0_w1);
uint64_t bench_fixed_borrow(const bench_resource * p0_w1, bench_resource * p1_w1);
uint64_t bench_fixed_move(bench_resource * p0_w1, bench_resource * p1_w1);
uint64_t bench_history_callback(bench_callback_bd272b159791e968 p0_callback);
typedef struct {
    bench_resource * v0;
} bench_history_flat_p0_w1_row;
typedef struct { bench_history_flat_p0_w1_row *data; size_t len; } bench_history_flat_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_history_flat(bench_history_flat_p0_w1_rows p0_w1);
typedef struct {
    bench_resource * v0;
} bench_history_nested_p0_w5_row;
typedef struct { bench_history_nested_p0_w5_row *data; size_t len; } bench_history_nested_p0_w5_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
typedef struct {
    bench_history_nested_p0_w5_rows v0;
} bench_history_nested_p0_w1_row;
typedef struct { bench_history_nested_p0_w1_row *data; size_t len; } bench_history_nested_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection values[]: p0_w5 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_history_nested(bench_history_nested_p0_w1_rows p0_w1);
typedef struct {
    const bench_resource * v0;
} bench_history_readonly_p0_w5_row;
typedef struct { bench_history_readonly_p0_w5_row *data; size_t len; } bench_history_readonly_p0_w5_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
typedef struct {
    bench_history_readonly_p0_w5_rows v0;
} bench_history_readonly_p0_w1_row;
typedef struct { bench_history_readonly_p0_w1_row *data; size_t len; } bench_history_readonly_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection values[]: p0_w5 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_history_readonly(bench_history_readonly_p0_w1_rows p0_w1);
typedef struct {
    const bench_resource * v0;
} bench_history_shared_p0_w5_row;
typedef struct { bench_history_shared_p0_w5_row *data; size_t len; } bench_history_shared_p0_w5_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
typedef struct {
    bench_history_shared_p0_w5_rows v0;
} bench_history_shared_p0_w1_row;
typedef struct { bench_history_shared_p0_w1_row *data; size_t len; } bench_history_shared_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection values[]: p0_w5 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_history_shared(bench_history_shared_p0_w1_rows p0_w1, bench_resource * p1_w1);
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
bench_native_packet bench_native_packet_new(uint64_t p0_w1);
bench_resource * bench_resource_new(uint64_t p0_w1);
uint64_t bench_resource_value(const bench_resource * p0_w1);
int64_t bench_shared_input_array(bench_i64_slice_view p0_w1);
uintptr_t bench_shared_input_inspect(bench_i64_slice_view p0_w1);
uintptr_t bench_shared_input_owned(bench_buffer p0_w1);
int64_t bench_shared_input_sum(bench_i64_slice_view p0_w1);
uintptr_t bench_shared_input_text(bench_buffer p0_w1);
typedef struct {
    bench_resource * v0;
    bench_buffer v1;
    bench_buffer v2;
} bench_take_buffers_p0_w1_row;
typedef struct { bench_take_buffers_p0_w1_row *data; size_t len; } bench_take_buffers_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 3 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_take_buffers(bench_take_buffers_p0_w1_rows p0_w1);
typedef struct {
    bench_resource * v0;
    bench_buffer v1;
} bench_take_bytes_p0_w1_row;
typedef struct { bench_take_bytes_p0_w1_row *data; size_t len; } bench_take_bytes_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_take_bytes(bench_take_bytes_p0_w1_rows p0_w1);
typedef struct {
    bench_resource * v0;
} bench_take_handles_p0_w1_row;
typedef struct { bench_take_handles_p0_w1_row *data; size_t len; } bench_take_handles_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_take_handles(bench_take_handles_p0_w1_rows p0_w1);
typedef struct {
    bench_native_packet * v0;
} bench_take_native_p0_w1_row;
typedef struct { bench_take_native_p0_w1_row *data; size_t len; } bench_take_native_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_take_native(bench_take_native_p0_w1_rows p0_w1);
typedef struct {
    bench_resource * v0;
    bench_native_packet * v1;
    bench_resource * v2;
} bench_take_native_mixed_p0_w1_row;
typedef struct { bench_take_native_mixed_p0_w1_row *data; size_t len; } bench_take_native_mixed_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 3 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_take_native_mixed(bench_take_native_mixed_p0_w1_rows p0_w1);
typedef struct {
    uint64_t v0;
} bench_take_numbers_p0_w1_row;
typedef struct { bench_take_numbers_p0_w1_row *data; size_t len; } bench_take_numbers_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_take_numbers(bench_take_numbers_p0_w1_rows p0_w1);
typedef struct {
    bench_resource * v0;
    uint64_t v1;
    bench_resource * v2;
} bench_take_records_p0_w1_row;
typedef struct { bench_take_records_p0_w1_row *data; size_t len; } bench_take_records_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 3 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_take_records(bench_take_records_p0_w1_rows p0_w1);
typedef struct {
    bench_resource * v0;
    bench_input_tag v1;
} bench_take_tagged_p0_w1_row;
typedef struct { bench_take_tagged_p0_w1_row *data; size_t len; } bench_take_tagged_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_take_tagged(bench_take_tagged_p0_w1_rows p0_w1);
typedef struct {
    bench_resource * v0;
    bench_input_tag v1;
    bench_buffer v2;
    bench_buffer v3;
} bench_take_tagged_buffers_p0_w1_row;
typedef struct { bench_take_tagged_buffers_p0_w1_row *data; size_t len; } bench_take_tagged_buffers_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 4 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_take_tagged_buffers(bench_take_tagged_buffers_p0_w1_rows p0_w1);
typedef struct {
    bench_resource * v0;
    bench_buffer v1;
} bench_take_text_p0_w1_row;
typedef struct { bench_take_text_p0_w1_row *data; size_t len; } bench_take_text_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_take_text(bench_take_text_p0_w1_rows p0_w1);
typedef struct {
    bench_resource * v0;
    uint64_t v1;
    uint64_t v2;
    uint64_t v3;
    uint64_t v4;
    uint64_t v5;
    uint64_t v6;
    uint64_t v7;
} bench_take_wide_p0_w1_row;
typedef struct { bench_take_wide_p0_w1_row *data; size_t len; } bench_take_wide_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 8 ABI fields; element-owned fields are released separately from row storage. */
uint64_t bench_take_wide(bench_take_wide_p0_w1_rows p0_w1);
#ifdef __cplusplus
}
#endif
#endif
