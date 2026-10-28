/* Borrowed output views must finish before the non-Clone owner is moved. */
static void check_borrowed_getters(void) {
    int64_t field_number = 0;
    perftest_buffer field_text = {0};
    perftest_getter_owner *field_owner = perftest_getter_fields_new(&field_number, &field_text);
    perftest_drop_getter_owner(field_owner);
    assert(field_number == 31 && field_text.len == 5 && memcmp(field_text.data, "first", 5) == 0);
    perftest_free_buffer(field_text);
    perftest_getter_leaf *second_clone = NULL;
    perftest_getter_leaf *first_clone = perftest_getter_two_clones_new(&second_clone);
    assert(first_clone != second_clone && perftest_getter_leaf_id(first_clone) == 19);
    perftest_drop_getter_leaf(first_clone);
    assert(perftest_getter_leaf_id(second_clone) == 19);
    perftest_drop_getter_leaf(second_clone);
    const char expected[] = "héllo\0世界😀";
    for (uint8_t present = 0; present <= 1; ++present) {
        perftest_getter_counts_ret_w0_rows before = perftest_getter_counts();
        int64_t number = 0;
        perftest_buffer name = {0};
        perftest_getter_leaf *leaf = NULL;
        perftest_getter_owner *owner = perftest_getter_owner_new(present, &number, &name, &leaf);
        assert(number == 71 && name.len == sizeof(expected)-1);
        assert(memcmp(name.data, expected, name.len) == 0);
        perftest_free_buffer(name);
        assert(perftest_getter_owner_id(owner) == 71 && perftest_getter_leaf_id(leaf) == 19);
        assert(perftest_getter_owner_read(owner, &name) == 71);
        assert(name.len == sizeof(expected)-1);
        assert(memcmp(name.data, expected, name.len) == 0);
        perftest_free_buffer(name);
        number = -1;
        name = (perftest_buffer){0};
        assert(perftest_getter_owner_optional(owner, &number, &name) == present);
        if (present) {
            assert(number == 71 && name.len == sizeof(expected)-1);
            assert(memcmp(name.data, expected, name.len) == 0);
            perftest_free_buffer(name);
        } else { assert(number == 0 && name.data == NULL && name.len == 0); }
        perftest_drop_getter_owner(owner);
        assert(perftest_getter_leaf_id(leaf) == 19);
        perftest_drop_getter_leaf(leaf);
        perftest_getter_counts_ret_w0_rows after = perftest_getter_counts();
        assert(after.data[0].v0 == before.data[0].v0 + 1);
        assert(after.data[1].v0 == before.data[1].v0 + 1);
        assert(after.data[2].v0 == before.data[2].v0 + 2);
        assert(after.data[3].v0 == before.data[3].v0 + 1);
        assert(after.data[4].v0 == before.data[4].v0 + 2 + present);
        perftest_getter_counts_ret_w0_rows_free(before);
        perftest_getter_counts_ret_w0_rows_free(after);
    }
    puts("C borrowed getters PASS: nested views, optional borrows, move ordering and independent cloned handles");
}

/* A returned loan points inside the owner and must never be destroyed. */
static void check_borrowed_pointers(void) {
    const uint8_t temporary_text[] = "temporary input";
    const perftest_getter_child *fixed = perftest_getter_static_child(
        (perftest_buffer){temporary_text, sizeof(temporary_text)-1});
    assert(perftest_getter_child_number(fixed) == 91);
    assert(perftest_getter_static_child((perftest_buffer){NULL, 0}) == fixed);
    for (uint8_t present = 0; present <= 1; ++present) {
        int64_t number = 0;
        perftest_buffer name = {0};
        perftest_getter_leaf *leaf = NULL;
        perftest_getter_owner *owner = perftest_getter_owner_new(present, &number, &name, &leaf);
        perftest_free_buffer(name);
        const perftest_getter_child *child = perftest_getter_owner_child(owner);
        assert(child != NULL && perftest_getter_owner_child_matches(owner, child));
        assert(perftest_getter_owner_child(owner) == child);
        assert(perftest_getter_child_number(child) == 71);
        name = perftest_getter_child_name(child);
        const char expected[] = "héllo\0世界😀";
        assert(name.len == sizeof(expected)-1 && memcmp(name.data, expected, name.len) == 0);
        perftest_free_buffer(name);
        assert(perftest_getter_owner_optional_pointer(owner) == (present ? child : NULL));
        char *error = NULL;
        assert(perftest_getter_owner_try_child(owner, 0, &error) == child && error == NULL);
        assert(perftest_getter_owner_try_child(owner, 1, &error) == NULL && error != NULL);
        perftest_free_error(error);
        error = NULL;
        assert(perftest_getter_owner_try_child(NULL, 0, &error) == NULL && error != NULL);
        perftest_free_error(error);
        for (int32_t mode = 0; mode <= 2; ++mode) {
            error = NULL;
            const perftest_getter_child *optional = child;
            bool success = perftest_getter_owner_try_optional_child(owner, mode, &optional, &error);
            assert(success == (mode < 2));
            assert(optional == (mode == 0 ? NULL : child)); // Errors preserve output storage.
            assert((error != NULL) == (mode == 2));
            perftest_free_error(error);
        }
        error = NULL;
        assert(!perftest_getter_owner_try_optional_child(owner, 1, NULL, &error) && error != NULL);
        perftest_free_error(error);
        assert(perftest_getter_owner_child_matches(owner, child));
        assert(perftest_getter_child_number(child) == 71);
        perftest_drop_getter_owner(owner); // Invalidates child; only owner is freed.
        perftest_drop_getter_leaf(leaf); // Existing expansion still gives an owned clone.
    }
    puts("C borrowed pointers PASS: address identity, non-Clone children, optional and Result loans");
}

static void check_mutable_borrowed_pointers(void) {
    for (uint8_t present = 0; present <= 1; ++present) {
        perftest_getter_counts_ret_w0_rows before = perftest_getter_counts();
        int64_t number = 0;
        perftest_buffer name = {0};
        perftest_getter_leaf *leaf = NULL;
        perftest_getter_owner *owner = perftest_getter_owner_new(present, &number, &name, &leaf);
        perftest_free_buffer(name);
        uintptr_t address = (uintptr_t)perftest_getter_owner_child(owner);
        // Each new owner borrow ends the previous child loan. Retain only its
        // numeric address for comparisons, and obtain a fresh loan before writing.
        perftest_getter_child *child = perftest_getter_owner_child_mut(owner);
        assert((uintptr_t)child == address);
        perftest_getter_child_set_number(child, 88);
        child = NULL;
        assert(perftest_getter_owner_id(owner) == 88);
        perftest_getter_child *optional = perftest_getter_owner_optional_mut(owner);
        assert((uintptr_t)optional == (present ? address : 0));
        if (present) perftest_getter_child_set_number(optional, 89);
        optional = NULL;
        number = present ? 89 : 88;
        assert(perftest_getter_owner_id(owner) == number);
        char *error = NULL;
        child = perftest_getter_owner_try_child_mut(owner, 0, &error);
        assert((uintptr_t)child == address && error == NULL);
        perftest_getter_child_set_number(child, 90);
        child = NULL;
        assert(perftest_getter_owner_id(owner) == 90);
        assert(perftest_getter_owner_try_child_mut(owner, 1, &error) == NULL && error != NULL);
        perftest_free_error(error);
        number = 90;
        for (int32_t mode = 0; mode <= 2; ++mode) {
            error = NULL;
            optional = NULL;
            assert(perftest_getter_owner_try_optional_mut(owner, mode, &optional, &error) == (mode < 2));
            assert((uintptr_t)optional == (mode == 1 ? address : 0));
            assert((error != NULL) == (mode == 2));
            if (mode == 1) { perftest_getter_child_set_number(optional, 91); number = 91; }
            optional = NULL;
            perftest_free_error(error);
            assert(perftest_getter_owner_id(owner) == number);
        }
        error = NULL;
        assert(!perftest_getter_owner_try_optional_mut(owner, 1, NULL, &error) && error != NULL);
        perftest_free_error(error);
        // Failures preserve the owning object; mutation uses a fresh child loan.
        child = perftest_getter_owner_child_mut(owner);
        assert((uintptr_t)child == address);
        perftest_getter_child_set_number(child, 99);
        child = NULL;
        assert(perftest_getter_owner_id(owner) == 99);
        perftest_drop_getter_owner(owner);
        perftest_drop_getter_leaf(leaf);
        perftest_getter_counts_ret_w0_rows after = perftest_getter_counts();
        assert(after.data[0].v0 == before.data[0].v0 + 1);
        assert(after.data[1].v0 == before.data[1].v0 + 1); // Only the explicit owned leaf projection clones.
        assert(after.data[2].v0 == before.data[2].v0 + 2);
        perftest_getter_counts_ret_w0_rows_free(before);
        perftest_getter_counts_ret_w0_rows_free(after);
    }
    puts("C mutable borrows PASS: original-object mutation, optional/Result loans and owner-only destruction");
}
