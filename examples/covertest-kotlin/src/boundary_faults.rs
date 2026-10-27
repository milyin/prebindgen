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
    armed: bool,
    failures: jint,
}
thread_local! { static FAULT: RefCell<Option<Fault>> = const { RefCell::new(None) }; }

fn current() -> Option<(Fault, bool)> {
    FAULT.with(|slot| {
        let mut slot = slot.borrow_mut();
        let fault = slot.as_mut()?;
        let inject = fault.armed;
        fault.armed = false;
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
        if mode == 0 {
            table.PushLocalFrame = Some(fail_frame);
        } else {
            table.NewObjectArray = Some(fail_array);
        }
        FAULT.with(|slot| {
            *slot.borrow_mut() = Some(Fault {
                original,
                oom: oom.as_obj().as_raw(),
                armed: true,
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
        if !(0..=1).contains(&mode) {
            return Err("unknown allocation probe".into());
        }
        // Resolve the Java entry point before arming the table hook. Only the
        // generated callback's first frame/array allocation is supposed to fail.
        let class = env
            .find_class("io/prebindgen/covertest/CallbackFailureEntryPoint")
            .map_err(|e| e.to_string())?;
        let hook = unsafe { Hook::install(&mut env, mode) }?;
        let result = env.call_static_method(&class, "run", "()V", &[]);
        let failures = FAULT.with(|slot| slot.borrow().as_ref().map_or(0, |fault| fault.failures));
        drop(hook);
        result.map_err(|e| e.to_string())?;
        Ok(failures)
    })
    .unwrap_or(-1)
}
