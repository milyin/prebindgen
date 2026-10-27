static void result_argument_number(void *context, int64_t value) {
    *((int64_t *)context) += value;
}
static void result_argument_unit(void *context) { *((int64_t *)context) += 1; }
static void result_argument_handles(void *context, perftest_callback_return_resource *first, int64_t value, perftest_callback_return_resource *last) {
    assert(perftest_callback_return_resource_id(first) == 11 && value == 7);
    assert(perftest_callback_return_resource_id(last) == 13);
    perftest_drop_callback_return_resource(first);
    perftest_drop_callback_return_resource(last);
    *((int64_t *)context) += 1;
}
static int64_t result_argument_return(void *context, int64_t value) { *((int64_t *)context) += 1; return value * 2; }
static void check_callback_result_arguments(void) {
    int64_t count = 0;
    perftest_callback_result_values((perftest_callback_result_values_callback){.run=result_argument_number,.context=&count});
    assert(count == 24);
    count = 0;
    perftest_callback_result_unit((perftest_callback_result_unit_callback){.run=result_argument_unit,.context=&count});
    assert(count == 2);
    count = 0;
    perftest_callback_result_mapped((perftest_callback_result_mapped_callback){.run=result_argument_number,.context=&count});
    assert(count == 17);
    count = 0;
    assert(perftest_callback_result_return((perftest_callback_result_return_callback){.run=result_argument_return,.context=&count}) == 42);
    assert(count == 1);
    count = 0;
    perftest_callback_result_handles((perftest_callback_result_handles_callback){.run=result_argument_handles,.context=&count});
    assert(count == 1 && perftest_callback_return_resource_live() == 0);
    puts("C callback Result arguments PASS: delivery, suppression, mapped errors and cleanup");
}
