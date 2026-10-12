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
    puts("example-cbindgen PASS: generated handle creation, release and null output rejection");
}
