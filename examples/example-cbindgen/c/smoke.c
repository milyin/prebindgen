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
    assert(example_drop_calculator(result.v0) == 0);
    assert(example_last_error()[0] == '\0');
    const unsigned char text[] = {'h', 'i', 0, 'x'};
    example_caption_new_result caption = {0};
    assert(example_caption_new(UINT64_MAX, (example_buffer){text, sizeof(text)}, 255, &caption) == 0);
    assert(caption.v0 != 0);
    assert(example_drop_caption(caption.v0) == 0);
    caption.v0 = 42;
    const unsigned char invalid[] = {255};
    assert(example_caption_new(0, (example_buffer){invalid, 1}, 0, &caption) != 0);
    assert(caption.v0 == 42);
    example_foo_new_result foo = {0};
    example_foo_get_id_result id = {0};
    assert(example_foo_new(UINT64_MAX, &foo) == 0);
    assert(example_foo_get_id(foo.v0, &id) == 0 && id.v0 == UINT64_MAX);
    assert(example_drop_foo(foo.v0) == 0);
    puts("example-cbindgen PASS: handles, reduced Caption constructor, invalid UTF-8 and null output rejection");
}
