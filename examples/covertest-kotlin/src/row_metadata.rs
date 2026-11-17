//! Test-only tracing of the real row readers on the invoking JNI thread.
use prebindgen_jni_runtime::{
    jni::{
        objects::{GlobalRef, JClass, JObjectArray},
        sys::*,
        JNIEnv as Env,
    },
    LocalFrame,
};
use std::{cell::RefCell, ffi::c_char};

struct Trace {
    original: *const JNINativeInterface_,
    oom: jclass,
    fail: jint,
    counts: [jlong; 7],
}
thread_local! { static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) }; }
fn event(index: usize, operation: jint) -> (*const JNINativeInterface_, jclass, bool, bool) {
    TRACE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let t = slot.as_mut().expect("metadata trace installed");
        t.counts[index] += 1;
        let inject = t.fail == operation || operation == 3 && t.fail == 4;
        let pending = t.fail != 4;
        if inject {
            t.fail = 0;
        }
        (t.original, t.oom, inject, pending)
    })
}
unsafe fn fail(env: *mut JNIEnv, original: *const JNINativeInterface_, oom: jclass) {
    ((*original).ThrowNew.unwrap())(
        env,
        oom,
        c"injected metadata initialization failure".as_ptr(),
    );
}
unsafe extern "system" fn find_class(env: *mut JNIEnv, name: *const c_char) -> jclass {
    let (o, oom, inject, _) = event(0, 1);
    if inject {
        fail(env, o, oom);
        std::ptr::null_mut()
    } else {
        ((*o).FindClass.unwrap())(env, name)
    }
}
unsafe extern "system" fn object_class(env: *mut JNIEnv, object: jobject) -> jclass {
    let (o, _, _, _) = event(1, -1);
    ((*o).GetObjectClass.unwrap())(env, object)
}
unsafe extern "system" fn method_id(
    env: *mut JNIEnv,
    class: jclass,
    name: *const c_char,
    sig: *const c_char,
) -> jmethodID {
    let (o, oom, inject, _) = event(2, 2);
    if inject {
        fail(env, o, oom);
        std::ptr::null_mut()
    } else {
        ((*o).GetMethodID.unwrap())(env, class, name, sig)
    }
}
unsafe extern "system" fn global_ref(env: *mut JNIEnv, object: jobject) -> jobject {
    let (o, oom, inject, pending) = event(3, 3);
    if inject {
        if pending {
            fail(env, o, oom);
        }
        std::ptr::null_mut()
    } else {
        ((*o).NewGlobalRef.unwrap())(env, object)
    }
}
unsafe extern "system" fn delete_global(env: *mut JNIEnv, object: jobject) {
    let (o, _, _, _) = event(4, -1);
    ((*o).DeleteGlobalRef.unwrap())(env, object)
}
unsafe extern "system" fn instance_of(
    env: *mut JNIEnv,
    object: jobject,
    class: jclass,
) -> jboolean {
    let (o, _, _, _) = event(5, -1);
    ((*o).IsInstanceOf.unwrap())(env, object, class)
}
macro_rules! call_trace {
    ($name:ident,$entry:ident,$result:ty) => {
        unsafe extern "system" fn $name(
            env: *mut JNIEnv,
            object: jobject,
            method: jmethodID,
            args: *const jvalue,
        ) -> $result {
            let (o, _, _, _) = event(6, -1);
            ((*o).$entry.unwrap())(env, object, method, args)
        }
    };
}
call_trace!(byte_call, CallByteMethodA, jbyte);
call_trace!(short_call, CallShortMethodA, jshort);
call_trace!(int_call, CallIntMethodA, jint);
call_trace!(long_call, CallLongMethodA, jlong);
call_trace!(float_call, CallFloatMethodA, jfloat);
call_trace!(double_call, CallDoubleMethodA, jdouble);
call_trace!(boolean_call, CallBooleanMethodA, jboolean);
struct Hook {
    env: *mut JNIEnv,
    original: *const JNINativeInterface_,
    _table: Box<JNINativeInterface_>,
    _oom: GlobalRef,
}
impl Hook {
    unsafe fn new(env: &mut Env<'_>, failure: jint) -> prebindgen_runtime::Result<Self> {
        let oom = env
            .find_class("java/lang/OutOfMemoryError")
            .map_err(|e| e.to_string())?;
        let oom = env.new_global_ref(oom).map_err(|e| e.to_string())?;
        let raw = env.get_native_interface();
        let original = *raw;
        let mut table = Box::new(*original);
        table.FindClass = Some(find_class);
        table.GetObjectClass = Some(object_class);
        table.GetMethodID = Some(method_id);
        table.NewGlobalRef = Some(global_ref);
        table.DeleteGlobalRef = Some(delete_global);
        table.IsInstanceOf = Some(instance_of);
        table.CallByteMethodA = Some(byte_call);
        table.CallShortMethodA = Some(short_call);
        table.CallIntMethodA = Some(int_call);
        table.CallLongMethodA = Some(long_call);
        table.CallFloatMethodA = Some(float_call);
        table.CallDoubleMethodA = Some(double_call);
        table.CallBooleanMethodA = Some(boolean_call);
        TRACE.with(|slot| {
            assert!(slot.borrow().is_none());
            *slot.borrow_mut() = Some(Trace {
                original,
                oom: oom.as_obj().as_raw(),
                fail: failure,
                counts: [0; 7],
            });
        });
        *raw = &*table;
        Ok(Self {
            env: raw,
            original,
            _table: table,
            _oom: oom,
        })
    }
    fn counts(&self) -> [jlong; 7] {
        TRACE.with(|slot| slot.borrow().as_ref().unwrap().counts)
    }
}
impl Drop for Hook {
    fn drop(&mut self) {
        unsafe {
            *self.env = self.original;
        }
        TRACE.with(|slot| *slot.borrow_mut() = None);
    }
}
fn value<'local>(
    env: &mut Env<'local>,
    row: &JObjectArray<'local>,
    kind: jint,
) -> prebindgen_runtime::Result<jlong> {
    use prebindgen_jni_runtime::{
        row_bool, row_bytes, row_f32, row_f64, row_i16, row_i32, row_i64, row_i8, row_rows,
        row_text,
    };
    match kind {
        0 => row_i8(env, row, 0).map(jlong::from),
        1 => row_i16(env, row, 0).map(jlong::from),
        2 => row_i32(env, row, 0).map(jlong::from),
        3 => row_i64(env, row, 0),
        4 => row_f32(env, row, 0).map(|v| v.to_bits() as jlong),
        5 => row_f64(env, row, 0).map(|v| v.to_bits() as jlong),
        6 => row_bool(env, row, 0).map(jlong::from),
        7 => row_text(env, row, 0).map(|v| if v.is_null() { -1 } else { 1 }),
        8 => row_bytes(env, row, 0).map(|v| if v.is_null() { -1 } else { 1 }),
        9 => row_rows(env, row, 0).map(|v| if v.is_null() { -1 } else { 1 }),
        10 => prebindgen_jni_runtime::row(env, row, 0, 1).map(|_| 1),
        _ => Err("unknown row metadata probe".into()),
    }
}
fn probe<'local>(
    env: &mut Env<'local>,
    row: &JObjectArray<'local>,
    kind: jint,
    failure: jint,
    repeats: jint,
) -> prebindgen_runtime::Result<Vec<jlong>> {
    let hook = unsafe { Hook::new(env, failure) }?;
    let mut result = Ok(0);
    for _ in 0..repeats {
        let frame = unsafe { LocalFrame::new(env, 16) }?;
        result = value(env, row, kind);
        drop(frame);
        if result.is_err() {
            break;
        }
    }
    let counts = hook.counts();
    drop(hook);
    let pending = env.exception_check().map_err(|e| e.to_string())?;
    if pending {
        env.exception_clear().map_err(|e| e.to_string())?;
    }
    let mut report = vec![jlong::from(result.is_ok()), result.unwrap_or_default()];
    report.extend(counts);
    report.push(jlong::from(pending));
    Ok(report)
}
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_RowMetadataProbe_read<'local>(
    mut env: Env<'local>,
    _: JClass<'_>,
    kind: jint,
    rows: JObjectArray<'local>,
    failure: jint,
    worker: jboolean,
    repeats: jint,
) -> jlongArray {
    let result = prebindgen_runtime::boundary(|| {
        if worker == JNI_FALSE {
            return probe(&mut env, &rows, kind, failure, repeats);
        }
        let vm = env.get_java_vm().map_err(|e| e.to_string())?;
        let keep = env.new_global_ref(&rows).map_err(|e| e.to_string())?;
        let rows = keep.clone();
        let result = std::thread::spawn(move || {
            let mut env = vm.attach_current_thread().map_err(|e| e.to_string())?;
            let local = env
                .new_local_ref(rows.as_obj())
                .map_err(|e| e.to_string())?;
            probe(&mut env, &JObjectArray::from(local), kind, failure, repeats)
        })
        .join()
        .map_err(|_| "metadata probe worker panicked".to_string())?;
        drop(keep);
        result
    });
    match result.and_then(|values| {
        let array = env
            .new_long_array(values.len() as i32)
            .map_err(|e| e.to_string())?;
        env.set_long_array_region(&array, 0, &values)
            .map_err(|e| e.to_string())?;
        Ok(array.into_raw())
    }) {
        Ok(array) => array,
        Err(message) => {
            prebindgen_jni_runtime::fail(&mut env, &message);
            std::ptr::null_mut()
        }
    }
}
