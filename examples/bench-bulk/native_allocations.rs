//! Benchmark-only native allocation accounting; disabled in latency builds.
#![allow(non_snake_case)]
use prebindgen_jni_runtime::jni::{
    objects::JClass,
    sys::{jboolean, jlongArray},
    JNIEnv,
};
use std::cell::Cell;
thread_local! {
    static ACTIVE: Cell<bool> = const { Cell::new(false) };
    static COUNTS: Cell<[u64; 3]> = const { Cell::new([0; 3]) };
}
#[cfg(feature = "native-allocations")]
mod allocator {
    use super::*;
    use std::alloc::{GlobalAlloc, Layout, System};
    struct Count;
    #[global_allocator]
    static ALLOCATOR: Count = Count;
    fn record(bytes: usize, realloc: bool) {
        let _ = ACTIVE.try_with(|active| {
            if active.get() {
                let _ = COUNTS.try_with(|counts| {
                    let mut value = counts.get();
                    let field = usize::from(realloc);
                    value[field] = value[field].wrapping_add(1);
                    value[2] = value[2].wrapping_add(bytes as u64);
                    counts.set(value);
                });
            }
        });
    }
    unsafe impl GlobalAlloc for Count {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let ptr = System.alloc(layout);
            if !ptr.is_null() {
                record(layout.size(), false);
            }
            ptr
        }
        unsafe fn alloc_zeroed(&self, layout: Layout) -> *mut u8 {
            let ptr = System.alloc_zeroed(layout);
            if !ptr.is_null() {
                record(layout.size(), false);
            }
            ptr
        }
        unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, size: usize) -> *mut u8 {
            let ptr = System.realloc(ptr, layout, size);
            if !ptr.is_null() {
                record(size, true);
            }
            ptr
        }
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            System.dealloc(ptr, layout);
        }
    }
}
#[no_mangle]
pub extern "system" fn Java_bench_allocations_Counters_start(
    _: JNIEnv<'_>,
    _: JClass<'_>,
) -> jboolean {
    COUNTS.with(|c| c.set([0; 3]));
    ACTIVE.with(|a| a.set(cfg!(feature = "native-allocations")));
    u8::from(cfg!(feature = "native-allocations"))
}
#[no_mangle]
pub extern "system" fn Java_bench_allocations_Counters_stop(
    env: JNIEnv<'_>,
    _: JClass<'_>,
) -> jlongArray {
    ACTIVE.with(|a| a.set(false));
    let values = COUNTS.with(|c| c.get().map(|n| n as i64));
    let Ok(array) = env.new_long_array(3) else {
        return std::ptr::null_mut();
    };
    if env.set_long_array_region(&array, 0, &values).is_err() {
        return std::ptr::null_mut();
    }
    array.into_raw()
}
