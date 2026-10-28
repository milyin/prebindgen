static void check_buffer_identity_views(void) {
    uintptr_t length = 0;
    perftest_buffer text = perftest_buffer_views_text(&length);
    assert(length == 9 && text.len == length);
    assert((uintptr_t)text.data == perftest_buffer_views_address());
    assert(memcmp(text.data, "move text", length) == 0);
    perftest_free_buffer(text);
    perftest_buffer bytes = perftest_buffer_views_bytes(&length);
    const unsigned char expected[] = {1, 2, 3, 4};
    assert(length == sizeof(expected) && bytes.len == length);
    assert((uintptr_t)bytes.data == perftest_buffer_views_address());
    assert(memcmp(bytes.data, expected, length) == 0);
    perftest_free_buffer(bytes);
    for (uint8_t present = 0; present < 2; present++) {
        text = (perftest_buffer){0};
        assert(perftest_buffer_views_optional_text(present, &text, &length) == present);
        assert(length == (present ? 9u : 0u));
        if (present) {
            assert((uintptr_t)text.data == perftest_buffer_views_address());
            assert(memcmp(text.data, "move text", length) == 0);
            perftest_free_buffer(text);
        }
        bytes = (perftest_buffer){0};
        assert(perftest_buffer_views_optional_bytes(present, &bytes, &length) == present);
        assert(length == (present ? 4u : 0u));
        if (present) {
            assert((uintptr_t)bytes.data == perftest_buffer_views_address());
            assert(memcmp(bytes.data, expected, length) == 0);
            perftest_free_buffer(bytes);
        }
    }
    puts("C buffer identity views PASS: getters observe source storage and outputs preserve its address");
}
