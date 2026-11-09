/* Runtime ownership checks through the generated C ABI, including source constructor errors. */
static void check_buffer_inputs(void) {
    const uint8_t bytes[] = {2, 3}, euro[] = {0xe2, 0x82, 0xac};
    perftest_buffer text = {(const uint8_t *)"abc", 3}, payload = {bytes, sizeof(bytes)};
    char *error = NULL;
    int64_t result = 12345;
    size_t consumed = 99, drops = perftest_owned_resource_drops();
    perftest_buffer_row_reset_counts();
    perftest_buffer_rows_sum_p0_w1_row rows[] = {
        {perftest_collection_resource_new(1), text, payload},
        {perftest_collection_resource_new(2), {NULL, 0}, {NULL, 0}},
        {perftest_collection_resource_new(3), {euro, sizeof(euro)}, payload}
    };
    assert(perftest_buffer_rows_sum((perftest_buffer_rows_sum_p0_w1_rows){rows, 3}, &result, &consumed, &error));
    assert(result == 22 && consumed == 3 && error == NULL);
    assert(perftest_owned_resource_drops() == drops + 3);
    assert(perftest_buffer_row_constructor_calls() == 3 && perftest_buffer_row_function_calls() == 1);

    /* A source constructor can panic after consuming rows. Rust drops the completed
       prefix and current owner; the unread suffix remains the caller's. */
    for (int mode = 3; mode < 4; ++mode) {
        drops = perftest_owned_resource_drops();
        perftest_buffer_row_reset_counts();
        for (size_t i = 0; i < 3; ++i)
            rows[i] = (perftest_buffer_rows_sum_p0_w1_row){perftest_collection_resource_new(i == 1 && mode == 3 ? -2 : (int64_t)i + 1), text, payload};
        result = 12345;
        assert(!perftest_buffer_rows_sum((perftest_buffer_rows_sum_p0_w1_rows){rows, 3}, &result, &consumed, &error));
        assert(result == 12345 && consumed == 2 && error != NULL);
        assert(perftest_owned_resource_drops() == drops + 2);
        assert(perftest_buffer_row_constructor_calls() == (mode == 3 ? 2u : 1u));
        assert(perftest_buffer_row_function_calls() == 0);
        assert(perftest_option_resource_id(rows[2].v0) == 3);
        perftest_drop_owned_resource(rows[2].v0);
        perftest_free_error(error); error = NULL;
        assert(perftest_owned_resource_drops() == drops + 3);
    }

    perftest_buffer_row_reset_counts();
    assert(perftest_buffer_rows_optional(0, (perftest_buffer_rows_optional_p0_w3_rows){(void *)1, SIZE_MAX}, &result, &consumed, &error));
    assert(result == -1 && consumed == 0);
    perftest_buffer_rows_optional_p0_w3_row optional[] = {
        {NULL, {NULL, SIZE_MAX}, {NULL, SIZE_MAX}},
        {perftest_collection_resource_new(7), text, payload}
    };
    assert(perftest_buffer_rows_optional(1, (perftest_buffer_rows_optional_p0_w3_rows){optional, 2}, &result, &consumed, &error));
    assert(result == 15 && consumed == 1 && perftest_buffer_row_constructor_calls() == 1);

    /* Shared row handles may repeat alongside a separate exclusive receiver. */
    perftest_owned_resource *shared = perftest_collection_resource_new(7);
    perftest_owned_resource *receiver = perftest_collection_resource_new(100);
    perftest_buffer_rows_borrowed_p1_w1_row borrowed[] = {{shared, text, payload}, {shared, text, payload}};
    perftest_buffer_row_reset_counts(); drops = perftest_owned_resource_drops();
    assert(perftest_buffer_rows_borrowed(receiver, (perftest_buffer_rows_borrowed_p1_w1_rows){borrowed, 2}, &result, &error));
    assert(result == 130 && perftest_buffer_row_constructor_calls() == 2);
    assert(perftest_owned_resource_drops() == drops && perftest_option_resource_id(shared) == 7);
    perftest_drop_owned_resource(shared); perftest_drop_owned_resource(receiver);

    /* &String/&Vec constructor parameters need temporaries lasting until the
       constructor finishes; they must neither escape nor be decoded twice. */
    perftest_buffer_row_reset_counts(); drops = perftest_owned_resource_drops();
    perftest_buffer_rows_temporaries_p0_w1_row temporary[] = {{perftest_collection_resource_new(7), text, payload}};
    assert(perftest_buffer_rows_temporaries((perftest_buffer_rows_temporaries_p0_w1_rows){temporary, 1}, &result, &consumed, &error));
    assert(result == 15 && consumed == 1 && perftest_owned_resource_drops() == drops + 1);
    assert(perftest_buffer_row_constructor_calls() == 1 && perftest_buffer_row_function_calls() == 1);
    /* Optional borrowed leaves keep storage outside the presence branch. */
    perftest_buffer_row_reset_counts(); drops = perftest_owned_resource_drops();
    perftest_buffer_rows_views_p0_w1_row views[] = {
        {perftest_collection_resource_new(1), 1, text, 1, payload},
        {perftest_collection_resource_new(2), 0, {NULL, SIZE_MAX}, 0, {NULL, SIZE_MAX}}
    };
    assert(perftest_buffer_rows_views((perftest_buffer_rows_views_p0_w1_rows){views, 2}, &result, &consumed, &error));
    assert(result == 11 && consumed == 2 && perftest_owned_resource_drops() == drops + 2);
    assert(perftest_buffer_row_constructor_calls() == 2);
    perftest_buffer_rows_optional_temporaries_p0_w1_row optional_temps[] = {
        {perftest_collection_resource_new(1), 1, text, 1, payload},
        {perftest_collection_resource_new(2), 0, {NULL, SIZE_MAX}, 0, {NULL, SIZE_MAX}}
    };
    assert(perftest_buffer_rows_optional_temporaries((perftest_buffer_rows_optional_temporaries_p0_w1_rows){optional_temps, 2}, &result, &consumed, &error));
    assert(result == 11 && consumed == 2);

    /* Selected constructors can borrow optional buffers without leaking a
       reference into an arm-local temporary. Inactive slots stay untouched. */
    shared = perftest_collection_resource_new(7);
    perftest_buffer_rows_alternatives_p0_w1_row alternatives[] = {
        {0, perftest_collection_resource_new(1), 1, text, 1, payload, NULL, {NULL, SIZE_MAX}, {NULL, SIZE_MAX}},
        {1, NULL, 0, {NULL, SIZE_MAX}, 0, {NULL, SIZE_MAX}, shared, text, payload}
    };
    perftest_buffer_row_reset_counts(); drops = perftest_owned_resource_drops();
    assert(perftest_buffer_rows_alternatives((perftest_buffer_rows_alternatives_p0_w1_rows){alternatives, 2}, &result, &consumed, &error));
    assert(result == 24 && consumed == 1 && perftest_owned_resource_drops() == drops + 1);
    assert(perftest_buffer_row_constructor_calls() == 2 && perftest_option_resource_id(shared) == 7);
    perftest_drop_owned_resource(shared);

    puts("C buffer input replay PASS: decode once, optional/selector borrows and prefix cleanup");
}
