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
/* Shared borrowed slice; never free or consume. Keep source storage alive; do not mutate it or obtain overlapping exclusive/mutable access until the view's last use. Empty views may have a non-null dangling data pointer; never dereference when len is zero. Callback argument views last only for that callback invocation. */
typedef struct { const int64_t *data; size_t len; } owner_i64_slice_view;
/* Binding target identity; use the matching Rust library. */
#define owner_TARGET "x86_64-unknown-linux-gnu"
#ifdef __cplusplus
static_assert(sizeof(void*) * 8 == 64, "prebindgen: header pointer width differs; regenerate for this target");
#else
_Static_assert(sizeof(void*) * 8 == 64, "prebindgen: header pointer width differs; regenerate for this target");
#endif
typedef int32_t owner_input_tag;
#define owner_input_tag_Low ((owner_input_tag)(-3))
#define owner_input_tag_Normal ((owner_input_tag)(0))
#define owner_input_tag_High ((owner_input_tag)(17))
/* Field paths below start at a source parameter or return and list their ABI values.
 * Optional groups below name their discriminator. Rules apply only in active
 * selector arms and present enclosing groups. Absent payload wires are ignored. */
typedef struct owner_child owner_child;
void owner_drop_child(owner_child *ptr);
typedef struct owner_boxed_numbers owner_boxed_numbers;
void owner_drop_boxed_numbers(owner_boxed_numbers *ptr);
typedef struct owner_resource owner_resource;
void owner_drop_resource(owner_resource *ptr);
typedef struct { const uint8_t *data; size_t len; } owner_buffer_view;
/* Exclusive borrowed bytes; never free or consume. Keep the owner alive, and do not access it or overlapping storage until the view's last use. */
typedef struct { uint8_t *data; size_t len; } owner_buffer_view_mut;
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
/* Borrowed view; never free it. The owner retains the allocation. A later mutable view invalidates earlier views. */
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
/* Borrowed view; never free. Keep the owner alive; do not mutate it or obtain overlapping exclusive/mutable access until the view's last use. */
owner_i64_slice_view owner_numbers_view(const owner_numbers *value);
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
/* Borrowed view; never free it. The owner retains the allocation. A later mutable view invalidates earlier views. */
owner_buffer_view owner_bytes_view(const owner_bytes *value);
/* Exclusive borrowed view; never free it. Keep the owner alive and preserve exclusive access until the view's last use. */
owner_buffer_view_mut owner_bytes_view_mut(owner_bytes *value);
/* Rust signature: (perftest_flat :: InlinePacket)->perftest_flat :: InlinePacket. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    owner_packet (*run)(void *context, owner_packet arg0);
    void *context;
    void (*drop)(void *context);
} owner_callback_0c47d5ea09f8970c;
typedef struct {
    owner_resource * v0;
    owner_packet v1;
    owner_resource * v2;
    uint32_t v3;
    uintptr_t v4;
    uintptr_t v5;
} owner_callback_37300b78c94d93fa_callback_return_w1_row;
typedef struct { owner_callback_37300b78c94d93fa_callback_return_w1_row *data; size_t len; } owner_callback_37300b78c94d93fa_callback_return_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Rust signature: ()->Vec < perftest_flat :: InlineCallbackRow >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    owner_callback_37300b78c94d93fa_callback_return_w1_rows (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const owner_callback_37300b78c94d93fa_callback_return_w1_rows *result);
} owner_callback_37300b78c94d93fa;
typedef struct {
    owner_resource * v0;
    owner_packet v1;
    owner_resource * v2;
    uint32_t v3;
    uintptr_t v4;
    uintptr_t v5;
} owner_callback_70db60a6f9dbbb06_callback_return_w3_row;
typedef struct { owner_callback_70db60a6f9dbbb06_callback_return_w3_row *data; size_t len; } owner_callback_70db60a6f9dbbb06_callback_return_w3_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
typedef struct {
    int32_t v0;
    owner_callback_70db60a6f9dbbb06_callback_return_w3_rows v1;
} owner_callback_70db60a6f9dbbb06_result;
/* Rust signature: ()->Option < Vec < Option < perftest_flat :: InlineCallbackRow > > >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    owner_callback_70db60a6f9dbbb06_result (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const owner_callback_70db60a6f9dbbb06_result *result);
} owner_callback_70db60a6f9dbbb06;
typedef struct {
    owner_packet v0;
    owner_child * v1;
} owner_callback_805117dbed50edfb_callback_return_w1_row;
typedef struct { owner_callback_805117dbed50edfb_callback_return_w1_row *data; size_t len; } owner_callback_805117dbed50edfb_callback_return_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Rust signature: ()->Vec < perftest_flat :: InlineMixed >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    owner_callback_805117dbed50edfb_callback_return_w1_rows (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const owner_callback_805117dbed50edfb_callback_return_w1_rows *result);
} owner_callback_805117dbed50edfb;
typedef struct {
    owner_packet v0;
    uint32_t v1;
} owner_callback_c381621a1cf6edc5_result;
/* Rust signature: ()->perftest_flat :: InlineReturn. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    owner_callback_c381621a1cf6edc5_result (*run)(void *context);
    void *context;
    void (*drop)(void *context);
} owner_callback_c381621a1cf6edc5;
/* Rust signature: (Vec < u8 >)->Vec < u8 >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    owner_bytes (*run)(void *context, owner_bytes arg0);
    void *context;
    void (*drop)(void *context);
} owner_callback_f5757ec9aa1c45d2;
typedef struct {
    owner_packet v0;
    uint32_t v1;
} owner_callback_f5d26722508e30c3_callback_return_w1_row;
typedef struct { owner_callback_f5d26722508e30c3_callback_return_w1_row *data; size_t len; } owner_callback_f5d26722508e30c3_callback_return_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Rust signature: ()->Vec < perftest_flat :: InlineReturn >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    owner_callback_f5d26722508e30c3_callback_return_w1_rows (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const owner_callback_f5d26722508e30c3_callback_return_w1_rows *result);
} owner_callback_f5d26722508e30c3;
typedef owner_callback_f5757ec9aa1c45d2 owner_inline_bytes_callback_callback;
typedef owner_bytes owner_inline_bytes_callback_callback_return;
typedef owner_callback_70db60a6f9dbbb06 owner_inline_callback_optional_callback;
typedef owner_callback_70db60a6f9dbbb06_result owner_inline_callback_optional_callback_return;
typedef owner_callback_37300b78c94d93fa owner_inline_callback_rows_callback;
typedef owner_callback_37300b78c94d93fa_callback_return_w1_rows owner_inline_callback_rows_callback_return;
typedef owner_callback_37300b78c94d93fa_callback_return_w1_row owner_inline_callback_rows_callback_return_row;
typedef owner_callback_37300b78c94d93fa owner_inline_callback_rows_fast_callback;
typedef owner_callback_37300b78c94d93fa_callback_return_w1_rows owner_inline_callback_rows_fast_callback_return;
typedef owner_callback_37300b78c94d93fa_callback_return_w1_row owner_inline_callback_rows_fast_callback_return_row;
typedef owner_callback_805117dbed50edfb owner_inline_mixed_rows_sum_callback;
typedef owner_callback_805117dbed50edfb_callback_return_w1_rows owner_inline_mixed_rows_sum_callback_return;
typedef owner_callback_805117dbed50edfb_callback_return_w1_row owner_inline_mixed_rows_sum_callback_return_row;
typedef owner_callback_0c47d5ea09f8970c owner_inline_packet_callback_callback;
typedef owner_packet owner_inline_packet_callback_callback_return;
typedef owner_callback_c381621a1cf6edc5 owner_inline_return_catch_callback;
typedef owner_callback_c381621a1cf6edc5_result owner_inline_return_catch_callback_return;
typedef owner_callback_f5d26722508e30c3 owner_inline_rows_catch_callback;
typedef owner_callback_f5d26722508e30c3_callback_return_w1_rows owner_inline_rows_catch_callback_return;
typedef owner_callback_f5d26722508e30c3_callback_return_w1_row owner_inline_rows_catch_callback_return_row;
/* Non-Result calls return their first value directly; additional values use output pointers.
 * Binding failures require explicit .panic() and abort. Destructors return void. */
owner_resource * owner_collection_resource_new(int64_t p0_w1);
uintptr_t owner_enum_inputs_constructors(void);
typedef struct {
    owner_packet * v0;
    owner_input_tag v1;
} owner_enum_inputs_inline_p0_w1_row;
typedef struct { owner_enum_inputs_inline_p0_w1_row *data; size_t len; } owner_enum_inputs_inline_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection rows: p0_w1 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_enum_inputs_inline(owner_enum_inputs_inline_p0_w1_rows p0_w1, int64_t *out0, size_t *ownership, char * *error);
void owner_enum_inputs_reset(void);
uintptr_t owner_inline_boxed_numbers_len(const owner_boxed_numbers * p0_w1);
owner_boxed_numbers * owner_inline_boxed_numbers_new(void);
int32_t owner_inline_boxed_numbers_take(owner_boxed_numbers * p0_w1);
owner_bytes owner_inline_bytes(void);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_bytes_alias(owner_bytes * p0_w1, const owner_bytes * p1_w1, uintptr_t *out0, size_t *ownership, char * *error);
uintptr_t owner_inline_bytes_callback(owner_callback_f5757ec9aa1c45d2 p0_callback);
uintptr_t owner_inline_bytes_capacity(const owner_bytes * p0_w1);
owner_bytes owner_inline_bytes_mutable(uintptr_t *out1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
owner_bytes owner_inline_bytes_new(owner_buffer_view *out1, uintptr_t *out2);
uintptr_t owner_inline_bytes_take(owner_bytes * p0_w1);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_callback_optional(owner_callback_70db60a6f9dbbb06 p0_callback, int64_t *out0, size_t *ownership, char * *error);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_callback_rows(owner_callback_37300b78c94d93fa p0_callback, int64_t *out0, size_t *ownership, char * *error);
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_callback_rows_fast(owner_callback_37300b78c94d93fa p0_callback, int64_t *out0, size_t *ownership, char * *error);
uint64_t owner_inline_child_id(const owner_child * p0_w1);
typedef struct {
    const owner_packet * v0;
} owner_inline_inputs_borrow_p1_w1_row;
typedef struct { owner_inline_inputs_borrow_p1_w1_row *data; size_t len; } owner_inline_inputs_borrow_p1_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection rows: p1_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_inputs_borrow(owner_packet * p0_w1, owner_inline_inputs_borrow_p1_w1_rows p1_w1, uintptr_t *out0, char * *error);
typedef struct {
    owner_bytes * v0;
} owner_inline_inputs_bytes_p0_w1_row;
typedef struct { owner_inline_inputs_bytes_p0_w1_row *data; size_t len; } owner_inline_inputs_bytes_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection rows: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_inputs_bytes(owner_inline_inputs_bytes_p0_w1_rows p0_w1, uintptr_t *out0, size_t *ownership, char * *error);
uintptr_t owner_inline_inputs_calls(void);
uintptr_t owner_inline_inputs_constructor_calls(void);
uintptr_t owner_inline_inputs_empty_calls(void);
typedef struct {
    owner_resource * v0;
    owner_packet * v1;
    owner_resource * v2;
} owner_inline_inputs_mixed_p0_w1_row;
typedef struct { owner_inline_inputs_mixed_p0_w1_row *data; size_t len; } owner_inline_inputs_mixed_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection rows: p0_w1 carries rows of 3 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_inputs_mixed(owner_inline_inputs_mixed_p0_w1_rows p0_w1, int64_t *out0, size_t *ownership, char * *error);
typedef struct {
    owner_resource * v0;
    owner_packet * v1;
    owner_resource * v2;
} owner_inline_inputs_mixed_fast_p0_w1_row;
typedef struct { owner_inline_inputs_mixed_fast_p0_w1_row *data; size_t len; } owner_inline_inputs_mixed_fast_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Collection rows: p0_w1 carries rows of 3 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_inputs_mixed_fast(owner_inline_inputs_mixed_fast_p0_w1_rows p0_w1, int64_t *out0, size_t *ownership, char * *error);
typedef struct {
    owner_resource * v0;
    owner_packet * v1;
    owner_resource * v2;
} owner_inline_inputs_optional_p0_w3_row;
typedef struct { owner_inline_inputs_optional_p0_w3_row *data; size_t len; } owner_inline_inputs_optional_p0_w3_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Collection rows: p0_w3 carries rows of 3 ABI fields; element-owned fields are released separately from row storage. */
/* Option option1: row.v0 == NULL means None; non-null means Some; only when option0 is present. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_inputs_optional(int32_t p0_w1, owner_inline_inputs_optional_p0_w3_rows p0_w3, int64_t *out0, size_t *ownership, char * *error);
typedef struct {
    owner_packet * v0;
} owner_inline_inputs_packets_p0_w1_row;
typedef struct { owner_inline_inputs_packets_p0_w1_row *data; size_t len; } owner_inline_inputs_packets_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection rows: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_inputs_packets(owner_inline_inputs_packets_p0_w1_rows p0_w1, uintptr_t *out0, size_t *ownership, char * *error);
typedef struct {
    owner_packet * v0;
} owner_inline_inputs_packets_fast_p0_w1_row;
typedef struct { owner_inline_inputs_packets_fast_p0_w1_row *data; size_t len; } owner_inline_inputs_packets_fast_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Collection rows: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_inputs_packets_fast(owner_inline_inputs_packets_fast_p0_w1_rows p0_w1, uintptr_t *out0, size_t *ownership, char * *error);
typedef struct {
    owner_packet * v0;
    owner_packet * v1;
} owner_inline_inputs_pair_p0_w1_row;
typedef struct { owner_inline_inputs_pair_p0_w1_row *data; size_t len; } owner_inline_inputs_pair_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection rows: p0_w1 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_inputs_pair(owner_inline_inputs_pair_p0_w1_rows p0_w1, int64_t *out0, size_t *ownership, char * *error);
typedef struct {
    owner_packet * v0;
    owner_packet * v1;
} owner_inline_inputs_pair_fast_p0_w1_row;
typedef struct { owner_inline_inputs_pair_fast_p0_w1_row *data; size_t len; } owner_inline_inputs_pair_fast_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Collection rows: p0_w1 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_inputs_pair_fast(owner_inline_inputs_pair_fast_p0_w1_rows p0_w1, int64_t *out0, size_t *ownership, char * *error);
void owner_inline_inputs_reset(uintptr_t p0_w1, uintptr_t p1_w1);
typedef struct {
    owner_text * v0;
} owner_inline_inputs_texts_p0_w1_row;
typedef struct { owner_inline_inputs_texts_p0_w1_row *data; size_t len; } owner_inline_inputs_texts_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection rows: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_inputs_texts(owner_inline_inputs_texts_p0_w1_rows p0_w1, uintptr_t *out0, size_t *ownership, char * *error);
uint64_t owner_inline_mixed_rows_sum(owner_callback_805117dbed50edfb p0_callback);
owner_numbers owner_inline_numbers(void);
uintptr_t owner_inline_numbers_capacity(const owner_numbers * p0_w1);
owner_numbers owner_inline_numbers_new(void);
int64_t owner_inline_numbers_take(owner_numbers * p0_w1);
uintptr_t owner_inline_packet_bytes_address(const owner_packet * p0_w1);
uintptr_t owner_inline_packet_callback(owner_callback_0c47d5ea09f8970c p0_callback);
uintptr_t owner_inline_packet_drops(void);
uintptr_t owner_inline_packet_len(const owner_packet * p0_w1);
owner_packet owner_inline_packet_new(void);
/* Option option0: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t owner_inline_packet_optional(uint8_t p0_w1, owner_packet *out1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
const owner_child * owner_inline_packet_ref(const owner_packet * p0_w1);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool owner_inline_packet_result(owner_packet *out0, uintptr_t *out1, char * *error);
uintptr_t owner_inline_packet_take(owner_packet * p0_w1);
uintptr_t owner_inline_packet_text_address(const owner_packet * p0_w1);
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
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. This excludes pointed-to inline owner storage: each consumed owner is replaced with an empty value in its original slot. Discard that empty slot or call its typed drop; do not free consumed opaque pointer handles again. */
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
uintptr_t owner_inline_packets_take(owner_inline_packets_take_p0_w1_rows p0_w1);
uint8_t owner_inline_return_catch(owner_callback_c381621a1cf6edc5 p0_callback);
uint8_t owner_inline_rows_catch(owner_callback_f5d26722508e30c3 p0_callback);
owner_text owner_inline_text(void);
uintptr_t owner_inline_text_capacity(const owner_text * p0_w1);
owner_text owner_inline_text_new(void);
uintptr_t owner_inline_text_take(owner_text * p0_w1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
owner_bytes owner_mutable_bytes_vector_expanded(uintptr_t p0_w1, owner_buffer_view_mut *out1);
uintptr_t owner_owned_resource_drops(void);
int64_t owner_owned_resource_id(const owner_resource * p0_w1);
#ifdef __cplusplus
}
#endif
#endif
