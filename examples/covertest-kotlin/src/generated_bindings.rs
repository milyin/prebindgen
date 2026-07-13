#[allow(dead_code)]
pub(crate) type __JniErr = ::prebindgen::lang::JniBindingError<()>;
/// See module-level docs at [`owned_object_prerequisite_items`].
#[allow(dead_code)]
pub(crate) struct OwnedObject<T: ?Sized> {
    ptr: *const T,
}
impl<T: ?Sized> std::ops::Deref for OwnedObject<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        unsafe { &*self.ptr }
    }
}
impl<T: ?Sized> std::ops::DerefMut for OwnedObject<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.ptr as *mut T) }
    }
}
impl<T: ?Sized> OwnedObject<T> {
    /// Borrow a `T` whose backing `Box<T>` lives on the
    /// Java side. Stores only the pointer; the wrapper
    /// does not own the heap allocation and never frees
    /// it on drop.
    ///
    /// # Safety
    ///
    /// `ptr` must be the result of an earlier
    /// `Box::into_raw(Box::new(v))` and the allocation
    /// must still be live (Java still owns it). The Java
    /// side is responsible for sequencing this call
    /// against any concurrent free or consume (via
    /// `NativeHandle.withPtr` read-lock vs `consume` /
    /// `close` write-lock) so the borrow cannot race a
    /// deallocation on the same pointer.
    #[allow(dead_code)]
    pub(crate) unsafe fn from_raw(ptr: *const T) -> Self {
        Self { ptr }
    }
}
#[allow(non_snake_case, dead_code)]
pub(crate) fn signal_error(
    env: &mut jni::JNIEnv,
    sink: &jni::objects::JObject,
    mid: &::prebindgen::lang::CachedIfaceMethod,
    fqn: &str,
    descr: &str,
    je: ::core::option::Option<&str>,
    ze: &[jni::sys::jvalue],
) {
    if env.exception_check().unwrap_or(false) {
        return;
    }
    let __je: jni::objects::JObject = match je {
        ::core::option::Option::Some(__m) => {
            match env.new_string(__m) {
                Ok(s) => s.into(),
                Err(e) => {
                    tracing::error!("signal_error: new_string failed: {}", e);
                    return;
                }
            }
        }
        ::core::option::Option::None => jni::objects::JObject::null(),
    };
    let mut __args: ::std::vec::Vec<jni::sys::jvalue> = ::std::vec::Vec::with_capacity(
        1 + ze.len(),
    );
    __args
        .push(jni::sys::jvalue {
            l: __je.as_raw(),
        });
    __args.extend_from_slice(ze);
    if let Err(e) = mid.call_object(env, fqn, "run", descr, sink, &__args) {
        tracing::error!("signal_error: error-callback invoke failed: {}", e);
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_PayloadHandler_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::PayloadHandler));
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_PayloadVecHandler_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::PayloadVecHandler));
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_StorageHandler_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::StorageHandler));
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_Storage_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::Storage));
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_analytics_SummaryVault_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::Archive));
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_analytics_Summary_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::Summary));
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_errors_StorageError_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::StorageError));
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_payloadVecFree(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    handle: jni::sys::jlong,
) {
    if handle != 0 {
        drop(Box::from_raw(handle as *mut Vec<perftest_flat::Payload>));
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_payloadVecNew(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    cap: jni::sys::jint,
) -> jni::sys::jlong {
    let __cap = if cap > 0 { cap as usize } else { 0usize };
    Box::into_raw(Box::new(Vec::<perftest_flat::Payload>::with_capacity(__cap)))
        as jni::sys::jlong
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_payloadVecPush<
    'a,
>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    handle: jni::sys::jlong,
    e_id: jni::sys::jlong,
    e_seq: jni::sys::jint,
    e_value: jni::sys::jdouble,
    e_flag: jni::sys::jboolean,
    e_label: jni::objects::JString<'a>,
) {
    if handle == 0 {
        return;
    }
    let __e_id = match jlong_to_i64_fbf9a9bc(&mut env, &e_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            tracing::error!("vecPush: decoding `{}`: {}", stringify!(id), __e);
            return;
        }
    };
    let __e_seq = match jint_to_i32_a3e3b6ef(&mut env, &e_seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            tracing::error!("vecPush: decoding `{}`: {}", stringify!(seq), __e);
            return;
        }
    };
    let __e_value = match jdouble_to_f64_9e4a8f70(&mut env, &e_value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            tracing::error!("vecPush: decoding `{}`: {}", stringify!(value), __e);
            return;
        }
    };
    let __e_flag = match jboolean_to_bool_31306d98(&mut env, &e_flag) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            tracing::error!("vecPush: decoding `{}`: {}", stringify!(flag), __e);
            return;
        }
    };
    let __e_label = match JString_to_Option_Box_String_071e4c8c(&mut env, &e_label) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            tracing::error!("vecPush: decoding `{}`: {}", stringify!(label), __e);
            return;
        }
    };
    let __elem = perftest_flat::Payload {
        id: __e_id,
        seq: __e_seq,
        value: __e_value,
        flag: __e_flag,
        label: __e_label,
    };
    let __vec = &mut *(handle as *mut Vec<perftest_flat::Payload>);
    __vec.push(__elem);
}
const _: () = {
    const fn __assert_copy<T: ::core::marker::Copy>() {}
    __assert_copy::<perftest_flat::Stamp>();
};
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_constGetCoverBanner<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JString<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = {
        #[allow(unused_imports)]
        use perftest_flat::*;
        #[allow(unused_imports)]
        use covertest_helpers::*;
        format!("{COVER_TAG}:{COVER_MAGIC:#x}")
    };
    match String_to_JString_c7f3ca43(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Annotated_to_JObject_b543f0d9<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Annotated,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___payload_id: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.payload.id.clone(),
        )?;
        let ___payload_seq: jni::sys::jint = i32_to_jint_a3e3b6ef(
            env,
            v.payload.seq.clone(),
        )?;
        let ___payload_value: jni::sys::jdouble = f64_to_jdouble_9e4a8f70(
            env,
            v.payload.value.clone(),
        )?;
        let ___payload_flag: jni::sys::jboolean = bool_to_jboolean_31306d98(
            env,
            v.payload.flag.clone(),
        )?;
        let ___payload_label: jni::objects::JObject = Option_Box_String_to_JString_071e4c8c(
                env,
                v.payload.label.clone(),
            )?
            .into();
        let ___ttl: jni::objects::JObject = Option_i64_to_JObject_2ba9a5ed(
            env,
            v.ttl.clone(),
        )?;
        let ___priority: jni::objects::JObject = Option_Priority_to_JObject_ad5cbb32(
            env,
            v.priority.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/Annotated",
                "fromParts",
                "(JIDZLjava/lang/String;Ljava/lang/Long;Ljava/lang/Integer;)Lio/prebindgen/covertest/model/Annotated;",
                &[
                    jni::objects::JValue::from(___payload_id),
                    jni::objects::JValue::from(___payload_seq),
                    jni::objects::JValue::from(___payload_value),
                    jni::objects::JValue::from(___payload_flag),
                    jni::objects::JValue::Object(&___payload_label),
                    jni::objects::JValue::Object(&___ttl),
                    jni::objects::JValue::Object(&___priority),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Archive_to_jlong_cd73502c<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Archive,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Celsius_to_i32_88c8e884<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Celsius,
) -> ::core::result::Result<i32, __JniErr> {
    Ok(<perftest_flat::Celsius as ::core::convert::Into<i32>>::into(v))
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JByteArray_to_Stamp_2fc9bd18<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JByteArray<'v>,
) -> ::core::result::Result<perftest_flat::Stamp, __JniErr> {
    Ok({
        let __bytes = env
            .convert_byte_array(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("value-blob decode: {}", e))
            })?;
        if __bytes.len() != ::core::mem::size_of::<perftest_flat::Stamp>() {
            return ::core::result::Result::Err(
                <__JniErr as ::core::convert::From<
                    String,
                >>::from("value-blob decode: wrong byte length".to_string()),
            );
        }
        unsafe {
            ::core::ptr::read_unaligned(__bytes.as_ptr() as *const perftest_flat::Stamp)
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JObject_to_Annotated_b543f0d9<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Annotated, __JniErr> {
    Ok({
        let __payload_raw: jni::objects::JObject = env
            .get_field(v, "payload", "Lio/prebindgen/covertest/Payload;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Annotated.payload: {}", e)))?;
        let payload = JObject_to_Payload_98f64326(env, &__payload_raw)?;
        let __ttl_raw: jni::objects::JObject = env
            .get_field(v, "ttl", "Ljava/lang/Long;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Annotated.ttl: {}", e)))?;
        let ttl = JObject_to_Option_i64_2ba9a5ed(env, &__ttl_raw)?;
        let __priority_jobj: jni::objects::JObject = env
            .get_field(v, "priority", "Lio/prebindgen/covertest/model/Priority;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Annotated.priority: {}", e)))?;
        let priority = if __priority_jobj.is_null() {
            ::core::option::Option::None
        } else {
            let __priority_raw: jni::sys::jint = env
                .call_method(&__priority_jobj, "getValue", "()I", &[])
                .and_then(|val| val.i())
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Annotated.priority: {}", e)))?;
            ::core::option::Option::Some(
                jint_to_Priority_447102d2(env, &__priority_raw)?,
            )
        };
        perftest_flat::Annotated {
            payload,
            ttl,
            priority,
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JObject_to_Option_Payload_97036642<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<perftest_flat::Payload>, __JniErr> {
    Ok({ if v.is_null() { None } else { Some(JObject_to_Payload_98f64326(env, v)?) } })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JObject_to_Option_Priority_ad5cbb32<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<perftest_flat::Priority>, __JniErr> {
    Ok({
        if !v.is_null() {
            let __unboxed: jni::sys::jint = env
                .call_method(&v, "intValue", "()I", &[])
                .and_then(|val| val.i())
                .map(|__x| __x as jni::sys::jint)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Option unbox: {}", e)))?;
            Some(jint_to_Priority_447102d2(env, &__unboxed)?)
        } else {
            None
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JObject_to_Option_f64_b3f3e9a9<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<f64>, __JniErr> {
    Ok({
        if !v.is_null() {
            let __unboxed: jni::sys::jdouble = env
                .call_method(&v, "doubleValue", "()D", &[])
                .and_then(|val| val.d())
                .map(|__x| __x as jni::sys::jdouble)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Option unbox: {}", e)))?;
            Some(jdouble_to_f64_9e4a8f70(env, &__unboxed)?)
        } else {
            None
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JObject_to_Option_i64_2ba9a5ed<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<i64>, __JniErr> {
    Ok({
        if !v.is_null() {
            let __unboxed: jni::sys::jlong = env
                .call_method(&v, "longValue", "()J", &[])
                .and_then(|val| val.j())
                .map(|__x| __x as jni::sys::jlong)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Option unbox: {}", e)))?;
            Some(jlong_to_i64_fbf9a9bc(env, &__unboxed)?)
        } else {
            None
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JObject_to_Payload_98f64326<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Payload, __JniErr> {
    Ok({
        let __id_raw: jni::sys::jlong = env
            .get_field(v, "id", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Payload.id: {}", e)))? as _;
        let id = jlong_to_i64_fbf9a9bc(env, &__id_raw)?;
        let __seq_raw: jni::sys::jint = env
            .get_field(v, "seq", "I")
            .and_then(|val| val.i())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Payload.seq: {}", e)))? as _;
        let seq = jint_to_i32_a3e3b6ef(env, &__seq_raw)?;
        let __value_raw: jni::sys::jdouble = env
            .get_field(v, "value", "D")
            .and_then(|val| val.d())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Payload.value: {}", e)))? as _;
        let value = jdouble_to_f64_9e4a8f70(env, &__value_raw)?;
        let __flag_raw: jni::sys::jboolean = env
            .get_field(v, "flag", "Z")
            .and_then(|val| val.z())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Payload.flag: {}", e)))? as _;
        let flag = jboolean_to_bool_31306d98(env, &__flag_raw)?;
        let __label_jobj: jni::objects::JObject = env
            .get_field(v, "label", "Ljava/lang/String;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Payload.label: {}", e)))?;
        let __label_raw: jni::objects::JString = __label_jobj.into();
        let label = JString_to_Option_Box_String_071e4c8c(env, &__label_raw)?;
        perftest_flat::Payload {
            id,
            seq,
            value,
            flag,
            label,
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JObject_to_Vec_Payload_8b7084d2<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Vec<perftest_flat::Payload>, __JniErr> {
    Ok({
        let __list = jni::objects::JList::from_env(env, v)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-from-env: {}", e)))?;
        let mut __it = __list
            .iter(env)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-iter: {}", e)))?;
        let mut __out: Vec<perftest_flat::Payload> = Vec::new();
        while let Some(__obj) = __it
            .next(env)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-next: {}", e)))?
        {
            let __elem_wire: jni::objects::JObject = __obj.into();
            let __elem: perftest_flat::Payload = JObject_to_Payload_98f64326(
                env,
                &__elem_wire,
            )?;
            __out.push(__elem);
        }
        __out
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JObject_to_impl_Fn_Payload_Send_Sync_static_95073668<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<
    impl Fn(&[perftest_flat::Payload]) + Send + Sync + 'static,
    __JniErr,
> {
    Ok({
        use std::sync::Arc;
        let java_vm = Arc::new(
            env
                .get_java_vm()
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Unable to retrieve JVM: {}", e)))?,
        );
        let callback_global_ref = env
            .new_global_ref(&v)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to global-ref callback: {}", e)))?;
        let __invoke_class = env
            .get_object_class(&v)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(
                format!("Unable to get callback class for {}: {}", "Fn(& [Payload])", e),
            ))?;
        let __invoke_id = env
            .get_method_id(&__invoke_class, "run", "(Ljava/util/List;)V")
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(
                format!("Unable to resolve run for {}: {}", "Fn(& [Payload])", e),
            ))?;
        let __fold0_obj = {
            let __cls = env
                .find_class("io/prebindgen/covertest/__PayloadFolderRawHolder")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        "find folder holder {}: {}",
                        "io/prebindgen/covertest/__PayloadFolderRawHolder", e
                    ),
                ))?;
            let __field = env
                .get_static_field(
                    &__cls,
                    "instance",
                    "Lio/prebindgen/covertest/PayloadFolderRaw;",
                )
                .and_then(|__v| __v.l())
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        "fetch folder singleton {}.{}: {}",
                        "io/prebindgen/covertest/__PayloadFolderRawHolder", "instance", e
                    ),
                ))?;
            env.new_global_ref(&__field)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("global-ref folder singleton: {}", e)))?
        };
        let __fold0_id = {
            let __cls = env
                .find_class("io/prebindgen/covertest/PayloadFolderRaw")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        "find folder iface {}: {}",
                        "io/prebindgen/covertest/PayloadFolderRaw", e
                    ),
                ))?;
            env.get_method_id(
                    &__cls,
                    "run",
                    "(Ljava/lang/Object;JIDZLjava/lang/String;)Ljava/lang/Object;",
                )
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        "resolve folder run {}: {}",
                        "io/prebindgen/covertest/PayloadFolderRaw", e
                    ),
                ))?
        };
        Box::new(move |__cb_arg0: &[perftest_flat::Payload]| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        format!("Attach thread for {}: {}", "Fn(& [Payload])", e),
                    ))?;
                env.push_local_frame(16)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        format!("push local frame for {}: {}", "Fn(& [Payload])", e),
                    ))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let __fold0_acc: jni::objects::JObject = env
                        .new_object("java/util/ArrayList", "()V", &[])
                        .map_err(|e| <__JniErr as ::core::convert::From<
                            String,
                        >>::from(format!("fold: new ArrayList: {}", e)))?;
                    for __cb_elem in __cb_arg0.iter() {
                        env.push_local_frame(16)
                            .map_err(|e| <__JniErr as ::core::convert::From<
                                String,
                            >>::from(format!("fold: push frame: {}", e)))?;
                        let __fold_res = (|| -> ::core::result::Result<(), __JniErr> {
                            let __cbfold0_obj0: jni::sys::jvalue = {
                                let __enc0 = match i64_to_jlong_fbf9a9bc(
                                    &mut env,
                                    __cb_elem.id.clone(),
                                ) {
                                    ::core::result::Result::Ok(__w) => __w,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<
                                                String,
                                            >>::from(__e.to_string()),
                                        );
                                    }
                                };
                                jni::sys::jvalue { j: __enc0 }
                            };
                            let __cbfold0_obj1: jni::sys::jvalue = {
                                let __enc1 = match i32_to_jint_a3e3b6ef(
                                    &mut env,
                                    __cb_elem.seq.clone(),
                                ) {
                                    ::core::result::Result::Ok(__w) => __w,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<
                                                String,
                                            >>::from(__e.to_string()),
                                        );
                                    }
                                };
                                jni::sys::jvalue { i: __enc1 }
                            };
                            let __cbfold0_obj2: jni::sys::jvalue = {
                                let __enc2 = match f64_to_jdouble_9e4a8f70(
                                    &mut env,
                                    __cb_elem.value.clone(),
                                ) {
                                    ::core::result::Result::Ok(__w) => __w,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<
                                                String,
                                            >>::from(__e.to_string()),
                                        );
                                    }
                                };
                                jni::sys::jvalue { d: __enc2 }
                            };
                            let __cbfold0_obj3: jni::sys::jvalue = {
                                let __enc3 = match bool_to_jboolean_31306d98(
                                    &mut env,
                                    __cb_elem.flag.clone(),
                                ) {
                                    ::core::result::Result::Ok(__w) => __w,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<
                                                String,
                                            >>::from(__e.to_string()),
                                        );
                                    }
                                };
                                jni::sys::jvalue { z: __enc3 }
                            };
                            let __cbfold0_obj4: jni::objects::JObject = {
                                let __enc4 = match Option_Box_String_to_JString_071e4c8c(
                                    &mut env,
                                    __cb_elem.label.clone(),
                                ) {
                                    ::core::result::Result::Ok(__w) => __w,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<
                                                String,
                                            >>::from(__e.to_string()),
                                        );
                                    }
                                };
                                __enc4.into()
                            };
                            let _ = unsafe {
                                env.call_method_unchecked(
                                    &__fold0_obj,
                                    __fold0_id,
                                    jni::signature::ReturnType::Object,
                                    &[
                                        jni::sys::jvalue {
                                            l: __fold0_acc.as_raw(),
                                        },
                                        __cbfold0_obj0,
                                        __cbfold0_obj1,
                                        __cbfold0_obj2,
                                        __cbfold0_obj3,
                                        jni::sys::jvalue {
                                            l: __cbfold0_obj4.as_raw(),
                                        },
                                    ],
                                )
                            }
                                .map_err(|e| {
                                    let _ = env.exception_describe();
                                    <__JniErr as ::core::convert::From<
                                        String,
                                    >>::from(format!("fold run: {}", e))
                                })?;
                            ::core::result::Result::Ok(())
                        })();
                        let _ = unsafe {
                            env.pop_local_frame(&jni::objects::JObject::null())
                        };
                        __fold_res?;
                    }
                    let __call_res: ::core::result::Result<(), __JniErr> = unsafe {
                        env.call_method_unchecked(
                            &callback_global_ref,
                            __invoke_id,
                            jni::signature::ReturnType::Primitive(
                                jni::signature::Primitive::Void,
                            ),
                            &[
                                jni::sys::jvalue {
                                    l: __fold0_acc.as_raw(),
                                },
                            ],
                        )
                    }
                        .map(|_| ())
                        .map_err(|e| {
                            let _ = env.exception_describe();
                            <__JniErr as ::core::convert::From<
                                String,
                            >>::from(e.to_string())
                        });
                    __call_res?;
                    Ok(())
                })();
                let _ = unsafe { env.pop_local_frame(&jni::objects::JObject::null()) };
                __frame_res?;
                Ok(())
            })()
                .map_err(|e| {
                    tracing::error!("{} callback error: {e}", "Fn(& [Payload])")
                });
        })
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JObject_to_impl_Fn_Payload_Send_Sync_static_96d50906<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<
    impl Fn(&perftest_flat::Payload) + Send + Sync + 'static,
    __JniErr,
> {
    Ok({
        use std::sync::Arc;
        let java_vm = Arc::new(
            env
                .get_java_vm()
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Unable to retrieve JVM: {}", e)))?,
        );
        let callback_global_ref = env
            .new_global_ref(&v)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to global-ref callback: {}", e)))?;
        let __invoke_class = env
            .get_object_class(&v)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(
                format!("Unable to get callback class for {}: {}", "Fn(& Payload)", e),
            ))?;
        let __invoke_id = env
            .get_method_id(&__invoke_class, "run", "(JIDZLjava/lang/String;)V")
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to resolve run for {}: {}", "Fn(& Payload)", e)))?;
        Box::new(move |__cb_arg0: &perftest_flat::Payload| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Attach thread for {}: {}", "Fn(& Payload)", e)))?;
                env.push_local_frame(16)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        format!("push local frame for {}: {}", "Fn(& Payload)", e),
                    ))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let __cb0_obj0: jni::sys::jvalue = {
                        let __enc0 = match i64_to_jlong_fbf9a9bc(
                            &mut env,
                            __cb_arg0.id.clone(),
                        ) {
                            ::core::result::Result::Ok(__w) => __w,
                            ::core::result::Result::Err(__e) => {
                                return ::core::result::Result::Err(
                                    <__JniErr as ::core::convert::From<
                                        String,
                                    >>::from(__e.to_string()),
                                );
                            }
                        };
                        jni::sys::jvalue { j: __enc0 }
                    };
                    let __cb0_obj1: jni::sys::jvalue = {
                        let __enc1 = match i32_to_jint_a3e3b6ef(
                            &mut env,
                            __cb_arg0.seq.clone(),
                        ) {
                            ::core::result::Result::Ok(__w) => __w,
                            ::core::result::Result::Err(__e) => {
                                return ::core::result::Result::Err(
                                    <__JniErr as ::core::convert::From<
                                        String,
                                    >>::from(__e.to_string()),
                                );
                            }
                        };
                        jni::sys::jvalue { i: __enc1 }
                    };
                    let __cb0_obj2: jni::sys::jvalue = {
                        let __enc2 = match f64_to_jdouble_9e4a8f70(
                            &mut env,
                            __cb_arg0.value.clone(),
                        ) {
                            ::core::result::Result::Ok(__w) => __w,
                            ::core::result::Result::Err(__e) => {
                                return ::core::result::Result::Err(
                                    <__JniErr as ::core::convert::From<
                                        String,
                                    >>::from(__e.to_string()),
                                );
                            }
                        };
                        jni::sys::jvalue { d: __enc2 }
                    };
                    let __cb0_obj3: jni::sys::jvalue = {
                        let __enc3 = match bool_to_jboolean_31306d98(
                            &mut env,
                            __cb_arg0.flag.clone(),
                        ) {
                            ::core::result::Result::Ok(__w) => __w,
                            ::core::result::Result::Err(__e) => {
                                return ::core::result::Result::Err(
                                    <__JniErr as ::core::convert::From<
                                        String,
                                    >>::from(__e.to_string()),
                                );
                            }
                        };
                        jni::sys::jvalue { z: __enc3 }
                    };
                    let __cb0_obj4: jni::objects::JObject = {
                        let __enc4 = match Option_Box_String_to_JString_071e4c8c(
                            &mut env,
                            __cb_arg0.label.clone(),
                        ) {
                            ::core::result::Result::Ok(__w) => __w,
                            ::core::result::Result::Err(__e) => {
                                return ::core::result::Result::Err(
                                    <__JniErr as ::core::convert::From<
                                        String,
                                    >>::from(__e.to_string()),
                                );
                            }
                        };
                        __enc4.into()
                    };
                    let __call_res: ::core::result::Result<(), __JniErr> = unsafe {
                        env.call_method_unchecked(
                            &callback_global_ref,
                            __invoke_id,
                            jni::signature::ReturnType::Primitive(
                                jni::signature::Primitive::Void,
                            ),
                            &[
                                __cb0_obj0,
                                __cb0_obj1,
                                __cb0_obj2,
                                __cb0_obj3,
                                jni::sys::jvalue {
                                    l: __cb0_obj4.as_raw(),
                                },
                            ],
                        )
                    }
                        .map(|_| ())
                        .map_err(|e| {
                            let _ = env.exception_describe();
                            <__JniErr as ::core::convert::From<
                                String,
                            >>::from(e.to_string())
                        });
                    __call_res?;
                    Ok(())
                })();
                let _ = unsafe { env.pop_local_frame(&jni::objects::JObject::null()) };
                __frame_res?;
                Ok(())
            })()
                .map_err(|e| tracing::error!("{} callback error: {e}", "Fn(& Payload)"));
        })
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JObject_to_impl_Fn_Storage_Send_Sync_static_2f26edcf<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<
    impl Fn(perftest_flat::Storage) + Send + Sync + 'static,
    __JniErr,
> {
    Ok({
        use std::sync::Arc;
        let java_vm = Arc::new(
            env
                .get_java_vm()
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Unable to retrieve JVM: {}", e)))?,
        );
        let callback_global_ref = env
            .new_global_ref(&v)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to global-ref callback: {}", e)))?;
        let __invoke_class = env
            .get_object_class(&v)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(
                format!("Unable to get callback class for {}: {}", "Fn(Storage)", e),
            ))?;
        let __invoke_id = env
            .get_method_id(&__invoke_class, "run", "(J)V")
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to resolve run for {}: {}", "Fn(Storage)", e)))?;
        Box::new(move |__cb_arg0: perftest_flat::Storage| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Attach thread for {}: {}", "Fn(Storage)", e)))?;
                env.push_local_frame(16)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("push local frame for {}: {}", "Fn(Storage)", e)))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let __cb0_enc = Storage_to_jlong_1b233abd(&mut env, __cb_arg0)?;
                    let __call_res: ::core::result::Result<(), __JniErr> = unsafe {
                        env.call_method_unchecked(
                            &callback_global_ref,
                            __invoke_id,
                            jni::signature::ReturnType::Primitive(
                                jni::signature::Primitive::Void,
                            ),
                            &[jni::sys::jvalue { j: __cb0_enc }],
                        )
                    }
                        .map(|_| ())
                        .map_err(|e| {
                            let _ = env.exception_describe();
                            <__JniErr as ::core::convert::From<
                                String,
                            >>::from(e.to_string())
                        });
                    __call_res?;
                    Ok(())
                })();
                let _ = unsafe { env.pop_local_frame(&jni::objects::JObject::null()) };
                __frame_res?;
                Ok(())
            })()
                .map_err(|e| tracing::error!("{} callback error: {e}", "Fn(Storage)"));
        })
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JString_to_Option_Box_String_071e4c8c<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JString<'v>,
) -> ::core::result::Result<Option<Box<String>>, __JniErr> {
    Ok({
        if v.is_null() {
            None
        } else {
            Some(JString_to_std_boxed_Box_std_string_String_cfbab680(env, v)?)
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JString_to_String_c7f3ca43<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JString<'v>,
) -> ::core::result::Result<String, __JniErr> {
    Ok({
        let s = env
            .get_string(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("decode_string: {}", e))
            })?;
        s.into()
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn JString_to_std_boxed_Box_std_string_String_cfbab680<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JString<'v>,
) -> ::core::result::Result<::std::boxed::Box<::std::string::String>, __JniErr> {
    Ok({
        let s = env
            .get_string(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("decode_string: {}", e))
            })?;
        ::std::boxed::Box::new(::std::string::String::from(s))
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Label_to_String_63dec766<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Label,
) -> ::core::result::Result<String, __JniErr> {
    Ok(crate::label_out(v))
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Millis_to_i64_61ecf054<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Millis,
) -> ::core::result::Result<i64, __JniErr> {
    Ok(covertest_helpers::millis_value(&v))
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Option_Box_String_to_JString_071e4c8c<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<Box<String>>,
) -> ::core::result::Result<jni::objects::JString<'a>, __JniErr> {
    Ok({
        match v {
            Some(value) => {
                std_boxed_Box_std_string_String_to_JString_cfbab680(env, value)?
            }
            None => jni::objects::JObject::null().into(),
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Option_Payload_to_JObject_97036642<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<perftest_flat::Payload>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        match v {
            Some(value) => Payload_to_JObject_98f64326(env, value)?,
            None => jni::objects::JObject::null().into(),
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Option_Priority_to_JObject_ad5cbb32<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<perftest_flat::Priority>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        match v {
            Some(value) => {
                let __raw: jni::sys::jint = Priority_to_jint_447102d2(env, value)?;
                ::prebindgen::lang::box_jint(env, __raw)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option box: {}", e)))?
            }
            None => jni::objects::JObject::null(),
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Option_Summary_to_jlong_828826f3<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<&perftest_flat::Summary>,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok({
        match v {
            Some(value) => Summary_to_jlong_ccacdeac(env, value)?,
            None => 0i64,
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Option_Vec_Payload_to_JObject_b9a4637e<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<Vec<perftest_flat::Payload>>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        match v {
            Some(value) => Vec_Payload_to_JObject_8b7084d2(env, value)?,
            None => jni::objects::JObject::null().into(),
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Option_i64_to_JObject_2ba9a5ed<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<i64>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        match v {
            Some(value) => {
                let __raw: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, value)?;
                ::prebindgen::lang::box_jlong(env, __raw)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option box: {}", e)))?
            }
            None => jni::objects::JObject::null(),
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn PayloadHandler_to_jlong_d61fd890<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::PayloadHandler,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn PayloadVecHandler_to_jlong_b32d2812<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::PayloadVecHandler,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Payload_to_JObject_25cd94ea<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: &[perftest_flat::Payload],
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __list_obj = env
            .new_object("java/util/ArrayList", "()V", &[])
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("&[_]: new ArrayList: {}", e)))?;
        let __list = jni::objects::JList::from_env(env, &__list_obj)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("&[_]: list-from-env: {}", e)))?;
        for __elem in v.iter() {
            let __elem_wire = Payload_to_JObject_98f64326(
                env,
                ::core::clone::Clone::clone(__elem),
            )?;
            let __elem_obj: jni::objects::JObject = __elem_wire.into();
            __list
                .add(env, &__elem_obj)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("&[_]: list-add: {}", e)))?;
        }
        __list_obj
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Payload_to_JObject_98f64326<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Payload,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___id: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.id.clone())?;
        let ___seq: jni::sys::jint = i32_to_jint_a3e3b6ef(env, v.seq.clone())?;
        let ___value: jni::sys::jdouble = f64_to_jdouble_9e4a8f70(env, v.value.clone())?;
        let ___flag: jni::sys::jboolean = bool_to_jboolean_31306d98(
            env,
            v.flag.clone(),
        )?;
        let ___label: jni::objects::JObject = Option_Box_String_to_JString_071e4c8c(
                env,
                v.label.clone(),
            )?
            .into();
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/Payload",
                "fromParts",
                "(JIDZLjava/lang/String;)Lio/prebindgen/covertest/Payload;",
                &[
                    jni::objects::JValue::from(___id),
                    jni::objects::JValue::from(___seq),
                    jni::objects::JValue::from(___value),
                    jni::objects::JValue::from(___flag),
                    jni::objects::JValue::Object(&___label),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Percent_to_i32_01484801<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Percent,
) -> ::core::result::Result<i32, __JniErr> {
    Ok(<perftest_flat::Percent as ::core::convert::Into<i32>>::into(v))
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Priority_to_jint_447102d2<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Priority,
) -> ::core::result::Result<jni::sys::jint, __JniErr> {
    Ok({ v as jni::sys::jint })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Result_Storage_StorageError_to_Storage_7ccce404<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Result<perftest_flat::Storage, perftest_flat::StorageError>,
) -> ::core::result::Result<perftest_flat::Storage, perftest_flat::StorageError> {
    v
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Stamp_to_JByteArray_2fc9bd18<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Stamp,
) -> ::core::result::Result<jni::objects::JByteArray<'a>, __JniErr> {
    Ok({
        let __bytes: &[u8] = unsafe {
            ::core::slice::from_raw_parts(
                (&v as *const perftest_flat::Stamp) as *const u8,
                ::core::mem::size_of::<perftest_flat::Stamp>(),
            )
        };
        env.byte_array_from_slice(__bytes)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("value-blob encode: {}", e))
            })?
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn StorageError_to_jlong_26b2d298<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::StorageError,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn StorageHandler_to_jlong_3b4d3ed3<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::StorageHandler,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Storage_to_jlong_1b233abd<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Storage,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn String_to_JString_c7f3ca43<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: String,
) -> ::core::result::Result<jni::objects::JString<'a>, __JniErr> {
    Ok({
        env.new_string(v.as_str())
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("encode_string: {}", e))
            })?
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn String_to_Label_c1a79668<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: String,
) -> ::core::result::Result<perftest_flat::Label, __JniErr> {
    Ok(crate::label_in(v))
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Summary_to_jlong_3cb103b9<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Summary,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Summary_to_jlong_ccacdeac<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: &perftest_flat::Summary,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v.clone())) as i64)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Vec_Payload_to_JObject_8b7084d2<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Vec<perftest_flat::Payload>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __list_obj = env
            .new_object("java/util/ArrayList", "()V", &[])
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: new ArrayList: {}", e)))?;
        let __list = jni::objects::JList::from_env(env, &__list_obj)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-from-env: {}", e)))?;
        for __elem in v.into_iter() {
            let __elem_wire = Payload_to_JObject_98f64326(env, __elem)?;
            let __elem_obj: jni::objects::JObject = __elem_wire.into();
            __list
                .add(env, &__elem_obj)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Vec<_>: list-add: {}", e)))?;
        }
        __list_obj
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Vec_Stamp_to_JObject_8954d9be<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Vec<perftest_flat::Stamp>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __list_obj = env
            .new_object("java/util/ArrayList", "()V", &[])
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: new ArrayList: {}", e)))?;
        let __list = jni::objects::JList::from_env(env, &__list_obj)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-from-env: {}", e)))?;
        for __elem in v.into_iter() {
            let __elem_wire = Stamp_to_JByteArray_2fc9bd18(env, __elem)?;
            let __elem_obj: jni::objects::JObject = __elem_wire.into();
            __list
                .add(env, &__elem_obj)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Vec<_>: list-add: {}", e)))?;
        }
        __list_obj
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn Vec_String_to_JObject_1e282499<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Vec<String>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __list_obj = env
            .new_object("java/util/ArrayList", "()V", &[])
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: new ArrayList: {}", e)))?;
        let __list = jni::objects::JList::from_env(env, &__list_obj)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-from-env: {}", e)))?;
        for __elem in v.into_iter() {
            let __elem_wire = String_to_JString_c7f3ca43(env, __elem)?;
            let __elem_obj: jni::objects::JObject = __elem_wire.into();
            __list
                .add(env, &__elem_obj)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Vec<_>: list-add: {}", e)))?;
        }
        __list_obj
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn bool_to_jboolean_31306d98<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: bool,
) -> ::core::result::Result<jni::sys::jboolean, __JniErr> {
    Ok(v as jni::sys::jboolean)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn f64_to_jdouble_9e4a8f70<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: f64,
) -> ::core::result::Result<jni::sys::jdouble, __JniErr> {
    Ok(v as jni::sys::jdouble)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn i32_to_Celsius_8c363100<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: i32,
) -> ::core::result::Result<perftest_flat::Celsius, __JniErr> {
    Ok(<i32 as ::core::convert::Into<perftest_flat::Celsius>>::into(v))
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn i32_to_Percent_db3641cc<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: i32,
) -> ::core::result::Result<
    perftest_flat::Percent,
    <i32 as ::core::convert::TryInto<perftest_flat::Percent>>::Error,
> {
    <i32 as ::core::convert::TryInto<perftest_flat::Percent>>::try_into(v)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn i32_to_jint_a3e3b6ef<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: i32,
) -> ::core::result::Result<jni::sys::jint, __JniErr> {
    Ok(v as jni::sys::jint)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn i64_to_Millis_bb88777a<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: i64,
) -> ::core::result::Result<perftest_flat::Millis, __JniErr> {
    Ok(covertest_helpers::millis_from_long(v))
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn i64_to_jlong_fbf9a9bc<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: i64,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(v as jni::sys::jlong)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jboolean_to_bool_31306d98<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jboolean,
) -> ::core::result::Result<bool, __JniErr> {
    Ok(*v != 0)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jdouble_to_f64_9e4a8f70<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jdouble,
) -> ::core::result::Result<f64, __JniErr> {
    Ok(*v)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jint_to_Priority_447102d2<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jint,
) -> ::core::result::Result<perftest_flat::Priority, __JniErr> {
    Ok({
        match *v as i64 {
            0 => perftest_flat::Priority::Low,
            1 => perftest_flat::Priority::Normal,
            2 => perftest_flat::Priority::High,
            other => {
                return ::core::result::Result::Err(
                    <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("invalid {} discriminant: {}", "Priority", other)),
                );
            }
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jint_to_i32_a3e3b6ef<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jint,
) -> ::core::result::Result<i32, __JniErr> {
    Ok(*v)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jlong_to_Archive_cd73502c<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::Archive>, __JniErr> {
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::Archive) })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jlong_to_Option_Summary_252ef2ba<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<Option<perftest_flat::Summary>, __JniErr> {
    Ok({
        if *v == 0 {
            None
        } else {
            Some(*std::boxed::Box::from_raw(*v as *mut perftest_flat::Summary))
        }
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jlong_to_PayloadHandler_d61fd890<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::PayloadHandler>, __JniErr> {
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::PayloadHandler) })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jlong_to_PayloadVecHandler_b32d2812<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::PayloadVecHandler>, __JniErr> {
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::PayloadVecHandler) })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jlong_to_StorageError_26b2d298<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::StorageError>, __JniErr> {
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::StorageError) })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jlong_to_StorageHandler_3b4d3ed3<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::StorageHandler>, __JniErr> {
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::StorageHandler) })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jlong_to_Storage_1b233abd<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::Storage>, __JniErr> {
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::Storage) })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jlong_to_Summary_3cb103b9<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::Summary>, __JniErr> {
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::Summary) })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn jlong_to_i64_fbf9a9bc<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<i64, __JniErr> {
    Ok(*v)
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn std_boxed_Box_std_string_String_to_JString_cfbab680<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: ::std::boxed::Box<::std::string::String>,
) -> ::core::result::Result<jni::objects::JString<'a>, __JniErr> {
    Ok({
        env.new_string(v.as_str())
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("encode_str: {}", e))
            })?
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn str_to_JString_7b77dc67<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: &str,
) -> ::core::result::Result<jni::objects::JString<'a>, __JniErr> {
    Ok({
        env.new_string(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("encode_str: {}", e))
            })?
    })
}
#[allow(non_snake_case, unused_mut, unused_variables, unused_braces, dead_code)]
pub(crate) unsafe fn unit_to_unit_9ecccf8e<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: (),
) -> ::core::result::Result<(), __JniErr> {
    Ok(v)
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_annotatedNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    payload_id: jni::sys::jlong,
    payload_seq: jni::sys::jint,
    payload_value: jni::sys::jdouble,
    payload_flag: jni::sys::jboolean,
    payload_label: jni::objects::JString<'a>,
    ttl_present: jni::sys::jboolean,
    ttl_value: jni::sys::jlong,
    priority_present: jni::sys::jboolean,
    priority_value: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __payload_id = match jlong_to_i64_fbf9a9bc(&mut env, &payload_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __payload_seq = match jint_to_i32_a3e3b6ef(&mut env, &payload_seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __payload_value = match jdouble_to_f64_9e4a8f70(&mut env, &payload_value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __payload_flag = match jboolean_to_bool_31306d98(&mut env, &payload_flag) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __payload_label = match JString_to_Option_Box_String_071e4c8c(
        &mut env,
        &payload_label,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let payload = perftest_flat::Payload {
        id: __payload_id,
        seq: __payload_seq,
        value: __payload_value,
        flag: __payload_flag,
        label: __payload_label,
    };
    let ttl = if ttl_present != 0u8 {
        let __ttl_val = match jlong_to_i64_fbf9a9bc(&mut env, &ttl_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__ttl_val)
    } else {
        ::core::option::Option::None
    };
    let priority = if priority_present != 0u8 {
        let __priority_val = match jint_to_Priority_447102d2(&mut env, &priority_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__priority_val)
    } else {
        ::core::option::Option::None
    };
    let __out = perftest_flat::annotated_new(payload, ttl, priority);
    match Annotated_to_JObject_b543f0d9(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_annotatedPayloadValue<
    'a,
>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jdouble {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let a = match JObject_to_Annotated_b543f0d9(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __out = perftest_flat::annotated_payload_value(&a);
    match f64_to_jdouble_9e4a8f70(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0.0 as jni::sys::jdouble
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_annotatedPriority<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let a = match JObject_to_Annotated_b543f0d9(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::annotated_priority(&a);
    match Option_Priority_to_JObject_ad5cbb32(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_annotatedTtl<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let a = match JObject_to_Annotated_b543f0d9(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::annotated_ttl(&a);
    match Option_i64_to_JObject_2ba9a5ed(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_archiveLatest<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let a = match jlong_to_Archive_cd73502c(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::archive_latest(&a);
    match Option_Summary_to_jlong_828826f3(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_archiveNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::archive_new();
    match Archive_to_jlong_cd73502c(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_archiveStore<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a: jni::sys::jlong,
    s_sel: jni::sys::jint,
    s_0_0_present: jni::sys::jboolean,
    s_0_0_value: jni::sys::jlong,
    s_0_1_present: jni::sys::jboolean,
    s_0_1_value: jni::sys::jdouble,
    s_1: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut a = match jlong_to_Archive_cd73502c(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __exp_s_sel = match jint_to_i32_a3e3b6ef(&mut env, &s_sel) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __exp_s_0_0: Option<i64> = if s_0_0_present != 0u8 {
        let __v = match jlong_to_i64_fbf9a9bc(&mut env, &s_0_0_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return ();
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_s_0_1: Option<f64> = if s_0_1_present != 0u8 {
        let __v = match jdouble_to_f64_9e4a8f70(&mut env, &s_0_1_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return ();
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_s_1 = match jlong_to_Option_Summary_252ef2ba(&mut env, &s_1) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __folded_s = match {
        match __exp_s_sel {
            0i32 => {
                match (__exp_s_0_0, __exp_s_0_1) {
                    (
                        ::core::option::Option::Some(__p0),
                        ::core::option::Option::Some(__p1),
                    ) => {
                        ::core::result::Result::Ok(
                            perftest_flat::summary_new(__p0, __p1),
                        )
                    }
                    _ => {
                        ::core::result::Result::Err(
                            ::std::string::String::from(
                                "constructor variant input missing",
                            ),
                        )
                    }
                }
            }
            1i32 => {
                match __exp_s_1 {
                    ::core::option::Option::Some(__v) => ::core::result::Result::Ok(__v),
                    ::core::option::Option::None => {
                        ::core::result::Result::Err(
                            ::std::string::String::from("identity variant value missing"),
                        )
                    }
                }
            }
            __sel => {
                ::core::result::Result::Err(
                    ::std::format!("invalid constructor selector: {}", __sel),
                )
            }
        }
    } {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __je = <__JniErr as ::core::convert::From<
                ::std::string::String,
            >>::from(__e);
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__je.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __out = perftest_flat::archive_store(&mut a, __folded_s);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_celsiusDouble<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    c: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jint {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __c_s0 = match jint_to_i32_a3e3b6ef(&mut env, &c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    let c = match i32_to_Celsius_8c363100(&mut env, __c_s0) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    let __out = perftest_flat::celsius_double(c);
    let __out_s0 = match Celsius_to_i32_88c8e884(&mut env, __out) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    match i32_to_jint_a3e3b6ef(&mut env, __out_s0) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jint
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_coverTagRuntime<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JString<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::cover_tag_runtime();
    match String_to_JString_c7f3ca43(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_labelReverse<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    l: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JString<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __l_s0 = match JString_to_String_c7f3ca43(&mut env, &l) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let l = match String_to_Label_c1a79668(&mut env, __l_s0) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::label_reverse(l);
    let __out_s0 = match Label_to_String_63dec766(&mut env, __out) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    match String_to_JString_c7f3ca43(&mut env, __out_s0) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_millisAdd<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a: jni::sys::jlong,
    b: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __a_s0 = match jlong_to_i64_fbf9a9bc(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let a = match i64_to_Millis_bb88777a(&mut env, __a_s0) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __b_s0 = match jlong_to_i64_fbf9a9bc(&mut env, &b) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let b = match i64_to_Millis_bb88777a(&mut env, __b_s0) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::millis_add(a, b);
    let __out_s0 = match Millis_to_i64_61ecf054(&mut env, __out) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    match i64_to_jlong_fbf9a9bc(&mut env, __out_s0) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_payloadHandlerNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    f: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let f = match JObject_to_impl_Fn_Payload_Send_Sync_static_96d50906(&mut env, &f) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::payload_handler_new(f);
    match PayloadHandler_to_jlong_d61fd890(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_payloadLabelLen<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    p_id: jni::sys::jlong,
    p_seq: jni::sys::jint,
    p_value: jni::sys::jdouble,
    p_flag: jni::sys::jboolean,
    p_label: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __p_id = match jlong_to_i64_fbf9a9bc(&mut env, &p_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __p_seq = match jint_to_i32_a3e3b6ef(&mut env, &p_seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __p_value = match jdouble_to_f64_9e4a8f70(&mut env, &p_value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __p_flag = match jboolean_to_bool_31306d98(&mut env, &p_flag) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __p_label = match JString_to_Option_Box_String_071e4c8c(&mut env, &p_label) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let p = perftest_flat::Payload {
        id: __p_id,
        seq: __p_seq,
        value: __p_value,
        flag: __p_flag,
        label: __p_label,
    };
    let __out = perftest_flat::payload_label_len(&p);
    match Option_i64_to_JObject_2ba9a5ed(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_payloadPriority<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    p_id: jni::sys::jlong,
    p_seq: jni::sys::jint,
    p_value: jni::sys::jdouble,
    p_flag: jni::sys::jboolean,
    p_label: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jint {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __p_id = match jlong_to_i64_fbf9a9bc(&mut env, &p_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    let __p_seq = match jint_to_i32_a3e3b6ef(&mut env, &p_seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    let __p_value = match jdouble_to_f64_9e4a8f70(&mut env, &p_value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    let __p_flag = match jboolean_to_bool_31306d98(&mut env, &p_flag) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    let __p_label = match JString_to_Option_Box_String_071e4c8c(&mut env, &p_label) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    let p = perftest_flat::Payload {
        id: __p_id,
        seq: __p_seq,
        value: __p_value,
        flag: __p_flag,
        label: __p_label,
    };
    let __out = perftest_flat::payload_priority(&p);
    match Priority_to_jint_447102d2(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jint
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_payloadVecHandlerNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    f: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let f = match JObject_to_impl_Fn_Payload_Send_Sync_static_95073668(&mut env, &f) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::payload_vec_handler_new(f);
    match PayloadVecHandler_to_jlong_b32d2812(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_percentScale<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    p: jni::sys::jint,
    factor: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jint {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __p_s0 = match jint_to_i32_a3e3b6ef(&mut env, &p) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    let p = match i32_to_Percent_db3641cc(&mut env, __p_s0) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    let factor = match jint_to_i32_a3e3b6ef(&mut env, &factor) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    let __out = perftest_flat::percent_scale(p, factor);
    let __out_s0 = match Percent_to_i32_01484801(&mut env, __out) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    match i32_to_jint_a3e3b6ef(&mut env, __out_s0) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jint
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_priorityOr<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    p_present: jni::sys::jboolean,
    p_value: jni::sys::jint,
    fallback: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jint {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let p = if p_present != 0u8 {
        let __p_val = match jint_to_Priority_447102d2(&mut env, &p_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return 0 as jni::sys::jint;
            }
        };
        ::core::option::Option::Some(__p_val)
    } else {
        ::core::option::Option::None
    };
    let fallback = match jint_to_Priority_447102d2(&mut env, &fallback) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    let __out = perftest_flat::priority_or(p, fallback);
    match Priority_to_jint_447102d2(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jint
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_priorityWeight<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    p: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jint {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let p = match jint_to_Priority_447102d2(&mut env, &p) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jint;
        }
    };
    let __out = perftest_flat::priority_weight(p);
    match i32_to_jint_a3e3b6ef(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jint
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_stampNanos<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::objects::JByteArray<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match JByteArray_to_Stamp_2fc9bd18(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::stamp_nanos(&s);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_stampNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    secs: jni::sys::jlong,
    nanos: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JByteArray<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let secs = match jlong_to_i64_fbf9a9bc(&mut env, &secs) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let nanos = match jlong_to_i64_fbf9a9bc(&mut env, &nanos) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::stamp_new(secs, nanos);
    match Stamp_to_JByteArray_2fc9bd18(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_stampSecs<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::objects::JByteArray<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match JByteArray_to_Stamp_2fc9bd18(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::stamp_secs(&s);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_stampSeries<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    count: jni::sys::jlong,
    __acc: jni::objects::JObject<'a>,
    __fold: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/StampFolderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Object;[B)Ljava/lang/Object;";
    let __vec = perftest_flat::stamp_series(count);
    let mut __acc = __acc;
    for __elem in __vec.into_iter() {
        let __enc = match Stamp_to_JByteArray_2fc9bd18(&mut env, __elem) {
            ::core::result::Result::Ok(__w) => __w,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __obj: jni::objects::JObject = __enc.into();
        __acc = match __CB_MID
            .call_object(
                &mut env,
                __CB_FQN,
                "run",
                __CB_DESCR,
                &__fold,
                &[
                    jni::sys::jvalue {
                        l: __acc.as_raw(),
                    },
                    jni::sys::jvalue {
                        l: __obj.as_raw(),
                    },
                ],
            )
        {
            ::core::result::Result::Ok(__o) => __o,
            ::core::result::Result::Err(__e) => {
                let _ = env.exception_describe();
                let __e2 = <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string());
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e2.to_string()),
                    &__zd,
                );
                return jni::objects::JObject::null().into();
            }
        };
    }
    __acc
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageCallback<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    handler: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let handler = match jlong_to_PayloadHandler_d61fd890(&mut env, &handler) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __out = perftest_flat::storage_callback(&s, &handler);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageCallbackVec<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    handler: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let handler = match jlong_to_PayloadVecHandler_b32d2812(&mut env, &handler) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __out = perftest_flat::storage_callback_vec(&s, &handler);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageContains<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    id: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jboolean {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let id = match jlong_to_i64_fbf9a9bc(&mut env, &id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __out = perftest_flat::storage_contains(&s, id);
    match bool_to_jboolean_31306d98(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jboolean
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageEmit<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    n: jni::sys::jlong,
    h: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let n = match jlong_to_i64_fbf9a9bc(&mut env, &n) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let h = match jlong_to_StorageHandler_3b4d3ed3(&mut env, &h) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __out = perftest_flat::storage_emit(n, &h);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageErrorMessage<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    e: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JString<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let e = match jlong_to_StorageError_26b2d298(&mut env, &e) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::storage_error_message(&e);
    match String_to_JString_c7f3ca43(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageExpectSummary<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    expected_sel: jni::sys::jint,
    expected_0_0_present: jni::sys::jboolean,
    expected_0_0_value: jni::sys::jlong,
    expected_0_1_present: jni::sys::jboolean,
    expected_0_1_value: jni::sys::jdouble,
    expected_1: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jboolean {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __exp_expected_sel = match jint_to_i32_a3e3b6ef(&mut env, &expected_sel) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __exp_expected_0_0: Option<i64> = if expected_0_0_present != 0u8 {
        let __v = match jlong_to_i64_fbf9a9bc(&mut env, &expected_0_0_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return 0 as jni::sys::jboolean;
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_expected_0_1: Option<f64> = if expected_0_1_present != 0u8 {
        let __v = match jdouble_to_f64_9e4a8f70(&mut env, &expected_0_1_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return 0 as jni::sys::jboolean;
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_expected_1 = match jlong_to_Option_Summary_252ef2ba(
        &mut env,
        &expected_1,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __folded_expected = match {
        match __exp_expected_sel {
            0i32 => {
                match (__exp_expected_0_0, __exp_expected_0_1) {
                    (
                        ::core::option::Option::Some(__p0),
                        ::core::option::Option::Some(__p1),
                    ) => {
                        ::core::result::Result::Ok(
                            perftest_flat::summary_new(__p0, __p1),
                        )
                    }
                    _ => {
                        ::core::result::Result::Err(
                            ::std::string::String::from(
                                "constructor variant input missing",
                            ),
                        )
                    }
                }
            }
            1i32 => {
                match __exp_expected_1 {
                    ::core::option::Option::Some(__v) => ::core::result::Result::Ok(__v),
                    ::core::option::Option::None => {
                        ::core::result::Result::Err(
                            ::std::string::String::from("identity variant value missing"),
                        )
                    }
                }
            }
            __sel => {
                ::core::result::Result::Err(
                    ::std::format!("invalid constructor selector: {}", __sel),
                )
            }
        }
    } {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __je = <__JniErr as ::core::convert::From<
                ::std::string::String,
            >>::from(__e);
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__je.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __out = perftest_flat::storage_expect_summary(&mut s, __folded_expected);
    match bool_to_jboolean_31306d98(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jboolean
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageGet<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/PayloadBuilder";
    const __CB_DESCR: &str = "(JIDZLjava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::storage_get(&s);
    match __out {
        ::core::option::Option::Some(__inner) => {
            let __obj0: jni::sys::jvalue = {
                let __enc0 = match i64_to_jlong_fbf9a9bc(&mut env, __inner.id.clone()) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env);
                        signal_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            ::core::option::Option::Some(&__e.to_string()),
                            &__zd,
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                jni::sys::jvalue { j: __enc0 }
            };
            let __obj1: jni::sys::jvalue = {
                let __enc1 = match i32_to_jint_a3e3b6ef(&mut env, __inner.seq.clone()) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env);
                        signal_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            ::core::option::Option::Some(&__e.to_string()),
                            &__zd,
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                jni::sys::jvalue { i: __enc1 }
            };
            let __obj2: jni::sys::jvalue = {
                let __enc2 = match f64_to_jdouble_9e4a8f70(
                    &mut env,
                    __inner.value.clone(),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env);
                        signal_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            ::core::option::Option::Some(&__e.to_string()),
                            &__zd,
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                jni::sys::jvalue { d: __enc2 }
            };
            let __obj3: jni::sys::jvalue = {
                let __enc3 = match bool_to_jboolean_31306d98(
                    &mut env,
                    __inner.flag.clone(),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env);
                        signal_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            ::core::option::Option::Some(&__e.to_string()),
                            &__zd,
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                jni::sys::jvalue { z: __enc3 }
            };
            let __obj4: jni::objects::JObject = {
                let __enc4 = match Option_Box_String_to_JString_071e4c8c(
                    &mut env,
                    __inner.label.clone(),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env);
                        signal_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            ::core::option::Option::Some(&__e.to_string()),
                            &__zd,
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                __enc4.into()
            };
            match __CB_MID
                .call_object(
                    &mut env,
                    __CB_FQN,
                    "run",
                    __CB_DESCR,
                    &__builder,
                    &[
                        __obj0,
                        __obj1,
                        __obj2,
                        __obj3,
                        jni::sys::jvalue {
                            l: __obj4.as_raw(),
                        },
                    ],
                )
            {
                ::core::result::Result::Ok(__o) => __o,
                ::core::result::Result::Err(__e) => {
                    let _ = env.exception_describe();
                    let __e2 = <__JniErr as ::core::convert::From<
                        String,
                    >>::from(__e.to_string());
                    let __zd = __ze_defaults(&mut env);
                    signal_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        ::core::option::Option::Some(&__e2.to_string()),
                        &__zd,
                    );
                    jni::objects::JObject::null().into()
                }
            }
        }
        ::core::option::Option::None => jni::objects::JObject::null().into(),
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageGetVec<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __acc: jni::objects::JObject<'a>,
    __fold: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/PayloadFolderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Object;JIDZLjava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::storage_get_vec(&s);
    match __out {
        ::core::option::Option::Some(__vec) => {
            let mut __acc = __acc;
            for __elem in __vec.into_iter() {
                let __obj0: jni::sys::jvalue = {
                    let __enc0 = match i64_to_jlong_fbf9a9bc(
                        &mut env,
                        __elem.id.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            let __zd = __ze_defaults(&mut env);
                            signal_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                ::core::option::Option::Some(&__e.to_string()),
                                &__zd,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    jni::sys::jvalue { j: __enc0 }
                };
                let __obj1: jni::sys::jvalue = {
                    let __enc1 = match i32_to_jint_a3e3b6ef(
                        &mut env,
                        __elem.seq.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            let __zd = __ze_defaults(&mut env);
                            signal_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                ::core::option::Option::Some(&__e.to_string()),
                                &__zd,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    jni::sys::jvalue { i: __enc1 }
                };
                let __obj2: jni::sys::jvalue = {
                    let __enc2 = match f64_to_jdouble_9e4a8f70(
                        &mut env,
                        __elem.value.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            let __zd = __ze_defaults(&mut env);
                            signal_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                ::core::option::Option::Some(&__e.to_string()),
                                &__zd,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    jni::sys::jvalue { d: __enc2 }
                };
                let __obj3: jni::sys::jvalue = {
                    let __enc3 = match bool_to_jboolean_31306d98(
                        &mut env,
                        __elem.flag.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            let __zd = __ze_defaults(&mut env);
                            signal_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                ::core::option::Option::Some(&__e.to_string()),
                                &__zd,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    jni::sys::jvalue { z: __enc3 }
                };
                let __obj4: jni::objects::JObject = {
                    let __enc4 = match Option_Box_String_to_JString_071e4c8c(
                        &mut env,
                        __elem.label.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            let __zd = __ze_defaults(&mut env);
                            signal_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                ::core::option::Option::Some(&__e.to_string()),
                                &__zd,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __enc4.into()
                };
                __acc = match __CB_MID
                    .call_object(
                        &mut env,
                        __CB_FQN,
                        "run",
                        __CB_DESCR,
                        &__fold,
                        &[
                            jni::sys::jvalue {
                                l: __acc.as_raw(),
                            },
                            __obj0,
                            __obj1,
                            __obj2,
                            __obj3,
                            jni::sys::jvalue {
                                l: __obj4.as_raw(),
                            },
                        ],
                    )
                {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        let _ = env.exception_describe();
                        let __e2 = <__JniErr as ::core::convert::From<
                            String,
                        >>::from(__e.to_string());
                        let __zd = __ze_defaults(&mut env);
                        signal_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            ::core::option::Option::Some(&__e2.to_string()),
                            &__zd,
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
            }
            __acc
        }
        ::core::option::Option::None => jni::objects::JObject::null().into(),
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageHandlerNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    f: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let f = match JObject_to_impl_Fn_Storage_Send_Sync_static_2f26edcf(&mut env, &f) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::storage_handler_new(f);
    match StorageHandler_to_jlong_3b4d3ed3(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageLabels<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __acc: jni::objects::JObject<'a>,
    __fold: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/StringFolder";
    const __CB_DESCR: &str = "(Ljava/lang/Object;Ljava/lang/String;)Ljava/lang/Object;";
    let __vec = perftest_flat::storage_labels(&s);
    let mut __acc = __acc;
    for __elem in __vec.into_iter() {
        let __enc = match String_to_JString_c7f3ca43(&mut env, __elem) {
            ::core::result::Result::Ok(__w) => __w,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __obj: jni::objects::JObject = __enc.into();
        __acc = match __CB_MID
            .call_object(
                &mut env,
                __CB_FQN,
                "run",
                __CB_DESCR,
                &__fold,
                &[
                    jni::sys::jvalue {
                        l: __acc.as_raw(),
                    },
                    jni::sys::jvalue {
                        l: __obj.as_raw(),
                    },
                ],
            )
        {
            ::core::result::Result::Ok(__o) => __o,
            ::core::result::Result::Err(__e) => {
                let _ = env.exception_describe();
                let __e2 = <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string());
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e2.to_string()),
                    &__zd,
                );
                return jni::objects::JObject::null().into();
            }
        };
    }
    __acc
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageLen<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::storage_len(&s);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageMatchesSummary<
    'a,
>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    expected_sel: jni::sys::jint,
    expected_0_0_present: jni::sys::jboolean,
    expected_0_0_value: jni::sys::jlong,
    expected_0_1_present: jni::sys::jboolean,
    expected_0_1_value: jni::sys::jdouble,
    expected_1: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jboolean {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __exp_expected_sel = match jint_to_i32_a3e3b6ef(&mut env, &expected_sel) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __exp_expected_0_0: Option<i64> = if expected_0_0_present != 0u8 {
        let __v = match jlong_to_i64_fbf9a9bc(&mut env, &expected_0_0_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return 0 as jni::sys::jboolean;
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_expected_0_1: Option<f64> = if expected_0_1_present != 0u8 {
        let __v = match jdouble_to_f64_9e4a8f70(&mut env, &expected_0_1_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return 0 as jni::sys::jboolean;
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_expected_1 = match jlong_to_Option_Summary_252ef2ba(
        &mut env,
        &expected_1,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __folded_expected = match {
        match __exp_expected_sel {
            0i32 => {
                match (__exp_expected_0_0, __exp_expected_0_1) {
                    (
                        ::core::option::Option::Some(__p0),
                        ::core::option::Option::Some(__p1),
                    ) => {
                        ::core::result::Result::Ok(
                            perftest_flat::summary_new(__p0, __p1),
                        )
                    }
                    _ => {
                        ::core::result::Result::Err(
                            ::std::string::String::from(
                                "constructor variant input missing",
                            ),
                        )
                    }
                }
            }
            1i32 => {
                match __exp_expected_1 {
                    ::core::option::Option::Some(__v) => ::core::result::Result::Ok(__v),
                    ::core::option::Option::None => {
                        ::core::result::Result::Err(
                            ::std::string::String::from("identity variant value missing"),
                        )
                    }
                }
            }
            __sel => {
                ::core::result::Result::Err(
                    ::std::format!("invalid constructor selector: {}", __sel),
                )
            }
        }
    } {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __je = <__JniErr as ::core::convert::From<
                ::std::string::String,
            >>::from(__e);
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__je.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __out = perftest_flat::storage_matches_summary(&s, __folded_expected);
    match bool_to_jboolean_31306d98(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jboolean
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::storage_new();
    match Storage_to_jlong_1b233abd(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storagePutByRead<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    payload_id: jni::sys::jlong,
    payload_seq: jni::sys::jint,
    payload_value: jni::sys::jdouble,
    payload_flag: jni::sys::jboolean,
    payload_label: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __payload_id = match jlong_to_i64_fbf9a9bc(&mut env, &payload_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __payload_seq = match jint_to_i32_a3e3b6ef(&mut env, &payload_seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __payload_value = match jdouble_to_f64_9e4a8f70(&mut env, &payload_value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __payload_flag = match jboolean_to_bool_31306d98(&mut env, &payload_flag) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __payload_label = match JString_to_Option_Box_String_071e4c8c(
        &mut env,
        &payload_label,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let payload = perftest_flat::Payload {
        id: __payload_id,
        seq: __payload_seq,
        value: __payload_value,
        flag: __payload_flag,
        label: __payload_label,
    };
    let __out = perftest_flat::storage_put_by_read(&mut s, &payload);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storagePutByTake<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    payload_id: jni::sys::jlong,
    payload_seq: jni::sys::jint,
    payload_value: jni::sys::jdouble,
    payload_flag: jni::sys::jboolean,
    payload_label: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __payload_id = match jlong_to_i64_fbf9a9bc(&mut env, &payload_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __payload_seq = match jint_to_i32_a3e3b6ef(&mut env, &payload_seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __payload_value = match jdouble_to_f64_9e4a8f70(&mut env, &payload_value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __payload_flag = match jboolean_to_bool_31306d98(&mut env, &payload_flag) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let __payload_label = match JString_to_Option_Box_String_071e4c8c(
        &mut env,
        &payload_label,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let payload = perftest_flat::Payload {
        id: __payload_id,
        seq: __payload_seq,
        value: __payload_value,
        flag: __payload_flag,
        label: __payload_label,
    };
    let __out = perftest_flat::storage_put_by_take(&mut s, payload);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storagePutOpt<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    p_present: jni::sys::jboolean,
    p_id: jni::sys::jlong,
    p_seq: jni::sys::jint,
    p_value: jni::sys::jdouble,
    p_flag: jni::sys::jboolean,
    p_label: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jboolean {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let p = if p_present != 0u8 {
        let __p_id = match jlong_to_i64_fbf9a9bc(&mut env, &p_id) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return 0 as jni::sys::jboolean;
            }
        };
        let __p_seq = match jint_to_i32_a3e3b6ef(&mut env, &p_seq) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return 0 as jni::sys::jboolean;
            }
        };
        let __p_value = match jdouble_to_f64_9e4a8f70(&mut env, &p_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return 0 as jni::sys::jboolean;
            }
        };
        let __p_flag = match jboolean_to_bool_31306d98(&mut env, &p_flag) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return 0 as jni::sys::jboolean;
            }
        };
        let __p_label = match JString_to_Option_Box_String_071e4c8c(&mut env, &p_label) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return 0 as jni::sys::jboolean;
            }
        };
        Some(perftest_flat::Payload {
            id: __p_id,
            seq: __p_seq,
            value: __p_value,
            flag: __p_flag,
            label: __p_label,
        })
    } else {
        None
    };
    let __out = perftest_flat::storage_put_opt(&mut s, p);
    match bool_to_jboolean_31306d98(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jboolean
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storagePutSlice<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    payloads_handle: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return ();
        }
    };
    let payloads: &[perftest_flat::Payload] = unsafe {
        &*(payloads_handle as *const Vec<perftest_flat::Payload>)
    };
    let __out = perftest_flat::storage_put_slice(&mut s, payloads);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageShards<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    count: jni::sys::jlong,
    each: jni::sys::jlong,
    __acc: jni::objects::JObject<'a>,
    __fold: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let each = match jlong_to_i64_fbf9a9bc(&mut env, &each) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/StorageFolderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Object;J)Ljava/lang/Object;";
    let __vec = perftest_flat::storage_shards(count, each);
    let mut __acc = __acc;
    for __elem in __vec.into_iter() {
        let __enc = match Storage_to_jlong_1b233abd(&mut env, __elem) {
            ::core::result::Result::Ok(__w) => __w,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return jni::objects::JObject::null().into();
            }
        };
        __acc = match __CB_MID
            .call_object(
                &mut env,
                __CB_FQN,
                "run",
                __CB_DESCR,
                &__fold,
                &[
                    jni::sys::jvalue {
                        l: __acc.as_raw(),
                    },
                    jni::sys::jvalue { j: __enc },
                ],
            )
        {
            ::core::result::Result::Ok(__o) => __o,
            ::core::result::Result::Err(__e) => {
                let _ = env.exception_describe();
                let __e2 = <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string());
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e2.to_string()),
                    &__zd,
                );
                return jni::objects::JObject::null().into();
            }
        };
    }
    __acc
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageShardsOpt<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    count: jni::sys::jlong,
    each: jni::sys::jlong,
    __acc: jni::objects::JObject<'a>,
    __fold: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let each = match jlong_to_i64_fbf9a9bc(&mut env, &each) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/StorageFolderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Object;J)Ljava/lang/Object;";
    let __out = perftest_flat::storage_shards_opt(count, each);
    match __out {
        ::core::option::Option::Some(__vec) => {
            let mut __acc = __acc;
            for __elem in __vec.into_iter() {
                let __enc = match Storage_to_jlong_1b233abd(&mut env, __elem) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env);
                        signal_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            ::core::option::Option::Some(&__e.to_string()),
                            &__zd,
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                __acc = match __CB_MID
                    .call_object(
                        &mut env,
                        __CB_FQN,
                        "run",
                        __CB_DESCR,
                        &__fold,
                        &[
                            jni::sys::jvalue {
                                l: __acc.as_raw(),
                            },
                            jni::sys::jvalue { j: __enc },
                        ],
                    )
                {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        let _ = env.exception_describe();
                        let __e2 = <__JniErr as ::core::convert::From<
                            String,
                        >>::from(__e.to_string());
                        let __zd = __ze_defaults(&mut env);
                        signal_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            ::core::option::Option::Some(&__e2.to_string()),
                            &__zd,
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
            }
            __acc
        }
        ::core::option::Option::None => jni::objects::JObject::null().into(),
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageSummary<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/analytics/SummaryBuilder";
    const __CB_DESCR: &str = "(JD)Ljava/lang/Object;";
    let __out = perftest_flat::storage_summary(&s);
    let __obj0: jni::sys::jvalue = {
        let __enc0 = match i64_to_jlong_fbf9a9bc(
            &mut env,
            perftest_flat::summary_count(&__out),
        ) {
            ::core::result::Result::Ok(__w) => __w,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return jni::objects::JObject::null().into();
            }
        };
        jni::sys::jvalue { j: __enc0 }
    };
    let __obj1: jni::sys::jvalue = {
        let __enc1 = match f64_to_jdouble_9e4a8f70(
            &mut env,
            perftest_flat::summary_total(&__out),
        ) {
            ::core::result::Result::Ok(__w) => __w,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return jni::objects::JObject::null().into();
            }
        };
        jni::sys::jvalue { d: __enc1 }
    };
    match __CB_MID
        .call_object(
            &mut env,
            __CB_FQN,
            "run",
            __CB_DESCR,
            &__builder,
            &[__obj0, __obj1],
        )
    {
        ::core::result::Result::Ok(__o) => __o,
        ::core::result::Result::Err(__e) => {
            let _ = env.exception_describe();
            let __e2 = <__JniErr as ::core::convert::From<
                String,
            >>::from(__e.to_string());
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e2.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageSummaryFull<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/analytics/SummaryStorageSummaryFullBuilderRaw";
    const __CB_DESCR: &str = "(JDJ)Ljava/lang/Object;";
    let __out = perftest_flat::storage_summary_full(&s);
    let __obj0: jni::sys::jvalue = {
        let __enc0 = match i64_to_jlong_fbf9a9bc(
            &mut env,
            perftest_flat::summary_count(&__out),
        ) {
            ::core::result::Result::Ok(__w) => __w,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return jni::objects::JObject::null().into();
            }
        };
        jni::sys::jvalue { j: __enc0 }
    };
    let __obj1: jni::sys::jvalue = {
        let __enc1 = match f64_to_jdouble_9e4a8f70(
            &mut env,
            perftest_flat::summary_total(&__out),
        ) {
            ::core::result::Result::Ok(__w) => __w,
            ::core::result::Result::Err(__e) => {
                let __zd = __ze_defaults(&mut env);
                signal_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    ::core::option::Option::Some(&__e.to_string()),
                    &__zd,
                );
                return jni::objects::JObject::null().into();
            }
        };
        jni::sys::jvalue { d: __enc1 }
    };
    let __obj2: jni::sys::jvalue = jni::sys::jvalue {
        j: std::boxed::Box::into_raw(std::boxed::Box::new(__out)) as jni::sys::jlong,
    };
    match __CB_MID
        .call_object(
            &mut env,
            __CB_FQN,
            "run",
            __CB_DESCR,
            &__builder,
            &[__obj0, __obj1, __obj2],
        )
    {
        ::core::result::Result::Ok(__o) => __o,
        ::core::result::Result::Err(__e) => {
            let _ = env.exception_describe();
            let __e2 = <__JniErr as ::core::convert::From<
                String,
            >>::from(__e.to_string());
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e2.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageSummaryHandle<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::storage_summary_handle(&s);
    match Summary_to_jlong_3cb103b9(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageTotalLen<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a: jni::sys::jlong,
    b: jni::sys::jlong,
    c: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let a = match jlong_to_Storage_1b233abd(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let b = match jlong_to_Storage_1b233abd(&mut env, &b) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let c = match jlong_to_Storage_1b233abd(&mut env, &c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::storage_total_len(&a, &b, &c);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageTryWithLabel<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    label: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![
            env.new_string("").map(| __s | jni::sys::jvalue { l : __s.into_raw() })
            .unwrap_or(jni::sys::jvalue { l : ::std::ptr::null_mut() }), jni::sys::jvalue
            { l : ::std::ptr::null_mut() }
        ]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/errors/StorageErrorHandlerRaw";
    const __SINK_DESCR: &str = "(Ljava/lang/String;Ljava/lang/String;J)Ljava/lang/Object;";
    let label = match JString_to_String_c7f3ca43(&mut env, &label) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = match perftest_flat::storage_try_with_label(&label) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__de) => {
            let __eze0: jni::objects::JObject = {
                let __enc0 = match String_to_JString_c7f3ca43(
                    &mut env,
                    perftest_flat::storage_error_message(&__de),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        let __zd = __ze_defaults(&mut env);
                        signal_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            ::core::option::Option::Some(&__e.to_string()),
                            &__zd,
                        );
                        return 0 as jni::sys::jlong;
                    }
                };
                __enc0.into()
            };
            let __eze1: jni::sys::jvalue = jni::sys::jvalue {
                j: std::boxed::Box::into_raw(std::boxed::Box::new(__de))
                    as jni::sys::jlong,
            };
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::None,
                &[
                    jni::sys::jvalue {
                        l: __eze0.as_raw(),
                    },
                    __eze1,
                ],
            );
            return 0 as jni::sys::jlong;
        }
    };
    match Storage_to_jlong_1b233abd(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageWithPayload<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    payload_id: jni::sys::jlong,
    payload_seq: jni::sys::jint,
    payload_value: jni::sys::jdouble,
    payload_flag: jni::sys::jboolean,
    payload_label: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __payload_id = match jlong_to_i64_fbf9a9bc(&mut env, &payload_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __payload_seq = match jint_to_i32_a3e3b6ef(&mut env, &payload_seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __payload_value = match jdouble_to_f64_9e4a8f70(&mut env, &payload_value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __payload_flag = match jboolean_to_bool_31306d98(&mut env, &payload_flag) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __payload_label = match JString_to_Option_Box_String_071e4c8c(
        &mut env,
        &payload_label,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let payload = perftest_flat::Payload {
        id: __payload_id,
        seq: __payload_seq,
        value: __payload_value,
        flag: __payload_flag,
        label: __payload_label,
    };
    let __out = perftest_flat::storage_with_payload(payload);
    match Storage_to_jlong_1b233abd(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_stringNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JString<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match JString_to_String_c7f3ca43(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::string_new(&s);
    match String_to_JString_c7f3ca43(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summaryCount<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Summary_3cb103b9(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::summary_count(&s);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summaryNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    count: jni::sys::jlong,
    total: jni::sys::jdouble,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let total = match jdouble_to_f64_9e4a8f70(&mut env, &total) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::summary_new(count, total);
    match Summary_to_jlong_3cb103b9(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summaryScaled<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    factor: jni::sys::jdouble,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jdouble {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Summary_3cb103b9(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let factor = match jdouble_to_f64_9e4a8f70(&mut env, &factor) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __out = perftest_flat::summary_scaled(&s, factor);
    match f64_to_jdouble_9e4a8f70(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0.0 as jni::sys::jdouble
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summaryTotal<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jdouble {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Summary_3cb103b9(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __out = perftest_flat::summary_total(&s);
    match f64_to_jdouble_9e4a8f70(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0.0 as jni::sys::jdouble
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summaryTotalRaw<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jdouble {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s: perftest_flat::Summary = unsafe {
        *std::boxed::Box::from_raw(s as *mut perftest_flat::Summary)
    };
    let __out = perftest_flat::summary_total_raw(s);
    match f64_to_jdouble_9e4a8f70(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0.0 as jni::sys::jdouble
        }
    }
}
/// The storage capacity limit advertised to bindings (a primitive const).
pub const COVER_MAGIC: i64 = perftest_flat::COVER_MAGIC;
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_constGetCoverMagic<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::COVER_MAGIC;
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            0 as jni::sys::jlong
        }
    }
}
/// The coverage surface's tag string (a string const).
pub const COVER_TAG: &str = perftest_flat::COVER_TAG;
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_constGetCoverTag<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JString<'a> {
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::COVER_TAG;
    match str_to_JString_7b77dc67(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            let __zd = __ze_defaults(&mut env);
            signal_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                ::core::option::Option::Some(&__e.to_string()),
                &__zd,
            );
            jni::objects::JObject::null().into()
        }
    }
}
const _: () = {
    konst::assertc_eq!(
        perftest_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
const _: () = {
    konst::assertc_eq!(
        covertest_helpers::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
