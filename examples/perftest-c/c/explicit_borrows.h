struct borrow_packet_probe { const perftest_borrow_packet *original; size_t calls; };
static void receive_packet_borrow(void *context, const perftest_borrow_packet *value) {
    struct borrow_packet_probe *probe = context;
    assert(value == probe->original);
    assert(perftest_borrow_packet_len(value) == 65536);
    probe->calls++;
}
static void receive_packet_copy(void *context, perftest_borrow_packet *value) {
    struct borrow_packet_probe *probe = context;
    assert(value != probe->original && perftest_borrow_packet_len(value) == 65536);
    probe->calls++;
    perftest_drop_borrow_packet(value);
}
static void receive_packet_optional(void *context, const perftest_borrow_packet *value) {
    struct borrow_packet_probe *probe = context;
    assert(value == (probe->calls == 0 ? NULL : probe->original));
    probe->calls++;
}
static void receive_packet_size(void *context, uintptr_t size) {
    struct borrow_packet_probe *probe = context;
    assert(size == 65536);
    probe->calls++;
}
static void receive_packet_size_plus_one(void *context, uintptr_t size) {
    struct borrow_packet_probe *probe = context;
    assert(size == 65537);
    probe->calls++;
}
static void receive_non_clone(void *context, const perftest_borrow_no_clone *value) {
    assert(context == (const void *)value);
    assert(perftest_borrow_no_clone_value(value) == 91);
}
static void check_explicit_borrows(void) {
    perftest_borrow_packet *packet = perftest_borrow_packet_new(65536);
    struct borrow_packet_probe probe = {packet, 0};
    uintptr_t clones = perftest_borrow_packet_counter(0);
    uintptr_t bytes = perftest_borrow_packet_counter(1);
    uintptr_t drops = perftest_borrow_packet_counter(2);
    perftest_borrow_packet_emit_callback borrowed = {receive_packet_borrow, &probe, NULL};
    perftest_borrow_packet_emit(packet, borrowed, 1000);
    perftest_borrow_packet_emit_worker(packet, borrowed);
    assert(probe.calls == 1001 && perftest_borrow_packet_ref(packet) == packet);
    assert(perftest_borrow_packet_optional(packet, 0) == NULL);
    assert(perftest_borrow_packet_optional(packet, 1) == packet);
    assert(perftest_borrow_packet_counter(0) == clones && perftest_borrow_packet_counter(1) == bytes);
    assert(perftest_borrow_packet_counter(2) == drops);
    perftest_borrow_packet_emit_copy_callback copied = {receive_packet_copy, &probe, NULL};
    perftest_borrow_packet_emit_copy(packet, copied, 32);
    assert(perftest_borrow_packet_counter(0) == clones + 32);
    assert(perftest_borrow_packet_counter(1) == bytes + 32 * 65536);
    assert(perftest_borrow_packet_counter(2) == drops + 32);
    probe.calls = 0;
    perftest_borrow_packet_emit_optional_callback optional = {receive_packet_optional, &probe, NULL};
    perftest_borrow_packet_emit_optional(packet, optional, 0);
    perftest_borrow_packet_emit_optional(packet, optional, 1);
    assert(probe.calls == 2);
    perftest_borrow_packet_emit_expanded_callback expanded = {receive_packet_size, &probe, NULL};
    perftest_borrow_packet_emit_expanded(packet, expanded, 3);
    assert(probe.calls == 5 && perftest_borrow_packet_counter(0) == clones + 32);
    perftest_borrow_packet_emit_expanded_other_callback other = {receive_packet_size_plus_one, &probe, NULL};
    perftest_borrow_packet_emit_expanded_other(packet, other, 3);
    assert(probe.calls == 8 && perftest_borrow_packet_counter(0) == clones + 32);
    perftest_drop_borrow_packet(packet);
    assert(perftest_borrow_packet_counter(2) == drops + 33);
    perftest_borrow_no_clone *unique = perftest_borrow_no_clone_new();
    perftest_borrow_no_clone_emit_callback no_clone = {receive_non_clone, unique, NULL};
    perftest_borrow_no_clone_emit(unique, no_clone);
    perftest_drop_borrow_no_clone(unique);
    puts("C explicit borrows PASS: pointer identity, non-Clone objects, optional/worker borrows and opt-in copy budgets");
}
