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
    puts("perftest-c PASS: strings, buffers, failure atomicity and handle release");
}
