//! Count full-buffer-sized native allocations on the invoking JVM test thread.
//! The baseline uses the same runtime encoders, including JNI string encoding.
use prebindgen_jni_runtime::jni::{objects::JClass, sys::jobject, JNIEnv};
use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;

struct Allocator;
thread_local! {
    static COUNTING: Cell<bool> = const { Cell::new(false) };
    static LARGE_ALLOCATIONS: Cell<i64> = const { Cell::new(0) };
}
fn record(size: usize) {
    if size >= 65536 && COUNTING.try_with(Cell::get).unwrap_or(false) {
        let _ = LARGE_ALLOCATIONS.try_with(|count| count.set(count.get().wrapping_add(1)));
    }
}
#[global_allocator]
static ALLOCATOR: Allocator = Allocator;
unsafe impl GlobalAlloc for Allocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        record(layout.size());
        unsafe { System.alloc(layout) }
    }
    unsafe fn dealloc(&self, pointer: *mut u8, layout: Layout) {
        unsafe { System.dealloc(pointer, layout) }
    }
    unsafe fn realloc(&self, pointer: *mut u8, layout: Layout, size: usize) -> *mut u8 {
        record(size);
        unsafe { System.realloc(pointer, layout, size) }
    }
}
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_BufferAllocationProbe_start(
    _: JNIEnv<'_>,
    _: JClass<'_>,
) {
    LARGE_ALLOCATIONS.with(|count| count.set(0));
    COUNTING.with(|flag| flag.set(true));
}
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_BufferAllocationProbe_stop(
    _: JNIEnv<'_>,
    _: JClass<'_>,
) -> i64 {
    COUNTING.with(|flag| flag.set(false));
    LARGE_ALLOCATIONS.with(Cell::get)
}
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_BufferAllocationProbe_directText<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
) -> jobject {
    prebindgen_jni_runtime::text_object(&mut env, perftest_flat::jni_buffer_text())
        .unwrap()
        .into_raw()
}
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_BufferAllocationProbe_directBytes<'a>(
    mut env: JNIEnv<'a>,
    _: JClass<'a>,
) -> jobject {
    prebindgen_jni_runtime::bytes_object(&mut env, perftest_flat::jni_buffer_bytes())
        .unwrap()
        .into_raw()
}
