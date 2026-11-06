/* Exercise the same C-to-Rust rows under both alias-check policies. */
#define EI_PREFIX perftest
#define EI_CHECKED 1
#include "enum_inputs_case.h"
#undef EI_CHECKED
#undef EI_PREFIX
#define EI_PREFIX fast
#define EI_CHECKED 0
#include "enum_inputs_case.h"
#undef EI_CHECKED
#undef EI_PREFIX
static void check_enum_inputs(void) {
    perftest_check_enum_inputs();
    fast_check_enum_inputs();
    puts("C enum input replay PASS: sparse tags, optional/selector borrows, aliases and failure cleanup");
}
