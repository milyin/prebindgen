/* Runs through generated C callbacks and the Rust source functions. */
#include <stdlib.h>
#include <stdatomic.h>

struct returned_callback_state { _Atomic size_t calls, drops, releases; };
static void returned_context_drop(void *context) { atomic_fetch_add(&((struct returned_callback_state *)context)->drops, 1); }
static int64_t returned_number(void *context, int64_t value) { atomic_fetch_add(&((struct returned_callback_state *)context)->calls, 1); return value * 2; }
static int8_t returned_i8(void *context, int8_t value) {(void)context;return value+1;}
static int16_t returned_i16(void *context, int16_t value) {(void)context;return value+1;}
static float returned_f32(void *context, float value) {(void)context;return value*2;}
static int64_t returned_zero(void *context) { (void)context; return 42; }
static uint8_t returned_bool(void *context, uint8_t value) { (void)context; return !value; }
static uint32_t returned_char(void *context, uint32_t value) { (void)context; return value + 1; }
static uint32_t returned_char_failure(void *context) { return atomic_fetch_add(&((struct returned_callback_state *)context)->calls, 1) == 0 ? 0xd800 : 'c'; }
static double returned_float(void *context, double value) { (void)context; return value * 2; }
static uint64_t returned_unsigned(void *context, uint64_t value) { (void)context; return value - 1; }
static perftest_int128 returned_wide(void *context, perftest_int128 value) { (void)context; value.lo--; return value; }
static perftest_callback_return_choice returned_enum(void *context, perftest_callback_return_choice value) { (void)context; assert(value == perftest_callback_return_choice_First); return perftest_callback_return_choice_Second; }
static perftest_buffer returned_text(void *context, perftest_buffer value) {
    (void)context;
    assert(value.len == sizeof("héllo\0世界") - 1);
    perftest_free_buffer(value);
    const char text[] = "returned\0answer";
    uint8_t *data = malloc(sizeof(text)-1); assert(data);
    memcpy(data, text, sizeof(text)-1);
    return (perftest_buffer){data, sizeof(text)-1};
}
static void returned_buffer_release(void *context, const perftest_buffer *value) { atomic_fetch_add(&((struct returned_callback_state *)context)->releases, 1); free((void *)value->data); }
static perftest_buffer returned_bytes(void *context, perftest_buffer value) {
    (void)context;
    assert(value.len == 3 && value.data[1] == 128 && value.data[2] == 255);
    perftest_free_buffer(value);
    static const uint8_t data[] = {255,128,0};
    return (perftest_buffer){data,3};
}
static perftest_callback_return_optional_callback_return returned_optional(void *context, int32_t present, int64_t value) { (void)context; return (perftest_callback_return_optional_callback_return){present, present ? value * 2 : 0}; }
static perftest_callback_return_vector_callback_return returned_vector(void *context, perftest_callback_return_vector_callback_arg0 value) {
    (void)context; assert(value.len == 3 && value.data[2].v0 == 3);
    perftest_callback_return_vector_callback_arg0_free(value);
    static perftest_callback_return_vector_callback_return_row rows[] = {{2},{4},{6}};
    return (perftest_callback_return_vector_callback_return){rows,3};
}
static perftest_callback_return_record_callback_return returned_record(void *context, int64_t number, perftest_buffer label) {
    (void)context; assert(number == 21 && label.len == 3); perftest_free_buffer(label);
    static const uint8_t text[] = "answer";
    return (perftest_callback_return_record_callback_return){42,{text,6}};
}
static perftest_callback_return_resource *returned_owner(void *context) { (void)context; return perftest_callback_return_resource_new(71); }
static perftest_callback_return_resource *returned_absent_owner(void *context) { (void)context; return NULL; }
static perftest_callback_return_owned_vector_catch_callback_return returned_owner_vector(void *context) {
    struct returned_callback_state *state = context;
    size_t call = atomic_fetch_add(&state->calls, 1);
    assert(perftest_callback_return_resource_live() == 0);
    size_t count = call == 0 ? 3 : 1;
    perftest_callback_return_owned_vector_catch_callback_return_row *rows = calloc(count, sizeof(*rows)); assert(rows);
    rows[0].v0 = perftest_callback_return_resource_new(81);
    if (call == 0) { rows[1].v0 = rows[0].v0; rows[2].v0 = perftest_callback_return_resource_new(82); }
    return (perftest_callback_return_owned_vector_catch_callback_return){rows,count};
}
static void returned_owner_vector_release(void *context, const perftest_callback_return_owned_vector_catch_callback_return *value) { atomic_fetch_add(&((struct returned_callback_state *)context)->releases,1); free(value->data); }
static perftest_callback_return_pair_catch_callback_return returned_pair(void *context) {
    size_t call = atomic_fetch_add(&((struct returned_callback_state *)context)->calls,1);
    assert(perftest_callback_return_resource_live() == 0);
    return (perftest_callback_return_pair_catch_callback_return){perftest_callback_return_resource_new(74),(uint32_t)(call == 0 ? 0xd800 : 'c')};
}
struct borrowed_return_state {perftest_callback_return_resource *value;size_t calls;};
static perftest_callback_return_borrowed_pair_catch_callback_return returned_borrowed_pair(void *context) {
    struct borrowed_return_state *state=context;
    assert(perftest_callback_return_resource_live()==1 && perftest_callback_return_resource_id(state->value)==71);
    return (perftest_callback_return_borrowed_pair_catch_callback_return){state->value,(uint32_t)(state->calls++==0?0xd800:'c')};
}
static uint32_t returned_invalid_char(void *context,uint32_t value) {(void)context;(void)value;return 0xd800;}
static void returned_unit(void *context, int64_t value) { assert(value == 21); atomic_fetch_add(&((struct returned_callback_state *)context)->calls,1); }

static void check_callback_returns(void) {
    struct returned_callback_state state = {0};
    assert(perftest_callback_return_i64((perftest_callback_return_i64_callback){.run=returned_number,.context=&state,.drop=returned_context_drop}) == 42);
    assert(state.calls == 1 && state.drops == 1);
    assert(perftest_callback_return_i8((perftest_callback_return_i8_callback){.run=returned_i8})==-7);
    assert(perftest_callback_return_i16((perftest_callback_return_i16_callback){.run=returned_i16})==-1599);
    assert(perftest_callback_return_f32((perftest_callback_return_f32_callback){.run=returned_f32})==2.5f);
    perftest_int128 unsigned_wide=perftest_callback_return_u128((perftest_callback_return_u128_callback){.run=returned_wide});
    assert(unsigned_wide.hi==UINT64_MAX && unsigned_wide.lo==UINT64_MAX-1);
    assert(perftest_callback_return_zero((perftest_callback_return_zero_callback){.run=returned_zero}) == 42);
    assert(!perftest_callback_return_bool((perftest_callback_return_bool_callback){.run=returned_bool}));
    assert(perftest_callback_return_char((perftest_callback_return_char_callback){.run=returned_char}) == 0x1f981);
    assert(perftest_callback_return_float((perftest_callback_return_float_callback){.run=returned_float}) == 2.5);
    assert(perftest_callback_return_unsigned((perftest_callback_return_unsigned_callback){.run=returned_unsigned}) == UINT64_MAX - 1);
    perftest_int128 wide = perftest_callback_return_wide((perftest_callback_return_wide_callback){.run=returned_wide});
    assert(wide.hi == INT64_MAX && wide.lo == UINT64_MAX - 1);
    assert(perftest_callback_return_enum((perftest_callback_return_enum_callback){.run=returned_enum}));
    perftest_buffer text = perftest_callback_return_text((perftest_callback_return_text_callback){.run=returned_text,.context=&state,.release_result=returned_buffer_release});
    assert(text.len == sizeof("returned\0answer")-1 && state.releases == 1); perftest_free_buffer(text);
    perftest_buffer bytes = perftest_callback_return_bytes((perftest_callback_return_bytes_callback){.run=returned_bytes});
    assert(bytes.len == 3 && bytes.data[0] == 255 && bytes.data[2] == 0); perftest_free_buffer(bytes);
    int64_t number = 0;
    perftest_callback_return_optional_callback optional = {.run=returned_optional};
    assert(perftest_callback_return_optional(optional,1,&number) == 1 && number == 42);
    assert(perftest_callback_return_optional(optional,0,&number) == 0);
    perftest_callback_return_vector_ret_w0_rows vector = perftest_callback_return_vector((perftest_callback_return_vector_callback){.run=returned_vector});
    assert(vector.len == 3 && vector.data[2].v0 == 6); perftest_callback_return_vector_ret_w0_rows_free(vector);
    assert(perftest_callback_return_record((perftest_callback_return_record_callback){.run=returned_record}) == 42);
    assert(perftest_callback_return_owned((perftest_callback_return_owned_callback){.run=returned_owner}) == 71 && perftest_callback_return_resource_live() == 0);
    assert(perftest_callback_return_owned_optional((perftest_callback_return_owned_optional_callback){.run=returned_absent_owner},&number) == 0);
    assert(perftest_callback_return_owned_optional((perftest_callback_return_owned_optional_callback){.run=returned_owner},&number) == 1 && number == 71);
    struct returned_callback_state vector_state = {0};
    assert(perftest_callback_return_owned_vector_catch((perftest_callback_return_owned_vector_catch_callback){.run=returned_owner_vector,.context=&vector_state,.release_result=returned_owner_vector_release}) == 81);
    assert(vector_state.calls == 2 && vector_state.releases == 2 && perftest_callback_return_resource_live() == 0);
    struct returned_callback_state pair_state = {0};
    assert(perftest_callback_return_pair_catch((perftest_callback_return_pair_catch_callback){.run=returned_pair,.context=&pair_state}) == 74);
    assert(pair_state.calls == 2 && perftest_callback_return_resource_live() == 0);
    struct returned_callback_state char_state = {0};
    assert(perftest_callback_return_char_catch((perftest_callback_return_char_catch_callback){.run=returned_char_failure,.context=&char_state}) == 'c' && char_state.calls == 2);
    struct returned_callback_state worker = {0};
    perftest_callback_return_job *job = perftest_callback_return_spawn((perftest_callback_return_spawn_callback){.run=returned_number,.context=&worker,.drop=returned_context_drop});
    assert(perftest_callback_return_join(job) == 19800 && worker.calls == 200 && worker.drops == 1);
    struct returned_callback_state retained = {0}; bool accepted = false; char *error = NULL;
    perftest_callback_return_handler_new_callback callback = {.run=returned_number,.context=&retained,.drop=returned_context_drop};
    perftest_callback_return_handler *handler = perftest_callback_return_handler_new(callback,0,&accepted,&error);
    assert(handler && accepted && !error && retained.drops == 0);
    assert(perftest_callback_return_handler_call(handler,21) == 42);
    perftest_drop_callback_return_handler(handler); assert(retained.drops == 1);
    accepted = false;
    assert(!perftest_callback_return_handler_new(callback,1,&accepted,&error) && accepted && error && retained.drops == 2);
    perftest_free_error(error); error = NULL; accepted = true; callback.run = NULL;
    assert(!perftest_callback_return_handler_new(callback,0,&accepted,&error) && !accepted && error && retained.drops == 2);
    perftest_free_error(error);
    struct returned_callback_state constructed = {0};
    assert(perftest_callback_return_handler_value((perftest_callback_return_handler_value_handler){.run=returned_number,.context=&constructed,.drop=returned_context_drop}) == 42);
    assert(constructed.calls == 1 && constructed.drops == 1);
    struct returned_callback_state many = {0};
    perftest_callback_return_handler_many_p0_w1_row handlers[] = {
        {{.run=returned_number,.context=&many,.drop=returned_context_drop}},
        {{.run=returned_number,.context=&many,.drop=returned_context_drop}}
    };
    assert(perftest_callback_return_handler_many((perftest_callback_return_handler_many_p0_w1_rows){handlers,2})==84);
    assert(many.calls==2 && many.drops==2);
    struct borrowed_return_state borrowed = {.value=perftest_callback_return_resource_new(71)};
    assert(perftest_callback_return_borrowed_pair_catch((perftest_callback_return_borrowed_pair_catch_callback){.run=returned_borrowed_pair,.context=&borrowed})==72);
    assert(borrowed.calls==2 && perftest_callback_return_resource_id(borrowed.value)==72 && perftest_callback_return_resource_live()==1);
    perftest_drop_callback_return_resource(borrowed.value);
    assert(perftest_callback_return_resource_live()==0);
    struct returned_callback_state failed = {0}; accepted = false; error = NULL;
    assert(perftest_callback_return_c_result((perftest_callback_return_c_result_callback){.run=returned_invalid_char,.context=&failed,.drop=returned_context_drop},&accepted,&error)==0x110000);
    assert(accepted && error && strstr(error,"panicked") && failed.drops==1);
    perftest_free_error(error);
    struct returned_callback_state unit = {0};
    perftest_callback_return_unit((perftest_callback_return_unit_callback){.run=returned_unit,.context=&unit,.drop=returned_context_drop});
    assert(unit.calls == 1 && unit.drops == 1);
    puts("C callback returns PASS: values, ownership, retained workers, constructors and invalid-result cleanup");
}
