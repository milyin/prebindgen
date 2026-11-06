#![allow(non_camel_case_types, unused_variables)]
mod callback_frames;
mod diagnostics;
mod output_rows;
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
    row_class: prebindgen_jni_runtime::jni::objects::JClass<'_>,
) -> i64 {
    use prebindgen_jni_runtime::{array, handle_object, jni::sys::jvalue, Allocation, Callback};
    let count = if mode == 1 || mode == 3 { 2 } else { 1 };
    let descriptor = format!("({})V", "[Ljava/lang/Object;".repeat(count));
    let callback = Callback::new(&mut env, &object, &descriptor).unwrap();
    let row_type = prebindgen_jni_runtime::OutputRowType::new();
    row_type
        .initialize(&mut env, &row_class, &["J", "Ljava/lang/String;", "J"])
        .unwrap();
    let constructor = row_type.get().unwrap();
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
                let mut handles = Vec::new();
                for column in [0, 2] {
                    let allocation = Box::new(Allocation {
                        value: perftest_flat::callback_row_resource_new(
                            (index * 10 + column) as i64,
                        ),
                    });
                    handles.push((&*allocation as *const _) as i64);
                    allocations.push(allocation);
                }
                // Typed rows cannot hold an object of the wrong JVM field type.
                // A null in the non-null logical text field still exercises the
                // generated decoder's failed current row and unread-row cleanup.
                let text = if mode <= 1 && index == 1 && argument == count - 1 {
                    prebindgen_jni_runtime::jni::objects::JObject::null()
                } else {
                    env.new_string("valid row")
                        .map_err(|e| e.to_string())?
                        .into()
                };
                let row = unsafe {
                    constructor.construct(
                        env,
                        &[
                            jvalue { j: handles[0] },
                            jvalue { l: text.as_raw() },
                            jvalue { j: handles[1] },
                        ],
                    )
                }?;
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

mod buffer_allocations;

mod row_metadata;
