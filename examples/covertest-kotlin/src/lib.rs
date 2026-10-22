#![allow(non_camel_case_types, unused_variables)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Runtime failure probes exercise exception lookup and pending-exception
// preservation with the same helper generated Result wrappers use.
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_ResultRuntimeProbe_missingClass(
    mut env: prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _: prebindgen_jni_runtime::jni::objects::JClass<'_>,
) {
    let _ = prebindgen_jni_runtime::domain_error(&mut env, "missing/Exception", "missing".into());
    prebindgen_jni_runtime::fail(&mut env, "fallback must not replace the lookup exception");
}
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_ResultRuntimeProbe_pendingException(
    mut env: prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _: prebindgen_jni_runtime::jni::objects::JClass<'_>,
) {
    env.throw_new("java/io/IOException", "original").unwrap();
    let _ = prebindgen_jni_runtime::domain_error(
        &mut env,
        "java/lang/IllegalStateException",
        "replacement".into(),
    );
    prebindgen_jni_runtime::fail(&mut env, "fallback");
}
