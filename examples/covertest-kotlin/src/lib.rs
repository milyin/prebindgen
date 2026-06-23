// The generated JNI bindings, written by build.rs from perftest-flat's
// #[prebindgen] surface (the perf surface plus the `ext` coverage surface). The
// generated code refers to source types fully qualified through the
// `source_module` (e.g. `perftest_flat::Payload`), so no extra `use` is needed.
include!("generated_bindings.rs");

/// Hand-written destructor for `PayloadVecHandler` — the one type declared
/// `.suppress_kotlin_code()` in `build.rs`. That flag suppresses both its Kotlin
/// class *and* its generated Rust `freePtr` extern, so the coverage example owns
/// both sides: this mirrors the generated `freePtr` externs (`Box::from_raw` +
/// drop) and pairs with the hand-written companion in
/// `kotlin/io/prebindgen/covertest/PayloadVecHandler.kt`.
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_PayloadVecHandler_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::PayloadVecHandler));
    }
}
