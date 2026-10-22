#include "../../c_abort.h"
#include "bindings.h"
#include <assert.h>
#include <stdio.h>
#include <string.h>

int main(void) {
    example_calculator * result = {0};
    result = example_calculator_new();
    assert(result != 0);
    example_drop_calculator(result);
    example_drop_calculator(NULL);
    const unsigned char text[] = {'h', 'i', 0, 'x'};
    example_caption * caption = {0};
    caption = example_caption_new(UINT64_MAX, (example_buffer){text, sizeof(text)}, 255);
    assert(caption != 0);
    example_drop_caption(caption);
    caption = NULL;
    const unsigned char invalid[] = {255};
    EXPECT_ABORT(example_caption_new(0, (example_buffer){invalid, 1}, 0));
    assert(caption == NULL);
    example_foo * foo = {0};
    uint64_t id = {0};
    foo = example_foo_new(UINT64_MAX);
    id = example_foo_get_id(foo); assert(id == UINT64_MAX);
    example_local_counter * counter = {0};
    example_local_counter * labeled = {0};
    uint64_t value = {0};
    counter = example_local_counter_new(42);
    value = example_local_counter_value(counter); assert(value == 42);
    EXPECT_ABORT(example_local_counter_label(counter, (example_buffer){invalid, 1}));
    value = example_local_counter_value(counter); assert(value == 42);
    labeled = example_local_counter_label(counter, (example_buffer){text, sizeof(text)});
    value = example_local_counter_value(labeled); assert(value == 46);
    example_drop_local_counter(labeled);
    puts("example-cbindgen PASS: typed pointers, non-Send counter, Caption constructor, invalid UTF-8 and aborting invalid input validation");
}
