#include "../../c_abort.h"
#include "bindings.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <math.h>
#include <limits.h>

struct sequence_acc { size_t calls; int64_t total; };
static void sequence_number(void *context, int64_t value) {
    struct sequence_acc *acc = context;
    acc->calls++; acc->total += value;
}
static void sequence_text(void *context, perftest_buffer value) {
    struct sequence_acc *acc = context;
    acc->calls++; acc->total += (int64_t)value.len;
    perftest_free_buffer(value);
}
static void sequence_nested(void *context, perftest_collection_nested_each_ret_w4_rows rows) {
    struct sequence_acc *acc = context;
    acc->calls++;
    for (size_t i=0; i<rows.len; ++i) acc->total += rows.data[i].v0;
    perftest_collection_nested_each_ret_w4_rows_free(rows);
}
static void sequence_part(void *context, int64_t number, perftest_buffer text, perftest_owned_resource *resource) {
    struct sequence_acc *acc = context;
    assert(number == perftest_option_resource_id(resource));
    assert(text.len > 6 && memcmp(text.data, "part ", 5) == 0);
    perftest_free_buffer(text);
    perftest_drop_owned_resource(resource);
    acc->calls++; acc->total += number;
}
static void sequence_resource(void *context, perftest_owned_resource *resource) {
    struct sequence_acc *acc = context;
    acc->calls++; acc->total += perftest_option_resource_id(resource);
    perftest_drop_owned_resource(resource);
}
static void sequence_scalar(void *context, uint8_t flag, int8_t byte, int16_t small, int32_t number, int64_t large, float single, double real, perftest_int128 wide, int32_t present, perftest_buffer text) {
    struct sequence_acc *acc = context;
    if (acc->calls++ == 0) {
        assert(flag == 1 && byte == -7 && small == -1234 && number == -123456);
        assert(large == INT64_MIN && single == 1.25f && real == -2.5);
        assert(wide.hi == UINT64_C(0x8000000000000000) && wide.lo == 0 && present == 1 && text.len == 6);
    } else {
        assert(flag == 0 && byte == 7 && small == 1234 && number == 123456);
        assert(large == INT64_MAX && single == -1.25f && real == 2.5);
        assert(wide.hi == UINT64_C(0x7fffffffffffffff) && wide.lo == UINT64_MAX && present == 0);
    }
    perftest_free_buffer(text);
}
static void check_sequence_delivery(void) {
    struct sequence_acc acc = {0};
    assert(perftest_sequence_scalars_each(sequence_scalar, &acc) == 2 && acc.calls == 2);
    acc = (struct sequence_acc){0};
    perftest_collection_i64_each_p0_w1_row values[] = {{1},{2},{3}};
    assert(perftest_collection_i64_each((perftest_collection_i64_each_p0_w1_rows){values,3}, sequence_number, &acc) == 3);
    assert(acc.calls == 3 && acc.total == 6);
    assert(perftest_collection_i64_each((perftest_collection_i64_each_p0_w1_rows){NULL,0}, sequence_number, &acc) == 0);
    assert(acc.calls == 3);
    EXPECT_ABORT(perftest_collection_i64_each(((perftest_collection_i64_each_p0_w1_rows){values,3}), NULL, &acc));
    perftest_collection_text_each_p0_w1_row text[] = {{{(const uint8_t *)"hello",5}},{{NULL,0}}};
    acc = (struct sequence_acc){0};
    assert(perftest_collection_text_each((perftest_collection_text_each_p0_w1_rows){text,2}, sequence_text, &acc) == 2);
    assert(acc.calls == 2 && acc.total == 5);
    perftest_collection_nested_each_p0_w5_row inner[] = {{7},{9}};
    perftest_collection_nested_each_p0_w1_row outer[] = {{{inner,2}},{{NULL,0}}};
    acc = (struct sequence_acc){0};
    assert(perftest_collection_nested_each((perftest_collection_nested_each_p0_w1_rows){outer,2}, sequence_nested, &acc) == 2);
    assert(acc.calls == 2 && acc.total == 16);
    uintptr_t count = 12345;
    acc = (struct sequence_acc){0};
    assert(perftest_collection_optional_each(0,(perftest_collection_optional_each_p0_w3_rows){NULL,0},sequence_number,&acc,&count) == 0);
    assert(count == 0 && acc.calls == 0);
    assert(perftest_collection_optional_each(1,(perftest_collection_optional_each_p0_w3_rows){NULL,0},sequence_number,&acc,&count) == 1);
    assert(count == 0 && acc.calls == 0);

    size_t start = perftest_owned_resource_drops();
    char *error = NULL;
    assert(perftest_sequence_parts_each(0,sequence_part,&acc,&count,&error));
    assert(count == 3 && acc.calls == 3 && acc.total == 6 && error == NULL);
    assert(perftest_owned_resource_drops() == start + 3);
    acc = (struct sequence_acc){0}; count = 12345;
    assert(!perftest_sequence_parts_each(1,sequence_part,&acc,&count,&error));
    assert(count == 12345 && acc.calls == 0 && error != NULL);
    perftest_free_error(error); error = NULL;
    assert(!perftest_sequence_parts_each(2,sequence_part,&acc,&count,&error));
    assert(count == 12345 && acc.calls == 1 && acc.total == 1 && error != NULL);
    perftest_free_error(error); error = NULL;
    assert(perftest_owned_resource_drops() == start + 6);
    perftest_collection_handles_each_p0_w1_row owned[] = {{perftest_collection_resource_new(42)}};
    bool accepted = true;
    assert(!perftest_collection_handles_each((perftest_collection_handles_each_p0_w1_rows){owned,1},65,0,NULL,&acc,&count,&accepted,&error));
    assert(!accepted && count == 12345 && error != NULL);
    perftest_free_error(error); error = NULL;
    acc = (struct sequence_acc){0};
    assert(perftest_collection_handles_each((perftest_collection_handles_each_p0_w1_rows){owned,1},65,0,sequence_resource,&acc,&count,&accepted,&error));
    assert(accepted && count == 1 && acc.total == 42 && error == NULL);
    assert(perftest_owned_resource_drops() == start + 7);
    acc = (struct sequence_acc){0}; count = 12345;
    assert(!perftest_sequence_late_drop_each(sequence_resource,&acc,&count,&error));
    assert(acc.calls == 2 && acc.total == 163 && count == 12345 && error != NULL);
    perftest_free_error(error);
    assert(perftest_owned_resource_drops() == start + 9);
    for (int mode=0; mode<3; ++mode) {
        perftest_collection_result_optional_each_p0_w1_row inputs[]={{perftest_collection_resource_new(101)},{perftest_collection_resource_new(102)}};
        acc=(struct sequence_acc){0}; count=12345; int32_t present=12345; accepted=false; error=NULL;
        bool ok=perftest_collection_result_optional_each((perftest_collection_result_optional_each_p0_w1_rows){inputs,2},mode,sequence_resource,&acc,&present,&count,&accepted,&error);
        assert(accepted);
        if (mode==2) {
            assert(!ok && present==12345 && count==12345 && error!=NULL && acc.calls==0);
            perftest_free_error(error);
        } else {
            assert(ok && error==NULL && present==mode && count==(mode==1 ? 2 : 0));
            assert(acc.calls==count);
        }
    }
    assert(perftest_owned_resource_drops() == start + 15);

}

static void check_collections(void) {
    int64_t tag = 0;
    perftest_collection_numbers_ret_w2_rows observed = perftest_collection_numbers(&tag);
    assert(tag == 42 && observed.len == 2 && observed.data[1].v0 == 9);
    perftest_collection_numbers_ret_w2_rows_free(observed);
    perftest_collection_i64_p0_w1_row values[] = {{INT64_MIN}, {0}, {INT64_MAX}};
    perftest_collection_i64_ret_w0_rows result = perftest_collection_i64((perftest_collection_i64_p0_w1_rows){values, 3});
    assert(result.len == 3 && result.data[0].v0 == INT64_MIN && result.data[2].v0 == INT64_MAX);
    perftest_collection_i64_ret_w0_rows_free(result);
    result = perftest_collection_i64((perftest_collection_i64_p0_w1_rows){NULL, 0});
    assert(result.len == 0);
    perftest_collection_i64_ret_w0_rows_free(result);
    EXPECT_ABORT(perftest_collection_i64(((perftest_collection_i64_p0_w1_rows){NULL, 1})));
    EXPECT_ABORT(perftest_collection_i64(((perftest_collection_i64_p0_w1_rows){(void *)1, 1})));
    EXPECT_ABORT(perftest_collection_i64(((perftest_collection_i64_p0_w1_rows){values, SIZE_MAX})));
    perftest_collection_text_p0_w1_row texts[] = {{{(const uint8_t *)"hello", 5}}, {{NULL, 0}}};
    perftest_collection_text_ret_w0_rows text = perftest_collection_text((perftest_collection_text_p0_w1_rows){texts, 2});
    assert(text.len == 2 && text.data[0].v0.len == 5 && memcmp(text.data[0].v0.data, "hello", 5) == 0);
    for (size_t i = 0; i < text.len; ++i) perftest_free_buffer(text.data[i].v0);
    perftest_collection_text_ret_w0_rows_free(text);
    const uint8_t bad[] = {0xff}; texts[1].v0 = (perftest_buffer){bad, 1};
    EXPECT_ABORT(perftest_collection_text(((perftest_collection_text_p0_w1_rows){texts, 2})));
    perftest_collection_nested_p0_w5_row inner[] = {{7}, {9}};
    perftest_collection_nested_p0_w1_row outer[] = {{{inner, 2}}, {{NULL, 0}}};
    perftest_collection_nested_ret_w0_rows nested = perftest_collection_nested((perftest_collection_nested_p0_w1_rows){outer, 2});
    assert(nested.len == 2 && nested.data[0].v0.len == 2 && nested.data[0].v0.data[1].v0 == 9 && nested.data[1].v0.len == 0);
    for (size_t i = 0; i < nested.len; ++i) perftest_collection_nested_ret_w4_rows_free(nested.data[i].v0);
    perftest_collection_nested_ret_w0_rows_free(nested);
    perftest_collection_options_p0_w1_row opts[] = {{0, INT64_MAX}, {1, INT64_MIN}};
    perftest_collection_options_ret_w0_rows options = perftest_collection_options((perftest_collection_options_p0_w1_rows){opts, 2}, 0);
    assert(options.data[0].v0 == 0 && options.data[1].v0 == 1 && options.data[1].v1 == INT64_MIN);
    perftest_collection_options_ret_w0_rows_free(options);
    opts[1].v0 = 2;
    EXPECT_ABORT(perftest_collection_options(((perftest_collection_options_p0_w1_rows){opts, 2}), 0));
    perftest_collection_optional_ret_w2_rows optional = {0};
    assert(perftest_collection_optional(0, (perftest_collection_optional_p0_w3_rows){(void *)1, SIZE_MAX}, &optional) == 0);
    assert(perftest_collection_optional(1, (perftest_collection_optional_p0_w3_rows){NULL, 0}, &optional) == 1 && optional.len == 0);
    perftest_collection_optional_ret_w2_rows_free(optional);

    size_t start = perftest_owned_resource_drops();
    perftest_owned_resource *a = perftest_collection_resource_new(1), *b = perftest_collection_resource_new(2);
    perftest_collection_handles_p0_w1_row handles[] = {{a}, {b}};
    perftest_collection_handles_ret_w2_rows moved = {NULL, 12345};
    bool ownership = true;
    char *error = NULL;
    assert(!perftest_collection_handles((perftest_collection_handles_p0_w1_rows){handles, 2}, 0xd800, 0, &moved, &ownership, &error));
    assert(!ownership && moved.len == 12345 && error != NULL);
    perftest_free_error(error); error = NULL;
    handles[1].v0 = a;
    assert(!perftest_collection_handles((perftest_collection_handles_p0_w1_rows){handles, 2}, 65, 0, &moved, &ownership, &error));
    assert(!ownership && moved.len == 12345 && error != NULL);
    perftest_free_error(error); error = NULL;
    handles[1].v0 = b;
    perftest_collection_alias_p0_w1_row aliases[] = {{a}};
    EXPECT_ABORT(perftest_collection_alias(((perftest_collection_alias_p0_w1_rows){aliases, 1}), a));
    perftest_collection_resource_ids_p0_w1_row shared[] = {{a}, {a}};
    perftest_collection_resource_ids_ret_w0_rows ids = perftest_collection_resource_ids((perftest_collection_resource_ids_p0_w1_rows){shared, 2});
    assert(ids.len == 2 && ids.data[0].v0 == 1 && ids.data[1].v0 == 1);
    perftest_collection_resource_ids_ret_w0_rows_free(ids);
    assert(perftest_collection_handles((perftest_collection_handles_p0_w1_rows){handles, 2}, 65, 0, &moved, &ownership, &error));
    assert(ownership && moved.len == 2 && error == NULL);
    for (size_t i = 0; i < moved.len; ++i) perftest_drop_owned_resource(moved.data[i].v0);
    perftest_collection_handles_ret_w2_rows_free(moved);
    moved = (perftest_collection_handles_ret_w2_rows){NULL, 12345};
    for (int mode = 1; mode <= 2; ++mode) {
        handles[0].v0 = perftest_collection_resource_new(3); handles[1].v0 = perftest_collection_resource_new(4);
        assert(!perftest_collection_handles((perftest_collection_handles_p0_w1_rows){handles, 2}, 65, mode, &moved, &ownership, &error));
        assert(ownership && moved.len == 12345 && error != NULL);
        perftest_free_error(error); error = NULL;
    }
    assert(perftest_owned_resource_drops() == start + 8);
    perftest_collection_try_construct_all_p0_w1_row pending[3];
    for (size_t i = 0; i < 3; ++i) pending[i].v0 = perftest_collection_resource_new((int64_t)i + 1);
    int64_t constructed = 12345;
    assert(!perftest_collection_try_construct_all((perftest_collection_try_construct_all_p0_w1_rows){pending, 3}, &constructed, &ownership, &error));
    assert(ownership && constructed == 12345 && error != NULL);
    perftest_free_error(error); error = NULL;
    assert(perftest_owned_resource_drops() == start + 11);
    perftest_collection_try_make_items_ret_w2_rows projected = {NULL, 12345};
    assert(!perftest_collection_try_make_items(&projected, &error));
    assert(projected.len == 12345 && error != NULL);
    perftest_free_error(error); error = NULL;
    assert(perftest_owned_resource_drops() == start + 14);
    perftest_collection_optional_handles_ret_w2_rows maybe_handles = {0};
    assert(perftest_collection_optional_handles(0, (perftest_collection_optional_handles_p0_w3_rows){NULL, 0}, &maybe_handles) == 0);
    perftest_collection_optional_handles_p0_w3_row maybe_input[] = {{perftest_collection_resource_new(5)}};
    assert(perftest_collection_optional_handles(1, (perftest_collection_optional_handles_p0_w3_rows){maybe_input, 1}, &maybe_handles) == 1 && maybe_handles.len == 1);
    perftest_drop_owned_resource(maybe_handles.data[0].v0);
    perftest_collection_optional_handles_ret_w2_rows_free(maybe_handles);
    perftest_collection_nested_handles_p0_w5_row nested_input[] = {{perftest_collection_resource_new(6)}};
    perftest_collection_nested_handles_p0_w1_row nested_rows[] = {{{NULL, 0}}, {{nested_input, 1}}};
    perftest_collection_nested_handles_ret_w0_rows nested_handles = perftest_collection_nested_handles((perftest_collection_nested_handles_p0_w1_rows){nested_rows, 2});
    assert(nested_handles.len == 2 && nested_handles.data[0].v0.len == 0 && nested_handles.data[1].v0.len == 1);
    perftest_drop_owned_resource(nested_handles.data[1].v0.data[0].v0);
    for (size_t i = 0; i < 2; ++i) perftest_collection_nested_handles_ret_w4_rows_free(nested_handles.data[i].v0);
    perftest_collection_nested_handles_ret_w0_rows_free(nested_handles);
    perftest_collection_bundle_ret_w2_rows bundle = perftest_collection_bundle(&tag);
    assert(bundle.len == 3 && tag == 42);
    for (size_t i = 0; i < 3; ++i) perftest_drop_owned_resource(bundle.data[i].v0);
    perftest_collection_bundle_ret_w2_rows_free(bundle);
    assert(perftest_owned_resource_drops() == start + 19);

    perftest_storage *storage = perftest_storage_new();
    perftest_storage_get_vec_ret_w2_rows stored = {0};
    assert(perftest_storage_get_vec(storage, &stored) == 0);
    perftest_storage_put_slice_p1_w1_row payloads[] = {{5, 2, 1.5, 1}, {9, -1, -2.0, 0}};
    perftest_storage_put_slice(storage, (perftest_storage_put_slice_p1_w1_rows){payloads, 2});
    assert(perftest_storage_get_vec(storage, &stored) == 1 && stored.len == 2 && stored.data[0].v0 == 5 && stored.data[1].v2 == -2.0);
    perftest_storage_get_vec_ret_w2_rows_free(stored);
    perftest_drop_storage(storage);

    perftest_collection_error typed;
    assert(!perftest_collection_fail(&typed));
    assert(typed.resources.len == 2 && typed.codes.len == 2);
    assert(typed.codes.data[0].v0.len == 2 && typed.codes.data[0].v0.data[1].v0 == 9);
    perftest_drop_collection_error(&typed);
    assert(typed.resources.data == NULL && typed.codes.data == NULL);
    perftest_drop_collection_error(&typed);
    assert(perftest_owned_resource_drops() == start + 21);
    assert(!perftest_collection_fail(NULL));
    assert(perftest_owned_resource_drops() == start + 23);
    perftest_collection_late_drop_ret_w3_rows unpublished = {NULL, 12345};
    perftest_result_error late_error;
    assert(!perftest_collection_late_drop(&unpublished, &late_error));
    assert(unpublished.len == 12345);
    assert(perftest_owned_resource_drops() == start + 25);
    perftest_drop_result_error(&late_error);
}

static void check_enums(void) {
    perftest_enum_payload * payload = {0};
    perftest_priority classified = {0};
    payload = perftest_enum_payload_new(50.0);
    classified = perftest_payload_priority(payload); assert(classified == perftest_priority_High);
    perftest_drop_enum_payload(payload);
    int32_t weight = {0};
    weight = perftest_priority_weight(perftest_priority_Low); assert(weight == 1);
    weight = perftest_priority_weight(perftest_priority_Normal); assert(weight == 5);
    weight = perftest_priority_weight(perftest_priority_High); assert(weight == 10);
    EXPECT_ABORT(perftest_priority_weight(3));
    perftest_priority priority = {0};
    priority = perftest_priority_or(-1, perftest_priority_High); assert(priority == perftest_priority_High);
    priority = perftest_priority_or(perftest_priority_Low, perftest_priority_High); assert(priority == perftest_priority_Low);
    const perftest_code codes[] = { perftest_code_Bottom, perftest_code_MinusTwo, perftest_code_MinusOne,
        perftest_code_Zero, perftest_code_Seven, perftest_code_Eight, perftest_code_Top };
    assert(perftest_code_Bottom == INT32_MIN && perftest_code_Top == INT32_MAX);
    assert(perftest_code_MinusOne == -1 && perftest_code_Eight == 8);
    perftest_code echoed = {0};
    perftest_code optional = {0};
    perftest_code nested = {0};
    int32_t borrowed = {0};
    perftest_code returned = {0};
    struct { perftest_priority v0; perftest_code v1; } record = {0};
    for (size_t i = 0; i < sizeof(codes) / sizeof(*codes); ++i) {
        perftest_code code = codes[i];
        echoed = perftest_sparse_echo(code); assert(echoed == code);
        optional = perftest_sparse_optional(code); assert(optional == code);
        nested = perftest_sparse_nested(code); assert(nested == code);
        borrowed = perftest_sparse_ref(code); assert(borrowed == code);
        returned = perftest_sparse_borrowed(code); assert(returned == code);
        record.v0 = perftest_enum_record_echo(perftest_priority_High, code, &record.v1); assert(record.v0 == perftest_priority_High && record.v1 == code);
    }
    record.v0 = perftest_enum_record_echo(perftest_priority_Low, -3, &record.v1); assert(record.v1 == -3);
    optional = perftest_sparse_optional(-3); assert(optional == -3);
    nested = perftest_sparse_nested(-3); assert(nested == -3);
    nested = perftest_sparse_nested(-4); assert(nested == -4);
    const int32_t invalid[] = {-5, -4, -3, 1, 6, 9};
    for (size_t i = 0; i < sizeof(invalid) / sizeof(*invalid); ++i) {
        echoed = 12345;
        EXPECT_ABORT(perftest_sparse_echo(invalid[i]));
    }
    optional = 12345; nested = 12345;
    EXPECT_ABORT(perftest_sparse_optional(-4));
    EXPECT_ABORT(perftest_sparse_nested(-5));
    perftest_enum_names names = {0};
    names = perftest_enum_names_echo(perftest_enum_names_type); assert(names == 14);
}
static void check_options(void) {
    struct { int32_t v0; perftest_buffer v1; } echoed = {0};
    echoed.v0 = perftest_option_str_echo(0, (perftest_buffer){NULL, 0}, &echoed.v1); assert(echoed.v0 == 0);
    echoed.v0 = perftest_option_str_echo(1, (perftest_buffer){(const uint8_t *)"hello", 5}, &echoed.v1); assert(echoed.v0 == 1 && echoed.v1.len == 5 && memcmp(echoed.v1.data, "hello", 5) == 0);
    perftest_free_buffer(echoed.v1);
    struct { int32_t v0; uintptr_t v1; } slen = {0};
    slen.v0 = perftest_option_str_len(0, (perftest_buffer){(const uint8_t *)1, SIZE_MAX}, &slen.v1); assert(slen.v0 == 0);
    slen.v0 = perftest_option_str_len(1, (perftest_buffer){(const uint8_t *)"hi", 2}, &slen.v1); assert(slen.v0 == 1 && slen.v1 == 2);
    int64_t ctor_id = {0};
    ctor_id = perftest_option_ctor_id(0, 0, 99, NULL); assert(ctor_id == -10);
    ctor_id = perftest_option_ctor_id(0, 1, 7, NULL); assert(ctor_id == 7);
    perftest_option_ctor * ctor = {0};
    ctor = perftest_option_ctor_new(0, 99);
    ctor_id = perftest_option_ctor_id(1, 2, 99, ctor); assert(ctor_id == -10);
    int64_t choice = {0};
    choice = perftest_option_choice_id(0, 8, 0, NULL); assert(choice == -1);
    choice = perftest_option_choice_id(1, 0, 8, NULL); assert(choice == 8);
    int64_t pair = {0};
    pair = perftest_option_resource_pair(NULL, NULL); assert(pair == 0);
    struct { int32_t v0; int64_t v1; } integer = {0};
    integer.v0 = perftest_option_i64(0, INT64_MAX, &integer.v1); assert(integer.v0 == 0);
    integer.v0 = perftest_option_i64(1, INT64_MIN, &integer.v1); assert(integer.v0 == 1 && integer.v1 == INT64_MIN);
    EXPECT_ABORT(perftest_option_i64(2, 0, &integer.v1));
    uint32_t ch = {0};
    ch = perftest_option_char(0x110000); assert(ch == 0x110000);
    ch = perftest_option_char(0x10ffff); assert(ch == 0x10ffff);
    EXPECT_ABORT(perftest_option_char(0xd800));
    EXPECT_ABORT(perftest_option_char(0x110001));
    int64_t niche_choice = {0};
    perftest_niche_choice * choice_handle = {0};
    niche_choice = perftest_niche_choice_id(0, 0, 0x110001, NULL); assert(niche_choice == -2);
    niche_choice = perftest_niche_choice_id(1, 0, 0x110000, NULL); assert(niche_choice == -1);
    niche_choice = perftest_niche_choice_id(1, 0, 65, NULL); assert(niche_choice == 65);
    choice_handle = perftest_niche_choice_new(66);
    niche_choice = perftest_niche_choice_id(1, 1, 0x110001, choice_handle); assert(niche_choice == 66);
    struct { int32_t v0; int32_t v1; uint32_t v2; } nested_unsigned = {0};
    nested_unsigned.v0 = perftest_option_nested_u32(0, 2, 0, &nested_unsigned.v1, &nested_unsigned.v2); assert(nested_unsigned.v0 == 0);
    nested_unsigned.v0 = perftest_option_nested_u32(1, 0, 0, &nested_unsigned.v1, &nested_unsigned.v2); assert(nested_unsigned.v0 == 1 && nested_unsigned.v1 == 0);
    nested_unsigned.v0 = perftest_option_nested_u32(1, 1, UINT32_MAX, &nested_unsigned.v1, &nested_unsigned.v2); assert(nested_unsigned.v0 == 1 && nested_unsigned.v1 == 1 && nested_unsigned.v2 == UINT32_MAX);
    uint32_t nested = {0};
    nested = perftest_option_nested_char(0x110001); assert(nested == 0x110001);
    nested = perftest_option_nested_char(0x110000); assert(nested == 0x110000);
    nested = perftest_option_nested_char(65); assert(nested == 65);
    EXPECT_ABORT(perftest_option_nested_char(0x110002));
    EXPECT_ABORT(perftest_option_nested_char(UINT32_MAX));
    struct { int32_t v0; int64_t v1; int32_t v2; perftest_buffer v3; uint32_t v4; } record = {0};
    record.v0 = perftest_optional_fields_echo(2, 99, 2, (perftest_buffer){(const uint8_t *)1, SIZE_MAX}, 0x110001, &record.v1, &record.v2, &record.v3, &record.v4); assert(record.v4 == 0x110001);
    record.v0 = perftest_optional_fields_echo(1, 99, 0, (perftest_buffer){NULL, 0}, 0x110000, &record.v1, &record.v2, &record.v3, &record.v4); assert(record.v0 == 1 && record.v1 == 99 && record.v2 == 0 && record.v4 == 0x110000);
    struct { int32_t v0; perftest_buffer v1; } text = {0};
    text.v0 = perftest_option_text(0, (perftest_buffer){(const uint8_t *)1, SIZE_MAX}, &text.v1); assert(text.v0 == 0);
    text.v0 = perftest_option_text(1, (perftest_buffer){NULL, 0}, &text.v1); assert(text.v0 == 1 && text.v1.len == 0);
    perftest_free_buffer(text.v1);
    int32_t unit = {0};
    unit = perftest_option_unit(0); assert(unit == 0);
    unit = perftest_option_unit(1); assert(unit == 1);
    uintptr_t drops = {0};
    drops = perftest_owned_resource_drops(); size_t start = drops;
    perftest_owned_resource * resource = {0};
    resource = perftest_option_resource_new(0); assert(resource == NULL);
    resource = perftest_option_resource_new(1); assert(resource != NULL);
    int64_t id = {0};
    id = perftest_option_resource_id(NULL); assert(id == -1);
    id = perftest_option_resource_id(resource); assert(id == 77);
    EXPECT_ABORT(perftest_option_resource_pair(resource, resource));
    id = perftest_option_resource_id(resource); assert(id == 77);
    perftest_owned_resource * moved = {0};
    EXPECT_ABORT(perftest_option_resource_echo(resource, 0xd800));
    id = perftest_option_resource_id(resource); assert(id == 77);
    moved = perftest_option_resource_echo(resource, 65);
    perftest_drop_owned_resource(moved);
    moved = perftest_option_resource_echo(NULL, 65); assert(moved == NULL);
    struct { int32_t v0; perftest_owned_resource * v1; } nested_resource = {0};
    nested_resource.v0 = perftest_option_nested_resource(0, NULL, &nested_resource.v1); assert(nested_resource.v0 == 0);
    nested_resource.v0 = perftest_option_nested_resource(1, NULL, &nested_resource.v1); assert(nested_resource.v0 == 1 && nested_resource.v1 == NULL);
    resource = perftest_option_resource_new(1);
    nested_resource.v0 = perftest_option_nested_resource(0, resource, &nested_resource.v1); assert(nested_resource.v0 == 0);
    id = perftest_option_resource_id(resource); assert(id == 77);
    nested_resource.v0 = perftest_option_nested_resource(1, resource, &nested_resource.v1); assert(nested_resource.v0 == 1);
    perftest_drop_owned_resource(nested_resource.v1);
    int64_t resource_choice = {0};
    resource_choice = perftest_resource_choice_id(NULL, 7); assert(resource_choice == -1);
    resource = perftest_option_resource_new(1);
    resource_choice = perftest_resource_choice_id(resource, 7); assert(resource_choice == 84);
    drops = perftest_owned_resource_drops(); assert(drops == start + 3);
}
static void check_results(void) {
    uintptr_t errors = {0};
    errors = perftest_result_error_drops(); size_t error_start = errors;
    char *error = NULL;
    int64_t scalar = 12345;
    assert(perftest_result_scalar(42, &scalar, &error) && scalar == 42 && error == NULL);
    scalar = 12345;
    assert(!perftest_result_scalar(-7, &scalar, &error) && scalar == 12345);
    assert(strcmp(error, "failure -7 世界") == 0);
    char *retained = error; error = NULL;
    assert(perftest_result_scalar(1, &scalar, &error) && error == NULL);
    assert(strcmp(retained, "failure -7 世界") == 0); // independent of subsequent calls
    perftest_free_error(retained);
    assert(!perftest_result_scalar(-8, &scalar, NULL));
    assert(!perftest_result_scalar(42, NULL, &error));
    assert(strcmp(error, "null success-output pointer") == 0);
    perftest_free_error(error); error = NULL;
    assert(!perftest_result_unit(0, &error));
    assert(strcmp(error, "failure 1 世界") == 0);
    perftest_free_error(error); error = NULL;
    assert(perftest_result_unit(1, &error) && error == NULL);
    // The optional enum already uses -3 for None; Result reserves the next niche.
    assert(perftest_result_optional(2, &error) == -4);
    assert(strcmp(error, "failure 2 世界") == 0);
    perftest_free_error(error); error = NULL;
    assert(perftest_result_optional(0, &error) == -3 && error == NULL);
    assert(perftest_result_optional(1, &error) == perftest_code_Seven);
    perftest_buffer text = {NULL, 99};
    assert(!perftest_result_text(0, &text, &error) && text.len == 99);
    perftest_free_error(error); error = NULL;
    assert(perftest_result_text(1, &text, &error));
    assert(text.len == strlen("héllo 世界") && memcmp(text.data, "héllo 世界", text.len) == 0);
    perftest_free_buffer(text);
    perftest_code code = 12345;
    assert(perftest_result_record(0, &code, &error) == -1 && code == 12345);
    perftest_free_error(error); error = NULL;
    assert(perftest_result_record(1, &code, &error) == perftest_priority_High && code == perftest_code_MinusOne);
    scalar = 12345;
    assert(!perftest_result_other(0, &scalar, &error) && scalar == 12345);
    assert(strcmp(error, "other error") == 0);
    perftest_free_error(error); error = NULL;
    assert(perftest_result_other(1, &scalar, &error) && scalar == 42);
    assert(perftest_storage_try_with_label((perftest_buffer){NULL, 0}, &error) == NULL);
    assert(strcmp(error, "label must not be empty") == 0);
    perftest_free_error(error); error = NULL;
    assert(perftest_storage_try_with_label((perftest_buffer){NULL, 1}, &error) == NULL);
    assert(strcmp(error, "invalid buffer") == 0);
    perftest_free_error(error); error = NULL;
    perftest_storage *storage = perftest_storage_try_with_label((perftest_buffer){(const uint8_t *)"label", 5}, &error);
    assert(storage != NULL && error == NULL);
    perftest_drop_storage(storage);
    uintptr_t drops = {0};
    drops = perftest_owned_resource_drops(); size_t start = drops;
    for (int32_t mode = 0; mode <= 3; ++mode) {
        perftest_owned_resource * resource = {NULL};
        resource = perftest_option_resource_new(1);
        bool consumed = true;
        assert(perftest_result_owned(resource, mode, 65, NULL, &error) == NULL);
        assert(strcmp(error, "null ownership-output pointer") == 0);
        perftest_free_error(error); error = NULL;
        assert(perftest_result_owned(resource, mode, 0xd800, &consumed, &error) == NULL && !consumed);
        perftest_free_error(error); error = NULL;
        int64_t id = {0};
        id = perftest_option_resource_id(resource); assert(id == 77);
        perftest_owned_resource *moved = perftest_result_owned(resource, mode, 65, &consumed, &error);
        assert(consumed);
        if (mode == 0) {
            assert(moved != NULL && error == NULL);
            perftest_drop_owned_resource(moved);
        } else {
            assert(moved == NULL);
            assert(strcmp(error, mode == 1 ? "failure 5 世界" : "Rust operation panicked") == 0);
            perftest_free_error(error); error = NULL;
        }
    }
    drops = perftest_owned_resource_drops(); assert(drops == start + 4);
    errors = perftest_result_error_drops(); assert(errors == error_start + 8);
    // Structured errors preserve fields and embedded NUL bytes in their buffer.
    perftest_result_error structured = {0};
    scalar = 12345;
    assert(!perftest_c_result_structured((perftest_buffer){NULL, 0}, 1, &scalar, &structured));
    assert(scalar == 12345 && structured.code == 7);
    const char message[] = "source 世界\0details";
    assert(structured.message.len == sizeof(message)-1 && memcmp(structured.message.data, message, sizeof(message)-1) == 0);
    perftest_drop_result_error(&structured);
    perftest_drop_result_error(&structured); assert(structured.message.data == NULL);
    const uint8_t invalid[] = {0xff};
    assert(!perftest_c_result_structured((perftest_buffer){invalid, 1}, 0, &scalar, &structured));
    assert(structured.code == -1 && structured.message.len == strlen("invalid UTF-8"));
    perftest_drop_result_error(&structured);
    assert(!perftest_c_result_structured((perftest_buffer){NULL, 0}, 2, &scalar, &structured));
    assert(structured.code == -1 && structured.message.len == strlen("Rust operation panicked"));
    perftest_drop_result_error(&structured);
    assert(!perftest_c_result_structured((perftest_buffer){NULL, 0}, 1, &scalar, NULL));
    structured.code = 12345;
    assert(perftest_c_result_structured((perftest_buffer){NULL, 0}, 0, &scalar, &structured));
    assert(scalar == 0 && structured.code == 12345);
    perftest_resource_error owned_error = {0};
    drops = perftest_owned_resource_drops(); start = drops;
    assert(!perftest_c_result_resource_error(1, &owned_error));
    assert(owned_error.resource != NULL);
    int64_t id = {0};
    id = perftest_option_resource_id(owned_error.resource); assert(id == 91);
    perftest_drop_resource_error(&owned_error); assert(owned_error.resource == NULL && owned_error.message.data == NULL);
    perftest_drop_resource_error(&owned_error);
    assert(!perftest_c_result_resource_error(1, NULL));
    drops = perftest_owned_resource_drops(); assert(drops == start + 2);
    uint8_t value = 99;
    assert(perftest_c_result_bool(0, &value, &structured) && value == 0);
    value = 99;
    assert(!perftest_c_result_bool(1, &value, &structured) && value == 99 && structured.code == 9);
    perftest_drop_result_error(&structured);
    scalar = 12345;
    assert(perftest_c_result_late_niche(1, &scalar, &structured) == 0x110000 && scalar == 12345 && structured.code == 8);
    perftest_drop_result_error(&structured);
    assert(perftest_c_result_late_niche(0, &scalar, &structured) == 65 && scalar == 42);
    perftest_panic_error panic_error;
    memset(&panic_error, 0xa5, sizeof(panic_error));
    assert(!perftest_c_result_panic_error(&panic_error));
    assert(panic_error.message.data == NULL && panic_error.message.len == 0);
    perftest_drop_panic_error(&panic_error);
    perftest_drop_panic_error(&panic_error);
    assert(!perftest_c_result_panic_error(NULL));
    perftest_rich_error rich = {0};
    assert(!perftest_c_result_rich_error(1, &rich));
    assert(rich.kind == perftest_code_Seven && rich.v1 == 1 && rich.v2.len == strlen("rich error"));
    perftest_drop_rich_error(&rich); assert(rich.v2.data == NULL);
    assert(!perftest_c_result_rich_error(0, &rich) && rich.kind == perftest_code_Seven && rich.v1 == 0);
    perftest_drop_rich_error(&rich);
    // Option already uses the handle's null niche, so Result must use bool.
    perftest_owned_resource *optional_handle = NULL;
    assert(perftest_c_result_optional_handle(0, &optional_handle, &structured) && optional_handle == NULL);
    assert(perftest_c_result_optional_handle(1, &optional_handle, &structured) && optional_handle != NULL);
    perftest_drop_owned_resource(optional_handle);
    optional_handle = (perftest_owned_resource *)(uintptr_t)8;
    assert(!perftest_c_result_optional_handle(2, &optional_handle, &structured));
    assert(optional_handle == (perftest_owned_resource *)(uintptr_t)8 && structured.code == 10);
    perftest_drop_result_error(&structured);
    // Late Drop panics cannot leak committed but unpublished success handles,
    // or overwrite an already committed structured error.
    drops = perftest_owned_resource_drops(); start = drops;
    assert(perftest_c_result_drop_panic(&structured) == NULL && structured.code == -1);
    perftest_drop_result_error(&structured);
    drops = perftest_owned_resource_drops(); assert(drops == start + 1);
    perftest_drop_error drop_error = {0};
    assert(!perftest_c_result_error_drop(&drop_error));
    assert(drop_error.resource != NULL);
    id = perftest_option_resource_id(drop_error.resource); assert(id == 12);
    perftest_drop_drop_error(&drop_error);
    drops = perftest_owned_resource_drops(); assert(drops == start + 2);
    perftest_free_error(NULL);
}
int main(void) {
    check_collections();
    check_sequence_delivery();
    EXPECT_ABORT(perftest_c_plain_panic());
    EXPECT_ABORT(perftest_drop_panic_output(perftest_c_panic_output_new()));
    EXPECT_ABORT(perftest_summary_new(0, 0.0, NULL));
    check_results();
    check_options();
    check_enums();
    perftest_storage * storage = {0};
    storage = perftest_storage_new();
    perftest_drop_storage(storage);
    perftest_drop_storage(NULL);
    const unsigned char text[] = {'h', 'i', 0, 'x'};
    perftest_buffer result = {0};
    result = perftest_string_new((perftest_buffer){text, sizeof(text)});
    assert(result.len == sizeof(text));
    assert(memcmp(result.data, text, sizeof(text)) == 0);
    perftest_free_buffer(result);
    uintptr_t len = {0};
    len = perftest_string_len((perftest_buffer){text, sizeof(text)});
    assert(len == sizeof(text));
    result = (perftest_buffer){NULL, 42};
    const unsigned char invalid[] = {255};
    EXPECT_ABORT(perftest_string_new((perftest_buffer){invalid, 1}));
    assert(result.data == NULL && result.len == 42);
    EXPECT_ABORT(perftest_string_new((perftest_buffer){NULL, 1}));
    result = perftest_string_new((perftest_buffer){NULL, 0});
    assert(result.len == 0);
    perftest_free_buffer(result);
    const unsigned char bytes[] = {0, 127, 128, 255};
    perftest_blob * blob = {0};
    perftest_blob * echoed = {0};
    blob = perftest_blob_value_new(INT64_MIN, (perftest_buffer){bytes, sizeof(bytes)});
    echoed = perftest_blob_value_echo(blob);
    EXPECT_ABORT(perftest_blob_value_echo(NULL));
    perftest_drop_blob(echoed);
    struct { int64_t v0; double v1; int64_t v2; int64_t v3; perftest_buffer v4; perftest_report * v5; } report = {0};
    report.v0 = perftest_report_new(42, 3.5, (perftest_buffer){text, sizeof(text)}, &report.v1, &report.v2, &report.v3, &report.v4, &report.v5);
    assert(report.v0 == 42 && report.v1 == 3.5 && report.v2 == 1 && report.v3 == 2);
    assert(report.v4.len == sizeof(text));
    assert(memcmp(report.v4.data, text, sizeof(text)) == 0);
    perftest_free_buffer(report.v4);
    perftest_report_struct * parts = {0};
    // Invalid selections abort; valid alternatives still work.
    EXPECT_ABORT(perftest_report_into_struct(2, report.v5, 0, 0.0, (perftest_buffer){NULL, 0}));
    assert(parts == NULL);
    // The selected handle arm ignores even an invalid buffer in the other arm.
    parts = perftest_report_into_struct(0, report.v5, 0, 0.0, (perftest_buffer){NULL, 1});
    perftest_drop_report_struct(parts);
    // Constructor arm receives the reduced label argument directly.
    parts = perftest_report_into_struct(1, 0, 7, 2.5, (perftest_buffer){text, sizeof(text)});
    perftest_drop_report_struct(parts);

    struct { int64_t v0; int64_t v1; } stamp = {0};
    stamp.v0 = perftest_stamp_new(INT64_MIN, INT64_MAX, &stamp.v1);
    assert(stamp.v0 == INT64_MIN && stamp.v1 == INT64_MAX);
    perftest_unsigned * unsigned_fields = {0};
    unsigned_fields = perftest_unsigned_round_trip(UINT8_MAX, UINT16_MAX, UINT32_MAX, UINT64_MAX, 1, UINT64_MAX);
    struct { int32_t v0; uint64_t v1; } maybe_unsigned = {0};
    maybe_unsigned.v0 = perftest_unsigned_data_maybe(unsigned_fields, &maybe_unsigned.v1);
    assert(maybe_unsigned.v0 == 1 && maybe_unsigned.v1 == UINT64_MAX);
    perftest_drop_unsigned(unsigned_fields);
    unsigned_fields = perftest_unsigned_round_trip(0, 0, 0, 0, 0, 0);
    maybe_unsigned.v0 = perftest_unsigned_data_maybe(unsigned_fields, &maybe_unsigned.v1); assert(maybe_unsigned.v0 == 0);
    perftest_drop_unsigned(unsigned_fields);
    struct { int32_t v0; perftest_buffer v1; } note = {0};
    note.v0 = perftest_plain_note_echo(0, (perftest_buffer){NULL, 99}, &note.v1); assert(note.v0 == 0);
    note.v0 = perftest_plain_note_echo(1, (perftest_buffer){NULL, 0}, &note.v1); assert(note.v0 == 1 && note.v1.len == 0);
    perftest_free_buffer(note.v1);
    const uint8_t note_text[] = {'h', 0, 'i'};
    note.v0 = perftest_plain_note_echo(1, (perftest_buffer){note_text, sizeof(note_text)}, &note.v1);
    assert(note.v0 == 1 && note.v1.len == sizeof(note_text) && memcmp(note.v1.data, note_text, sizeof(note_text)) == 0);
    perftest_free_buffer(note.v1);
    struct { int32_t v0; uint64_t v1; } optional_unsigned = {0};
    optional_unsigned.v0 = perftest_unsigned_optional(0, UINT64_MAX, &optional_unsigned.v1); assert(optional_unsigned.v0 == 0);
    optional_unsigned.v0 = perftest_unsigned_optional(1, UINT64_MAX, &optional_unsigned.v1);
    assert(optional_unsigned.v0 == 1 && optional_unsigned.v1 == UINT64_MAX);
    EXPECT_ABORT(perftest_unsigned_optional(2, 0, &optional_unsigned.v1));
    perftest_archive * archive = {0};
    perftest_summary *latest = NULL, *second_latest = NULL;

    archive = perftest_archive_new();
    latest = perftest_archive_latest(archive); assert(latest == NULL);
    perftest_archive_store(archive, 0, 2, 40.0, NULL);
    latest = perftest_archive_latest(archive); assert(latest != NULL);
    second_latest = perftest_archive_latest(archive); assert(second_latest != latest);
    perftest_drop_archive(archive);
    double archived_total = {0};
    archived_total = perftest_summary_total_raw(1, 0, 0.0, latest); assert(archived_total == 40.0);
    archived_total = perftest_summary_total_raw(1, 0, 0.0, second_latest); assert(archived_total == 40.0);
    perftest_storage * optional_storage = {0};
    struct { int32_t v0; int64_t v1; int32_t v2; double v3; uint8_t v4; } optional_payload = {0};
    uint8_t pushed = {0};
    optional_storage = perftest_storage_new();
    optional_payload.v0 = perftest_storage_get(optional_storage, &optional_payload.v1, &optional_payload.v2, &optional_payload.v3, &optional_payload.v4); assert(optional_payload.v0 == 0);
    pushed = perftest_storage_put_opt(optional_storage, 0, 0, 0, 0.0, 0); assert(pushed == 0);
    pushed = perftest_storage_put_opt(optional_storage, 1, 42, 7, 1.5, 0); assert(pushed == 1);
    optional_payload.v0 = perftest_storage_get(optional_storage, &optional_payload.v1, &optional_payload.v2, &optional_payload.v3, &optional_payload.v4);
    assert(optional_payload.v0 == 1 && optional_payload.v1 == 42 && optional_payload.v2 == 7 && optional_payload.v3 == 1.5 && optional_payload.v4 == 0);
    perftest_drop_storage(optional_storage);
    struct { int64_t v0; double v1; } summary = {0};
    summary.v0 = perftest_summary_new(INT64_MAX, -1.25, &summary.v1);
    assert(summary.v0 == INT64_MAX && summary.v1 == -1.25);
    double total = {0};
    total = perftest_summary_total_raw(0, 42, -1.25, 0); assert(total == -1.25);
    struct { int64_t v0; int32_t v1; double v2; uint8_t v3; } payload = {0};
    payload.v0 = perftest_payload_new(INT64_MIN, INT32_MAX, -0.0, 255, &payload.v1, &payload.v2, &payload.v3);
    assert(payload.v0 == INT64_MIN && payload.v1 == INT32_MAX && signbit(payload.v2) && payload.v3 == 1);
    perftest_storage * stored = {0};
    stored = perftest_storage_with_payload(INT64_MAX, INT32_MIN, INFINITY, 0);
    perftest_drop_storage(stored);
    perftest_storage * borrowed = {0};
    borrowed = perftest_storage_with_payload(42, 1, 2.5, 0);
    int64_t length = {0};
    length = perftest_storage_len(borrowed); assert(length == 1);

    perftest_storage_put_by_take(borrowed, 7, 2, 4.0, 1);
    uint8_t contains = {0};
    contains = perftest_storage_contains(borrowed, 7); assert(contains == 1);
    int64_t lengths = {0};
    lengths = perftest_storage_total_len(borrowed, borrowed, borrowed); assert(lengths == 3);
    perftest_summary *first = NULL, *second = NULL;
    first = perftest_storage_summary_handle(borrowed);
    second = perftest_storage_summary_handle(borrowed);
    struct { int64_t v0; double v1; } merged = {0};
    EXPECT_ABORT(perftest_summary_merge(1, 0, 0, first, 2, 0, 0, second, &merged.v1));
    EXPECT_ABORT(perftest_summary_merge(1, 0, 0, first, 1, 0, 0, NULL, &merged.v1));
    EXPECT_ABORT(perftest_summary_merge(1, 0, 0, first, 1, 0, 0, first, &merged.v1));
    // Abort probes run in children; these handles remain usable in the parent.
    merged.v0 = perftest_summary_merge(1, 0, 0, first, 1, 0, 0, second, &merged.v1);
    assert(merged.v0 == 2 && merged.v1 == 8.0);
    perftest_summary * nested = {0};
    perftest_buffer labeled_report = {0};
    nested = perftest_storage_summary_handle(borrowed);
    EXPECT_ABORT(perftest_report_label(1, 0, 0, nested, (perftest_buffer){invalid, 1}));
    labeled_report = perftest_report_label(1, 0, 0, nested, (perftest_buffer){text, sizeof(text)});
    assert(labeled_report.len == sizeof(text));
    perftest_free_buffer(labeled_report);
    perftest_drop_storage(borrowed);

    int8_t i8 = {0};
    i8 = perftest_primitive_i8(INT8_MIN); assert(i8 == INT8_MIN);
    i8 = perftest_primitive_i8(0); assert(i8 == 0);
    i8 = perftest_primitive_i8(INT8_MAX); assert(i8 == INT8_MAX);
    int16_t i16 = {0};
    i16 = perftest_primitive_i16(INT16_MIN); assert(i16 == INT16_MIN);
    i16 = perftest_primitive_i16(0); assert(i16 == 0);
    i16 = perftest_primitive_i16(INT16_MAX); assert(i16 == INT16_MAX);
    int32_t i32 = {0};
    i32 = perftest_primitive_i32(INT32_MIN); assert(i32 == INT32_MIN);
    i32 = perftest_primitive_i32(0); assert(i32 == 0);
    i32 = perftest_primitive_i32(INT32_MAX); assert(i32 == INT32_MAX);
    int64_t i64 = {0};
    i64 = perftest_primitive_i64(INT64_MIN); assert(i64 == INT64_MIN);
    i64 = perftest_primitive_i64(0); assert(i64 == 0);
    i64 = perftest_primitive_i64(INT64_MAX); assert(i64 == INT64_MAX);
    uint8_t u8 = {0};
    u8 = perftest_primitive_u8(0); assert(u8 == 0);
    u8 = perftest_primitive_u8(UINT8_MAX); assert(u8 == UINT8_MAX);
    uint16_t u16 = {0};
    u16 = perftest_primitive_u16(0); assert(u16 == 0);
    u16 = perftest_primitive_u16(UINT16_MAX); assert(u16 == UINT16_MAX);
    uint32_t u32 = {0};
    u32 = perftest_primitive_u32(0); assert(u32 == 0);
    u32 = perftest_primitive_u32(UINT32_MAX); assert(u32 == UINT32_MAX);
    uint64_t u64 = {0};
    u64 = perftest_primitive_u64(0); assert(u64 == 0);
    u64 = perftest_primitive_u64(UINT64_MAX); assert(u64 == UINT64_MAX);
    intptr_t isize = {0};
    isize = perftest_primitive_isize(INTPTR_MIN); assert(isize == INTPTR_MIN);
    isize = perftest_primitive_isize(INTPTR_MAX); assert(isize == INTPTR_MAX);
    uintptr_t usize = {0};
    usize = perftest_primitive_usize(0); assert(usize == 0);
    usize = perftest_primitive_usize(UINTPTR_MAX); assert(usize == UINTPTR_MAX);
    uint8_t boolean = {0};
    boolean = perftest_primitive_bool(0); assert(boolean == 0);
    boolean = perftest_primitive_bool(255); assert(boolean == 1);
    uint32_t character = {0};
    const uint32_t valid_chars[] = {0, 0xD7FF, 0xE000, 0x1F642, 0x10FFFF};
    for (size_t i = 0; i < sizeof(valid_chars)/sizeof(*valid_chars); ++i) {
        character = perftest_primitive_char(valid_chars[i]); assert(character == valid_chars[i]);
    }
    const uint32_t invalid_chars[] = {0xD800, 0xDFFF, 0x110000, UINT32_MAX};
    for (size_t i = 0; i < sizeof(invalid_chars)/sizeof(*invalid_chars); ++i) {
        character = 42;
        EXPECT_ABORT(perftest_primitive_char(invalid_chars[i]));
    }
    perftest_int128 signed128 = {0};
    perftest_int128 unsigned128 = {0};
    const perftest_int128 wide[] = {{0, 0}, {UINT64_MAX, UINT64_MAX}, {UINT64_C(1) << 63, 0}, {INT64_MAX, UINT64_MAX}};
    for (size_t i = 0; i < sizeof(wide)/sizeof(*wide); ++i) {
        signed128 = perftest_primitive_i128(wide[i]);
        assert(signed128.hi == wide[i].hi && signed128.lo == wide[i].lo);
        unsigned128 = perftest_primitive_u128(wide[i]);
        assert(unsigned128.hi == wide[i].hi && unsigned128.lo == wide[i].lo);
    }
    float f32 = {0};
    double f64 = {0};
    const double floats[] = {0.0, -0.0, 1.25, INFINITY, -INFINITY, NAN};
    for (size_t i = 0; i < sizeof(floats)/sizeof(*floats); ++i) {
        f64 = perftest_primitive_f64(floats[i]);
        f32 = perftest_primitive_f32((float)floats[i]);
        if (isnan(floats[i])) { assert(isnan(f64) && isnan(f32)); }
        else { assert(f64 == floats[i] && f32 == (float)floats[i]); }
    }
    f32 = perftest_primitive_f32(-0.0f); assert(signbit(f32));

    perftest_primitive_unit();
    puts("perftest-c PASS: all primitives, restored fields, constructor selectors and ownership");
}
