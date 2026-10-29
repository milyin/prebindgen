//! Allocation budgets at the actual generated C boundary, isolated per test thread.
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct Allocator;
thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static ALLOCATIONS: Cell<usize> = const { Cell::new(0) };
}
fn record() {
    if COUNTING.try_with(Cell::get).unwrap_or(false) {
        let _ = ALLOCATIONS.try_with(|n| n.set(n.get().wrapping_add(1)));
    }
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record();
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record();
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
    COUNTING.with(|flag| flag.set(true));
    let stop = Stop;
    call();
    drop(stop);
    ALLOCATIONS.with(Cell::get)
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
