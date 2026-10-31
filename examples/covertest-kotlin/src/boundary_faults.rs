//! Test-only JNI allocation failures on the invoking thread, restored with RAII.
use prebindgen_jni_runtime::jni::{
    objects::{GlobalRef, JClass},
    sys::*,
};
use std::cell::RefCell;

#[derive(Clone, Copy)]
struct Fault {
    original: *const JNINativeInterface_,
    oom: jclass,
    remaining: jint,
    failures: jint,
}
thread_local! { static FAULT: RefCell<Option<Fault>> = const { RefCell::new(None) }; }

fn current() -> Option<(Fault, bool)> {
    FAULT.with(|slot| {
        let mut slot = slot.borrow_mut();
        let fault = slot.as_mut()?;
        let inject = fault.remaining == 1;
        fault.remaining = (fault.remaining - 1).max(0);
        if inject {
            fault.failures += 1;
        }
        Some((*fault, inject))
    })
}
unsafe fn out_of_memory(env: *mut JNIEnv, fault: Fault) {
    if let Some(throw) = (*fault.original).ThrowNew {
        throw(
            env,
            fault.oom,
            c"injected callback allocation failure".as_ptr(),
        );
    }
}
unsafe extern "system" fn fail_frame(env: *mut JNIEnv, capacity: jint) -> jint {
    let Some((fault, inject)) = current() else {
        return JNI_ERR;
    };
    if inject {
        out_of_memory(env, fault);
        JNI_ERR
    } else {
        ((*fault.original).PushLocalFrame.unwrap())(env, capacity)
    }
}
unsafe extern "system" fn fail_array(
    env: *mut JNIEnv,
    length: jsize,
    class: jclass,
    value: jobject,
) -> jobjectArray {
    let Some((fault, inject)) = current() else {
        return std::ptr::null_mut();
    };
    if inject {
        out_of_memory(env, fault);
        std::ptr::null_mut()
    } else {
        ((*fault.original).NewObjectArray.unwrap())(env, length, class, value)
    }
}

unsafe extern "system" fn fail_string(env: *mut JNIEnv, text: *const std::ffi::c_char) -> jstring {
    let Some((fault, inject)) = current() else {
        return std::ptr::null_mut();
    };
    if inject {
        out_of_memory(env, fault);
        std::ptr::null_mut()
    } else {
        ((*fault.original).NewStringUTF.unwrap())(env, text)
    }
}
unsafe extern "system" fn fail_bytes(env: *mut JNIEnv, length: jsize) -> jbyteArray {
    let Some((fault, inject)) = current() else {
        return std::ptr::null_mut();
    };
    if inject {
        out_of_memory(env, fault);
        std::ptr::null_mut()
    } else {
        ((*fault.original).NewByteArray.unwrap())(env, length)
    }
}

struct Hook {
    env: *mut JNIEnv,
    original: *const JNINativeInterface_,
    _table: Box<JNINativeInterface_>,
    _oom: GlobalRef,
}
impl Hook {
    unsafe fn install(
        env: &mut prebindgen_jni_runtime::jni::JNIEnv<'_>,
        mode: jint,
    ) -> prebindgen_runtime::Result<Self> {
        let oom = env
            .find_class("java/lang/OutOfMemoryError")
            .map_err(|e| e.to_string())?;
        let oom = env.new_global_ref(oom).map_err(|e| e.to_string())?;
        let raw = env.get_native_interface();
        let original = *raw;
        let mut table = Box::new(*original);
        match mode {
            0 => table.PushLocalFrame = Some(fail_frame),
            // Materialized collections use object arrays or a bulk byte frame.
            1 => {
                table.NewObjectArray = Some(fail_array);
                table.NewByteArray = Some(fail_bytes);
            }
            2 | 4 | 6 | 8 => table.NewStringUTF = Some(fail_string),
            3 | 5 | 7 | 9 => table.NewByteArray = Some(fail_bytes),
            _ => unreachable!(),
        }
        FAULT.with(|slot| {
            *slot.borrow_mut() = Some(Fault {
                original,
                oom: oom.as_obj().as_raw(),
                remaining: if mode >= 8 { 2 } else { 1 },
                failures: 0,
            })
        });
        *raw = &*table;
        Ok(Self {
            env: raw,
            original,
            _table: table,
            _oom: oom,
        })
    }
}
impl Drop for Hook {
    fn drop(&mut self) {
        unsafe {
            *self.env = self.original;
        }
        FAULT.with(|slot| *slot.borrow_mut() = None);
    }
}

#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_CallbackFailureProbe_exercise(
    mut env: prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _: JClass<'_>,
    mode: jint,
) -> jint {
    prebindgen_runtime::boundary(|| {
        if !(0..=9).contains(&mode) {
            return Err("unknown allocation probe".into());
        }
        // Resolve Java entry points before arming the selected JNI allocation
        // hook. Collection probes skip the first successfully prepared row.
        let class = env
            .find_class(if mode < 2 {
                "io/prebindgen/covertest/CallbackFailureEntryPoint"
            } else if mode < 4 {
                "io/prebindgen/covertest/BufferFailureEntryPoint"
            } else {
                "io/prebindgen/covertest/BufferReturnFailureEntryPoint"
            })
            .map_err(|e| e.to_string())?;
        let hook = unsafe { Hook::install(&mut env, mode) }?;
        let result = if mode < 4 {
            env.call_static_method(&class, "run", "()V", &[])
        } else {
            env.call_static_method(
                &class,
                "run",
                "(I)V",
                &[prebindgen_jni_runtime::jni::objects::JValue::Int(mode)],
            )
        };
        let failures = FAULT.with(|slot| slot.borrow().as_ref().map_or(0, |fault| fault.failures));
        drop(hook);
        result.map_err(|e| e.to_string())?;
        Ok(failures)
    })
    .unwrap_or(-1)
}
