#include "bindings.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
#include <math.h>
#include <limits.h>
int main(void) {
    perftest_storage_new_result storage = {0};
    assert(perftest_storage_new(&storage) == 0);
    assert(perftest_drop_storage(storage.v0) == 0);
    assert(perftest_drop_storage(NULL) == 0);
    const unsigned char text[] = {'h', 'i', 0, 'x'};
    perftest_string_new_result result = {0};
    assert(perftest_string_new((perftest_buffer){text, sizeof(text)}, &result) == 0);
    assert(result.v0.len == sizeof(text));
    assert(memcmp(result.v0.data, text, sizeof(text)) == 0);
    perftest_free_buffer(result.v0);
    result.v0 = (perftest_buffer){NULL, 42};
    const unsigned char invalid[] = {255};
    assert(perftest_string_new((perftest_buffer){invalid, 1}, &result) != 0);
    assert(result.v0.data == NULL && result.v0.len == 42);
    assert(perftest_string_new((perftest_buffer){NULL, 1}, &result) != 0);
    assert(perftest_string_new((perftest_buffer){NULL, 0}, &result) == 0);
    assert(result.v0.len == 0);
    perftest_free_buffer(result.v0);
    const unsigned char bytes[] = {0, 127, 128, 255};
    perftest_blob_value_new_result blob = {0};
    perftest_blob_value_echo_result echoed = {0};
    assert(perftest_blob_value_new(INT64_MIN, (perftest_buffer){bytes, sizeof(bytes)}, &blob) == 0);
    assert(perftest_blob_value_echo(blob.v0, &echoed) == 0);
    assert(perftest_blob_value_echo(NULL, &echoed) != 0);
    assert(perftest_drop_blob(echoed.v0) == 0);
    perftest_report_new_result report = {0};
    assert(perftest_report_new(42, 3.5, (perftest_buffer){text, sizeof(text)}, &report) == 0);
    assert(report.v0 == 42 && report.v1 == 3.5 && report.v2 == 1 && report.v3 == 2);
    assert(report.v4.len == sizeof(text));
    assert(memcmp(report.v4.data, text, sizeof(text)) == 0);
    perftest_free_buffer(report.v4);
    perftest_report_into_struct_result parts = {0};
    // Invalid selection leaves the handle unvisited and output untouched.
    assert(perftest_report_into_struct(2, report.v5, 0, 0.0, (perftest_buffer){NULL, 0}, &parts) != 0);
    assert(parts.v0 == NULL);
    // The selected handle arm ignores even an invalid buffer in the other arm.
    assert(perftest_report_into_struct(0, report.v5, 0, 0.0, (perftest_buffer){NULL, 1}, &parts) == 0);
    assert(perftest_drop_report_struct(parts.v0) == 0);
    // Constructor arm receives the reduced label argument directly.
    assert(perftest_report_into_struct(1, 0, 7, 2.5, (perftest_buffer){text, sizeof(text)}, &parts) == 0);
    assert(perftest_drop_report_struct(parts.v0) == 0);

    perftest_stamp_new_result stamp = {0};
    assert(perftest_stamp_new(INT64_MIN, INT64_MAX, &stamp) == 0);
    assert(stamp.v0 == INT64_MIN && stamp.v1 == INT64_MAX);
    perftest_unsigned_round_trip_result unsigned_fields = {0};
    assert(perftest_unsigned_round_trip(UINT8_MAX, UINT16_MAX, UINT32_MAX, UINT64_MAX, &unsigned_fields) == 0);
    assert(perftest_drop_unsigned(unsigned_fields.v0) == 0);
    perftest_summary_new_result summary = {0};
    assert(perftest_summary_new(INT64_MAX, -1.25, &summary) == 0);
    assert(summary.v0 == INT64_MAX && summary.v1 == -1.25);
    perftest_summary_total_raw_result total = {0};
    assert(perftest_summary_total_raw(0, 42, -1.25, 0, &total) == 0 && total.v0 == -1.25);
    perftest_payload_new_result payload = {0};
    assert(perftest_payload_new(INT64_MIN, INT32_MAX, -0.0, 255, &payload) == 0);
    assert(payload.v0 == INT64_MIN && payload.v1 == INT32_MAX && signbit(payload.v2) && payload.v3 == 1);
    perftest_storage_with_payload_result stored = {0};
    assert(perftest_storage_with_payload(INT64_MAX, INT32_MIN, INFINITY, 0, &stored) == 0);
    assert(perftest_drop_storage(stored.v0) == 0);
    perftest_storage_with_payload_result borrowed = {0};
    assert(perftest_storage_with_payload(42, 1, 2.5, 0, &borrowed) == 0);
    perftest_storage_len_result length = {0};
    assert(perftest_storage_len(borrowed.v0, &length) == 0 && length.v0 == 1);
    perftest_storage_put_by_take_result put = {0};
    assert(perftest_storage_put_by_take(borrowed.v0, 7, 2, 4.0, 1, &put) == 0);
    perftest_storage_contains_result contains = {0};
    assert(perftest_storage_contains(borrowed.v0, 7, &contains) == 0 && contains.v0 == 1);
    perftest_storage_total_len_result lengths = {0};
    assert(perftest_storage_total_len(borrowed.v0, borrowed.v0, borrowed.v0, &lengths) == 0 && lengths.v0 == 3);
    perftest_storage_summary_handle_result first = {0}, second = {0};
    assert(perftest_storage_summary_handle(borrowed.v0, &first) == 0);
    assert(perftest_storage_summary_handle(borrowed.v0, &second) == 0);
    perftest_summary_merge_result merged = {0};
    assert(perftest_summary_merge(1, 0, 0, first.v0, 2, 0, 0, second.v0, &merged) != 0);
    assert(perftest_summary_merge(1, 0, 0, first.v0, 1, 0, 0, NULL, &merged) != 0);
    assert(perftest_summary_merge(1, 0, 0, first.v0, 1, 0, 0, first.v0, &merged) != 0);
    // Both pointers survived validation failures, including duplicate consumption.
    assert(perftest_summary_merge(1, 0, 0, first.v0, 1, 0, 0, second.v0, &merged) == 0);
    assert(merged.v0 == 2 && merged.v1 == 8.0);
    perftest_storage_summary_handle_result nested = {0};
    perftest_report_label_result labeled_report = {0};
    assert(perftest_storage_summary_handle(borrowed.v0, &nested) == 0);
    assert(perftest_report_label(1, 0, 0, nested.v0, (perftest_buffer){invalid, 1}, &labeled_report) != 0);
    assert(perftest_report_label(1, 0, 0, nested.v0, (perftest_buffer){text, sizeof(text)}, &labeled_report) == 0);
    assert(labeled_report.v0.len == sizeof(text));
    perftest_free_buffer(labeled_report.v0);
    assert(perftest_drop_storage(borrowed.v0) == 0);

    perftest_primitive_i8_result i8 = {0};
    assert(perftest_primitive_i8(INT8_MIN, &i8) == 0 && i8.v0 == INT8_MIN);
    assert(perftest_primitive_i8(0, &i8) == 0 && i8.v0 == 0);
    assert(perftest_primitive_i8(INT8_MAX, &i8) == 0 && i8.v0 == INT8_MAX);
    perftest_primitive_i16_result i16 = {0};
    assert(perftest_primitive_i16(INT16_MIN, &i16) == 0 && i16.v0 == INT16_MIN);
    assert(perftest_primitive_i16(0, &i16) == 0 && i16.v0 == 0);
    assert(perftest_primitive_i16(INT16_MAX, &i16) == 0 && i16.v0 == INT16_MAX);
    perftest_primitive_i32_result i32 = {0};
    assert(perftest_primitive_i32(INT32_MIN, &i32) == 0 && i32.v0 == INT32_MIN);
    assert(perftest_primitive_i32(0, &i32) == 0 && i32.v0 == 0);
    assert(perftest_primitive_i32(INT32_MAX, &i32) == 0 && i32.v0 == INT32_MAX);
    perftest_primitive_i64_result i64 = {0};
    assert(perftest_primitive_i64(INT64_MIN, &i64) == 0 && i64.v0 == INT64_MIN);
    assert(perftest_primitive_i64(0, &i64) == 0 && i64.v0 == 0);
    assert(perftest_primitive_i64(INT64_MAX, &i64) == 0 && i64.v0 == INT64_MAX);
    perftest_primitive_u8_result u8 = {0};
    assert(perftest_primitive_u8(0, &u8) == 0 && u8.v0 == 0);
    assert(perftest_primitive_u8(UINT8_MAX, &u8) == 0 && u8.v0 == UINT8_MAX);
    perftest_primitive_u16_result u16 = {0};
    assert(perftest_primitive_u16(0, &u16) == 0 && u16.v0 == 0);
    assert(perftest_primitive_u16(UINT16_MAX, &u16) == 0 && u16.v0 == UINT16_MAX);
    perftest_primitive_u32_result u32 = {0};
    assert(perftest_primitive_u32(0, &u32) == 0 && u32.v0 == 0);
    assert(perftest_primitive_u32(UINT32_MAX, &u32) == 0 && u32.v0 == UINT32_MAX);
    perftest_primitive_u64_result u64 = {0};
    assert(perftest_primitive_u64(0, &u64) == 0 && u64.v0 == 0);
    assert(perftest_primitive_u64(UINT64_MAX, &u64) == 0 && u64.v0 == UINT64_MAX);
    perftest_primitive_isize_result isize = {0};
    assert(perftest_primitive_isize(INTPTR_MIN, &isize) == 0 && isize.v0 == INTPTR_MIN);
    assert(perftest_primitive_isize(INTPTR_MAX, &isize) == 0 && isize.v0 == INTPTR_MAX);
    perftest_primitive_usize_result usize = {0};
    assert(perftest_primitive_usize(0, &usize) == 0 && usize.v0 == 0);
    assert(perftest_primitive_usize(UINTPTR_MAX, &usize) == 0 && usize.v0 == UINTPTR_MAX);
    perftest_primitive_bool_result boolean = {0};
    assert(perftest_primitive_bool(0, &boolean) == 0 && boolean.v0 == 0);
    assert(perftest_primitive_bool(255, &boolean) == 0 && boolean.v0 == 1);
    perftest_primitive_char_result character = {0};
    const uint32_t valid_chars[] = {0, 0xD7FF, 0xE000, 0x1F642, 0x10FFFF};
    for (size_t i = 0; i < sizeof(valid_chars)/sizeof(*valid_chars); ++i) {
        assert(perftest_primitive_char(valid_chars[i], &character) == 0 && character.v0 == valid_chars[i]);
    }
    const uint32_t invalid_chars[] = {0xD800, 0xDFFF, 0x110000, UINT32_MAX};
    for (size_t i = 0; i < sizeof(invalid_chars)/sizeof(*invalid_chars); ++i) {
        character.v0 = 42;
        assert(perftest_primitive_char(invalid_chars[i], &character) != 0 && character.v0 == 42);
    }
    perftest_primitive_i128_result signed128 = {0};
    perftest_primitive_u128_result unsigned128 = {0};
    const perftest_int128 wide[] = {{0, 0}, {UINT64_MAX, UINT64_MAX}, {UINT64_C(1) << 63, 0}, {INT64_MAX, UINT64_MAX}};
    for (size_t i = 0; i < sizeof(wide)/sizeof(*wide); ++i) {
        assert(perftest_primitive_i128(wide[i], &signed128) == 0);
        assert(signed128.v0.hi == wide[i].hi && signed128.v0.lo == wide[i].lo);
        assert(perftest_primitive_u128(wide[i], &unsigned128) == 0);
        assert(unsigned128.v0.hi == wide[i].hi && unsigned128.v0.lo == wide[i].lo);
    }
    perftest_primitive_f32_result f32 = {0};
    perftest_primitive_f64_result f64 = {0};
    const double floats[] = {0.0, -0.0, 1.25, INFINITY, -INFINITY, NAN};
    for (size_t i = 0; i < sizeof(floats)/sizeof(*floats); ++i) {
        assert(perftest_primitive_f64(floats[i], &f64) == 0);
        assert(perftest_primitive_f32((float)floats[i], &f32) == 0);
        if (isnan(floats[i])) { assert(isnan(f64.v0) && isnan(f32.v0)); }
        else { assert(f64.v0 == floats[i] && f32.v0 == (float)floats[i]); }
    }
    assert(perftest_primitive_f32(-0.0f, &f32) == 0 && signbit(f32.v0));
    perftest_primitive_unit_result unit = {0};
    assert(perftest_primitive_unit(&unit) == 0);
    puts("perftest-c PASS: all primitives, restored fields, constructor selectors and ownership");
}
