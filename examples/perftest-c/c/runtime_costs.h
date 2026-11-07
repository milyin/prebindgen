static void check_buffer_identity_views(void) {
    uintptr_t length = 0;
    perftest_owned_string text = perftest_buffer_views_text(&length);
    assert(length == 9 && text_view(&text).len == length);
    assert((uintptr_t)text_view(&text).data == perftest_buffer_views_address());
    assert(memcmp(text_view(&text).data, "move text", length) == 0);
    perftest_drop_owned_string(&text);
    perftest_owned_vec_u8_672e72343c1b4cb4 bytes = perftest_buffer_views_bytes(&length);
    const unsigned char expected[] = {1, 2, 3, 4};
    assert(length == sizeof(expected) && bytes_view(&bytes).len == length);
    assert((uintptr_t)bytes_view(&bytes).data == perftest_buffer_views_address());
    assert(memcmp(bytes_view(&bytes).data, expected, length) == 0);
    perftest_drop_owned_vec_u8_672e72343c1b4cb4(&bytes);
    for (uint8_t present = 0; present < 2; present++) {
        text = (perftest_owned_string){0};
        assert(perftest_buffer_views_optional_text(present, &text, &length) == present);
        assert(length == (present ? 9u : 0u));
        if (present) {
            assert((uintptr_t)text_view(&text).data == perftest_buffer_views_address());
            assert(memcmp(text_view(&text).data, "move text", length) == 0);
            perftest_drop_owned_string(&text);
        }
        bytes = (perftest_owned_vec_u8_672e72343c1b4cb4){0};
        assert(perftest_buffer_views_optional_bytes(present, &bytes, &length) == present);
        assert(length == (present ? 4u : 0u));
        if (present) {
            assert((uintptr_t)bytes_view(&bytes).data == perftest_buffer_views_address());
            assert(memcmp(bytes_view(&bytes).data, expected, length) == 0);
            perftest_drop_owned_vec_u8_672e72343c1b4cb4(&bytes);
        }
    }
    puts("C buffer identity views PASS: getters observe source storage and outputs preserve its address");
}
