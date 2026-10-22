#![allow(non_camel_case_types, unused_variables)]
include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

// Binding-error delivery and its native fallback must preserve pending JVM exceptions.
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_ResultRuntimeProbe_pendingException(
    mut env: prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _: prebindgen_jni_runtime::jni::objects::JClass<'_>,
) {
    env.throw_new("java/io/IOException", "original").unwrap();
    assert!(prebindgen_jni_runtime::binding_packet(&mut env, "replacement").is_err());
    prebindgen_jni_runtime::fail(&mut env, "fallback");
}
