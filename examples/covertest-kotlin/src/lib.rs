#![allow(non_camel_case_types, unused_variables)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Runtime probes cover pending exceptions in the binding-packet path and
// preserve coverage for the explicitly callable native exception helper.
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
    assert!(prebindgen_jni_runtime::binding_packet(&mut env, "replacement").is_err());
    prebindgen_jni_runtime::fail(&mut env, "fallback");
}
