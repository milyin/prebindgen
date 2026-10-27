#![allow(non_camel_case_types, unused_variables)]
mod diagnostics;
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

// Feed valid handle addresses plus an invalid field, row or null carrier into
// the compiled bridge to test cleanup without losing later rows or arguments.
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_CallbackRowsRuntimeProbe_malformed(
    mut env: prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _: prebindgen_jni_runtime::jni::objects::JClass<'_>,
    object: prebindgen_jni_runtime::jni::objects::JObject<'_>,
    mode: prebindgen_jni_runtime::jni::sys::jint,
) -> i64 {
    use prebindgen_jni_runtime::{array, handle_object, jni::sys::jvalue, Allocation, Callback};
    let count = if mode == 1 || mode == 3 { 2 } else { 1 };
    let descriptor = format!("({})V", "[Ljava/lang/Object;".repeat(count));
    let callback = Callback::new(&mut env, &object, &descriptor).unwrap();
    let mut failed = false;
    callback.with_env(128, |env| {
        let mut allocations = vec![];
        let mut arrays = vec![];
        for argument in 0..count {
            if (mode == 3 && argument == 0) || mode == 5 {
                arrays.push(prebindgen_jni_runtime::jni::objects::JObjectArray::from(
                    prebindgen_jni_runtime::jni::objects::JObject::null(),
                ));
                continue;
            }
            let rows = array(env, 3)?;
            for index in 0..3 {
                if mode == 2 && index == 1 {
                    // No allocations belong to this deliberately malformed row.
                    let invalid = handle_object(env, 123)?;
                    env.set_object_array_element(&rows, index, invalid)
                        .map_err(|e| e.to_string())?;
                    continue;
                }
                let row = array(env, 3)?;
                for column in [0, 2] {
                    let allocation = Box::new(Allocation {
                        value: perftest_flat::callback_row_resource_new(
                            (index * 10 + column) as i64,
                        ),
                    });
                    let pointer = (&*allocation as *const _) as u64;
                    allocations.push(allocation);
                    let value = handle_object(env, pointer)?;
                    env.set_object_array_element(&row, column, value)
                        .map_err(|e| e.to_string())?;
                }
                if mode <= 1 && index == 1 && argument == count - 1 {
                    let invalid = handle_object(env, 123)?;
                    env.set_object_array_element(&row, 1, invalid)
                        .map_err(|e| e.to_string())?;
                } else {
                    let text = env.new_string("valid row").map_err(|e| e.to_string())?;
                    env.set_object_array_element(&row, 1, text)
                        .map_err(|e| e.to_string())?;
                }
                env.set_object_array_element(&rows, index, row)
                    .map_err(|e| e.to_string())?;
            }
            arrays.push(rows);
        }
        let arguments = arrays
            .iter()
            .map(|array| jvalue { l: array.as_raw() })
            .collect::<Vec<_>>();
        // Prepare every row before transferring any native allocation to Kotlin.
        for allocation in allocations {
            let _ = Box::into_raw(allocation);
        }
        let result = unsafe { callback.invoke(env, &arguments) };
        failed = result.is_err();
        result
    });
    if mode >= 4 && failed {
        -1
    } else {
        perftest_flat::callback_row_resource_live()
    }
}

mod boundary_faults;
