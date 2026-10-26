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

// Exercise runtime failures that generated public callers cannot manufacture.
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_SourceCallbackRuntimeProbe_pending(
    mut env: prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _: prebindgen_jni_runtime::jni::objects::JClass<'_>,
    object: prebindgen_jni_runtime::jni::objects::JObject<'_>,
) {
    let callback = prebindgen_jni_runtime::Callback::new(&mut env, &object, "()V").unwrap();
    env.throw_new("java/io/IOException", "outer exception")
        .unwrap();
    callback.with_env(16, |_| {
        panic!("callback must not run with an outer pending exception")
    });
}

#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_SourceCallbackRuntimeProbe_rejected(
    mut env: prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _: prebindgen_jni_runtime::jni::objects::JClass<'_>,
    object: prebindgen_jni_runtime::jni::objects::JObject<'_>,
) -> i64 {
    use prebindgen_jni_runtime::{jni::sys::jvalue, Allocation, Callback, CallbackOutputs};
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    struct Tracked(Arc<AtomicUsize>);
    impl Drop for Tracked {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }
    let count = Arc::new(AtomicUsize::new(0));
    let callback = Callback::new(&mut env, &object, "([ZJ)V").unwrap();
    callback.with_env(16, |env| {
        let accepted = env.new_boolean_array(1).map_err(|e| e.to_string())?;
        let pointer = Box::into_raw(Box::new(Allocation {
            value: Tracked(count.clone()),
        })) as i64;
        let mut outputs = CallbackOutputs::new();
        unsafe {
            outputs.push::<Tracked>(pointer);
        }
        outputs.arm();
        unsafe {
            callback.invoke(
                env,
                Some(&accepted),
                &[
                    jvalue {
                        l: accepted.as_raw(),
                    },
                    jvalue { j: pointer },
                ],
                &mut outputs,
            )
        }
    });
    count.load(Ordering::SeqCst) as i64
}
