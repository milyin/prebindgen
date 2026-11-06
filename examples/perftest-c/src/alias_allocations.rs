//! Allocation budgets at the actual generated C boundary, isolated per test thread.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct Allocator;
thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
    static ALLOCATED_BYTES: Cell<usize> = const { Cell::new(0) };
    static REALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}
fn record(size: usize, reallocation: bool) {
    if COUNTING.try_with(Cell::get).unwrap_or(false) {
        let _ = ALLOCATIONS.try_with(|n| n.set(n.get().wrapping_add(1)));
        let _ = ALLOCATED_BYTES.try_with(|n| n.set(n.get().wrapping_add(size)));
        if reallocation {
            let _ = REALLOCATIONS.try_with(|n| n.set(n.get().wrapping_add(1)));
        }
    }
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(layout.size(), false);
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record(size, true);
        unsafe { System.realloc(pointer, layout, size) }
    }
}
fn allocations(call: impl FnOnce()) -> usize {
    struct Stop;
    impl Drop for Stop {
        fn drop(&mut self) {
            COUNTING.with(|flag| flag.set(false));
        }
    }
    ALLOCATIONS.with(|n| n.set(0));
    ALLOCATED_BYTES.with(|n| n.set(0));
    REALLOCATIONS.with(|n| n.set(0));
    COUNTING.with(|flag| flag.set(true));
    let stop = Stop;
    call();
    drop(stop);
    ALLOCATIONS.with(Cell::get)
}

#[test]
fn borrowed_callbacks_allocate_only_when_copying_is_requested() {
    unsafe extern "C" fn borrowed(
        context: *mut core::ffi::c_void,
        value: *const perftest_flat::BorrowPacket,
    ) {
        assert_eq!(value.cast::<core::ffi::c_void>(), context.cast_const());
        assert_eq!(super::perftest_borrow_packet_len(value), 65536);
    }
    unsafe extern "C" fn copied(
        context: *mut core::ffi::c_void,
        value: *mut perftest_flat::BorrowPacket,
    ) {
        assert_ne!(value.cast::<core::ffi::c_void>(), context);
        assert_eq!(super::perftest_borrow_packet_len(value), 65536);
        super::perftest_drop_borrow_packet(value);
    }
    unsafe {
        let packet = super::perftest_borrow_packet_new(65536);
        let clones = perftest_flat::borrow_packet_counter(0);
        let bytes = perftest_flat::borrow_packet_counter(1);
        let drops = perftest_flat::borrow_packet_counter(2);
        let shared = allocations(|| {
            super::perftest_borrow_packet_emit(
                packet,
                super::perftest_callback_ad0b57894c7bbea6 {
                    run: Some(borrowed),
                    context: packet.cast(),
                    drop: None,
                },
                1000,
            )
        });
        assert_eq!(shared, 0);
        assert_eq!(perftest_flat::borrow_packet_counter(0), clones);
        assert_eq!(perftest_flat::borrow_packet_counter(1), bytes);
        assert_eq!(perftest_flat::borrow_packet_counter(2), drops);
        let copied = allocations(|| {
            super::perftest_borrow_packet_emit_copy(
                packet,
                super::perftest_callback_078f14fba1e1a0bf {
                    run: Some(copied),
                    context: packet.cast(),
                    drop: None,
                },
                1000,
            )
        });
        assert_eq!(
            copied, 2000,
            "one payload and one handle allocation per explicit copy"
        );
        assert_eq!(perftest_flat::borrow_packet_counter(0), clones + 1000);
        assert_eq!(
            perftest_flat::borrow_packet_counter(1),
            bytes + 1000 * 65536
        );
        assert_eq!(perftest_flat::borrow_packet_counter(2), drops + 1000);
        super::perftest_drop_borrow_packet(packet);
        println!("C borrowed callback allocation budget: shared=0, explicit_copy=2000 allocations, copied_bytes={} for 1000 x 65536-byte callbacks",1000*65536);
    }
}

#[test]
fn generated_shared_and_consuming_calls_do_not_allocate_for_alias_checks() {
    let _getters = super::GETTER_TESTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    unsafe {
        let owner = perftest_flat::getter_owner_new(false);
        let child = perftest_flat::getter_owner_child(&owner) as *const _;
        let storage = super::perftest_storage_with_payload(1, 1, 2.5, 0);
        let shared = allocations(|| {
            for _ in 0..1000 {
                assert_eq!(
                    super::perftest_getter_owner_child_matches(
                        std::hint::black_box(&owner as *const _),
                        std::hint::black_box(child)
                    ),
                    1
                );
                assert_eq!(
                    super::perftest_storage_total_len(
                        std::hint::black_box(storage),
                        storage,
                        storage
                    ),
                    3
                );
            }
        });
        assert_eq!(
            shared, 0,
            "two/three shared handle calls must not allocate alias bookkeeping"
        );
        let target = super::perftest_storage_with_payload(0, 0, 0.0, 0);
        let large = allocations(|| {
            for _ in 0..1000 {
                let _ = super::perftest_storage_accumulate_lengths(
                    std::hint::black_box(target),
                    storage,
                    storage,
                    storage,
                    storage,
                    storage,
                    storage,
                    storage,
                    storage,
                );
            }
        });
        assert_eq!(
            large, 0,
            "nine fixed accesses must use stack-only alias bookkeeping"
        );
        assert_eq!(super::perftest_storage_contains(target, 8000), 1);
        super::perftest_drop_storage(target);
        let mut consuming = 0;
        for _ in 0..1000 {
            let first = super::perftest_storage_summary_handle(storage);
            let second = super::perftest_storage_summary_handle(storage);
            let mut count = 0;
            let mut total = 0.0;
            consuming += allocations(|| {
                count = super::perftest_summary_merge(
                    1,
                    0,
                    0.0,
                    std::hint::black_box(first),
                    1,
                    0,
                    0.0,
                    std::hint::black_box(second),
                    &mut total,
                );
            });
            assert_eq!((count, total), (2, 5.0));
        }
        assert_eq!(
            consuming, 0,
            "selected consuming handle inputs must not allocate alias bookkeeping"
        );
        super::perftest_drop_storage(storage);
    }
}

#[test]
fn native_vec_and_string_transfers_preserve_capacity_without_allocating() {
    use super::inline_owners as ffi;
    let mut bytes = Vec::with_capacity(4096);
    bytes.extend_from_slice(b"a\0b");
    let mut text = String::with_capacity(8192);
    text.push_str("héllo");
    let mut numbers = Vec::with_capacity(1024);
    numbers.extend_from_slice(&[7, 9]);
    let bytes_address = bytes.as_ptr();
    let text_address = text.as_ptr();
    perftest_flat::inline_stage(bytes, text, numbers);
    let cost = allocations(|| unsafe {
        let mut bytes = ffi::owner_inline_bytes();
        let mut text = ffi::owner_inline_text();
        let mut numbers = ffi::owner_inline_numbers();
        assert_eq!(ffi::owner_bytes_view(&bytes).data, bytes_address);
        assert_eq!(ffi::owner_text_view(&text).data, text_address);
        assert_eq!(ffi::owner_inline_bytes_capacity(&bytes), 4096);
        assert_eq!(ffi::owner_inline_text_capacity(&text), 8192);
        assert_eq!(ffi::owner_inline_numbers_capacity(&numbers), 1024);
        assert_eq!(ffi::owner_inline_bytes_take(&mut bytes), 4096);
        assert_eq!(ffi::owner_inline_text_take(&mut text), 8192);
        assert_eq!(ffi::owner_inline_numbers_take(&mut numbers), 16);
        ffi::owner_drop_bytes(&mut bytes);
        ffi::owner_drop_text(&mut text);
        ffi::owner_drop_numbers(&mut numbers);
    });
    assert_eq!(cost,0,"native transfers, accessors, consumption and empty-owner drops must not allocate or reallocate");
    // A reserved, empty allocation must also survive the transfer unchanged.
    let bytes = Vec::with_capacity(4096);
    let pointer = bytes.as_ptr();
    perftest_flat::inline_stage(bytes, String::new(), Vec::new());
    assert_eq!(
        allocations(|| unsafe {
            let mut bytes = ffi::owner_inline_bytes();
            assert_eq!(ffi::owner_bytes_len(&bytes), 0);
            assert_eq!(ffi::owner_bytes_view(&bytes).data, pointer);
            assert_eq!(ffi::owner_inline_bytes_capacity(&bytes), 4096);
            ffi::owner_drop_bytes(&mut bytes);
            ffi::owner_drop_bytes(&mut bytes);
        }),
        0
    );
}

#[test]
fn generated_collection_inputs_allocate_once_without_growth() {
    for size in [0usize, 1, 8, 64, 512, 4096] {
        let plain: Vec<_> = (0..size)
            .map(|v| super::__PbRow_collection_input_plain_p0_w1 { v0: v as i64 })
            .collect();
        let slice: Vec<_> = (0..size)
            .map(|v| super::__PbRow_collection_input_slice_p0_w1 { v0: v as i64 })
            .collect();
        let custom: Vec<_> = (0..size)
            .map(|v| super::__PbRow_collection_input_custom_p0_w1 { v0: v as i64 })
            .collect();
        let expected = (size * size.saturating_sub(1) / 2) as i64;
        let plain_cost = allocations(|| unsafe {
            assert_eq!(
                super::perftest_collection_input_plain(
                    super::__PbRows {
                        data: plain.as_ptr().cast(),
                        len: size
                    },
                    7
                ),
                expected + 7
            );
        });
        assert_eq!(plain_cost, usize::from(size != 0));
        assert_eq!(
            ALLOCATED_BYTES.with(Cell::get),
            size * std::mem::size_of::<i64>()
        );
        assert_eq!(REALLOCATIONS.with(Cell::get), 0);
        let slice_cost = allocations(|| unsafe {
            assert_eq!(
                super::perftest_collection_input_slice(super::__PbRows {
                    data: slice.as_ptr().cast(),
                    len: size
                }),
                expected
            );
        });
        assert_eq!(slice_cost, usize::from(size != 0));
        assert_eq!(
            ALLOCATED_BYTES.with(Cell::get),
            size * std::mem::size_of::<i64>()
        );
        assert_eq!(REALLOCATIONS.with(Cell::get), 0);
        // Custom constructors use the same final vector allocation as primitives.
        let custom_cost = allocations(|| unsafe {
            assert_eq!(
                super::perftest_collection_input_custom(
                    super::__PbRows {
                        data: custom.as_ptr().cast(),
                        len: size
                    },
                    7
                ),
                expected + 7
            );
        });
        assert_eq!(custom_cost, usize::from(size != 0));
        assert_eq!(ALLOCATED_BYTES.with(Cell::get), size * 8);
        assert_eq!(REALLOCATIONS.with(Cell::get), 0);
    }
}

#[test]
fn consuming_collection_inputs_bound_preparation_and_alias_storage() {
    let _getters = super::GETTER_TESTS
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    for size in [0usize, 1, 8, 64, 512, 4096] {
        let rows: Vec<_> = (0..size)
            .map(|index| super::__PbRow_collection_try_construct_all_p0_w1 {
                v0: unsafe { super::perftest_collection_resource_new(index as i64 + 100) },
            })
            .collect();
        let mut output = -1;
        let mut consumed = usize::MAX;
        let mut error = std::ptr::null_mut();
        let cost = allocations(|| unsafe {
            assert!(super::perftest_collection_try_construct_all(
                super::__PbRows {
                    data: rows.as_ptr().cast(),
                    len: size
                },
                &mut output,
                &mut consumed,
                &mut error,
            ));
        });
        assert_eq!(
            cost,
            2 * usize::from(size != 0),
            "size={size}: grouped addresses and final values; no prepared-row buffer"
        );
        assert_eq!(REALLOCATIONS.with(Cell::get), 0);
        assert_eq!(consumed, size);
        assert_eq!(
            output,
            (size * 100 + size * size.saturating_sub(1) / 2) as i64
        );
        assert!(error.is_null());
    }
}

#[test]
fn enum_handle_inputs_omit_prepared_row_storage() {
    let _getters = super::GETTER_TESTS
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    macro_rules! check {
        ($ffi:ident, $row:ident, $make:ident, $take:ident, $checks:expr) => {{
            for size in [0usize, 1, 8, 64, 512, 4096] {
                let rows: Vec<_> = (0..size)
                    .map(|i| $ffi::$row {
                        v0: unsafe { $ffi::$make(i as i64) },
                        v1: 17,
                    })
                    .collect();
                let mut result = -1;
                let mut consumed = usize::MAX;
                let mut error = core::ptr::null_mut();
                let count = allocations(|| unsafe {
                    assert!($ffi::$take(
                        $ffi::__PbRows {
                            data: rows.as_ptr().cast(),
                            len: size
                        },
                        &mut result,
                        &mut consumed,
                        &mut error,
                    ));
                });
                assert_eq!(count, (1 + usize::from($checks)) * usize::from(size != 0));
                assert_eq!(REALLOCATIONS.with(Cell::get), 0);
                if !$checks {
                    assert_eq!(
                        ALLOCATED_BYTES.with(Cell::get),
                        size * core::mem::size_of::<i64>()
                    );
                }
                assert_eq!(
                    result,
                    (size * size.saturating_sub(1) / 2 + 17 * size) as i64
                );
                assert_eq!(consumed, size);
                assert!(error.is_null());
            }
        }};
    }
    use super::unchecked;
    check!(
        super,
        __PbRow_enum_inputs_take_p0_w1,
        perftest_collection_resource_new,
        perftest_enum_inputs_take,
        true
    );
    check!(
        unchecked,
        __PbRow_enum_inputs_take_p0_w1,
        fast_collection_resource_new,
        fast_enum_inputs_take,
        false
    );
}

#[test]
fn inline_owner_inputs_allocate_only_alias_history_and_final_values() {
    use super::inline_owners as ffi;
    let _getters = super::GETTER_TESTS
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    for size in [0usize, 1, 8, 64, 512, 4096] {
        for checked in [true, false] {
            let mut owners: Vec<_> = (0..size)
                .map(|_| unsafe { ffi::owner_inline_packet_new() })
                .collect();
            let rows: Vec<_> = owners
                .iter_mut()
                .map(|owner| ffi::__PbRow_inline_inputs_packets_p0_w1 { v0: owner })
                .collect();
            let mut result = usize::MAX;
            let mut consumed = usize::MAX;
            let mut error = core::ptr::null_mut();
            perftest_flat::inline_inputs_reset(0, 0);
            let count = allocations(|| unsafe {
                let rows = ffi::__PbRows {
                    data: rows.as_ptr().cast(),
                    len: size,
                };
                let ok = if checked {
                    ffi::owner_inline_inputs_packets(rows, &mut result, &mut consumed, &mut error)
                } else {
                    ffi::owner_inline_inputs_packets_fast(
                        rows,
                        &mut result,
                        &mut consumed,
                        &mut error,
                    )
                };
                assert!(ok);
            });
            assert_eq!(count, (1 + usize::from(checked)) * usize::from(size != 0));
            assert_eq!(REALLOCATIONS.with(Cell::get), 0);
            if !checked {
                assert_eq!(
                    ALLOCATED_BYTES.with(Cell::get),
                    size * core::mem::size_of::<perftest_flat::InlinePacket>()
                );
            }
            assert_eq!(result, 3 * size);
            assert_eq!(consumed, size);
            assert!(error.is_null());
            assert_eq!(perftest_flat::inline_inputs_empty_calls(), size);
            for owner in &mut owners {
                unsafe {
                    assert_eq!(ffi::owner_inline_packet_len(owner), 0);
                    ffi::owner_drop_packet(owner);
                }
            }
        }
    }
}

#[test]
fn mutable_byte_loans_and_vec_accessors_do_not_allocate_or_copy() {
    for size in [0usize, 1, 65536] {
        unsafe {
            let owner = super::perftest_mutable_bytes_new(size);
            let address = super::perftest_mutable_bytes_address(owner);
            let capacity = super::perftest_mutable_bytes_capacity(owner);
            let before_drops = super::perftest_mutable_bytes_drops();
            let count = allocations(|| {
                for i in 0..1000 {
                    let view = super::perftest_mutable_bytes_mut(std::hint::black_box(owner));
                    assert_eq!(view.data as usize, address);
                    assert_eq!(view.len, size);
                    if size > 0 {
                        *view.data = (i % 256) as u8;
                    }
                }
            });
            assert_eq!(
                count, 0,
                "size={size}: view acquisition and in-place writes must not allocate"
            );
            assert_eq!(REALLOCATIONS.with(Cell::get), 0);
            assert_eq!(super::perftest_mutable_bytes_address(owner), address);
            assert_eq!(super::perftest_mutable_bytes_capacity(owner), capacity);
            if size > 0 {
                assert_eq!(
                    super::perftest_mutable_bytes_value(owner, 0),
                    (999 % 256) as u8
                );
            }
            assert_eq!(super::perftest_mutable_bytes_drops(), before_drops);
            super::perftest_drop_mutable_bytes(owner);
            assert_eq!(super::perftest_mutable_bytes_drops(), before_drops + 1);

            use super::inline_owners as ffi;
            let mut initial = ffi::__PbBufferMut {
                data: std::ptr::null_mut(),
                len: 0,
            };
            let mut vector = ffi::owner_mutable_bytes_vector_expanded(size, &mut initial);
            let address = ffi::owner_bytes_view(&vector).data as usize;
            let count = allocations(|| {
                for i in 0..1000 {
                    let view = ffi::owner_bytes_view_mut(std::hint::black_box(&mut vector));
                    assert_eq!(view.data as usize, address);
                    if size > 0 {
                        *view.data = (i % 256) as u8;
                    }
                }
            });
            assert_eq!(count, 0);
            if size > 0 {
                assert_eq!(*ffi::owner_bytes_view(&vector).data, (999 % 256) as u8);
            }
            ffi::owner_drop_bytes(&mut vector);

            let mut view = ffi::__PbBufferMut {
                data: std::ptr::null_mut(),
                len: 0,
            };
            let count = allocations(|| {
                let mut expanded = ffi::owner_mutable_bytes_vector_expanded(size, &mut view);
                assert_eq!(view.len, size);
                let address = view.data as usize;
                if size > 0 {
                    *view.data = 73;
                }
                let observed = ffi::owner_bytes_view(&expanded);
                assert_eq!(observed.data as usize, address);
                if size > 0 {
                    assert_eq!(*observed.data, 73);
                }
                ffi::owner_drop_bytes(&mut expanded);
            });
            // The sole allocation constructs the source Vec, including reserved empty storage.
            assert_eq!(
                count, 1,
                "size={size}: only source Vec construction may allocate"
            );
            assert_eq!(REALLOCATIONS.with(Cell::get), 0);
            println!("mutable byte budget: size={size}, deliveries=1000, direct_allocations=0, vec_accessor_allocations=0, expanded_allocations=1 (source Vec only)");
        }
    }
}

#[test]
fn fixed_callback_results_need_no_cleanup_allocations() {
    use super::unchecked;
    let _getters = super::GETTER_TESTS
        .lock()
        .unwrap_or_else(|error| error.into_inner());
    struct Inputs {
        pairs: Vec<(
            *mut perftest_flat::OwnedResource,
            *mut perftest_flat::OwnedResource,
        )>,
        next: usize,
    }
    unsafe extern "C" fn run(
        context: *mut core::ffi::c_void,
    ) -> unchecked::fast_callback_396f37c4ec343c74_result {
        let inputs = unsafe { &mut *context.cast::<Inputs>() };
        let (first, second) = inputs.pairs[inputs.next];
        inputs.next += 1;
        unchecked::fast_callback_396f37c4ec343c74_result {
            v0: first,
            v1: second,
        }
    }
    let mut inputs = Inputs {
        pairs: (0..1000)
            .map(|_| unsafe {
                (
                    unchecked::fast_collection_resource_new(7),
                    unchecked::fast_collection_resource_new(9),
                )
            })
            .collect(),
        next: 0,
    };
    unsafe {
        let handler =
            unchecked::fast_alias_handler_new(unchecked::fast_callback_396f37c4ec343c74 {
                run: Some(run),
                context: (&mut inputs as *mut Inputs).cast(),
                drop: None,
            });
        let count = allocations(|| {
            for _ in 0..1000 {
                let mut result = 0;
                let mut error = core::ptr::null_mut();
                assert!(unchecked::fast_alias_handler_call(
                    handler,
                    &mut result,
                    &mut error
                ));
                assert_eq!(result, 16);
                assert!(error.is_null());
            }
        });
        unchecked::fast_drop_alias_handler(handler);
        assert_eq!(inputs.next, 1000);
        assert_eq!(
            count, 0,
            "fixed callback result bookkeeping must stay on the stack"
        );
    }
}

#[test]
fn collection_callback_results_allocate_only_the_final_rust_vector() {
    use super::unchecked;
    let _getters = super::GETTER_TESTS
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    type Row = unchecked::__PbRow_callback_8735da2fee137b57_callback_return_w1;
    struct Input {
        rows: Vec<Row>,
        next: usize,
    }
    unsafe extern "C" fn run(context: *mut core::ffi::c_void) -> unchecked::__PbRows {
        let input = unsafe { &mut *context.cast::<Input>() };
        let offset = input.next;
        input.next += 512;
        unchecked::__PbRows {
            data: unsafe { input.rows.as_ptr().add(offset) }.cast(),
            len: 512,
        }
    }
    let mut input = Input {
        rows: (0..(32 * 512))
            .map(|i| Row {
                v0: unsafe { unchecked::fast_collection_resource_new((i % 512) as i64) },
            })
            .collect(),
        next: 0,
    };
    let count = allocations(|| {
        for _ in 0..32 {
            let mut result = 0;
            let mut accepted = 0;
            let mut error = core::ptr::null_mut();
            unsafe {
                assert!(unchecked::fast_alias_callback_rows(
                    unchecked::fast_callback_8735da2fee137b57 {
                        run: Some(run),
                        context: (&mut input as *mut Input).cast(),
                        drop: None,
                        release_result: None
                    },
                    &mut result,
                    &mut accepted,
                    &mut error
                ));
            }
            assert_eq!(result, 511 * 512 / 2);
            assert_eq!(accepted, 1);
            assert!(error.is_null());
        }
    });
    assert_eq!(
        count, 32,
        "one final Vec per callback; no ownership-tracker allocation or growth"
    );
    assert_eq!(REALLOCATIONS.with(Cell::get), 0);
    assert_eq!(
        ALLOCATED_BYTES.with(Cell::get),
        32 * 512 * core::mem::size_of::<perftest_flat::OwnedResource>()
    );
}
