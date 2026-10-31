#ifndef owner_H
#define owner_H
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct { uint64_t hi; uint64_t lo; } owner_int128;
 typedef struct { const uint8_t *data; size_t len; } owner_buffer;
void owner_free_buffer(owner_buffer b);
void owner_free_error(char *error);
/* Binding target identity; use the matching Rust library. */
#define owner_TARGET "x86_64-unknown-linux-gnu"
#ifdef __cplusplus
static_assert(sizeof(void*) * 8 == 64, "prebindgen: header pointer width differs; regenerate for this target");
#else
_Static_assert(sizeof(void*) * 8 == 64, "prebindgen: header pointer width differs; regenerate for this target");
#endif
/* Field paths below start at a source parameter or return and list their ABI values.
 * Optional groups below name their discriminator. Rules apply only in active
 * selector arms and present enclosing groups. Absent payload wires are ignored. */
typedef struct owner_child owner_child;
void owner_drop_child(owner_child *ptr);
typedef struct owner_boxed_numbers owner_boxed_numbers;
void owner_drop_boxed_numbers(owner_boxed_numbers *ptr);
typedef struct { const uint8_t *data; size_t len; } owner_buffer_view;
typedef struct {
#ifdef __cplusplus
    alignas(8)
#else
    _Alignas(8)
#endif
    unsigned char _storage[64];
} owner_packet;
/* Owning value: move, never duplicate ownership. Drop after the last use of its views; consumption replaces it with an empty value. */
void owner_drop_packet(owner_packet *value);
typedef struct {
#ifdef __cplusplus
    alignas(8)
#else
    _Alignas(8)
#endif
    unsigned char _storage[24];
} owner_text;
/* Owning value: move, never duplicate ownership. Drop after the last use of its views; consumption replaces it with an empty value. */
void owner_drop_text(owner_text *value);
size_t owner_text_len(const owner_text *value);
/* Borrowed view; never free it. The owner retains the allocation. */
owner_buffer_view owner_text_view(const owner_text *value);
typedef struct {
#ifdef __cplusplus
    alignas(8)
#else
    _Alignas(8)
#endif
    unsigned char _storage[24];
} owner_numbers;
/* Owning value: move, never duplicate ownership. Drop after the last use of its views; consumption replaces it with an empty value. */
void owner_drop_numbers(owner_numbers *value);
size_t owner_numbers_len(const owner_numbers *value);
/* Borrowed element; null for an out-of-range index. */
const int64_t *owner_numbers_get(const owner_numbers *value,size_t index);
typedef struct {
#ifdef __cplusplus
    alignas(8)
#else
    _Alignas(8)
#endif
    unsigned char _storage[24];
} owner_bytes;
/* Owning value: move, never duplicate ownership. Drop after the last use of its views; consumption replaces it with an empty value. */
void owner_drop_bytes(owner_bytes *value);
size_t owner_bytes_len(const owner_bytes *value);
/* Borrowed view; never free it. The owner retains the allocation. */
owner_buffer_view owner_bytes_view(const owner_bytes *value);
typedef struct {
    owner_packet v0;
    uint32_t v1;
} owner_callback_1782e472ad837f0b_callback_return_w1_row;
typedef struct { owner_callback_1782e472ad837f0b_callback_return_w1_row *data; size_t len; } owner_callback_1782e472ad837f0b_callback_return_w1_rows;
/* Rust signature: ()->Vec < perftest_flat :: InlineReturn >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    owner_callback_1782e472ad837f0b_callback_return_w1_rows (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const owner_callback_1782e472ad837f0b_callback_return_w1_rows *result);
} owner_callback_1782e472ad837f0b;
/* Rust signature: (perftest_flat :: InlinePacket)->perftest_flat :: InlinePacket. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    owner_packet (*run)(void *context, owner_packet arg0);
    void *context;
    void (*drop)(void *context);
} owner_callback_7a0d2e4eb3c7fbb1;
typedef struct {
    owner_packet v0;
    owner_child * v1;
} owner_callback_d1e1f55a20f3880c_callback_return_w1_row;
typedef struct { owner_callback_d1e1f55a20f3880c_callback_return_w1_row *data; size_t len; } owner_callback_d1e1f55a20f3880c_callback_return_w1_rows;
/* Rust signature: ()->Vec < perftest_flat :: InlineMixed >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    owner_callback_d1e1f55a20f3880c_callback_return_w1_rows (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const owner_callback_d1e1f55a20f3880c_callback_return_w1_rows *result);
} owner_callback_d1e1f55a20f3880c;
/* Rust signature: (Vec < u8 >)->Vec < u8 >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    owner_bytes (*run)(void *context, owner_bytes arg0);
    void *context;
    void (*drop)(void *context);
} owner_callback_df547e71faee3c03;
typedef struct {
    owner_packet v0;
    uint32_t v1;
} owner_callback_e857880e5782d2cb_result;
/* Rust signature: ()->perftest_flat :: InlineReturn. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    owner_callback_e857880e5782d2cb_result (*run)(void *context);
    void *context;
    void (*drop)(void *context);
} owner_callback_e857880e5782d2cb;
typedef owner_callback_df547e71faee3c03 owner_inline_bytes_callback_callback;
typedef owner_bytes owner_inline_bytes_callback_callback_return;
typedef owner_callback_d1e1f55a20f3880c owner_inline_mixed_rows_sum_callback;
typedef owner_callback_d1e1f55a20f3880c_callback_return_w1_rows owner_inline_mixed_rows_sum_callback_return;
typedef owner_callback_d1e1f55a20f3880c_callback_return_w1_row owner_inline_mixed_rows_sum_callback_return_row;
typedef owner_callback_7a0d2e4eb3c7fbb1 owner_inline_packet_callback_callback;
typedef owner_packet owner_inline_packet_callback_callback_return;
typedef owner_callback_e857880e5782d2cb owner_inline_return_catch_callback;
typedef owner_callback_e857880e5782d2cb_result owner_inline_return_catch_callback_return;
typedef owner_callback_1782e472ad837f0b owner_inline_rows_catch_callback;
typedef owner_callback_1782e472ad837f0b_callback_return_w1_rows owner_inline_rows_catch_callback_return;
typedef owner_callback_1782e472ad837f0b_callback_return_w1_row owner_inline_rows_catch_callback_return_row;
/* Non-Result calls return their first value directly; additional values use output pointers.
 * Binding failures require explicit .panic() and abort. Destructors return void. */
uintptr_t owner_inline_boxed_numbers_len(const owner_boxed_numbers * p0_w1);
owner_boxed_numbers * owner_inline_boxed_numbers_new(void);
int32_t owner_inline_boxed_numbers_take(owner_boxed_numbers * p0_w1);
owner_bytes owner_inline_bytes(void);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_bytes_alias(owner_bytes * p0_w1, const owner_bytes * p1_w1, uintptr_t *out0, size_t *ownership, char * *error);
uintptr_t owner_inline_bytes_callback(owner_callback_df547e71faee3c03 p0_callback);
uintptr_t owner_inline_bytes_capacity(const owner_bytes * p0_w1);
owner_bytes owner_inline_bytes_mutable(uintptr_t *out1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
owner_bytes owner_inline_bytes_new(owner_buffer_view *out1, uintptr_t *out2);
uintptr_t owner_inline_bytes_take(owner_bytes * p0_w1);
uint64_t owner_inline_child_id(const owner_child * p0_w1);
uint64_t owner_inline_mixed_rows_sum(owner_callback_d1e1f55a20f3880c p0_callback);
owner_numbers owner_inline_numbers(void);
uintptr_t owner_inline_numbers_capacity(const owner_numbers * p0_w1);
owner_numbers owner_inline_numbers_new(void);
int64_t owner_inline_numbers_take(owner_numbers * p0_w1);
uintptr_t owner_inline_packet_callback(owner_callback_7a0d2e4eb3c7fbb1 p0_callback);
uintptr_t owner_inline_packet_drops(void);
owner_packet owner_inline_packet_new(void);
/* Option option0: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t owner_inline_packet_optional(uint8_t p0_w1, owner_packet *out1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
const owner_child * owner_inline_packet_ref(const owner_packet * p0_w1);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_packet_result(owner_packet *out0, uintptr_t *out1, char * *error);
uintptr_t owner_inline_packet_take(owner_packet * p0_w1);
typedef struct {
    owner_packet v0;
} owner_inline_packets_rows_ret_w3_row;
typedef struct { owner_inline_packets_rows_ret_w3_row *data; size_t len; } owner_inline_packets_rows_ret_w3_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void owner_inline_packets_rows_ret_w3_rows_free(owner_inline_packets_rows_ret_w3_rows rows);
/* Collection return: ret_w3 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_packets_rows(owner_inline_packets_rows_ret_w3_rows *out0, uintptr_t *out1, char * *error);
typedef struct {
    owner_packet * v0;
} owner_inline_packets_take_p0_w1_row;
typedef struct { owner_inline_packets_take_p0_w1_row *data; size_t len; } owner_inline_packets_take_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
uintptr_t owner_inline_packets_take(owner_inline_packets_take_p0_w1_rows p0_w1);
uint8_t owner_inline_return_catch(owner_callback_e857880e5782d2cb p0_callback);
uint8_t owner_inline_rows_catch(owner_callback_1782e472ad837f0b p0_callback);
owner_text owner_inline_text(void);
uintptr_t owner_inline_text_capacity(const owner_text * p0_w1);
owner_text owner_inline_text_new(void);
uintptr_t owner_inline_text_take(owner_text * p0_w1);
#ifdef __cplusplus
}
#endif
#endif
