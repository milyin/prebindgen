#include "bindings.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>
int main(void) {
    perftest_storage_new_result storage = {0};
    assert(perftest_storage_new(&storage) == 0);
    assert(perftest_drop_storage(storage.v0) == 0);
    assert(perftest_drop_storage(storage.v0) == 0);
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
    assert(perftest_blob_value_new((perftest_buffer){bytes, sizeof(bytes)}, &blob) == 0);
    assert(perftest_blob_value_echo(blob.v0, &echoed) == 0);
    assert(echoed.v0 != blob.v0);
    assert(perftest_blob_value_echo(blob.v0, &echoed) != 0);
    assert(perftest_drop_blob(echoed.v0) == 0);
    assert(perftest_drop_blob(blob.v0) == 0);
    // Wrong-type input must not consume the unrelated live handle.
    assert(perftest_storage_new(&storage) == 0);
    assert(perftest_blob_value_echo(storage.v0, &echoed) != 0);
    assert(perftest_drop_storage(storage.v0) == 0);

    perftest_report_new_result report = {0};
    assert(perftest_report_new((perftest_buffer){text, sizeof(text)}, &report) == 0);
    assert(report.v0.len == sizeof(text));
    assert(memcmp(report.v0.data, text, sizeof(text)) == 0);
    perftest_free_buffer(report.v0);
    perftest_report_into_struct_result parts = {42};
    // Invalid selection leaves the handle unvisited and output untouched.
    assert(perftest_report_into_struct(2, report.v1, (perftest_buffer){NULL, 0}, &parts) != 0);
    assert(parts.v0 == 42);
    // The selected handle arm ignores even an invalid buffer in the other arm.
    assert(perftest_report_into_struct(0, report.v1, (perftest_buffer){NULL, 1}, &parts) == 0);
    assert(perftest_drop_report_struct(parts.v0) == 0);
    assert(perftest_report_into_struct(0, report.v1, (perftest_buffer){NULL, 0}, &parts) != 0);
    assert(perftest_drop_report(report.v1) == 0);
    // Constructor arm receives the reduced label argument directly.
    assert(perftest_report_into_struct(1, 0, (perftest_buffer){text, sizeof(text)}, &parts) == 0);
    assert(perftest_drop_report_struct(parts.v0) == 0);
    puts("perftest-c PASS: strings, byte-vector records, constructor selectors, field outputs and ownership");
}
