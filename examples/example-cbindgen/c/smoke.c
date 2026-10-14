#include "bindings.h"
#include <assert.h>
#include <stdio.h>
int main(void) {
    example_calculator_new_result result = {0};
    assert(example_calculator_new(NULL) != 0);
    assert(example_last_error()[0] != '\0');
    assert(example_calculator_new(&result) == 0);
    assert(result.v0 != 0);
    assert(example_drop_calculator(result.v0) == 0);
    assert(example_drop_calculator(NULL) == 0);
    assert(example_last_error()[0] == '\0');
    const unsigned char text[] = {'h', 'i', 0, 'x'};
    example_caption_new_result caption = {0};
    assert(example_caption_new(UINT64_MAX, (example_buffer){text, sizeof(text)}, 255, &caption) == 0);
    assert(caption.v0 != 0);
    assert(example_drop_caption(caption.v0) == 0);
    caption.v0 = NULL;
    const unsigned char invalid[] = {255};
    assert(example_caption_new(0, (example_buffer){invalid, 1}, 0, &caption) != 0);
    assert(caption.v0 == NULL);
    example_foo_new_result foo = {0};
    example_foo_get_id_result id = {0};
    assert(example_foo_new(UINT64_MAX, &foo) == 0);
    assert(example_foo_get_id(foo.v0, &id) == 0 && id.v0 == UINT64_MAX);
    example_local_counter_new_result counter = {0};
    example_local_counter_label_result labeled = {0};
    example_local_counter_value_result value = {0};
    assert(example_local_counter_new(42, &counter) == 0);
    assert(example_local_counter_value(counter.v0, &value) == 0 && value.v0 == 42);
    assert(example_local_counter_label(counter.v0, (example_buffer){invalid, 1}, &labeled) != 0);
    assert(example_local_counter_value(counter.v0, &value) == 0 && value.v0 == 42);
    assert(example_local_counter_label(counter.v0, (example_buffer){text, sizeof(text)}, &labeled) == 0);
    assert(example_local_counter_value(labeled.v0, &value) == 0 && value.v0 == 46);
    assert(example_drop_local_counter(labeled.v0) == 0);
    puts("example-cbindgen PASS: typed pointers, non-Send counter, Caption constructor, invalid UTF-8 and null output rejection");
}
