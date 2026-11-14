/* The generated ABI contains representation values, not CanonMillis handles. */
static int64_t canonical_millis_callback(void *context, int64_t value) {
    assert(value == 19);
    ++*(int *)context;
    return value + 1;
}
static void canonical_millis_notify(void *context, int64_t value) {
    assert(value == 23);
    ++*(int *)context;
}
static perftest_canonical_token *canonical_owned_callback(void *context, perftest_canonical_token *value) {
    ++*(int *)context;
    assert(perftest_canonical_token_value(value) == 53);
    /* C owns the callback argument and must release or transfer it. */
    perftest_drop_canonical_token(value);
    return perftest_canonical_token_new(54);
}
static void check_canonical_values(void) {
    const int64_t values[] = {INT64_MIN, -1, 0, INT64_MAX};
    for (size_t i = 0; i < sizeof(values)/sizeof(*values); ++i) {
        assert(perftest_canonical_millis_echo(values[i]) == values[i]);
        assert(perftest_canonical_millis_borrow(values[i]) == values[i]);
        int64_t result = 0;
        assert(perftest_canonical_millis_optional(1, values[i], &result) == 1);
        assert(result == values[i]);
    }
    int64_t absent = 99;
    assert(perftest_canonical_millis_optional(0, 0, &absent) == 0);
    assert(perftest_canonical_millis_static() == 31);
    assert(perftest_canonical_millis_local(13) == 13);
    assert(perftest_canonical_celsius(22) == 23);
    perftest_canonical_millis_rows_p0_w1_row rows[] = {{3}, {7}, {11}};
    perftest_canonical_millis_rows_ret_w0_rows result = perftest_canonical_millis_rows(
        (perftest_canonical_millis_rows_p0_w1_rows){rows, 3});
    assert(result.len == 3);
    for (size_t i = 0; i < result.len; ++i) assert(result.data[i].v0 == rows[i].v0);
    perftest_canonical_millis_rows_ret_w0_rows_free(result);
    int calls = 0;
    assert(perftest_canonical_millis_callback((perftest_canonical_millis_callback_callback){canonical_millis_callback, &calls, NULL}) == 20);
    perftest_canonical_millis_notify((perftest_canonical_millis_notify_callback){canonical_millis_notify, &calls, NULL});
    assert(calls == 2);
    int64_t output = 99;
    perftest_canonical_error error = {0};
    assert(!perftest_canonical_domain_error(-1, &output, &error));
    assert(error.code == 17 && output == 99);
    perftest_drop_canonical_error(&error);
    assert(perftest_canonical_domain_error(7, &output, &error) && output == 7);
    assert(perftest_canonical_live() == 0);
    perftest_canonical_token *token = perftest_canonical_token_new(47);
    assert(perftest_canonical_millis_from_token(token) == 47);
    assert(perftest_canonical_token_value(token) == 47);
    perftest_drop_canonical_token(token);
    perftest_canonical_token *first = perftest_canonical_token_new(1);
    perftest_canonical_token *second = perftest_canonical_token_new(-2);
    perftest_canonical_token *third = perftest_canonical_token_new(3);
    size_t consumed = 999;
    assert(!perftest_canonical_consume(first, second, third, &output, &consumed, &error));
    assert(consumed == 2 && error.code == -1);
    perftest_drop_canonical_error(&error);
    assert(perftest_canonical_live() == 1 && perftest_canonical_token_value(third) == 3);
    perftest_drop_canonical_token(third);
    token = perftest_canonical_owned_callback((perftest_canonical_owned_callback_callback){canonical_owned_callback, &calls, NULL});
    assert(calls == 3 && perftest_canonical_token_value(token) == 54);
    perftest_drop_canonical_token(token);
    assert(perftest_canonical_live() == 0);
}
