#ifndef TEST_MUTABLE_BYTE_VIEWS_H
#define TEST_MUTABLE_BYTE_VIEWS_H

static void check_mutable_byte_views(void) {
    const size_t sizes[] = {0, 1, 65536};
    const uintptr_t drops = perftest_mutable_bytes_drops();
    for (size_t n = 0; n < sizeof(sizes) / sizeof(sizes[0]); ++n) {
        const size_t size = sizes[n];
        perftest_mutable_bytes *owner = perftest_mutable_bytes_new(size);
        const uintptr_t address = perftest_mutable_bytes_address(owner);
        const uintptr_t capacity = perftest_mutable_bytes_capacity(owner);
        perftest_buffer_view_mut view = perftest_mutable_bytes_mut(owner);
        assert((uintptr_t)view.data == address && view.len == size);
        if (size) { view.data[0] = 19; view.data[size - 1] = 23; }
        view = (perftest_buffer_view_mut){NULL, 0};
        if (size) assert(perftest_mutable_bytes_value(owner, size - 1) == 23);
        for (int present = 0; present <= 1; ++present) {
            assert(perftest_mutable_bytes_optional(owner, (uint8_t)present, &view) == present);
            assert((uintptr_t)view.data == (present ? address : 0));
            assert(view.len == (present ? size : 0));
            if (present && size) view.data[0] = 31;
            view = (perftest_buffer_view_mut){NULL, 0};
        }
        if (size) assert(perftest_mutable_bytes_value(owner, 0) == 31);
        for (int mode = 0; mode < 3; ++mode) {
            int32_t inner = -1;
            assert(perftest_mutable_bytes_nested(owner, mode, &inner, &view) == (mode != 0));
            assert(inner == (mode == 2));
            assert((uintptr_t)view.data == (mode == 2 ? address : 0));
            assert(view.len == (mode == 2 ? size : 0));
            view = (perftest_buffer_view_mut){NULL, 0};
        }
        char *error = NULL;
        assert(perftest_mutable_bytes_result(owner, 0, &view, &error));
        assert(error == NULL && (uintptr_t)view.data == address && view.len == size);
        if (size) view.data[0] = 41;
        view = (perftest_buffer_view_mut){NULL, 0};
        assert(!perftest_mutable_bytes_result(owner, 1, &view, &error) && error != NULL);
        assert(view.data == NULL && view.len == 0);
        perftest_free_error(error); error = NULL;
        for (int present = 0; present <= 1; ++present) {
            int32_t actual = -1;
            assert(perftest_mutable_bytes_result_optional(owner, (uint8_t)present, 0, &actual, &view, &error));
            assert(error == NULL && actual == present);
            assert((uintptr_t)view.data == (present ? address : 0));
            assert(view.len == (present ? size : 0));
            view = (perftest_buffer_view_mut){NULL, 0};
        }
        int32_t actual = -1;
        assert(!perftest_mutable_bytes_result_optional(owner, 1, 1, &actual, &view, &error) && error != NULL);
        perftest_free_error(error);
        if (size) assert(perftest_mutable_bytes_value(owner, 0) == 41);
        assert(perftest_mutable_bytes_address(owner) == address);
        assert(perftest_mutable_bytes_capacity(owner) == capacity);
        perftest_drop_mutable_bytes(owner);
    }
    assert(perftest_mutable_bytes_drops() == drops + 3);

    perftest_owned_vec_u8_672e72343c1b4cb4 vector = perftest_mutable_bytes_vector(8);
    perftest_buffer_view shared = perftest_owned_vec_u8_672e72343c1b4cb4_view(&vector);
    const uintptr_t address = (uintptr_t)shared.data;
    shared = (perftest_buffer_view){NULL, 0};
    perftest_buffer_view_mut mutable = perftest_owned_vec_u8_672e72343c1b4cb4_view_mut(&vector);
    assert((uintptr_t)mutable.data == address && mutable.len == 8);
    mutable.data[3] = 99;
    mutable = (perftest_buffer_view_mut){NULL, 0};
    shared = perftest_owned_vec_u8_672e72343c1b4cb4_view(&vector);
    assert(shared.data[3] == 99);
    shared = (perftest_buffer_view){NULL, 0};
    perftest_drop_owned_vec_u8_672e72343c1b4cb4(&vector);

    owner_buffer_view_mut projected = {NULL, 0};
    owner_bytes expanded = owner_mutable_bytes_vector_expanded(8, &projected);
    assert(projected.len == 8);
    const uintptr_t projected_address = (uintptr_t)projected.data;
    projected.data[0] = 87;
    projected = (owner_buffer_view_mut){NULL, 0};
    owner_buffer_view observed = owner_bytes_view(&expanded);
    assert((uintptr_t)observed.data == projected_address && observed.data[0] == 87);
    observed = (owner_buffer_view){NULL, 0};
    owner_drop_bytes(&expanded);
    puts("C mutable byte views PASS: address identity, original mutations, empty/optional/Result loans, inline Vec accessors and expansion");
}
#endif
