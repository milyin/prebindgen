#ifndef fast_H
#define fast_H
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct { uint64_t hi; uint64_t lo; } fast_int128;
 typedef struct { const uint8_t *data; size_t len; } fast_buffer;
void fast_free_buffer(fast_buffer b);
void fast_free_error(char *error);
/* Binding target identity; use the matching Rust library. */
#define fast_TARGET "x86_64-unknown-linux-gnu"
#ifdef __cplusplus
static_assert(sizeof(void*) * 8 == 64, "prebindgen: header pointer width differs; regenerate for this target");
#else
_Static_assert(sizeof(void*) * 8 == 64, "prebindgen: header pointer width differs; regenerate for this target");
#endif
typedef int32_t fast_input_tag;
#define fast_input_tag_Low ((fast_input_tag)(-3))
#define fast_input_tag_Normal ((fast_input_tag)(0))
#define fast_input_tag_High ((fast_input_tag)(17))
typedef int32_t fast_code;
#define fast_code_Bottom ((fast_code)(INT32_MIN))
#define fast_code_MinusTwo ((fast_code)(-2))
#define fast_code_MinusOne ((fast_code)(-1))
#define fast_code_Zero ((fast_code)(0))
#define fast_code_Seven ((fast_code)(7))
#define fast_code_Eight ((fast_code)(8))
#define fast_code_Top ((fast_code)(2147483647))
/* Field paths below start at a source parameter or return and list their ABI values.
 * Optional groups below name their discriminator. Rules apply only in active
 * selector arms and present enclosing groups. Absent payload wires are ignored. */
typedef struct fast_owned_resource fast_owned_resource;
void fast_drop_owned_resource(fast_owned_resource *ptr);
typedef struct fast_alias_handler fast_alias_handler;
void fast_drop_alias_handler(fast_alias_handler *ptr);
typedef struct { const uint8_t *data; size_t len; } fast_buffer_view;
/* Exclusive borrowed bytes; never free or consume. Keep the owner alive, and do not access it or overlapping storage until the view's last use. */
typedef struct { uint8_t *data; size_t len; } fast_buffer_view_mut;
typedef struct {
#ifdef __cplusplus
    alignas(8)
#else
    _Alignas(8)
#endif
    unsigned char _storage[24];
} fast_owned_string;
/* Owning value: move, never duplicate ownership. Drop after the last use of its views; consumption replaces it with an empty value. */
void fast_drop_owned_string(fast_owned_string *value);
size_t fast_owned_string_len(const fast_owned_string *value);
/* Borrowed view; never free it. The owner retains the allocation. A later mutable view invalidates earlier views. */
fast_buffer_view fast_owned_string_view(const fast_owned_string *value);
typedef struct {
    fast_owned_resource * v0;
    uint32_t v1;
    fast_buffer v2;
    fast_owned_resource * v3;
} fast_callback_0605b38a2f053a56_callback_return_w3_row;
typedef struct { fast_callback_0605b38a2f053a56_callback_return_w3_row *data; size_t len; } fast_callback_0605b38a2f053a56_callback_return_w3_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
typedef struct {
    int32_t v0;
    fast_callback_0605b38a2f053a56_callback_return_w3_rows v1;
} fast_callback_0605b38a2f053a56_result;
/* Rust signature: ()->Option < Vec < Option < perftest_flat :: FixedCleanupRecord > > >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    fast_callback_0605b38a2f053a56_result (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const fast_callback_0605b38a2f053a56_result *result);
} fast_callback_0605b38a2f053a56;
typedef struct {
    fast_owned_resource * v0;
    fast_owned_resource * v1;
} fast_callback_396f37c4ec343c74_result;
/* Rust signature: ()->perftest_flat :: AliasPair. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    fast_callback_396f37c4ec343c74_result (*run)(void *context);
    void *context;
    void (*drop)(void *context);
} fast_callback_396f37c4ec343c74;
typedef struct {
    fast_owned_resource * v0;
    uint32_t v1;
    fast_buffer v2;
    fast_owned_resource * v3;
} fast_callback_5b6f807c27356f4b_callback_return_w8_row;
typedef struct { fast_callback_5b6f807c27356f4b_callback_return_w8_row *data; size_t len; } fast_callback_5b6f807c27356f4b_callback_return_w8_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
typedef struct {
    fast_callback_5b6f807c27356f4b_callback_return_w8_rows v0;
} fast_callback_5b6f807c27356f4b_callback_return_w4_row;
typedef struct { fast_callback_5b6f807c27356f4b_callback_return_w4_row *data; size_t len; } fast_callback_5b6f807c27356f4b_callback_return_w4_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
typedef struct {
    fast_owned_resource * v0;
    fast_callback_5b6f807c27356f4b_callback_return_w4_rows v1;
    fast_owned_resource * v2;
} fast_callback_5b6f807c27356f4b_result;
/* Rust signature: ()->perftest_flat :: CleanupGroup. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    fast_callback_5b6f807c27356f4b_result (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const fast_callback_5b6f807c27356f4b_result *result);
} fast_callback_5b6f807c27356f4b;
typedef struct {
    fast_owned_resource * v0;
} fast_callback_8735da2fee137b57_callback_return_w1_row;
typedef struct { fast_callback_8735da2fee137b57_callback_return_w1_row *data; size_t len; } fast_callback_8735da2fee137b57_callback_return_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
/* Rust signature: ()->Vec < perftest_flat :: OwnedResource >. The callback context transfers after its run pointer validates and counts as one consumed input; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Owning values passed to run are callback-owned; borrowed views are valid only during run. Returned buffers, rows and descriptors must remain live and unchanged until Rust finishes decoding them, including source constructor invocations; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles or inline owners (including owners left in returned rows). */
typedef struct {
    fast_callback_8735da2fee137b57_callback_return_w1_rows (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const fast_callback_8735da2fee137b57_callback_return_w1_rows *result);
} fast_callback_8735da2fee137b57;
typedef fast_callback_8735da2fee137b57 fast_alias_callback_rows_callback;
typedef fast_callback_8735da2fee137b57_callback_return_w1_rows fast_alias_callback_rows_callback_return;
typedef fast_callback_8735da2fee137b57_callback_return_w1_row fast_alias_callback_rows_callback_return_row;
typedef fast_callback_396f37c4ec343c74 fast_alias_handler_new_callback;
typedef fast_callback_396f37c4ec343c74_result fast_alias_handler_new_callback_return;
typedef fast_callback_5b6f807c27356f4b fast_cleanup_group_call_callback;
typedef fast_callback_5b6f807c27356f4b_result fast_cleanup_group_call_callback_return;
typedef fast_callback_0605b38a2f053a56 fast_cleanup_optional_call_callback;
typedef fast_callback_0605b38a2f053a56_result fast_cleanup_optional_call_callback_return;
/* Non-Result calls return their first value directly; additional values use output pointers.
 * Binding failures require explicit .panic() and abort. Destructors return void. */
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool fast_alias_callback_rows(fast_callback_8735da2fee137b57 p0_callback, int64_t *out0, size_t *ownership, char * *error);
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
int64_t fast_alias_fixed(fast_owned_resource * p0_w1, const fast_owned_resource * p1_w1);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool fast_alias_handler_call(const fast_alias_handler * p0_w1, int64_t *out0, char * *error);
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
fast_alias_handler * fast_alias_handler_new(fast_callback_396f37c4ec343c74 p0_callback);
typedef struct {
    fast_owned_resource * v0;
} fast_alias_nested_sum_p0_w5_row;
typedef struct { fast_alias_nested_sum_p0_w5_row *data; size_t len; } fast_alias_nested_sum_p0_w5_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
typedef struct {
    fast_alias_nested_sum_p0_w5_rows v0;
} fast_alias_nested_sum_p0_w1_row;
typedef struct { fast_alias_nested_sum_p0_w1_row *data; size_t len; } fast_alias_nested_sum_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection values[]: p0_w5 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
int64_t fast_alias_nested_sum(fast_alias_nested_sum_p0_w1_rows p0_w1);
typedef struct {
    fast_owned_resource * v0;
    fast_code v1;
    int32_t v2;
    fast_buffer v3;
    int32_t v4;
    fast_buffer v5;
} fast_buffer_rows_enum_p0_w1_row;
typedef struct { fast_buffer_rows_enum_p0_w1_row *data; size_t len; } fast_buffer_rows_enum_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Collection rows: p0_w1 carries rows of 6 ABI fields; element-owned fields are released separately from row storage. */
/* Option option0: row.v2: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: row.v4: 0 = None, 1 = Some; other active flags are errors. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool fast_buffer_rows_enum(fast_buffer_rows_enum_p0_w1_rows p0_w1, int64_t *out0, size_t *ownership, char * *error);
typedef struct {
    fast_owned_resource * v0;
    fast_buffer v1;
    fast_buffer v2;
} fast_buffer_rows_optional_p0_w3_row;
typedef struct { fast_buffer_rows_optional_p0_w3_row *data; size_t len; } fast_buffer_rows_optional_p0_w3_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Collection rows: p0_w3 carries rows of 3 ABI fields; element-owned fields are released separately from row storage. */
/* Option option1: row.v0 == NULL means None; non-null means Some; only when option0 is present. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool fast_buffer_rows_optional(int32_t p0_w1, fast_buffer_rows_optional_p0_w3_rows p0_w3, int64_t *out0, size_t *ownership, char * *error);
typedef struct {
    fast_owned_resource * v0;
    fast_buffer v1;
    fast_buffer v2;
} fast_buffer_rows_sum_p0_w1_row;
typedef struct { fast_buffer_rows_sum_p0_w1_row *data; size_t len; } fast_buffer_rows_sum_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Collection rows: p0_w1 carries rows of 3 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool fast_buffer_rows_sum(fast_buffer_rows_sum_p0_w1_rows p0_w1, int64_t *out0, size_t *ownership, char * *error);
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool fast_cleanup_group_call(fast_callback_5b6f807c27356f4b p0_callback, int64_t *out0, size_t *ownership, char * *error);
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool fast_cleanup_optional_call(fast_callback_0605b38a2f053a56 p0_callback, int64_t *out0, size_t *ownership, char * *error);
typedef struct {
    fast_owned_resource * v0;
} fast_collection_alias_p0_w1_row;
typedef struct { fast_collection_alias_p0_w1_row *data; size_t len; } fast_collection_alias_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
int64_t fast_collection_alias(fast_collection_alias_p0_w1_rows p0_w1, const fast_owned_resource * p1_w1);
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
fast_owned_resource * fast_collection_resource_new(int64_t p0_w1);
typedef struct {
    int32_t v0;
    fast_owned_resource * v1;
    fast_input_tag v2;
    const fast_owned_resource * v3;
    fast_input_tag v4;
} fast_enum_inputs_alternatives_p0_w1_row;
typedef struct { fast_enum_inputs_alternatives_p0_w1_row *data; size_t len; } fast_enum_inputs_alternatives_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Collection rows: p0_w1 carries rows of 5 ABI fields; element-owned fields are released separately from row storage. */
/* Option option0: row.v4 == -1 means None; other values encode Some and must be valid payloads. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool fast_enum_inputs_alternatives(fast_enum_inputs_alternatives_p0_w1_rows p0_w1, int64_t *out0, size_t *ownership, char * *error);
typedef struct {
    const fast_owned_resource * v0;
    fast_input_tag v1;
} fast_enum_inputs_borrow_p1_w1_row;
typedef struct { fast_enum_inputs_borrow_p1_w1_row *data; size_t len; } fast_enum_inputs_borrow_p1_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Collection rows: p1_w1 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
/* Option option0: row.v1 == -1 means None; other values encode Some and must be valid payloads. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool fast_enum_inputs_borrow(fast_owned_resource * p0_w1, fast_enum_inputs_borrow_p1_w1_rows p1_w1, int64_t *out0, char * *error);
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
uintptr_t fast_enum_inputs_calls(void);
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
uintptr_t fast_enum_inputs_constructors(void);
typedef struct {
    fast_owned_resource * v0;
    fast_input_tag v1;
} fast_enum_inputs_optional_p0_w3_row;
typedef struct { fast_enum_inputs_optional_p0_w3_row *data; size_t len; } fast_enum_inputs_optional_p0_w3_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Collection rows: p0_w3 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
/* Option option1: row.v0 == NULL means None; non-null means Some; only when option0 is present. */
/* Option option2: row.v1 == -1 means None; other values encode Some and must be valid payloads; only when option0 and option1 is present. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool fast_enum_inputs_optional(int32_t p0_w1, fast_enum_inputs_optional_p0_w3_rows p0_w3, int64_t *out0, size_t *ownership, char * *error);
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
void fast_enum_inputs_reset(void);
typedef struct {
    fast_owned_resource * v0;
    fast_input_tag v1;
} fast_enum_inputs_take_p0_w1_row;
typedef struct { fast_enum_inputs_take_p0_w1_row *data; size_t len; } fast_enum_inputs_take_p0_w1_rows;
/* Keep input rows and descriptors alive and unchanged for the whole call, including calls made by source constructors. */
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Collection rows: p0_w1 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: the count of owned wire values consumed in argument/row/field order, including on failure. Consumed allocations and all their aliases must not be reused or destroyed. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool fast_enum_inputs_take(fast_enum_inputs_take_p0_w1_rows p0_w1, int64_t *out0, size_t *ownership, char * *error);
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
/* Option option0: p0_w3 == NULL means None; non-null means Some. */
int64_t fast_option_resource_id(const fast_owned_resource * p0_w3);
/* Input alias conflict checks disabled. The caller must uphold Rust ownership and aliasing rules, including callback results registered by this binding. */
uintptr_t fast_owned_resource_drops(void);
#ifdef __cplusplus
}
#endif
#endif
