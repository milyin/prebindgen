#ifndef perftest_H
#define perftest_H
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#ifdef __cplusplus
extern "C" {
#endif
typedef struct { uint64_t hi; uint64_t lo; } perftest_int128;
 typedef struct { const uint8_t *data; size_t len; } perftest_buffer;
void perftest_free_buffer(perftest_buffer b);
void perftest_free_error(char *error);
typedef int32_t perftest_callback_return_choice;
#define perftest_callback_return_choice_First ((perftest_callback_return_choice)(0))
#define perftest_callback_return_choice_Second ((perftest_callback_return_choice)(1))
typedef int32_t perftest_enum_names;
#define perftest_enum_names_Companion ((perftest_enum_names)(4))
#define perftest_enum_names_wireValue ((perftest_enum_names)(9))
#define perftest_enum_names_fromWire ((perftest_enum_names)(12))
#define perftest_enum_names_type ((perftest_enum_names)(14))
#define perftest_enum_names_when ((perftest_enum_names)(16))
#define perftest_enum_names_name ((perftest_enum_names)(20))
#define perftest_enum_names_ordinal ((perftest_enum_names)(21))
#define perftest_enum_names_entries ((perftest_enum_names)(22))
#define perftest_enum_names_value ((perftest_enum_names)(24))
#define perftest_enum_names_kotlin ((perftest_enum_names)(25))
typedef int32_t perftest_priority;
#define perftest_priority_Low ((perftest_priority)(0))
#define perftest_priority_Normal ((perftest_priority)(1))
#define perftest_priority_High ((perftest_priority)(2))
typedef int32_t perftest_code;
#define perftest_code_Bottom ((perftest_code)(INT32_MIN))
#define perftest_code_MinusTwo ((perftest_code)(-2))
#define perftest_code_MinusOne ((perftest_code)(-1))
#define perftest_code_Zero ((perftest_code)(0))
#define perftest_code_Seven ((perftest_code)(7))
#define perftest_code_Eight ((perftest_code)(8))
#define perftest_code_Top ((perftest_code)(2147483647))
/* Field paths below start at a source parameter or return and list their ABI values.
 * Optional groups below name their discriminator. Rules apply only in active
 * selector arms and present enclosing groups. Absent payload wires are ignored. */
typedef struct perftest_getter_owner perftest_getter_owner;
void perftest_drop_getter_owner(perftest_getter_owner *ptr);
typedef struct perftest_getter_child perftest_getter_child;
void perftest_drop_getter_child(perftest_getter_child *ptr);
typedef struct perftest_getter_leaf perftest_getter_leaf;
void perftest_drop_getter_leaf(perftest_getter_leaf *ptr);
typedef struct perftest_panic_output perftest_panic_output;
void perftest_drop_panic_output(perftest_panic_output *ptr);
typedef struct perftest_callback_return_resource perftest_callback_return_resource;
void perftest_drop_callback_return_resource(perftest_callback_return_resource *ptr);
typedef struct perftest_callback_return_job perftest_callback_return_job;
void perftest_drop_callback_return_job(perftest_callback_return_job *ptr);
typedef struct perftest_callback_return_handler perftest_callback_return_handler;
void perftest_drop_callback_return_handler(perftest_callback_return_handler *ptr);
typedef struct perftest_enum_payload perftest_enum_payload;
void perftest_drop_enum_payload(perftest_enum_payload *ptr);
typedef struct perftest_storage perftest_storage;
void perftest_drop_storage(perftest_storage *ptr);
typedef struct perftest_blob perftest_blob;
void perftest_drop_blob(perftest_blob *ptr);
typedef struct perftest_report perftest_report;
void perftest_drop_report(perftest_report *ptr);
typedef struct perftest_report_struct perftest_report_struct;
void perftest_drop_report_struct(perftest_report_struct *ptr);
typedef struct perftest_summary perftest_summary;
void perftest_drop_summary(perftest_summary *ptr);
typedef struct perftest_unsigned perftest_unsigned;
void perftest_drop_unsigned(perftest_unsigned *ptr);
typedef struct perftest_archive perftest_archive;
void perftest_drop_archive(perftest_archive *ptr);
typedef struct perftest_owned_resource perftest_owned_resource;
void perftest_drop_owned_resource(perftest_owned_resource *ptr);
typedef struct perftest_niche_choice perftest_niche_choice;
void perftest_drop_niche_choice(perftest_niche_choice *ptr);
typedef struct perftest_option_ctor perftest_option_ctor;
void perftest_drop_option_ctor(perftest_option_ctor *ptr);
typedef struct perftest_option_choice perftest_option_choice;
void perftest_drop_option_choice(perftest_option_choice *ptr);
/* Rust signature: ()->perftest_flat :: CallbackReturnResource. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_callback_return_resource * (*run)(void *context);
    void *context;
    void (*drop)(void *context);
} perftest_callback_0e60b44d7236d10d;
/* Rust signature: ()->i64. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    int64_t (*run)(void *context);
    void *context;
    void (*drop)(void *context);
} perftest_callback_15f1a05217a2ceea;
typedef struct {
    perftest_callback_return_resource * v0;
    uint32_t v1;
} perftest_callback_24a5c7966da050a9_result;
/* Rust signature: ()->perftest_flat :: CallbackReturnPair. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_callback_24a5c7966da050a9_result (*run)(void *context);
    void *context;
    void (*drop)(void *context);
} perftest_callback_24a5c7966da050a9;
/* Rust signature: (i64)->(). The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    void (*run)(void *context, int64_t arg0);
    void *context;
    void (*drop)(void *context);
} perftest_callback_321f7c4288706a27;
/* Rust signature: (bool)->bool. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    uint8_t (*run)(void *context, uint8_t arg0);
    void *context;
    void (*drop)(void *context);
} perftest_callback_490413898821d50d;
typedef struct {
    int64_t v0;
    perftest_buffer v1;
} perftest_callback_589eef1af0230719_result;
/* Rust signature: (perftest_flat :: CallbackReturnRecord)->perftest_flat :: CallbackReturnRecord. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_callback_589eef1af0230719_result (*run)(void *context, int64_t arg0, perftest_buffer arg1);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const perftest_callback_589eef1af0230719_result *result);
} perftest_callback_589eef1af0230719;
typedef struct {
    perftest_callback_return_resource * v0;
    uint32_t v1;
} perftest_callback_5f675dc767965255_result;
/* Rust signature: ()->perftest_flat :: CallbackReturnBorrowedPair. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_callback_5f675dc767965255_result (*run)(void *context);
    void *context;
    void (*drop)(void *context);
} perftest_callback_5f675dc767965255;
typedef struct {
    int64_t v0;
} perftest_callback_5fdca16b0dadfb25_callback_arg_0_w0_row;
typedef struct { perftest_callback_5fdca16b0dadfb25_callback_arg_0_w0_row *data; size_t len; } perftest_callback_5fdca16b0dadfb25_callback_arg_0_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_callback_5fdca16b0dadfb25_callback_arg_0_w0_rows_free(perftest_callback_5fdca16b0dadfb25_callback_arg_0_w0_rows rows);
typedef struct {
    int64_t v0;
} perftest_callback_5fdca16b0dadfb25_callback_return_w1_row;
typedef struct { perftest_callback_5fdca16b0dadfb25_callback_return_w1_row *data; size_t len; } perftest_callback_5fdca16b0dadfb25_callback_return_w1_rows;
/* Rust signature: (Vec < i64 >)->Vec < i64 >. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_callback_5fdca16b0dadfb25_callback_return_w1_rows (*run)(void *context, perftest_callback_5fdca16b0dadfb25_callback_arg_0_w0_rows arg0);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const perftest_callback_5fdca16b0dadfb25_callback_return_w1_rows *result);
} perftest_callback_5fdca16b0dadfb25;
/* Rust signature: (f32)->f32. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    float (*run)(void *context, float arg0);
    void *context;
    void (*drop)(void *context);
} perftest_callback_6a62e9f8e6773c45;
/* Rust signature: ()->Option < perftest_flat :: CallbackReturnResource >. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_callback_return_resource * (*run)(void *context);
    void *context;
    void (*drop)(void *context);
} perftest_callback_6e20e8b97ebb3c88;
/* Rust signature: (char)->char. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    uint32_t (*run)(void *context, uint32_t arg0);
    void *context;
    void (*drop)(void *context);
} perftest_callback_73c3c177e4a73331;
/* Rust signature: ()->char. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    uint32_t (*run)(void *context);
    void *context;
    void (*drop)(void *context);
} perftest_callback_8831734aa5203b29;
/* Rust signature: (i128)->i128. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_int128 (*run)(void *context, perftest_int128 arg0);
    void *context;
    void (*drop)(void *context);
} perftest_callback_9067021b3e3252ad;
/* Rust signature: (u128)->u128. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_int128 (*run)(void *context, perftest_int128 arg0);
    void *context;
    void (*drop)(void *context);
} perftest_callback_91e558069da45345;
typedef struct {
    perftest_callback_return_resource * v0;
} perftest_callback_b0a36b18d83434ed_callback_return_w1_row;
typedef struct { perftest_callback_b0a36b18d83434ed_callback_return_w1_row *data; size_t len; } perftest_callback_b0a36b18d83434ed_callback_return_w1_rows;
/* Rust signature: ()->Vec < perftest_flat :: CallbackReturnResource >. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_callback_b0a36b18d83434ed_callback_return_w1_rows (*run)(void *context);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const perftest_callback_b0a36b18d83434ed_callback_return_w1_rows *result);
} perftest_callback_b0a36b18d83434ed;
/* Rust signature: (i64)->i64. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    int64_t (*run)(void *context, int64_t arg0);
    void *context;
    void (*drop)(void *context);
} perftest_callback_cef6210f15ef4535;
/* Rust signature: (f64)->f64. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    double (*run)(void *context, double arg0);
    void *context;
    void (*drop)(void *context);
} perftest_callback_d58f9fdf78f56c8d;
/* Rust signature: (perftest_flat :: CallbackReturnChoice)->perftest_flat :: CallbackReturnChoice. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_callback_return_choice (*run)(void *context, perftest_callback_return_choice arg0);
    void *context;
    void (*drop)(void *context);
} perftest_callback_de793b4a6fb814fd;
/* Rust signature: (Vec < u8 >)->Vec < u8 >. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_buffer (*run)(void *context, perftest_buffer arg0);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const perftest_buffer *result);
} perftest_callback_df547e71faee3c03;
/* Rust signature: (i16)->i16. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    int16_t (*run)(void *context, int16_t arg0);
    void *context;
    void (*drop)(void *context);
} perftest_callback_e51dd6cac1e6761d;
/* Rust signature: (u64)->u64. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    uint64_t (*run)(void *context, uint64_t arg0);
    void *context;
    void (*drop)(void *context);
} perftest_callback_ea174d88bac9b485;
typedef struct {
    int32_t v0;
    int64_t v1;
} perftest_callback_ea6b56ec3d11492d_result;
/* Rust signature: (Option < i64 >)->Option < i64 >. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_callback_ea6b56ec3d11492d_result (*run)(void *context, int32_t arg0, int64_t arg1);
    void *context;
    void (*drop)(void *context);
} perftest_callback_ea6b56ec3d11492d;
/* Rust signature: (String)->String. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    perftest_buffer (*run)(void *context, perftest_buffer arg0);
    void *context;
    void (*drop)(void *context);
    void (*release_result)(void *context, const perftest_buffer *result);
} perftest_callback_efc3c673f144160f;
/* Rust signature: (i8)->i8. The callback context transfers to Rust when input ownership is accepted; drop(context) runs once when the Rust closure is released. run must support the source's thread bounds. Output allocations passed to run are callback-owned. Returned buffers must remain valid until Rust finishes copying them; optional release_result runs after decoding (also on failure) and frees only returned buffer/row storage, never moved handles. */
typedef struct {
    int8_t (*run)(void *context, int8_t arg0);
    void *context;
    void (*drop)(void *context);
} perftest_callback_f5f6b40488a06bd3;
typedef perftest_callback_490413898821d50d perftest_callback_return_bool_callback;
typedef uint8_t perftest_callback_return_bool_callback_return;
typedef perftest_callback_5f675dc767965255 perftest_callback_return_borrowed_pair_catch_callback;
typedef perftest_callback_5f675dc767965255_result perftest_callback_return_borrowed_pair_catch_callback_return;
typedef perftest_callback_df547e71faee3c03 perftest_callback_return_bytes_callback;
typedef perftest_buffer perftest_callback_return_bytes_callback_return;
typedef perftest_callback_73c3c177e4a73331 perftest_callback_return_c_result_callback;
typedef uint32_t perftest_callback_return_c_result_callback_return;
typedef perftest_callback_cef6210f15ef4535 perftest_callback_return_catch_callback;
typedef int64_t perftest_callback_return_catch_callback_return;
typedef perftest_callback_73c3c177e4a73331 perftest_callback_return_char_callback;
typedef uint32_t perftest_callback_return_char_callback_return;
typedef perftest_callback_8831734aa5203b29 perftest_callback_return_char_catch_callback;
typedef uint32_t perftest_callback_return_char_catch_callback_return;
typedef perftest_callback_de793b4a6fb814fd perftest_callback_return_enum_callback;
typedef perftest_callback_return_choice perftest_callback_return_enum_callback_return;
typedef perftest_callback_6a62e9f8e6773c45 perftest_callback_return_f32_callback;
typedef float perftest_callback_return_f32_callback_return;
typedef perftest_callback_d58f9fdf78f56c8d perftest_callback_return_float_callback;
typedef double perftest_callback_return_float_callback_return;
typedef perftest_callback_cef6210f15ef4535 perftest_callback_return_handler_many_handlers;
typedef int64_t perftest_callback_return_handler_many_handlers_return;
typedef perftest_callback_cef6210f15ef4535 perftest_callback_return_handler_new_callback;
typedef int64_t perftest_callback_return_handler_new_callback_return;
typedef perftest_callback_cef6210f15ef4535 perftest_callback_return_handler_value_handler;
typedef int64_t perftest_callback_return_handler_value_handler_return;
typedef perftest_callback_e51dd6cac1e6761d perftest_callback_return_i16_callback;
typedef int16_t perftest_callback_return_i16_callback_return;
typedef perftest_callback_cef6210f15ef4535 perftest_callback_return_i64_callback;
typedef int64_t perftest_callback_return_i64_callback_return;
typedef perftest_callback_f5f6b40488a06bd3 perftest_callback_return_i8_callback;
typedef int8_t perftest_callback_return_i8_callback_return;
typedef perftest_callback_ea6b56ec3d11492d perftest_callback_return_optional_callback;
typedef perftest_callback_ea6b56ec3d11492d_result perftest_callback_return_optional_callback_return;
typedef perftest_callback_0e60b44d7236d10d perftest_callback_return_owned_callback;
typedef perftest_callback_return_resource * perftest_callback_return_owned_callback_return;
typedef perftest_callback_6e20e8b97ebb3c88 perftest_callback_return_owned_optional_callback;
typedef perftest_callback_return_resource * perftest_callback_return_owned_optional_callback_return;
typedef perftest_callback_b0a36b18d83434ed perftest_callback_return_owned_vector_callback;
typedef perftest_callback_b0a36b18d83434ed_callback_return_w1_rows perftest_callback_return_owned_vector_callback_return;
typedef perftest_callback_b0a36b18d83434ed_callback_return_w1_row perftest_callback_return_owned_vector_callback_return_row;
typedef perftest_callback_b0a36b18d83434ed perftest_callback_return_owned_vector_catch_callback;
typedef perftest_callback_b0a36b18d83434ed_callback_return_w1_rows perftest_callback_return_owned_vector_catch_callback_return;
typedef perftest_callback_b0a36b18d83434ed_callback_return_w1_row perftest_callback_return_owned_vector_catch_callback_return_row;
typedef perftest_callback_24a5c7966da050a9 perftest_callback_return_pair_catch_callback;
typedef perftest_callback_24a5c7966da050a9_result perftest_callback_return_pair_catch_callback_return;
typedef perftest_callback_589eef1af0230719 perftest_callback_return_record_callback;
typedef perftest_callback_589eef1af0230719_result perftest_callback_return_record_callback_return;
typedef perftest_callback_cef6210f15ef4535 perftest_callback_return_spawn_callback;
typedef int64_t perftest_callback_return_spawn_callback_return;
typedef perftest_callback_efc3c673f144160f perftest_callback_return_text_callback;
typedef perftest_buffer perftest_callback_return_text_callback_return;
typedef perftest_callback_91e558069da45345 perftest_callback_return_u128_callback;
typedef perftest_int128 perftest_callback_return_u128_callback_return;
typedef perftest_callback_321f7c4288706a27 perftest_callback_return_unit_callback;
typedef perftest_callback_ea174d88bac9b485 perftest_callback_return_unsigned_callback;
typedef uint64_t perftest_callback_return_unsigned_callback_return;
typedef perftest_callback_5fdca16b0dadfb25 perftest_callback_return_vector_callback;
typedef perftest_callback_5fdca16b0dadfb25_callback_return_w1_rows perftest_callback_return_vector_callback_return;
typedef perftest_callback_5fdca16b0dadfb25_callback_return_w1_row perftest_callback_return_vector_callback_return_row;
typedef perftest_callback_5fdca16b0dadfb25_callback_arg_0_w0_rows perftest_callback_return_vector_callback_arg0;
static inline void perftest_callback_return_vector_callback_arg0_free(perftest_callback_return_vector_callback_arg0 value) { perftest_callback_5fdca16b0dadfb25_callback_arg_0_w0_rows_free(value); }
typedef perftest_callback_9067021b3e3252ad perftest_callback_return_wide_callback;
typedef perftest_int128 perftest_callback_return_wide_callback_return;
typedef perftest_callback_15f1a05217a2ceea perftest_callback_return_zero_callback;
typedef int64_t perftest_callback_return_zero_callback_return;
typedef struct {
    int64_t code;
    perftest_buffer message;
} perftest_result_error;
/* Frees owned error fields and clears them; repeated calls are harmless. */
void perftest_drop_result_error(perftest_result_error *error);
/* Field error.code: error->code. */
/* Field error.message: error->message. */
typedef struct {
    perftest_buffer message;
} perftest_panic_error;
/* Frees owned error fields and clears them; repeated calls are harmless. */
void perftest_drop_panic_error(perftest_panic_error *error);
/* Field error.message: error->message. */
typedef struct {
    perftest_code kind;
    int32_t v1;
    perftest_buffer v2;
} perftest_rich_error;
/* Frees owned error fields and clears them; repeated calls are harmless. */
void perftest_drop_rich_error(perftest_rich_error *error);
/* Field error.kind: error->kind. */
/* Field error.message: error->v1, error->v2. */
/* Option option0: error->v1: 0 = None, 1 = Some; other active flags are errors. */
typedef struct {
    perftest_owned_resource * resource;
} perftest_drop_error;
/* Frees owned error fields and clears them; repeated calls are harmless. */
void perftest_drop_drop_error(perftest_drop_error *error);
/* Field error.resource: error->resource. */
typedef struct {
    perftest_owned_resource * resource;
    perftest_buffer message;
} perftest_resource_error;
/* Frees owned error fields and clears them; repeated calls are harmless. */
void perftest_drop_resource_error(perftest_resource_error *error);
/* Field error.resource: error->resource. */
/* Field error.message: error->message. */
typedef struct {
    perftest_owned_resource * v0;
} perftest_error_collection_error_ret_w1_row;
typedef struct { perftest_error_collection_error_ret_w1_row *data; size_t len; } perftest_error_collection_error_ret_w1_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_error_collection_error_ret_w1_rows_free(perftest_error_collection_error_ret_w1_rows rows);
typedef struct {
    int64_t v0;
} perftest_error_collection_error_ret_w10_row;
typedef struct { perftest_error_collection_error_ret_w10_row *data; size_t len; } perftest_error_collection_error_ret_w10_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_error_collection_error_ret_w10_rows_free(perftest_error_collection_error_ret_w10_rows rows);
typedef struct {
    perftest_error_collection_error_ret_w10_rows v0;
} perftest_error_collection_error_ret_w6_row;
typedef struct { perftest_error_collection_error_ret_w6_row *data; size_t len; } perftest_error_collection_error_ret_w6_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_error_collection_error_ret_w6_rows_free(perftest_error_collection_error_ret_w6_rows rows);
typedef struct {
    perftest_error_collection_error_ret_w1_rows resources;
    perftest_error_collection_error_ret_w6_rows codes;
} perftest_collection_error;
/* Frees owned error fields and clears them; repeated calls are harmless. */
void perftest_drop_collection_error(perftest_collection_error *error);
/* Field error.resources: error->resources. */
/* Collection error.resources: ret_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Field error.codes: error->codes. */
/* Collection error.codes: ret_w6 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection error.codes[]: ret_w10 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Non-Result calls return their first value directly; additional values use output pointers.
 * Binding failures require explicit .panic() and abort. Destructors return void. */
/* Field return.count: return value. */
/* Field return.omitted: no boundary values. */
/* Field return.omitted_opt: no boundary values. */
/* Field return.omitted_vec: no boundary values. */
int64_t perftest_api_record_new(void);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
/* Option option0: return value == NULL means None; non-null means Some. */
const perftest_summary * perftest_archive_latest(const perftest_archive * p0_w1);
perftest_archive * perftest_archive_new(void);
void perftest_archive_store(perftest_archive * p0_w1, int32_t p1_w1, int64_t p1_w4, double p1_w6, perftest_summary * p1_w8);
perftest_blob * perftest_blob_value_echo(perftest_blob * p0_w1);
perftest_blob * perftest_blob_value_new(int64_t p0_w1, perftest_buffer p1_w1);
uintptr_t perftest_buffer_views_address(void);
perftest_buffer perftest_buffer_views_bytes(uintptr_t *out1);
/* Option option0: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_buffer_views_optional_bytes(uint8_t p0_w1, perftest_buffer *out1, uintptr_t *out2);
/* Option option0: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_buffer_views_optional_text(uint8_t p0_w1, perftest_buffer *out1, uintptr_t *out2);
perftest_buffer perftest_buffer_views_text(uintptr_t *out1);
perftest_panic_output * perftest_c_panic_output_new(void);
int64_t perftest_c_plain_panic(void);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_c_result_bool(uint8_t p0_w1, uint8_t *out0, perftest_result_error *error);
/* Result: success field v0 is returned directly; NULL means failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
perftest_owned_resource * perftest_c_result_drop_panic(perftest_result_error *error);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_c_result_error_drop(perftest_drop_error *error);
/* Field return.number: *out0. */
/* Field return.character: the return value. */
/* Result: success field v1 is returned directly; 1114112 means failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
uint32_t perftest_c_result_late_niche(uint8_t p0_w1, int64_t *out0, perftest_result_error *error);
/* Option option0: *out0 == NULL means None; non-null means Some. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_c_result_optional_handle(int32_t p0_w1, perftest_owned_resource * *out0, perftest_result_error *error);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_c_result_panic_error(perftest_panic_error *error);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_c_result_resource_error(uint8_t p0_w1, perftest_resource_error *error);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_c_result_rich_error(uint8_t p0_w1, perftest_rich_error *error);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_c_result_structured(perftest_buffer p0_w1, int32_t p1_w1, int64_t *out0, perftest_result_error *error);
uint8_t perftest_callback_return_bool(perftest_callback_490413898821d50d p0_callback);
int64_t perftest_callback_return_borrowed_pair_catch(perftest_callback_5f675dc767965255 p0_callback);
perftest_buffer perftest_callback_return_bytes(perftest_callback_df547e71faee3c03 p0_callback);
/* Result: success field v0 is returned directly; 1114112 means failure. */
/* ownership is required: false before acceptance; true after active owned inputs transfer, including on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
uint32_t perftest_callback_return_c_result(perftest_callback_73c3c177e4a73331 p0_callback, bool *ownership, char * *error);
int64_t perftest_callback_return_catch(perftest_callback_cef6210f15ef4535 p0_callback);
uint32_t perftest_callback_return_char(perftest_callback_73c3c177e4a73331 p0_callback);
uint32_t perftest_callback_return_char_catch(perftest_callback_8831734aa5203b29 p0_callback);
uint8_t perftest_callback_return_enum(perftest_callback_de793b4a6fb814fd p0_callback);
float perftest_callback_return_f32(perftest_callback_6a62e9f8e6773c45 p0_callback);
double perftest_callback_return_float(perftest_callback_d58f9fdf78f56c8d p0_callback);
int64_t perftest_callback_return_handler_call(const perftest_callback_return_handler * p0_w1, int64_t p1_w1);
typedef struct {
    perftest_callback_cef6210f15ef4535 v0;
} perftest_callback_return_handler_many_p0_w1_row;
typedef struct { perftest_callback_return_handler_many_p0_w1_row *data; size_t len; } perftest_callback_return_handler_many_p0_w1_rows;
/* Collection handlers: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
int64_t perftest_callback_return_handler_many(perftest_callback_return_handler_many_p0_w1_rows p0_w1);
/* Result: success field v0 is returned directly; NULL means failure. */
/* ownership is required: false before acceptance; true after active owned inputs transfer, including on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
perftest_callback_return_handler * perftest_callback_return_handler_new(perftest_callback_cef6210f15ef4535 p0_callback, uint8_t p1_w1, bool *ownership, char * *error);
int64_t perftest_callback_return_handler_value(perftest_callback_cef6210f15ef4535 p0_w2);
int16_t perftest_callback_return_i16(perftest_callback_e51dd6cac1e6761d p0_callback);
int64_t perftest_callback_return_i64(perftest_callback_cef6210f15ef4535 p0_callback);
int8_t perftest_callback_return_i8(perftest_callback_f5f6b40488a06bd3 p0_callback);
int64_t perftest_callback_return_join(perftest_callback_return_job * p0_w1);
/* Option option0: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_callback_return_optional(perftest_callback_ea6b56ec3d11492d p0_callback, uint8_t p1_w1, int64_t *out1);
int64_t perftest_callback_return_owned(perftest_callback_0e60b44d7236d10d p0_callback);
/* Option option0: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_callback_return_owned_optional(perftest_callback_6e20e8b97ebb3c88 p0_callback, int64_t *out1);
int64_t perftest_callback_return_owned_vector(perftest_callback_b0a36b18d83434ed p0_callback);
int64_t perftest_callback_return_owned_vector_catch(perftest_callback_b0a36b18d83434ed p0_callback);
int64_t perftest_callback_return_pair_catch(perftest_callback_24a5c7966da050a9 p0_callback);
int64_t perftest_callback_return_record(perftest_callback_589eef1af0230719 p0_callback);
int64_t perftest_callback_return_resource_id(const perftest_callback_return_resource * p0_w1);
uint64_t perftest_callback_return_resource_live(void);
perftest_callback_return_resource * perftest_callback_return_resource_new(int64_t p0_w1);
perftest_callback_return_job * perftest_callback_return_spawn(perftest_callback_cef6210f15ef4535 p0_callback);
perftest_buffer perftest_callback_return_text(perftest_callback_efc3c673f144160f p0_callback);
perftest_int128 perftest_callback_return_u128(perftest_callback_91e558069da45345 p0_callback);
void perftest_callback_return_unit(perftest_callback_321f7c4288706a27 p0_callback);
uint64_t perftest_callback_return_unsigned(perftest_callback_ea174d88bac9b485 p0_callback);
typedef struct {
    int64_t v0;
} perftest_callback_return_vector_ret_w0_row;
typedef struct { perftest_callback_return_vector_ret_w0_row *data; size_t len; } perftest_callback_return_vector_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_callback_return_vector_ret_w0_rows_free(perftest_callback_return_vector_ret_w0_rows rows);
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
perftest_callback_return_vector_ret_w0_rows perftest_callback_return_vector(perftest_callback_5fdca16b0dadfb25 p0_callback);
perftest_int128 perftest_callback_return_wide(perftest_callback_9067021b3e3252ad p0_callback);
int64_t perftest_callback_return_zero(perftest_callback_15f1a05217a2ceea p0_callback);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_alias_p0_w1_row;
typedef struct { perftest_collection_alias_p0_w1_row *data; size_t len; } perftest_collection_alias_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
int64_t perftest_collection_alias(perftest_collection_alias_p0_w1_rows p0_w1, const perftest_owned_resource * p1_w1);
typedef struct {
    int64_t v0;
} perftest_collection_borrow_p0_w1_row;
typedef struct { perftest_collection_borrow_p0_w1_row *data; size_t len; } perftest_collection_borrow_p0_w1_rows;
typedef struct {
    int64_t v0;
} perftest_collection_borrow_ret_w0_row;
typedef struct { perftest_collection_borrow_ret_w0_row *data; size_t len; } perftest_collection_borrow_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_borrow_ret_w0_rows_free(perftest_collection_borrow_ret_w0_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
perftest_collection_borrow_ret_w0_rows perftest_collection_borrow(perftest_collection_borrow_p0_w1_rows p0_w1);
typedef struct {
    int64_t v0;
} perftest_collection_borrow_each_p0_w1_row;
typedef struct { perftest_collection_borrow_each_p0_w1_row *data; size_t len; } perftest_collection_borrow_each_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Callback sequence return: ret_w0 reports the delivered element count; the visitor receives 1 fields per element. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
uintptr_t perftest_collection_borrow_each(perftest_collection_borrow_each_p0_w1_rows p0_w1, void (*callback)(void *context, int64_t v0), void *context);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_bundle_ret_w2_row;
typedef struct { perftest_collection_bundle_ret_w2_row *data; size_t len; } perftest_collection_bundle_ret_w2_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_bundle_ret_w2_rows_free(perftest_collection_bundle_ret_w2_rows rows);
/* Field return.values: return value. */
/* Collection return.values: ret_w2 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Field return.tag: *out1. */
perftest_collection_bundle_ret_w2_rows perftest_collection_bundle(int64_t *out1);
typedef struct {
    perftest_buffer v0;
} perftest_collection_bytes_p0_w1_row;
typedef struct { perftest_collection_bytes_p0_w1_row *data; size_t len; } perftest_collection_bytes_p0_w1_rows;
typedef struct {
    perftest_buffer v0;
} perftest_collection_bytes_ret_w0_row;
typedef struct { perftest_collection_bytes_ret_w0_row *data; size_t len; } perftest_collection_bytes_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_bytes_ret_w0_rows_free(perftest_collection_bytes_ret_w0_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
perftest_collection_bytes_ret_w0_rows perftest_collection_bytes(perftest_collection_bytes_p0_w1_rows p0_w1);
typedef struct {
    perftest_buffer v0;
} perftest_collection_bytes_each_p0_w1_row;
typedef struct { perftest_collection_bytes_each_p0_w1_row *data; size_t len; } perftest_collection_bytes_each_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Callback sequence return: ret_w0 reports the delivered element count; the visitor receives 1 fields per element. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
uintptr_t perftest_collection_bytes_each(perftest_collection_bytes_each_p0_w1_rows p0_w1, void (*callback)(void *context, perftest_buffer v0), void *context);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_construct_all_p0_w1_row;
typedef struct { perftest_collection_construct_all_p0_w1_row *data; size_t len; } perftest_collection_construct_all_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
int64_t perftest_collection_construct_all(perftest_collection_construct_all_p0_w1_rows p0_w1);
typedef struct {
    perftest_code v0;
} perftest_collection_enum_p0_w1_row;
typedef struct { perftest_collection_enum_p0_w1_row *data; size_t len; } perftest_collection_enum_p0_w1_rows;
typedef struct {
    perftest_code v0;
} perftest_collection_enum_ret_w0_row;
typedef struct { perftest_collection_enum_ret_w0_row *data; size_t len; } perftest_collection_enum_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_enum_ret_w0_rows_free(perftest_collection_enum_ret_w0_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
perftest_collection_enum_ret_w0_rows perftest_collection_enum(perftest_collection_enum_p0_w1_rows p0_w1);
typedef struct {
    perftest_code v0;
} perftest_collection_enum_each_p0_w1_row;
typedef struct { perftest_collection_enum_each_p0_w1_row *data; size_t len; } perftest_collection_enum_each_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Callback sequence return: ret_w0 reports the delivered element count; the visitor receives 1 fields per element. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
uintptr_t perftest_collection_enum_each(perftest_collection_enum_each_p0_w1_rows p0_w1, void (*callback)(void *context, perftest_code v0), void *context);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_collection_fail(perftest_collection_error *error);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_handles_p0_w1_row;
typedef struct { perftest_collection_handles_p0_w1_row *data; size_t len; } perftest_collection_handles_p0_w1_rows;
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_handles_ret_w2_row;
typedef struct { perftest_collection_handles_ret_w2_row *data; size_t len; } perftest_collection_handles_ret_w2_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_handles_ret_w2_rows_free(perftest_collection_handles_ret_w2_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return: ret_w2 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: false before acceptance; true after active owned inputs transfer, including on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_collection_handles(perftest_collection_handles_p0_w1_rows p0_w1, uint32_t p1_w1, int32_t p2_w1, perftest_collection_handles_ret_w2_rows *out0, bool *ownership, char * *error);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_handles_each_p0_w1_row;
typedef struct { perftest_collection_handles_each_p0_w1_row *data; size_t len; } perftest_collection_handles_each_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
/* Callback sequence return: ret_w2 reports the delivered element count; the visitor receives 1 fields per element. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: false before acceptance; true after active owned inputs transfer, including on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_collection_handles_each(perftest_collection_handles_each_p0_w1_rows p0_w1, uint32_t p1_w1, int32_t p2_w1, void (*callback)(void *context, perftest_owned_resource * v0), void *context, uintptr_t *out0, bool *ownership, char * *error);
typedef struct {
    int64_t v0;
} perftest_collection_i64_p0_w1_row;
typedef struct { perftest_collection_i64_p0_w1_row *data; size_t len; } perftest_collection_i64_p0_w1_rows;
typedef struct {
    int64_t v0;
} perftest_collection_i64_ret_w0_row;
typedef struct { perftest_collection_i64_ret_w0_row *data; size_t len; } perftest_collection_i64_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_i64_ret_w0_rows_free(perftest_collection_i64_ret_w0_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
perftest_collection_i64_ret_w0_rows perftest_collection_i64(perftest_collection_i64_p0_w1_rows p0_w1);
typedef struct {
    int64_t v0;
} perftest_collection_i64_each_p0_w1_row;
typedef struct { perftest_collection_i64_each_p0_w1_row *data; size_t len; } perftest_collection_i64_each_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Callback sequence return: ret_w0 reports the delivered element count; the visitor receives 1 fields per element. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
uintptr_t perftest_collection_i64_each(perftest_collection_i64_each_p0_w1_rows p0_w1, void (*callback)(void *context, int64_t v0), void *context);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_late_drop_ret_w7_row;
typedef struct { perftest_collection_late_drop_ret_w7_row *data; size_t len; } perftest_collection_late_drop_ret_w7_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_late_drop_ret_w7_rows_free(perftest_collection_late_drop_ret_w7_rows rows);
typedef struct {
    perftest_collection_late_drop_ret_w7_rows v0;
} perftest_collection_late_drop_ret_w3_row;
typedef struct { perftest_collection_late_drop_ret_w3_row *data; size_t len; } perftest_collection_late_drop_ret_w3_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_late_drop_ret_w3_rows_free(perftest_collection_late_drop_ret_w3_rows rows);
/* Collection return: ret_w3 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return[]: ret_w7 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_collection_late_drop(perftest_collection_late_drop_ret_w3_rows *out0, perftest_result_error *error);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_make_items_ret_w0_row;
typedef struct { perftest_collection_make_items_ret_w0_row *data; size_t len; } perftest_collection_make_items_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_make_items_ret_w0_rows_free(perftest_collection_make_items_ret_w0_rows rows);
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Field return[].resource: row.v0. */
perftest_collection_make_items_ret_w0_rows perftest_collection_make_items(void);
typedef struct {
    int64_t v0;
} perftest_collection_nested_p0_w5_row;
typedef struct { perftest_collection_nested_p0_w5_row *data; size_t len; } perftest_collection_nested_p0_w5_rows;
typedef struct {
    perftest_collection_nested_p0_w5_rows v0;
} perftest_collection_nested_p0_w1_row;
typedef struct { perftest_collection_nested_p0_w1_row *data; size_t len; } perftest_collection_nested_p0_w1_rows;
typedef struct {
    int64_t v0;
} perftest_collection_nested_ret_w4_row;
typedef struct { perftest_collection_nested_ret_w4_row *data; size_t len; } perftest_collection_nested_ret_w4_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_nested_ret_w4_rows_free(perftest_collection_nested_ret_w4_rows rows);
typedef struct {
    perftest_collection_nested_ret_w4_rows v0;
} perftest_collection_nested_ret_w0_row;
typedef struct { perftest_collection_nested_ret_w0_row *data; size_t len; } perftest_collection_nested_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_nested_ret_w0_rows_free(perftest_collection_nested_ret_w0_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection values[]: p0_w5 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return[]: ret_w4 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
perftest_collection_nested_ret_w0_rows perftest_collection_nested(perftest_collection_nested_p0_w1_rows p0_w1);
typedef struct {
    int64_t v0;
} perftest_collection_nested_each_p0_w5_row;
typedef struct { perftest_collection_nested_each_p0_w5_row *data; size_t len; } perftest_collection_nested_each_p0_w5_rows;
typedef struct {
    perftest_collection_nested_each_p0_w5_rows v0;
} perftest_collection_nested_each_p0_w1_row;
typedef struct { perftest_collection_nested_each_p0_w1_row *data; size_t len; } perftest_collection_nested_each_p0_w1_rows;
typedef struct {
    int64_t v0;
} perftest_collection_nested_each_ret_w4_row;
typedef struct { perftest_collection_nested_each_ret_w4_row *data; size_t len; } perftest_collection_nested_each_ret_w4_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_nested_each_ret_w4_rows_free(perftest_collection_nested_each_ret_w4_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection values[]: p0_w5 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Callback sequence return: ret_w0 reports the delivered element count; the visitor receives 1 fields per element. */
/* Collection return[]: ret_w4 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
uintptr_t perftest_collection_nested_each(perftest_collection_nested_each_p0_w1_rows p0_w1, void (*callback)(void *context, perftest_collection_nested_each_ret_w4_rows v0), void *context);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_nested_handles_p0_w5_row;
typedef struct { perftest_collection_nested_handles_p0_w5_row *data; size_t len; } perftest_collection_nested_handles_p0_w5_rows;
typedef struct {
    perftest_collection_nested_handles_p0_w5_rows v0;
} perftest_collection_nested_handles_p0_w1_row;
typedef struct { perftest_collection_nested_handles_p0_w1_row *data; size_t len; } perftest_collection_nested_handles_p0_w1_rows;
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_nested_handles_ret_w4_row;
typedef struct { perftest_collection_nested_handles_ret_w4_row *data; size_t len; } perftest_collection_nested_handles_ret_w4_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_nested_handles_ret_w4_rows_free(perftest_collection_nested_handles_ret_w4_rows rows);
typedef struct {
    perftest_collection_nested_handles_ret_w4_rows v0;
} perftest_collection_nested_handles_ret_w0_row;
typedef struct { perftest_collection_nested_handles_ret_w0_row *data; size_t len; } perftest_collection_nested_handles_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_nested_handles_ret_w0_rows_free(perftest_collection_nested_handles_ret_w0_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection values[]: p0_w5 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return[]: ret_w4 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
perftest_collection_nested_handles_ret_w0_rows perftest_collection_nested_handles(perftest_collection_nested_handles_p0_w1_rows p0_w1);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_nested_handles_each_p0_w5_row;
typedef struct { perftest_collection_nested_handles_each_p0_w5_row *data; size_t len; } perftest_collection_nested_handles_each_p0_w5_rows;
typedef struct {
    perftest_collection_nested_handles_each_p0_w5_rows v0;
} perftest_collection_nested_handles_each_p0_w1_row;
typedef struct { perftest_collection_nested_handles_each_p0_w1_row *data; size_t len; } perftest_collection_nested_handles_each_p0_w1_rows;
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_nested_handles_each_ret_w4_row;
typedef struct { perftest_collection_nested_handles_each_ret_w4_row *data; size_t len; } perftest_collection_nested_handles_each_ret_w4_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_nested_handles_each_ret_w4_rows_free(perftest_collection_nested_handles_each_ret_w4_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection values[]: p0_w5 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Callback sequence return: ret_w0 reports the delivered element count; the visitor receives 1 fields per element. */
/* Collection return[]: ret_w4 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
uintptr_t perftest_collection_nested_handles_each(perftest_collection_nested_handles_each_p0_w1_rows p0_w1, void (*callback)(void *context, perftest_collection_nested_handles_each_ret_w4_rows v0), void *context);
typedef struct {
    int64_t v0;
} perftest_collection_numbers_ret_w2_row;
typedef struct { perftest_collection_numbers_ret_w2_row *data; size_t len; } perftest_collection_numbers_ret_w2_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_numbers_ret_w2_rows_free(perftest_collection_numbers_ret_w2_rows rows);
/* Field return.values: return value. */
/* Collection return.values: ret_w2 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Field return.tag: *out1. */
perftest_collection_numbers_ret_w2_rows perftest_collection_numbers(int64_t *out1);
typedef struct {
    int64_t v0;
} perftest_collection_optional_p0_w3_row;
typedef struct { perftest_collection_optional_p0_w3_row *data; size_t len; } perftest_collection_optional_p0_w3_rows;
typedef struct {
    int64_t v0;
} perftest_collection_optional_ret_w2_row;
typedef struct { perftest_collection_optional_ret_w2_row *data; size_t len; } perftest_collection_optional_ret_w2_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_optional_ret_w2_rows_free(perftest_collection_optional_ret_w2_rows rows);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Collection values: p0_w3 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
/* Collection return: ret_w2 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
int32_t perftest_collection_optional(int32_t p0_w1, perftest_collection_optional_p0_w3_rows p0_w3, perftest_collection_optional_ret_w2_rows *out1);
typedef struct {
    int64_t v0;
} perftest_collection_optional_each_p0_w3_row;
typedef struct { perftest_collection_optional_each_p0_w3_row *data; size_t len; } perftest_collection_optional_each_p0_w3_rows;
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Collection values: p0_w3 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
/* Callback sequence return: ret_w2 reports the delivered element count; the visitor receives 1 fields per element. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
int32_t perftest_collection_optional_each(int32_t p0_w1, perftest_collection_optional_each_p0_w3_rows p0_w3, void (*callback)(void *context, int64_t v0), void *context, uintptr_t *out1);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_optional_handles_p0_w3_row;
typedef struct { perftest_collection_optional_handles_p0_w3_row *data; size_t len; } perftest_collection_optional_handles_p0_w3_rows;
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_optional_handles_ret_w2_row;
typedef struct { perftest_collection_optional_handles_ret_w2_row *data; size_t len; } perftest_collection_optional_handles_ret_w2_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_optional_handles_ret_w2_rows_free(perftest_collection_optional_handles_ret_w2_rows rows);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Collection values: p0_w3 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
/* Collection return: ret_w2 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
int32_t perftest_collection_optional_handles(int32_t p0_w1, perftest_collection_optional_handles_p0_w3_rows p0_w3, perftest_collection_optional_handles_ret_w2_rows *out1);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_optional_handles_each_p0_w3_row;
typedef struct { perftest_collection_optional_handles_each_p0_w3_row *data; size_t len; } perftest_collection_optional_handles_each_p0_w3_rows;
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Collection values: p0_w3 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
/* Callback sequence return: ret_w2 reports the delivered element count; the visitor receives 1 fields per element. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
int32_t perftest_collection_optional_handles_each(int32_t p0_w1, perftest_collection_optional_handles_each_p0_w3_rows p0_w3, void (*callback)(void *context, perftest_owned_resource * v0), void *context, uintptr_t *out1);
typedef struct {
    int32_t v0;
    int64_t v1;
} perftest_collection_optional_items_p0_w3_row;
typedef struct { perftest_collection_optional_items_p0_w3_row *data; size_t len; } perftest_collection_optional_items_p0_w3_rows;
typedef struct {
    int32_t v0;
    int64_t v1;
} perftest_collection_optional_items_ret_w2_row;
typedef struct { perftest_collection_optional_items_ret_w2_row *data; size_t len; } perftest_collection_optional_items_ret_w2_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_optional_items_ret_w2_rows_free(perftest_collection_optional_items_ret_w2_rows rows);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Collection values: p0_w3 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
/* Option option1: row.v0: 0 = None, 1 = Some; other active flags are errors; only when option0 is present. */
/* Option option2: return value: 0 = None, 1 = Some; other active flags are errors. */
/* Collection return: ret_w2 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
/* Option option3: row.v0: 0 = None, 1 = Some; other active flags are errors; only when option2 is present. */
int32_t perftest_collection_optional_items(int32_t p0_w1, perftest_collection_optional_items_p0_w3_rows p0_w3, perftest_collection_optional_items_ret_w2_rows *out1);
typedef struct {
    int32_t v0;
    int64_t v1;
} perftest_collection_optional_items_each_p0_w3_row;
typedef struct { perftest_collection_optional_items_each_p0_w3_row *data; size_t len; } perftest_collection_optional_items_each_p0_w3_rows;
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Collection values: p0_w3 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
/* Option option1: row.v0: 0 = None, 1 = Some; other active flags are errors; only when option0 is present. */
/* Option option2: return value: 0 = None, 1 = Some; other active flags are errors. */
/* Callback sequence return: ret_w2 reports the delivered element count; the visitor receives 2 fields per element. */
/* Option option3: row.v0: 0 = None, 1 = Some; other active flags are errors; only when option2 is present. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
int32_t perftest_collection_optional_items_each(int32_t p0_w1, perftest_collection_optional_items_each_p0_w3_rows p0_w3, void (*callback)(void *context, int32_t v0, int64_t v1), void *context, uintptr_t *out1);
typedef struct {
    int32_t v0;
    int64_t v1;
} perftest_collection_options_p0_w1_row;
typedef struct { perftest_collection_options_p0_w1_row *data; size_t len; } perftest_collection_options_p0_w1_rows;
typedef struct {
    int32_t v0;
    int64_t v1;
} perftest_collection_options_ret_w0_row;
typedef struct { perftest_collection_options_ret_w0_row *data; size_t len; } perftest_collection_options_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_options_ret_w0_rows_free(perftest_collection_options_ret_w0_rows rows);
/* Collection values: p0_w1 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
/* Option option0: row.v0: 0 = None, 1 = Some; other active flags are errors. */
/* Collection return: ret_w0 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
/* Option option1: row.v0: 0 = None, 1 = Some; other active flags are errors. */
perftest_collection_options_ret_w0_rows perftest_collection_options(perftest_collection_options_p0_w1_rows p0_w1, int16_t p1_w1);
typedef struct {
    int32_t v0;
    int64_t v1;
} perftest_collection_options_each_p0_w1_row;
typedef struct { perftest_collection_options_each_p0_w1_row *data; size_t len; } perftest_collection_options_each_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 2 ABI fields; element-owned fields are released separately from row storage. */
/* Option option0: row.v0: 0 = None, 1 = Some; other active flags are errors. */
/* Callback sequence return: ret_w0 reports the delivered element count; the visitor receives 2 fields per element. */
/* Option option1: row.v0: 0 = None, 1 = Some; other active flags are errors. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
uintptr_t perftest_collection_options_each(perftest_collection_options_each_p0_w1_rows p0_w1, int16_t p1_w1, void (*callback)(void *context, int32_t v0, int64_t v1), void *context);
typedef struct {
    uint32_t v0;
} perftest_collection_probe_p0_w1_row;
typedef struct { perftest_collection_probe_p0_w1_row *data; size_t len; } perftest_collection_probe_p0_w1_rows;
typedef struct {
    uint32_t v0;
} perftest_collection_probe_ret_w0_row;
typedef struct { perftest_collection_probe_ret_w0_row *data; size_t len; } perftest_collection_probe_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_probe_ret_w0_rows_free(perftest_collection_probe_ret_w0_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
perftest_collection_probe_ret_w0_rows perftest_collection_probe(perftest_collection_probe_p0_w1_rows p0_w1, perftest_buffer p1_w1);
typedef struct {
    const perftest_owned_resource * v0;
} perftest_collection_resource_ids_p0_w1_row;
typedef struct { perftest_collection_resource_ids_p0_w1_row *data; size_t len; } perftest_collection_resource_ids_p0_w1_rows;
typedef struct {
    int64_t v0;
} perftest_collection_resource_ids_ret_w0_row;
typedef struct { perftest_collection_resource_ids_ret_w0_row *data; size_t len; } perftest_collection_resource_ids_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_resource_ids_ret_w0_rows_free(perftest_collection_resource_ids_ret_w0_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
perftest_collection_resource_ids_ret_w0_rows perftest_collection_resource_ids(perftest_collection_resource_ids_p0_w1_rows p0_w1);
perftest_owned_resource * perftest_collection_resource_new(int64_t p0_w1);
typedef struct {
    int64_t v0;
} perftest_collection_result_p0_w1_row;
typedef struct { perftest_collection_result_p0_w1_row *data; size_t len; } perftest_collection_result_p0_w1_rows;
typedef struct {
    int64_t v0;
} perftest_collection_result_ret_w2_row;
typedef struct { perftest_collection_result_ret_w2_row *data; size_t len; } perftest_collection_result_ret_w2_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_result_ret_w2_rows_free(perftest_collection_result_ret_w2_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return: ret_w2 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_collection_result(perftest_collection_result_p0_w1_rows p0_w1, uint8_t p1_w1, perftest_collection_result_ret_w2_rows *out0, char * *error);
typedef struct {
    int64_t v0;
} perftest_collection_result_each_p0_w1_row;
typedef struct { perftest_collection_result_each_p0_w1_row *data; size_t len; } perftest_collection_result_each_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
/* Callback sequence return: ret_w2 reports the delivered element count; the visitor receives 1 fields per element. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_collection_result_each(perftest_collection_result_each_p0_w1_rows p0_w1, uint8_t p1_w1, void (*callback)(void *context, int64_t v0), void *context, uintptr_t *out0, char * *error);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_result_optional_p0_w1_row;
typedef struct { perftest_collection_result_optional_p0_w1_row *data; size_t len; } perftest_collection_result_optional_p0_w1_rows;
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_result_optional_ret_w4_row;
typedef struct { perftest_collection_result_optional_ret_w4_row *data; size_t len; } perftest_collection_result_optional_ret_w4_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_result_optional_ret_w4_rows_free(perftest_collection_result_optional_ret_w4_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Option option0: *out0: 0 = None, 1 = Some; other active flags are errors. */
/* Collection return: ret_w4 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: false before acceptance; true after active owned inputs transfer, including on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_collection_result_optional(perftest_collection_result_optional_p0_w1_rows p0_w1, int32_t p1_w1, int32_t *out0, perftest_collection_result_optional_ret_w4_rows *out1, bool *ownership, char * *error);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_result_optional_each_p0_w1_row;
typedef struct { perftest_collection_result_optional_each_p0_w1_row *data; size_t len; } perftest_collection_result_optional_each_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
/* Option option0: *out0: 0 = None, 1 = Some; other active flags are errors. */
/* Callback sequence return: ret_w4 reports the delivered element count; the visitor receives 1 fields per element. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: false before acceptance; true after active owned inputs transfer, including on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_collection_result_optional_each(perftest_collection_result_optional_each_p0_w1_rows p0_w1, int32_t p1_w1, void (*callback)(void *context, perftest_owned_resource * v0), void *context, int32_t *out0, uintptr_t *out1, bool *ownership, char * *error);
typedef struct {
    int64_t v0;
} perftest_collection_slice_p0_w1_row;
typedef struct { perftest_collection_slice_p0_w1_row *data; size_t len; } perftest_collection_slice_p0_w1_rows;
typedef struct {
    int64_t v0;
} perftest_collection_slice_ret_w0_row;
typedef struct { perftest_collection_slice_ret_w0_row *data; size_t len; } perftest_collection_slice_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_slice_ret_w0_rows_free(perftest_collection_slice_ret_w0_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
perftest_collection_slice_ret_w0_rows perftest_collection_slice(perftest_collection_slice_p0_w1_rows p0_w1);
typedef struct {
    int64_t v0;
} perftest_collection_slice_each_p0_w1_row;
typedef struct { perftest_collection_slice_each_p0_w1_row *data; size_t len; } perftest_collection_slice_each_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Callback sequence return: ret_w0 reports the delivered element count; the visitor receives 1 fields per element. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
uintptr_t perftest_collection_slice_each(perftest_collection_slice_each_p0_w1_rows p0_w1, void (*callback)(void *context, int64_t v0), void *context);
typedef struct {
    perftest_buffer v0;
} perftest_collection_text_p0_w1_row;
typedef struct { perftest_collection_text_p0_w1_row *data; size_t len; } perftest_collection_text_p0_w1_rows;
typedef struct {
    perftest_buffer v0;
} perftest_collection_text_ret_w0_row;
typedef struct { perftest_collection_text_ret_w0_row *data; size_t len; } perftest_collection_text_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_text_ret_w0_rows_free(perftest_collection_text_ret_w0_rows rows);
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
perftest_collection_text_ret_w0_rows perftest_collection_text(perftest_collection_text_p0_w1_rows p0_w1);
typedef struct {
    perftest_buffer v0;
} perftest_collection_text_each_p0_w1_row;
typedef struct { perftest_collection_text_each_p0_w1_row *data; size_t len; } perftest_collection_text_each_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Callback sequence return: ret_w0 reports the delivered element count; the visitor receives 1 fields per element. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
uintptr_t perftest_collection_text_each(perftest_collection_text_each_p0_w1_rows p0_w1, void (*callback)(void *context, perftest_buffer v0), void *context);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_try_construct_all_p0_w1_row;
typedef struct { perftest_collection_try_construct_all_p0_w1_row *data; size_t len; } perftest_collection_try_construct_all_p0_w1_rows;
/* Collection values: p0_w1 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* ownership is required: false before acceptance; true after active owned inputs transfer, including on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_collection_try_construct_all(perftest_collection_try_construct_all_p0_w1_rows p0_w1, int64_t *out0, bool *ownership, char * *error);
typedef struct {
    perftest_owned_resource * v0;
} perftest_collection_try_make_items_ret_w2_row;
typedef struct { perftest_collection_try_make_items_ret_w2_row *data; size_t len; } perftest_collection_try_make_items_ret_w2_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_collection_try_make_items_ret_w2_rows_free(perftest_collection_try_make_items_ret_w2_rows rows);
/* Collection return: ret_w2 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
/* Field return[].resource: row.v0. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_collection_try_make_items(perftest_collection_try_make_items_ret_w2_rows *out0, char * *error);
perftest_enum_names perftest_enum_names_echo(perftest_enum_names p0_w1);
perftest_enum_payload * perftest_enum_payload_new(double p0_w1);
/* Option option0: p0_w6 == -3 means None; other values encode Some and must be valid payloads. */
/* Field return.priority: return value. */
/* Field return.code: *out1. */
/* Option option1: *out1 == -3 means None; other values encode Some and must be valid payloads. */
perftest_priority perftest_enum_record_echo(perftest_priority p0_w2, perftest_code p0_w6, perftest_code *out1);
perftest_buffer perftest_getter_child_name(const perftest_getter_child * p0_w1);
int64_t perftest_getter_child_number(const perftest_getter_child * p0_w1);
void perftest_getter_child_set_number(perftest_getter_child * p0_w1, int64_t p1_w1);
typedef struct {
    uintptr_t v0;
} perftest_getter_counts_ret_w0_row;
typedef struct { perftest_getter_counts_ret_w0_row *data; size_t len; } perftest_getter_counts_ret_w0_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_getter_counts_ret_w0_rows_free(perftest_getter_counts_ret_w0_rows rows);
/* Collection return: ret_w0 carries rows of 1 ABI fields; element-owned fields are released separately from row storage. */
perftest_getter_counts_ret_w0_rows perftest_getter_counts(void);
/* Field return.number: *out1. */
/* Field return.text: *out2. */
perftest_getter_owner * perftest_getter_fields_new(int64_t *out1, perftest_buffer *out2);
int64_t perftest_getter_leaf_id(const perftest_getter_leaf * p0_w1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
const perftest_getter_child * perftest_getter_owner_child(const perftest_getter_owner * p0_w1);
uint8_t perftest_getter_owner_child_matches(const perftest_getter_owner * p0_w1, const perftest_getter_child * p1_w1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
perftest_getter_child * perftest_getter_owner_child_mut(perftest_getter_owner * p0_w1);
int64_t perftest_getter_owner_id(const perftest_getter_owner * p0_w1);
perftest_getter_owner * perftest_getter_owner_new(uint8_t p0_w1, int64_t *out1, perftest_buffer *out2, perftest_getter_leaf * *out3);
/* Option option0: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_getter_owner_optional(const perftest_getter_owner * p0_w1, int64_t *out1, perftest_buffer *out2);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
/* Option option0: return value == NULL means None; non-null means Some. */
perftest_getter_child * perftest_getter_owner_optional_mut(perftest_getter_owner * p0_w1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
/* Option option0: return value == NULL means None; non-null means Some. */
const perftest_getter_child * perftest_getter_owner_optional_pointer(const perftest_getter_owner * p0_w1);
int64_t perftest_getter_owner_read(const perftest_getter_owner * p0_w1, perftest_buffer *out1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
/* Result: success field v0 is returned directly; NULL means failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
const perftest_getter_child * perftest_getter_owner_try_child(const perftest_getter_owner * p0_w1, uint8_t p1_w1, char * *error);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
/* Result: success field v0 is returned directly; NULL means failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
perftest_getter_child * perftest_getter_owner_try_child_mut(perftest_getter_owner * p0_w1, uint8_t p1_w1, char * *error);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
/* Option option0: *out0 == NULL means None; non-null means Some. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_getter_owner_try_optional_child(const perftest_getter_owner * p0_w1, int32_t p1_w1, const perftest_getter_child * *out0, char * *error);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
/* Option option0: *out0 == NULL means None; non-null means Some. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_getter_owner_try_optional_mut(perftest_getter_owner * p0_w1, int32_t p1_w1, perftest_getter_child * *out0, char * *error);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
const perftest_getter_child * perftest_getter_static_child(perftest_buffer p0_w1);
perftest_getter_leaf * perftest_getter_two_clones_new(perftest_getter_leaf * *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: p0_w8 == 1114112 means None; other values encode Some and must be valid payloads; only when option0 is present. */
int64_t perftest_niche_choice_id(int32_t p0_w1, int32_t p0_w3, uint32_t p0_w8, perftest_niche_choice * p0_w10);
/* Option option0: p0_w3 == 1114112 means None; other values encode Some and must be valid payloads. */
perftest_niche_choice * perftest_niche_choice_new(uint32_t p0_w3);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_option_bool(int32_t p0_w1, uint8_t p0_w3, uint8_t *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_option_bytes(int32_t p0_w1, perftest_buffer p0_w3, perftest_buffer *out1);
/* Option option0: p0_w3 == 1114112 means None; other values encode Some and must be valid payloads. */
/* Option option1: return value == 1114112 means None; other values encode Some and must be valid payloads. */
uint32_t perftest_option_char(uint32_t p0_w3);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
int64_t perftest_option_choice_id(int32_t p0_w1, int32_t p0_w3, int64_t p0_w6, perftest_option_choice * p0_w8);
/* Option option0: p0_w4: 0 = None, 1 = Some; other active flags are errors. */
int64_t perftest_option_ctor_id(int32_t p0_w1, int32_t p0_w4, int64_t p0_w6, perftest_option_ctor * p0_w8);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
perftest_option_ctor * perftest_option_ctor_new(int32_t p0_w1, int64_t p0_w3);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_option_i64(int32_t p0_w1, int64_t p0_w3, int64_t *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_option_i64_ref(int32_t p0_w1, int64_t p0_w3, int64_t *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: p0_w3: 0 = None, 1 = Some; other active flags are errors; only when option0 is present. */
/* Option option2: return value: 0 = None, 1 = Some; other active flags are errors. */
/* Option option3: *out1: 0 = None, 1 = Some; other active flags are errors; only when option2 is present. */
int32_t perftest_option_nested(int32_t p0_w1, int32_t p0_w3, int64_t p0_w5, int32_t *out1, int64_t *out2);
/* Option option0: p0_w5 == 1114113 means None; other values encode Some and must be valid payloads. */
/* Option option1: p0_w5 == 1114112 means None; other values encode Some and must be valid payloads; only when option0 is present. */
/* Option option2: return value == 1114113 means None; other values encode Some and must be valid payloads. */
/* Option option3: return value == 1114112 means None; other values encode Some and must be valid payloads; only when option2 is present. */
uint32_t perftest_option_nested_char(uint32_t p0_w5);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: p0_w5 == NULL means None; non-null means Some; only when option0 is present. */
/* Option option2: return value: 0 = None, 1 = Some; other active flags are errors. */
/* Option option3: *out1 == NULL means None; non-null means Some; only when option2 is present. */
int32_t perftest_option_nested_resource(int32_t p0_w1, perftest_owned_resource * p0_w5, perftest_owned_resource * *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: p0_w3: 0 = None, 1 = Some; other active flags are errors; only when option0 is present. */
/* Option option2: return value: 0 = None, 1 = Some; other active flags are errors. */
/* Option option3: *out1: 0 = None, 1 = Some; other active flags are errors; only when option2 is present. */
int32_t perftest_option_nested_u32(int32_t p0_w1, int32_t p0_w3, uint32_t p0_w5, int32_t *out1, uint32_t *out2);
/* Option option0: p0_w3 == NULL means None; non-null means Some. */
/* Option option1: return value == NULL means None; non-null means Some. */
perftest_owned_resource * perftest_option_resource_echo(perftest_owned_resource * p0_w3, uint32_t p1_w1);
/* Option option0: p0_w3 == NULL means None; non-null means Some. */
int64_t perftest_option_resource_id(const perftest_owned_resource * p0_w3);
/* Option option0: return value == NULL means None; non-null means Some. */
perftest_owned_resource * perftest_option_resource_new(uint8_t p0_w1);
/* Option option0: p0_w3 == NULL means None; non-null means Some. */
/* Option option1: p1_w3 == NULL means None; non-null means Some. */
int64_t perftest_option_resource_pair(perftest_owned_resource * p0_w3, perftest_owned_resource * p1_w3);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_option_slice_echo(int32_t p0_w1, perftest_buffer p0_w3, perftest_buffer *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_option_slice_len(int32_t p0_w1, perftest_buffer p0_w3, uintptr_t *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_option_str_echo(int32_t p0_w1, perftest_buffer p0_w3, perftest_buffer *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_option_str_len(int32_t p0_w1, perftest_buffer p0_w3, uintptr_t *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_option_string_len(int32_t p0_w1, perftest_buffer p0_w3, uintptr_t *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_option_text(int32_t p0_w1, perftest_buffer p0_w3, perftest_buffer *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_option_u8(int32_t p0_w1, uint8_t p0_w3, uint8_t *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_option_unit(int32_t p0_w1);
/* Option option0: p0_w14 == 1114113 means None; other values encode Some and must be valid payloads. */
/* Option option1: p0_w4: 0 = None, 1 = Some; other active flags are errors; only when option0 is present. */
/* Option option2: p0_w8: 0 = None, 1 = Some; other active flags are errors; only when option0 is present. */
/* Option option3: p0_w14 == 1114112 means None; other values encode Some and must be valid payloads; only when option0 is present. */
/* Option option4: *out4 == 1114113 means None; other values encode Some and must be valid payloads. */
/* Field return.count: return value, *out1. */
/* Option option5: return value: 0 = None, 1 = Some; other active flags are errors; only when option4 is present. */
/* Field return.text: *out2, *out3. */
/* Option option6: *out2: 0 = None, 1 = Some; other active flags are errors; only when option4 is present. */
/* Field return.letter: *out4. */
/* Option option7: *out4 == 1114112 means None; other values encode Some and must be valid payloads; only when option4 is present. */
int32_t perftest_optional_fields_echo(int32_t p0_w4, int64_t p0_w6, int32_t p0_w8, perftest_buffer p0_w10, uint32_t p0_w14, int64_t *out1, int32_t *out2, perftest_buffer *out3, uint32_t *out4);
uintptr_t perftest_owned_resource_drops(void);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
/* Result: success field v0 is returned directly; NULL means failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
perftest_getter_owner * perftest_owner_view_checked(uint8_t p0_w1, const perftest_getter_child * *out1, int64_t *out2, char * *error);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
perftest_getter_owner * perftest_owner_view_from_temporary(perftest_buffer p0_w1, const perftest_getter_child * *out1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
/* Option option0: return value == NULL means None; non-null means Some. */
perftest_getter_owner * perftest_owner_view_maybe(uint8_t p0_w1, const perftest_getter_child * *out1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
perftest_getter_owner * perftest_owner_view_mutable(uint8_t p0_w1, perftest_getter_child * *out1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
/* Option option0: *out1 == NULL means None; non-null means Some. */
perftest_getter_owner * perftest_owner_view_optional(uint8_t p0_w1, const perftest_getter_child * *out1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
/* Option option0: *out1 == NULL means None; non-null means Some. */
perftest_getter_owner * perftest_owner_view_optional_mut(uint8_t p0_w1, perftest_getter_child * *out1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
const perftest_getter_child * perftest_owner_view_reordered(uint8_t p0_w1, perftest_getter_owner * *out1);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
/* Result: success field v0 is returned directly; NULL means failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
perftest_getter_owner * perftest_owner_view_rollback(const perftest_getter_child * *out1, perftest_owned_resource * *out2, char * *error);
/* Borrowed opaque outputs must never be freed or consumed. Keep their source storage alive at a stable address; mutable borrows require exclusive access. */
perftest_getter_owner * perftest_owner_view_shared(uint8_t p0_w1, const perftest_getter_child * *out1);
/* Field return.id: return value. */
/* Field return.seq: *out1. */
/* Field return.value: *out2. */
/* Field return.flag: *out3. */
int64_t perftest_payload_new(int64_t p0_w1, int32_t p1_w1, double p2_w1, uint8_t p3_w1, int32_t *out1, double *out2, uint8_t *out3);
perftest_priority perftest_payload_priority(const perftest_enum_payload * p0_w1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_plain_note_echo(int32_t p0_w1, perftest_buffer p0_w3, perftest_buffer *out1);
uint8_t perftest_primitive_bool(uint8_t p0_w1);
uint32_t perftest_primitive_char(uint32_t p0_w1);
float perftest_primitive_f32(float p0_w1);
double perftest_primitive_f64(double p0_w1);
perftest_int128 perftest_primitive_i128(perftest_int128 p0_w1);
int16_t perftest_primitive_i16(int16_t p0_w1);
int32_t perftest_primitive_i32(int32_t p0_w1);
int64_t perftest_primitive_i64(int64_t p0_w1);
int8_t perftest_primitive_i8(int8_t p0_w1);
intptr_t perftest_primitive_isize(intptr_t p0_w1);
perftest_int128 perftest_primitive_u128(perftest_int128 p0_w1);
uint16_t perftest_primitive_u16(uint16_t p0_w1);
uint32_t perftest_primitive_u32(uint32_t p0_w1);
uint64_t perftest_primitive_u64(uint64_t p0_w1);
uint8_t perftest_primitive_u8(uint8_t p0_w1);
void perftest_primitive_unit(void);
uintptr_t perftest_primitive_usize(uintptr_t p0_w1);
/* Option option0: p0_w3 == -1 means None; other values encode Some and must be valid payloads. */
perftest_priority perftest_priority_or(perftest_priority p0_w3, perftest_priority p1_w1);
int32_t perftest_priority_weight(perftest_priority p0_w1);
perftest_report_struct * perftest_report_into_struct(int32_t p0_w1, perftest_report * p0_w3, int64_t p0_w6, double p0_w8, perftest_buffer p0_w10);
perftest_buffer perftest_report_label(int32_t p0_w2, int64_t p0_w5, double p0_w7, perftest_summary * p0_w9, perftest_buffer p0_w11);
/* Field return.summary: return value, *out1. */
/* Field return.origin: *out2, *out3. */
/* Field return.label: *out4. */
int64_t perftest_report_new(int64_t p0_w1, double p1_w1, perftest_buffer p2_w1, double *out1, int64_t *out2, int64_t *out3, perftest_buffer *out4, perftest_report * *out5);
/* Option option0: p0_w4 == NULL means None; non-null means Some. */
int64_t perftest_resource_choice_id(perftest_owned_resource * p0_w4, int64_t p0_w6);
uintptr_t perftest_result_error_drops(void);
/* Option option0: the return value == -3 means None; other values encode Some and must be valid payloads. */
/* Result: success field v0 is returned directly; -4 means failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
perftest_code perftest_result_optional(int32_t p0_w1, char * *error);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_result_other(uint8_t p0_w1, int64_t *out0, char * *error);
/* Result: success field v0 is returned directly; NULL means failure. */
/* ownership is required: false before acceptance; true after active owned inputs transfer, including on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
perftest_owned_resource * perftest_result_owned(perftest_owned_resource * p0_w1, int32_t p1_w1, uint32_t p2_w1, bool *ownership, char * *error);
/* Field return.priority: the return value. */
/* Field return.code: *out1. */
/* Option option0: *out1 == -3 means None; other values encode Some and must be valid payloads. */
/* Result: success field v0 is returned directly; -1 means failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
perftest_priority perftest_result_record(uint8_t p0_w1, perftest_code *out1, char * *error);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_result_scalar(int64_t p0_w1, int64_t *out0, char * *error);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_result_text(uint8_t p0_w1, perftest_buffer *out0, char * *error);
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_result_unit(uint8_t p0_w1, char * *error);
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
/* Callback sequence return: ret_w3 reports the delivered element count; the visitor receives 1 fields per element. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_sequence_late_drop_each(void (*callback)(void *context, perftest_owned_resource * v0), void *context, uintptr_t *out0, char * *error);
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
/* Callback sequence return: ret_w2 reports the delivered element count; the visitor receives 3 fields per element. */
/* Field return[].number: row.v0. */
/* Field return[].text: row.v1. */
/* Field return[].resource: row.v2. */
/* Result: true = success; false = failure. Success outputs are unchanged on failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
bool perftest_sequence_parts_each(int32_t p0_w1, void (*callback)(void *context, int64_t v0, perftest_buffer v1, perftest_owned_resource * v2), void *context, uintptr_t *out0, char * *error);
/* Callback sequence return: ret_w0 reports the delivered element count; the visitor receives 10 fields per element. */
/* Field return[].flag: row.v0. */
/* Field return[].byte: row.v1. */
/* Field return[].short: row.v2. */
/* Field return[].number: row.v3. */
/* Field return[].long: row.v4. */
/* Field return[].float: row.v5. */
/* Field return[].double: row.v6. */
/* Field return[].wide: row.v7. */
/* Field return[].text: row.v8, row.v9. */
/* Option option0: row.v8: 0 = None, 1 = Some; other active flags are errors. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
uintptr_t perftest_sequence_scalars_each(void (*callback)(void *context, uint8_t v0, int8_t v1, int16_t v2, int32_t v3, int64_t v4, float v5, double v6, perftest_int128 v7, int32_t v8, perftest_buffer v9), void *context);
perftest_code perftest_sparse_borrowed(perftest_code p0_w1);
perftest_code perftest_sparse_echo(perftest_code p0_w1);
/* Option option0: p0_w5 == -4 means None; other values encode Some and must be valid payloads. */
/* Option option1: p0_w5 == -3 means None; other values encode Some and must be valid payloads; only when option0 is present. */
/* Option option2: return value == -4 means None; other values encode Some and must be valid payloads. */
/* Option option3: return value == -3 means None; other values encode Some and must be valid payloads; only when option2 is present. */
perftest_code perftest_sparse_nested(perftest_code p0_w5);
/* Option option0: p0_w3 == -3 means None; other values encode Some and must be valid payloads. */
/* Option option1: return value == -3 means None; other values encode Some and must be valid payloads. */
perftest_code perftest_sparse_optional(perftest_code p0_w3);
int32_t perftest_sparse_ref(perftest_code p0_w1);
int64_t perftest_stamp_new(int64_t p0_w1, int64_t p1_w1, int64_t *out1);
uint8_t perftest_storage_contains(const perftest_storage * p0_w1, int64_t p1_w1);
/* Option option0: return value: 0 = None, 1 = Some; other active flags are errors. */
/* Field return.id: *out1. */
/* Field return.seq: *out2. */
/* Field return.value: *out3. */
/* Field return.flag: *out4. */
int32_t perftest_storage_get(const perftest_storage * p0_w1, int64_t *out1, int32_t *out2, double *out3, uint8_t *out4);
typedef struct {
    int64_t v0;
    int32_t v1;
    double v2;
    uint8_t v3;
} perftest_storage_get_vec_ret_w2_row;
typedef struct { perftest_storage_get_vec_ret_w2_row *data; size_t len; } perftest_storage_get_vec_ret_w2_rows;
/* Release only the row allocation. Free or transfer owned element fields separately. Only pass buffers returned by this library. */
void perftest_storage_get_vec_ret_w2_rows_free(perftest_storage_get_vec_ret_w2_rows rows);
/* Option option0: return value: 0 = None, 1 = Some; other active flags are errors. */
/* Collection return: ret_w2 carries rows of 4 ABI fields; element-owned fields are released separately from row storage. */
/* Field return[].id: row.v0. */
/* Field return[].seq: row.v1. */
/* Field return[].value: row.v2. */
/* Field return[].flag: row.v3. */
int32_t perftest_storage_get_vec(const perftest_storage * p0_w1, perftest_storage_get_vec_ret_w2_rows *out1);
/* Option option0: return value: 0 = None, 1 = Some; other active flags are errors. */
/* Callback sequence return: ret_w2 reports the delivered element count; the visitor receives 4 fields per element. */
/* Field return[].id: row.v0. */
/* Field return[].seq: row.v1. */
/* Field return[].value: row.v2. */
/* Field return[].flag: row.v3. */
/* Each callback invocation transfers ownership of its element fields. Earlier deliveries remain caller-owned on later failure. Moved inputs already belong to Rust during callbacks; the ownership output is written on return. Borrowed inputs must stay live throughout callbacks; callbacks must not unwind or longjmp. */
int32_t perftest_storage_get_vec_each(const perftest_storage * p0_w1, void (*callback)(void *context, int64_t v0, int32_t v1, double v2, uint8_t v3), void *context, uintptr_t *out1);
int64_t perftest_storage_len(const perftest_storage * p0_w1);
perftest_storage * perftest_storage_new(void);
void perftest_storage_put_by_take(perftest_storage * p0_w1, int64_t p1_w2, int32_t p1_w4, double p1_w6, uint8_t p1_w8);
/* Option option0: p1_w1: 0 = None, 1 = Some; other active flags are errors. */
uint8_t perftest_storage_put_opt(perftest_storage * p0_w1, int32_t p1_w1, int64_t p1_w4, int32_t p1_w6, double p1_w8, uint8_t p1_w10);
typedef struct {
    int64_t v0;
    int32_t v1;
    double v2;
    uint8_t v3;
} perftest_storage_put_slice_p1_w1_row;
typedef struct { perftest_storage_put_slice_p1_w1_row *data; size_t len; } perftest_storage_put_slice_p1_w1_rows;
/* Collection payloads: p1_w1 carries rows of 4 ABI fields; element-owned fields are released separately from row storage. */
void perftest_storage_put_slice(perftest_storage * p0_w1, perftest_storage_put_slice_p1_w1_rows p1_w1);
perftest_summary * perftest_storage_summary_handle(const perftest_storage * p0_w1);
int64_t perftest_storage_total_len(const perftest_storage * p0_w1, const perftest_storage * p1_w1, const perftest_storage * p2_w1);
/* Result: success field v0 is returned directly; NULL means failure. */
/* error may be NULL to discard details. Failure initializes nonnull error with caller-owned data (or an empty record if error conversion fails). Success leaves error unchanged. Free a prior error before reusing its storage. All output/ownership/error storage must be disjoint. */
perftest_storage * perftest_storage_try_with_label(perftest_buffer p0_w1, char * *error);
perftest_storage * perftest_storage_with_payload(int64_t p0_w2, int32_t p0_w4, double p0_w6, uint8_t p0_w8);
uintptr_t perftest_string_len(perftest_buffer p0_w1);
perftest_buffer perftest_string_new(perftest_buffer p0_w1);
int64_t perftest_summary_merge(int32_t p0_w1, int64_t p0_w4, double p0_w6, perftest_summary * p0_w8, int32_t p1_w1, int64_t p1_w4, double p1_w6, perftest_summary * p1_w8, double *out1);
int64_t perftest_summary_new(int64_t p0_w1, double p1_w1, double *out1);
double perftest_summary_total(const perftest_summary * p0_w1);
double perftest_summary_total_raw(int32_t p0_w1, int64_t p0_w4, double p0_w6, perftest_summary * p0_w8);
/* Option option0: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_unsigned_data_maybe(const perftest_unsigned * p0_w1, uint64_t *out1);
/* Option option0: p0_w1: 0 = None, 1 = Some; other active flags are errors. */
/* Option option1: return value: 0 = None, 1 = Some; other active flags are errors. */
int32_t perftest_unsigned_optional(int32_t p0_w1, uint64_t p0_w3, uint64_t *out1);
/* Option option0: p4_w1: 0 = None, 1 = Some; other active flags are errors. */
perftest_unsigned * perftest_unsigned_round_trip(uint8_t p0_w1, uint16_t p1_w1, uint32_t p2_w1, uint64_t p3_w1, int32_t p4_w1, uint64_t p4_w3);
#ifdef __cplusplus
}
#endif
#endif
