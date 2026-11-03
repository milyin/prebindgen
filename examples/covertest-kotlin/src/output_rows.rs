//! Rollback must preserve a pending exception and continue after a destructor panic.
use prebindgen_jni_runtime::{
    jni::{
        objects::{JClass, JObjectArray},
        JNIEnv,
    },
    Allocation, OutputRows,
};

fn release<'local>(env: &mut JNIEnv<'local>, row: &JObjectArray<'local>) {
    prebindgen_jni_runtime::reclaim_output(env, |env| {
        let raw = unsafe { prebindgen_jni_runtime::output_handle(env, row, 0)? };
        let _ = prebindgen_runtime::boundary(|| unsafe {
            prebindgen_jni_runtime::drop_pointer::<perftest_flat::CallbackRowResource>(raw)
        });
        Ok(())
    });
}

#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_JniOutputRuntimeProbe_rollback(
    mut env: JNIEnv<'_>,
    _: JClass<'_>,
) {
    let result = prebindgen_runtime::boundary(|| {
        let rows = prebindgen_jni_runtime::array(&mut env, 2)?;
        let mut owner = unsafe { OutputRows::new(&mut env, &rows, release)? };
        for (index, id) in [777, 778].into_iter().enumerate() {
            let value = Box::new(Allocation {
                value: perftest_flat::callback_row_resource_new(id),
            });
            let row = prebindgen_jni_runtime::array(&mut env, 1)?;
            let handle = prebindgen_jni_runtime::handle_object(
                &mut env,
                (&*value as *const Allocation<_>) as u64,
            )?;
            env.set_object_array_element(&row, 0, handle)
                .map_err(|e| e.to_string())?;
            env.set_object_array_element(&rows, index as i32, &row)
                .map_err(|e| e.to_string())?;
            std::mem::forget(value);
            unsafe {
                owner.complete_row();
            }
        }
        env.throw_new("java/io/IOException", "original output rollback")
            .map_err(|e| e.to_string())?;
        drop(owner);
        Ok(())
    });
    if let Err(message) = result {
        prebindgen_jni_runtime::fail(&mut env, &message);
    }
}
