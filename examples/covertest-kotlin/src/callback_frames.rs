//! Test-only HotSpot JNI-table interception, restored by RAII before references drop.
//! Count frames enclosing collector upcalls and reject a primitive handle row.
use prebindgen_jni_runtime::jni::{
    objects::{GlobalRef, JClass, JValue},
    sys::*,
    JNIEnv as Env,
};
use std::cell::RefCell;

struct Trace {
    original: *const JNINativeInterface_,
    exception: jthrowable,
    sink_class: jclass,
    frames: Vec<usize>,
    reject: jint,
    reserve_method: jmethodID,
    counts: [jlong; 6],
}
thread_local! { static TRACE: RefCell<Option<Trace>> = const { RefCell::new(None) }; }

unsafe extern "system" fn push(env: *mut JNIEnv, capacity: jint) -> jint {
    let original = TRACE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let trace = slot.as_mut().expect("installed frame trace");
        trace.frames.push(0);
        trace.original
    });
    ((*original).PushLocalFrame.unwrap())(env, capacity)
}

unsafe extern "system" fn pop(env: *mut JNIEnv, result: jobject) -> jobject {
    let original = TRACE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let trace = slot.as_mut().expect("installed frame trace");
        if trace.frames.pop().expect("matched frame pop") > 0 {
            trace.counts[0] += 1;
            trace.counts[1] += 1;
        }
        trace.original
    });
    ((*original).PopLocalFrame.unwrap())(env, result)
}

unsafe extern "system" fn upcall(
    env: *mut JNIEnv,
    object: jobject,
    method: jmethodID,
    args: *const jvalue,
) {
    let (original, sink_class) = TRACE.with(|slot| {
        let slot = slot.borrow();
        let trace = slot.as_ref().expect("installed frame trace");
        (trace.original, trace.sink_class)
    });
    // Match the generated collector interface, not the order of void upcalls.
    let sink = ((*original).IsInstanceOf.unwrap())(env, object, sink_class) != 0;
    let (exception, reject, reserve) = TRACE.with(|slot| {
        let mut slot = slot.borrow_mut();
        let trace = slot.as_mut().expect("installed frame trace");
        let reserve = sink && method == trace.reserve_method;
        let reject = if reserve {
            assert_eq!(trace.counts[2], 0, "reserve precedes every row");
            trace.counts[4] += 1;
            trace.reject == 7
        } else if sink {
            trace.counts[2] += 1;
            if let Some(frame) = trace.frames.last_mut() {
                *frame += 1;
            }
            trace.reject == 5 && trace.counts[2] == 2
        } else {
            false
        };
        if reject {
            trace.counts[3] += 1;
        }
        (trace.exception, reject, reserve)
    });
    if reject {
        // Synthetic IOException before reservation or row acceptance, testing
        // the same ownership unwind as an allocation failure (not a real OOM).
        ((*original).Throw.unwrap())(env, exception);
    } else {
        ((*original).CallVoidMethodA.unwrap())(env, object, method, args);
        if reserve && ((*original).ExceptionCheck.unwrap())(env) == JNI_FALSE {
            // HotSpot-only test introspection: reservation must have allocated
            // enough reference slots before the first row can be accepted.
            let capacity = collector_capacity(env, object).expect("read collector capacity");
            TRACE.with(|slot| slot.borrow_mut().as_mut().unwrap().counts[5] = capacity);
        }
    }
}

unsafe fn collector_capacity(
    raw: *mut JNIEnv,
    object: jobject,
) -> prebindgen_runtime::Result<jlong> {
    let mut env = Env::from_raw(raw).map_err(|e| e.to_string())?;
    let object = prebindgen_jni_runtime::jni::objects::JObject::from_raw(object);
    let rows = env
        .get_field(&object, "rows", "Ljava/util/ArrayList;")
        .and_then(|v| v.l())
        .map_err(|e| e.to_string())?;
    let rows = env.auto_local(rows);
    let data = env
        .get_field(&*rows, "elementData", "[Ljava/lang/Object;")
        .and_then(|v| v.l())
        .map_err(|e| e.to_string())?;
    let data = env.auto_local(prebindgen_jni_runtime::jni::objects::JObjectArray::from(
        data,
    ));
    env.get_array_length(&*data)
        .map(jlong::from)
        .map_err(|e| e.to_string())
}

struct Hook {
    env: *mut JNIEnv,
    original: *const JNINativeInterface_,
    _table: Box<JNINativeInterface_>,
    _exception: GlobalRef,
    _sink_class: GlobalRef,
}
impl Hook {
    unsafe fn install(
        env: &mut Env<'_>,
        reject: jint,
        reserve_method: jmethodID,
        exception: GlobalRef,
        sink_class: GlobalRef,
    ) -> Self {
        let raw = env.get_native_interface();
        let original = *raw;
        let mut table = Box::new(*original);
        table.PushLocalFrame = Some(push);
        table.PopLocalFrame = Some(pop);
        table.CallVoidMethodA = Some(upcall);
        TRACE.with(|slot| {
            assert!(slot.borrow().is_none(), "nested frame tracing");
            *slot.borrow_mut() = Some(Trace {
                original,
                exception: exception.as_obj().as_raw(),
                sink_class: sink_class.as_obj().as_raw(),
                frames: Vec::new(),
                reject,
                reserve_method,
                counts: [0; 6],
            });
        });
        *raw = &*table;
        Self {
            env: raw,
            original,
            _table: table,
            _exception: exception,
            _sink_class: sink_class,
        }
    }
    fn counts(&self) -> [jlong; 6] {
        TRACE.with(|slot| {
            let slot = slot.borrow();
            let trace = slot.as_ref().expect("installed frame trace");
            assert!(trace.frames.is_empty(), "unbalanced JNI frames");
            trace.counts
        })
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

#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_CallbackFrameProbe_measure(
    mut env: Env<'_>,
    _: JClass<'_>,
    mode: jint,
    size: jint,
) -> jlongArray {
    let result = prebindgen_runtime::boundary(|| {
        if !(0..=10).contains(&mode) || !(0..=20000).contains(&size) {
            return Err("invalid frame probe".into());
        }
        let class = env
            .find_class("io/prebindgen/covertest/CallbackFrameEntryPoint")
            .map_err(|e| e.to_string())?;
        // Initialize generated captures/layouts and exercise the path before tracing.
        let warm_mode = if mode == 5 || mode == 7 { 1 } else { mode };
        env.call_static_method(
            &class,
            "run",
            "(II)V",
            &[JValue::Int(warm_mode), JValue::Int(size)],
        )
        .map_err(|e| e.to_string())?;
        let exception = env
            .get_static_field(&class, "rejected", "Ljava/io/IOException;")
            .and_then(|value| value.l())
            .map_err(|e| e.to_string())?;
        let exception = env.new_global_ref(exception).map_err(|e| e.to_string())?;
        let sink_class = env
            .call_static_method(
                &class,
                "collectorClass",
                "(I)Ljava/lang/Class;",
                &[JValue::Int(mode)],
            )
            .and_then(|value| value.l())
            .map_err(|e| e.to_string())?;
        let collector_class: &JClass<'_> = (&sink_class).into();
        let reserve_method = env
            .get_method_id(collector_class, "reserve", "(I)V")
            .map_err(|e| e.to_string())?
            .into_raw();
        let sink_class = env.new_global_ref(sink_class).map_err(|e| e.to_string())?;
        let hook = unsafe { Hook::install(&mut env, mode, reserve_method, exception, sink_class) };
        let run = env.call_static_method(
            &class,
            "run",
            "(II)V",
            &[JValue::Int(mode), JValue::Int(size)],
        );
        let counts = hook.counts();
        drop(hook);
        run.map_err(|e| e.to_string())?;
        let result = env
            .new_long_array(counts.len() as jint)
            .map_err(|e| e.to_string())?;
        env.set_long_array_region(&result, 0, &counts)
            .map_err(|e| e.to_string())?;
        Ok(result.into_raw())
    });
    match result {
        Ok(result) => result,
        Err(message) => {
            prebindgen_jni_runtime::fail(&mut env, &message);
            std::ptr::null_mut()
        }
    }
}
