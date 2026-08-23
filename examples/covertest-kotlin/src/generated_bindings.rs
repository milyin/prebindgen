#[allow(dead_code)]
pub(crate) type __JniErr = ::prebindgen_jni_runtime::JniBindingError<()>;
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
pub(crate) fn signal_binding_error(
    env: &mut jni::JNIEnv,
    sink: &jni::objects::JObject,
    mid: &::prebindgen_jni_runtime::CachedIfaceMethod,
    fqn: &str,
    descr: &str,
    je: &str,
) {
    if env.exception_check().unwrap_or(false) {
        return;
    }
    let __je: jni::objects::JObject = match env.new_string(je) {
        Ok(s) => s.into(),
        Err(e) => {
            tracing::error!("signal_binding_error: new_string failed: {}", e);
            return;
        }
    };
    let __args = [
        jni::sys::jvalue {
            l: __je.as_raw(),
        },
    ];
    if let Err(e) = mid.call_object(env, fqn, "run", descr, sink, &__args) {
        tracing::error!("signal_binding_error: error-callback invoke failed: {}", e);
    }
}
#[allow(non_snake_case, dead_code)]
pub(crate) fn signal_domain_error(
    env: &mut jni::JNIEnv,
    sink: &jni::objects::JObject,
    mid: &::prebindgen_jni_runtime::CachedIfaceMethod,
    fqn: &str,
    descr: &str,
    ze: &[jni::sys::jvalue],
) {
    if env.exception_check().unwrap_or(false) {
        return;
    }
    if let Err(e) = mid.call_object(env, fqn, "run", descr, sink, ze) {
        tracing::error!("signal_domain_error: error-callback invoke failed: {}", e);
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_PayloadHandler_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::PayloadHandler));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::PayloadHandler>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_PayloadVecHandler_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::PayloadVecHandler));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::PayloadVecHandler>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_StorageHandler_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::StorageHandler));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::StorageHandler>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_Storage_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::Storage));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::Storage>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_analytics_SummaryVault_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::Archive));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::Archive>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_analytics_Summary_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::Summary));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::Summary>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_errors_StorageError_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::StorageError));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::StorageError>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_esc_1pkg_Esc_1Probe_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::EscapeProbe));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::EscapeProbe>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_model_Ingot_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::Ingot));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::Ingot>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_model_Probe_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::Probe));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::Probe>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_model_Report_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::Report));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::Report>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_model_SpanHolder_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::SpanHolder));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::SpanHolder>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_model_Span_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::Span));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::Span>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_model_VaultHolder_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::VaultHolder));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::VaultHolder>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_covertest_model_Vault_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::Vault));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::Vault>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
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
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_constGetCoverVersion<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JString<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = {
        #[allow(unused_imports)]
        use perftest_flat::*;
        #[allow(unused_imports)]
        use cov_helpers::*;
        crate::cover_version()
    };
    match String_to_JString_c7f3ca43(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_constGetCoverBanner<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JString<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = {
        #[allow(unused_imports)]
        use perftest_flat::*;
        #[allow(unused_imports)]
        use cov_helpers::*;
        format!("{COVER_TAG}:{COVER_MAGIC:#x}")
    };
    match String_to_JString_c7f3ca43(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
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
        let ___alternate_present: jni::sys::jboolean;
        let ___alternate_o0: jni::sys::jlong;
        let ___alternate_o1: jni::sys::jint;
        let ___alternate_o2: jni::sys::jdouble;
        let ___alternate_o3: jni::sys::jboolean;
        let ___alternate_o4: jni::objects::JObject;
        let __on0: &::core::option::Option<_> = &v.alternate;
        match __on0 {
            ::core::option::Option::Some(__c0) => {
                let ___alternate_id: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
                    env,
                    __c0.id.clone(),
                )?;
                let ___alternate_seq: jni::sys::jint = i32_to_jint_a3e3b6ef(
                    env,
                    __c0.seq.clone(),
                )?;
                let ___alternate_value: jni::sys::jdouble = f64_to_jdouble_9e4a8f70(
                    env,
                    __c0.value.clone(),
                )?;
                let ___alternate_flag: jni::sys::jboolean = bool_to_jboolean_31306d98(
                    env,
                    __c0.flag.clone(),
                )?;
                let ___alternate_label: jni::objects::JObject = Option_Box_String_to_JString_071e4c8c(
                        env,
                        __c0.label.clone(),
                    )?
                    .into();
                ___alternate_present = 1u8;
                ___alternate_o0 = ___alternate_id;
                ___alternate_o1 = ___alternate_seq;
                ___alternate_o2 = ___alternate_value;
                ___alternate_o3 = ___alternate_flag;
                ___alternate_o4 = ___alternate_label;
            }
            ::core::option::Option::None => {
                ___alternate_present = 0u8;
                ___alternate_o0 = 0i64;
                ___alternate_o1 = 0i32;
                ___alternate_o2 = 0.0f64;
                ___alternate_o3 = 0u8;
                ___alternate_o4 = jni::objects::JObject::null();
            }
        }
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
                "(JIDZLjava/lang/String;ZJIDZLjava/lang/String;Ljava/lang/Long;Ljava/lang/Integer;)Lio/prebindgen/covertest/model/Annotated;",
                &[
                    jni::objects::JValue::from(___payload_id),
                    jni::objects::JValue::from(___payload_seq),
                    jni::objects::JValue::from(___payload_value),
                    jni::objects::JValue::from(___payload_flag),
                    jni::objects::JValue::Object(&___payload_label),
                    jni::objects::JValue::from(___alternate_present),
                    jni::objects::JValue::from(___alternate_o0),
                    jni::objects::JValue::from(___alternate_o1),
                    jni::objects::JValue::from(___alternate_o2),
                    jni::objects::JValue::from(___alternate_o3),
                    jni::objects::JValue::Object(&___alternate_o4),
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
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Archive_to_jlong_cd73502c<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Archive,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Arrays_to_JObject_71120c08<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Arrays,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___bytes: jni::objects::JObject = u8_4_to_JByteArray_39abedfa(
                env,
                v.bytes.clone(),
            )?
            .into();
        let ___shorts: jni::objects::JObject = i16_2_to_JShortArray_098f4ad5(
                env,
                v.shorts.clone(),
            )?
            .into();
        let ___ints: jni::objects::JObject = i32_3_to_JIntArray_60e5e35a(
                env,
                v.ints.clone(),
            )?
            .into();
        let ___longs: jni::objects::JObject = i64_2_to_JLongArray_73596912(
                env,
                v.longs.clone(),
            )?
            .into();
        let ___doubles: jni::objects::JObject = f64_2_to_JDoubleArray_dc30d1f9(
                env,
                v.doubles.clone(),
            )?
            .into();
        let ___flags: jni::objects::JObject = bool_3_to_JBooleanArray_3f960c58(
                env,
                v.flags.clone(),
            )?
            .into();
        let ___raw: jni::objects::JObject = u64_2_to_JLongArray_60bcc6a5(
                env,
                v.raw.clone(),
            )?
            .into();
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/Arrays",
                "fromParts",
                "([B[S[I[J[D[Z[J)Lio/prebindgen/covertest/model/Arrays;",
                &[
                    jni::objects::JValue::Object(&___bytes),
                    jni::objects::JValue::Object(&___shorts),
                    jni::objects::JValue::Object(&___ints),
                    jni::objects::JValue::Object(&___longs),
                    jni::objects::JValue::Object(&___doubles),
                    jni::objects::JValue::Object(&___flags),
                    jni::objects::JValue::Object(&___raw),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn Arrays_to_tuple7_c0fbd13f<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Arrays,
) -> ::core::result::Result<
    (
        jni::objects::JByteArray<'a>,
        jni::objects::JShortArray<'a>,
        jni::objects::JIntArray<'a>,
        jni::objects::JLongArray<'a>,
        jni::objects::JDoubleArray<'a>,
        jni::objects::JBooleanArray<'a>,
        jni::objects::JLongArray<'a>,
    ),
    __JniErr,
> {
    ::core::result::Result::Ok((
        u8_4_to_JByteArray_39abedfa(env, v.bytes)?,
        i16_2_to_JShortArray_098f4ad5(env, v.shorts)?,
        i32_3_to_JIntArray_60e5e35a(env, v.ints)?,
        i64_2_to_JLongArray_73596912(env, v.longs)?,
        f64_2_to_JDoubleArray_dc30d1f9(env, v.doubles)?,
        bool_3_to_JBooleanArray_3f960c58(env, v.flags)?,
        u64_2_to_JLongArray_60bcc6a5(env, v.raw)?,
    ))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn BlobValue_to_JObject_89b5dab7<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::BlobValue,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___stamp_secs: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.stamp.secs.clone(),
        )?;
        let ___stamp_nanos: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.stamp.nanos.clone(),
        )?;
        let ___id: jni::objects::JObject = Vec_u8_to_JByteArray_7936d5de(
                env,
                v.id.clone(),
            )?
            .into();
        let ___chunks: jni::objects::JObject = Vec_Vec_u8_to_JObject_43404875(
            env,
            v.chunks.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/BlobValue",
                "fromParts",
                "(JJ[BLjava/util/List;)Lio/prebindgen/covertest/model/BlobValue;",
                &[
                    jni::objects::JValue::from(___stamp_secs),
                    jni::objects::JValue::from(___stamp_nanos),
                    jni::objects::JValue::Object(&___id),
                    jni::objects::JValue::Object(&___chunks),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn BlobValue_to_tuple3_2c75fc67<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::BlobValue,
) -> ::core::result::Result<
    (
        (jni::sys::jlong, jni::sys::jlong),
        jni::objects::JByteArray<'a>,
        jni::objects::JObject<'a>,
    ),
    __JniErr,
> {
    ::core::result::Result::Ok((
        Stamp_to_tuple2_8d33d015(env, v.stamp)?,
        Vec_u8_to_JByteArray_7936d5de(env, v.id)?,
        Vec_Vec_u8_to_JObject_43404875(env, v.chunks)?,
    ))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Box_Box_Option_String_to_JString_299999e0<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Box<Box<Option<String>>>,
) -> ::core::result::Result<jni::objects::JString<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match **v {
            ::core::option::Option::Some(__value) => {
                String_to_JString_c7f3ca43(env, __value)?
            }
            ::core::option::Option::None => jni::objects::JObject::null().into(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Box_Duration_to_jlong_0776c1ca<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Box<perftest_flat::Duration>,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok({
        let __inner = *v;
        {
            let __inner_s0 = Duration_to_u64_e3980876(env, __inner)
                .map_err(|__e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string()))?;
            u64_to_jlong_4384a5d6(env, __inner_s0)?
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Box_Option_i64_to_JObject_cf5a3724<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Box<Option<i64>>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match *v {
            ::core::option::Option::Some(__value) => {
                let __raw: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, __value)?;
                ::prebindgen_jni_runtime::box_jlong(env, __raw)
                    .map_err(|__error| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option box: {}", __error)))?
            }
            ::core::option::Option::None => jni::objects::JObject::null(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Box_Priority_to_jint_a16653ae<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Box<perftest_flat::Priority>,
) -> ::core::result::Result<jni::sys::jint, __JniErr> {
    Ok({
        let __inner = *v;
        Priority_to_jint_447102d2(env, __inner)?
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Box_String_to_JString_027f6250<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Box<String>,
) -> ::core::result::Result<jni::objects::JString<'a>, __JniErr> {
    Ok({
        env.new_string(&*v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("encode_str: {}", e))
            })?
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn CacheConfig_to_JObject_db89a97c<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::CacheConfig,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___replies_priority: jni::sys::jint = Priority_to_jint_447102d2(
            env,
            v.replies.priority.clone(),
        )?;
        let ___replies_max_samples: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.replies.max_samples.clone(),
        )?;
        let ___ttl: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.ttl.clone())?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/CacheConfig",
                "fromParts",
                "(IJJ)Lio/prebindgen/covertest/model/CacheConfig;",
                &[
                    jni::objects::JValue::from(___replies_priority),
                    jni::objects::JValue::from(___replies_max_samples),
                    jni::objects::JValue::from(___ttl),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn CallbackHolder_to_JObject_81e45598<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::CallbackHolder,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___tag: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.tag.clone())?;
        let ___token: jni::sys::jlong = {
            let ___token_s0 = CallbackToken_to_Ingot_c7696aa6(env, v.token.clone())
                .map_err(|__e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string()))?;
            Ingot_to_jlong_020c3a86(env, ___token_s0)?
        };
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/CallbackHolder",
                "fromParts",
                "(JJ)Lio/prebindgen/covertest/CallbackHolder;",
                &[
                    jni::objects::JValue::from(___tag),
                    jni::objects::JValue::from(___token),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn CallbackHolder_to_tuple2_14aebb91<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::CallbackHolder,
) -> ::core::result::Result<(jni::sys::jlong, jni::sys::jlong), __JniErr> {
    ::core::result::Result::Ok((
        i64_to_jlong_fbf9a9bc(env, v.tag)?,
        {
            let __chain_s0 = CallbackToken_to_Ingot_c7696aa6(env, v.token)
                .map_err(|__e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string()))?;
            Ingot_to_jlong_020c3a86(env, __chain_s0)
        }?,
    ))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn CallbackToken_to_Ingot_c7696aa6<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::CallbackToken,
) -> ::core::result::Result<perftest_flat::Ingot, __JniErr> {
    Ok(perftest_flat::callback_token_into_ingot(v))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Celsius_to_i32_88c8e884<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Celsius,
) -> ::core::result::Result<i32, __JniErr> {
    Ok(<perftest_flat::Celsius as ::core::convert::Into<i32>>::into(v))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Dossier_to_JObject_eabbdbfa<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Dossier,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___note: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.note.clone())?;
        let ___holder_tag: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.holder.tag.clone(),
        )?;
        let ___holder_summary: jni::sys::jlong = Summary_to_jlong_3cb103b9(
            env,
            v.holder.summary.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/Dossier",
                "fromParts",
                "(JJJ)Lio/prebindgen/covertest/Dossier;",
                &[
                    jni::objects::JValue::from(___note),
                    jni::objects::JValue::from(___holder_tag),
                    jni::objects::JValue::from(___holder_summary),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn DurationBoundary_to_JObject_9c5bf9bc<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::DurationBoundary,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___required: jni::sys::jlong = {
            let ___required_s0 = Duration_to_u64_e3980876(env, v.required.clone())
                .map_err(|__e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string()))?;
            u64_to_jlong_4384a5d6(env, ___required_s0)?
        };
        let ___delay: jni::sys::jlong = Option_Duration_to_jlong_1cfa4d44(
            env,
            v.delay.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/DurationBoundary",
                "fromParts",
                "(JJ)Lio/prebindgen/covertest/model/DurationBoundary;",
                &[
                    jni::objects::JValue::from(___required),
                    jni::objects::JValue::from(___delay),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn DurationBoundary_to_tuple2_3834b601<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::DurationBoundary,
) -> ::core::result::Result<(jni::sys::jlong, jni::sys::jlong), __JniErr> {
    ::core::result::Result::Ok((
        {
            let __chain_s0 = Duration_to_u64_e3980876(env, v.required)
                .map_err(|__e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string()))?;
            u64_to_jlong_4384a5d6(env, __chain_s0)
        }?,
        Option_Duration_to_jlong_1cfa4d44(env, v.delay)?,
    ))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Duration_to_u64_e3980876<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Duration,
) -> ::core::result::Result<u64, __JniErr> {
    {
        match (crate::duration_to_millis(v))
            .map_err(|__e| {
                <__JniErr as ::core::convert::From<String>>::from(__e.to_string())
            })
        {
            ::core::result::Result::Ok(
                __repr,
            ) if (true && true && (__repr) <= 86400000u64) && !(false) => {
                ::core::result::Result::Ok(__repr)
            }
            ::core::result::Result::Ok(_) => {
                ::core::result::Result::Err(
                    <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        format!(
                            "{} representation is outside its declared domain",
                            "Duration"
                        ),
                    ),
                )
            }
            ::core::result::Result::Err(__e) => ::core::result::Result::Err(__e),
        }
    }
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn EscapeProbe_to_jlong_416aab42<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::EscapeProbe,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn HoldPolicy_to_JObject_d2a5bcc4<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::HoldPolicy,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___hold__tag: jni::sys::jint;
        let ___hold_g0: jni::sys::jlong;
        match &v.hold {
            perftest_flat::Hold::Indefinite => {
                ___hold__tag = 0;
                ___hold_g0 = 0i64;
            }
            perftest_flat::Hold::For(__s0_0) => {
                let ___hold_for_v0: jni::sys::jlong = {
                    let ___hold_for_v0_s0 = Duration_to_u64_e3980876(env, __s0_0.clone())
                        .map_err(|__e| <__JniErr as ::core::convert::From<
                            String,
                        >>::from(__e.to_string()))?;
                    u64_to_jlong_4384a5d6(env, ___hold_for_v0_s0)?
                };
                ___hold__tag = 1;
                ___hold_g0 = ___hold_for_v0;
            }
        }
        let ___grace_present: jni::sys::jboolean;
        let ___grace__tag: jni::sys::jint;
        let ___grace_g0: jni::sys::jlong;
        let __oc0: &::core::option::Option<_> = &v.grace;
        match __oc0 {
            ::core::option::Option::Some(__o0) => {
                ___grace_present = 1u8;
                match __o0 {
                    perftest_flat::Hold::Indefinite => {
                        ___grace__tag = 0;
                        ___grace_g0 = 0i64;
                    }
                    perftest_flat::Hold::For(__s0_0) => {
                        let ___grace_for_v0: jni::sys::jlong = {
                            let ___grace_for_v0_s0 = Duration_to_u64_e3980876(
                                    env,
                                    __s0_0.clone(),
                                )
                                .map_err(|__e| <__JniErr as ::core::convert::From<
                                    String,
                                >>::from(__e.to_string()))?;
                            u64_to_jlong_4384a5d6(env, ___grace_for_v0_s0)?
                        };
                        ___grace__tag = 1;
                        ___grace_g0 = ___grace_for_v0;
                    }
                }
            }
            ::core::option::Option::None => {
                ___grace_present = 0u8;
                ___grace__tag = 0i32;
                ___grace_g0 = 0i64;
            }
        }
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/HoldPolicy",
                "fromParts",
                "(IJZIJ)Lio/prebindgen/covertest/model/HoldPolicy;",
                &[
                    jni::objects::JValue::from(___hold__tag),
                    jni::objects::JValue::from(___hold_g0),
                    jni::objects::JValue::from(___grace_present),
                    jni::objects::JValue::from(___grace__tag),
                    jni::objects::JValue::from(___grace_g0),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn Hold_to_tuple3_bf18c116<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Hold,
) -> ::core::result::Result<(jni::sys::jint, (), (jni::sys::jlong,)), __JniErr> {
    ::core::result::Result::Ok({
        match v {
            perftest_flat::Hold::Indefinite => (0i32, (), (0 as jni::sys::jlong,)),
            perftest_flat::Hold::For(__part0) => {
                (
                    1i32,
                    (),
                    (
                        {
                            let __chain_s0 = Duration_to_u64_e3980876(env, __part0)
                                .map_err(|__e| <__JniErr as ::core::convert::From<
                                    String,
                                >>::from(__e.to_string()))?;
                            u64_to_jlong_4384a5d6(env, __chain_s0)
                        }?,
                    ),
                )
            }
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Holder_to_JObject_c36a9705<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Holder,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___tag: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.tag.clone())?;
        let ___summary: jni::sys::jlong = Summary_to_jlong_3cb103b9(
            env,
            v.summary.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/Holder",
                "fromParts",
                "(JJ)Lio/prebindgen/covertest/Holder;",
                &[
                    jni::objects::JValue::from(___tag),
                    jni::objects::JValue::from(___summary),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Ingot_to_jlong_020c3a86<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Ingot,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JBooleanArray_to_bool_3_3f960c58<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JBooleanArray<'v>,
) -> ::core::result::Result<[bool; 3], __JniErr> {
    Ok({
        let __len = env
            .get_array_length(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })? as usize;
        let mut __buf: ::std::vec::Vec<jni::sys::jboolean> = ::std::vec![
            0 as jni::sys::jboolean; __len
        ];
        env.get_boolean_array_region(v, 0, &mut __buf)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })?;
        let __vals: ::std::vec::Vec<bool> = __buf.iter().map(|__x| *__x != 0).collect();
        let __arr: [bool; 3] = __vals
            .as_slice()
            .try_into()
            .map_err(|_| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    "fixed-size array decode: `[bool ; 3]` expects a different length"
                        .to_string(),
                )
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JByteArray_to_Vec_u8_7936d5de<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JByteArray<'v>,
) -> ::core::result::Result<Vec<u8>, __JniErr> {
    Ok({
        env.convert_byte_array(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("decode_byte_array: {}", e))
            })?
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JByteArray_to_u8_2_9ca14e44<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JByteArray<'v>,
) -> ::core::result::Result<[u8; 2], __JniErr> {
    Ok({
        let __buf = env
            .convert_byte_array(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })?;
        let __arr: [u8; 2] = __buf
            .as_slice()
            .try_into()
            .map_err(|_| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    "fixed-size array decode: `[u8 ; 2]` expects a different length"
                        .to_string(),
                )
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JByteArray_to_u8_4_39abedfa<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JByteArray<'v>,
) -> ::core::result::Result<[u8; 4], __JniErr> {
    Ok({
        let __buf = env
            .convert_byte_array(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })?;
        let __arr: [u8; 4] = __buf
            .as_slice()
            .try_into()
            .map_err(|_| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    "fixed-size array decode: `[u8 ; 4]` expects a different length"
                        .to_string(),
                )
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JDoubleArray_to_f64_2_dc30d1f9<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JDoubleArray<'v>,
) -> ::core::result::Result<[f64; 2], __JniErr> {
    Ok({
        let __len = env
            .get_array_length(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })? as usize;
        let mut __buf: ::std::vec::Vec<jni::sys::jdouble> = ::std::vec![
            0 as jni::sys::jdouble; __len
        ];
        env.get_double_array_region(v, 0, &mut __buf)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })?;
        let __vals: ::std::vec::Vec<f64> = __buf.iter().map(|__x| *__x as f64).collect();
        let __arr: [f64; 2] = __vals
            .as_slice()
            .try_into()
            .map_err(|_| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    "fixed-size array decode: `[f64 ; 2]` expects a different length"
                        .to_string(),
                )
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JIntArray_to_i32_3_60e5e35a<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JIntArray<'v>,
) -> ::core::result::Result<[i32; 3], __JniErr> {
    Ok({
        let __len = env
            .get_array_length(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })? as usize;
        let mut __buf: ::std::vec::Vec<jni::sys::jint> = ::std::vec![
            0 as jni::sys::jint; __len
        ];
        env.get_int_array_region(v, 0, &mut __buf)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })?;
        let __vals: ::std::vec::Vec<i32> = __buf.iter().map(|__x| *__x as i32).collect();
        let __arr: [i32; 3] = __vals
            .as_slice()
            .try_into()
            .map_err(|_| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    "fixed-size array decode: `[i32 ; 3]` expects a different length"
                        .to_string(),
                )
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JLongArray_to_i64_2_73596912<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JLongArray<'v>,
) -> ::core::result::Result<[i64; 2], __JniErr> {
    Ok({
        let __len = env
            .get_array_length(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })? as usize;
        let mut __buf: ::std::vec::Vec<jni::sys::jlong> = ::std::vec![
            0 as jni::sys::jlong; __len
        ];
        env.get_long_array_region(v, 0, &mut __buf)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })?;
        let __vals: ::std::vec::Vec<i64> = __buf.iter().map(|__x| *__x as i64).collect();
        let __arr: [i64; 2] = __vals
            .as_slice()
            .try_into()
            .map_err(|_| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    "fixed-size array decode: `[i64 ; 2]` expects a different length"
                        .to_string(),
                )
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JLongArray_to_u64_2_60bcc6a5<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JLongArray<'v>,
) -> ::core::result::Result<[u64; 2], __JniErr> {
    Ok({
        let __len = env
            .get_array_length(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })? as usize;
        let mut __buf: ::std::vec::Vec<jni::sys::jlong> = ::std::vec![
            0 as jni::sys::jlong; __len
        ];
        env.get_long_array_region(v, 0, &mut __buf)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })?;
        let __vals: ::std::vec::Vec<u64> = __buf.iter().map(|__x| *__x as u64).collect();
        let __arr: [u64; 2] = __vals
            .as_slice()
            .try_into()
            .map_err(|_| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    "fixed-size array decode: `[u64 ; 2]` expects a different length"
                        .to_string(),
                )
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
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
        let __alternate_raw: jni::objects::JObject = env
            .get_field(v, "alternate", "Lio/prebindgen/covertest/Payload;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Annotated.alternate: {}", e)))?;
        let alternate = JObject_to_Option_Payload_97036642(env, &__alternate_raw)?;
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
            alternate,
            ttl,
            priority,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Arrays_71120c08<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Arrays, __JniErr> {
    Ok({
        let __bytes_jobj: jni::objects::JObject = env
            .get_field(v, "bytes", "[B")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Arrays.bytes: {}", e)))?;
        let __bytes_raw: jni::objects::JByteArray = __bytes_jobj.into();
        let bytes = JByteArray_to_u8_4_39abedfa(env, &__bytes_raw)?;
        let __shorts_jobj: jni::objects::JObject = env
            .get_field(v, "shorts", "[S")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Arrays.shorts: {}", e)))?;
        let __shorts_raw: jni::objects::JShortArray = __shorts_jobj.into();
        let shorts = JShortArray_to_i16_2_098f4ad5(env, &__shorts_raw)?;
        let __ints_jobj: jni::objects::JObject = env
            .get_field(v, "ints", "[I")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Arrays.ints: {}", e)))?;
        let __ints_raw: jni::objects::JIntArray = __ints_jobj.into();
        let ints = JIntArray_to_i32_3_60e5e35a(env, &__ints_raw)?;
        let __longs_jobj: jni::objects::JObject = env
            .get_field(v, "longs", "[J")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Arrays.longs: {}", e)))?;
        let __longs_raw: jni::objects::JLongArray = __longs_jobj.into();
        let longs = JLongArray_to_i64_2_73596912(env, &__longs_raw)?;
        let __doubles_jobj: jni::objects::JObject = env
            .get_field(v, "doubles", "[D")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Arrays.doubles: {}", e)))?;
        let __doubles_raw: jni::objects::JDoubleArray = __doubles_jobj.into();
        let doubles = JDoubleArray_to_f64_2_dc30d1f9(env, &__doubles_raw)?;
        let __flags_jobj: jni::objects::JObject = env
            .get_field(v, "flags", "[Z")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Arrays.flags: {}", e)))?;
        let __flags_raw: jni::objects::JBooleanArray = __flags_jobj.into();
        let flags = JBooleanArray_to_bool_3_3f960c58(env, &__flags_raw)?;
        let __raw_jobj: jni::objects::JObject = env
            .get_field(v, "raw", "[J")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Arrays.raw: {}", e)))?;
        let __raw_raw: jni::objects::JLongArray = __raw_jobj.into();
        let raw = JLongArray_to_u64_2_60bcc6a5(env, &__raw_raw)?;
        perftest_flat::Arrays {
            bytes,
            shorts,
            ints,
            longs,
            doubles,
            flags,
            raw,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_BlobValue_89b5dab7<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::BlobValue, __JniErr> {
    Ok({
        let __stamp_raw: jni::objects::JObject = env
            .get_field(v, "stamp", "Lio/prebindgen/covertest/model/Stamp;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("BlobValue.stamp: {}", e)))?;
        let stamp = JObject_to_Stamp_f6b1e942(env, &__stamp_raw)?;
        let __id_jobj: jni::objects::JObject = env
            .get_field(v, "id", "[B")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("BlobValue.id: {}", e)))?;
        let __id_raw: jni::objects::JByteArray = __id_jobj.into();
        let id = JByteArray_to_Vec_u8_7936d5de(env, &__id_raw)?;
        let __chunks_raw: jni::objects::JObject = env
            .get_field(v, "chunks", "Ljava/util/List;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("BlobValue.chunks: {}", e)))?;
        let chunks = JObject_to_Vec_Vec_u8_43404875(env, &__chunks_raw)?;
        perftest_flat::BlobValue {
            stamp,
            id,
            chunks,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Box_Option_Payload_8d993ebb<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Box<Option<perftest_flat::Payload>>, __JniErr> {
    ::core::result::Result::Ok(
        ::std::boxed::Box::new({
            if v.is_null() {
                ::core::option::Option::None
            } else {
                let __present = v;
                ::core::option::Option::Some(
                    JObject_to_Payload_98f64326(env, __present)?,
                )
            }
        }),
    )
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Box_Option_i64_cf5a3724<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Box<Option<i64>>, __JniErr> {
    ::core::result::Result::Ok(
        ::std::boxed::Box::new({
            if (v).is_null() {
                ::core::option::Option::None
            } else {
                let __present = {
                    env.call_method(&v, "longValue", "()J", &[])
                        .and_then(|__value| __value.j())
                        .map(|__value| __value as jni::sys::jlong)
                        .map_err(|__error| <__JniErr as ::core::convert::From<
                            String,
                        >>::from(format!("Option unbox: {}", __error)))?
                };
                ::core::option::Option::Some(jlong_to_i64_fbf9a9bc(env, &(__present))?)
            }
        }),
    )
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Box_Payload_0d2d19da<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Box<perftest_flat::Payload>, __JniErr> {
    Ok({
        let __inner = JObject_to_Payload_98f64326(env, v)?;
        ::std::boxed::Box::new(__inner)
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_CacheConfig_db89a97c<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::CacheConfig, __JniErr> {
    Ok({
        let __replies_raw: jni::objects::JObject = env
            .get_field(v, "replies", "Lio/prebindgen/covertest/model/RepliesConfig;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("CacheConfig.replies: {}", e)))?;
        let replies = JObject_to_RepliesConfig_eb8e9079(env, &__replies_raw)?;
        let __ttl_raw: jni::sys::jlong = env
            .get_field(v, "ttl", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("CacheConfig.ttl: {}", e)))? as _;
        let ttl = jlong_to_i64_fbf9a9bc(env, &__ttl_raw)?;
        perftest_flat::CacheConfig {
            replies,
            ttl,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_CallbackHolder_81e45598<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::CallbackHolder, __JniErr> {
    Ok({
        let __tag_raw: jni::sys::jlong = env
            .get_field(v, "tag", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("CallbackHolder.tag: {}", e)))? as _;
        let tag = jlong_to_i64_fbf9a9bc(env, &__tag_raw)?;
        let __token_raw: jni::objects::JObject = env
            .get_field(v, "token", "Ljava/lang/Object;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("CallbackHolder.token: {}", e)))?;
        let token = JObject_to_CallbackToken_432e8cc0(env, &__token_raw)?;
        perftest_flat::CallbackHolder {
            tag,
            token,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_CallbackToken_432e8cc0<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::CallbackToken, __JniErr> {
    Ok({
        let __ingot_jobj: jni::objects::JObject = env
            .get_field(v, "ingot", "Lio/prebindgen/covertest/model/Ingot;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("CallbackToken.ingot: {}", e)))?;
        let __ingot_raw: jni::sys::jlong = if __ingot_jobj.is_null() {
            0
        } else {
            env.call_method(&__ingot_jobj, "peek", "()J", &[])
                .and_then(|val| val.j())
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("CallbackToken.ingot: {}", e)))?
        };
        if __ingot_raw == 0 || (__ingot_raw & 1) == 1 {
            return ::core::result::Result::Err(
                <__JniErr as ::core::convert::From<
                    String,
                >>::from("Operation on a closed native handle.".to_string()),
            );
        }
        let ingot: perftest_flat::Ingot = unsafe {
            *std::boxed::Box::from_raw(__ingot_raw as *mut perftest_flat::Ingot)
        };
        perftest_flat::CallbackToken {
            ingot,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Dossier_eabbdbfa<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Dossier, __JniErr> {
    Ok({
        let __note_raw: jni::sys::jlong = env
            .get_field(v, "note", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Dossier.note: {}", e)))? as _;
        let note = jlong_to_i64_fbf9a9bc(env, &__note_raw)?;
        let __holder_raw: jni::objects::JObject = env
            .get_field(v, "holder", "Lio/prebindgen/covertest/Holder;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Dossier.holder: {}", e)))?;
        let holder = JObject_to_Holder_c36a9705(env, &__holder_raw)?;
        perftest_flat::Dossier {
            note,
            holder,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_DurationBoundary_9c5bf9bc<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::DurationBoundary, __JniErr> {
    Ok({
        let __required_raw: jni::sys::jlong = env
            .get_field(v, "required", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("DurationBoundary.required: {}", e)))?;
        let required = {
            let required_s0 = jlong_to_u64_4384a5d6(env, &__required_raw)?;
            let required_s1 = u64_to_Duration_7c0845f9(env, required_s0)
                .map_err(|__e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string()))?;
            required_s1
        };
        let __delay_jobj: jni::objects::JObject = env
            .get_field(v, "delay", "Lkotlin/ULong;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("DurationBoundary.delay: {}", e)))?;
        let delay = if __delay_jobj.is_null() {
            ::core::option::Option::None
        } else {
            let __delay_raw: jni::sys::jlong = env
                .call_method(&__delay_jobj, "unbox-impl", "()J", &[])
                .and_then(|val| val.j())
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("DurationBoundary.delay: {}", e)))?;
            jlong_to_Option_Duration_1cfa4d44(env, &__delay_raw)?
        };
        perftest_flat::DurationBoundary {
            required,
            delay,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_HoldPolicy_d2a5bcc4<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::HoldPolicy, __JniErr> {
    Ok({
        let __hold_raw: jni::objects::JObject = env
            .get_field(v, "hold", "Lio/prebindgen/covertest/model/Hold;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("HoldPolicy.hold: {}", e)))?;
        let hold = JObject_to_Hold_5f85caaf(env, &__hold_raw)?;
        let __grace_raw: jni::objects::JObject = env
            .get_field(v, "grace", "Lio/prebindgen/covertest/model/Hold;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("HoldPolicy.grace: {}", e)))?;
        let grace = JObject_to_Option_Hold_230d7f9b(env, &__grace_raw)?;
        perftest_flat::HoldPolicy {
            hold,
            grace,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Hold_5f85caaf<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Hold, __JniErr> {
    Ok({
        let __obj = v;
        (|| -> ::core::result::Result<perftest_flat::Hold, __JniErr> {
            if __obj.is_null() {
                return ::core::result::Result::Err(
                    <__JniErr as ::core::convert::From<
                        String,
                    >>::from("Hold: null value where a variant was required".to_string()),
                );
            }
            if env
                .is_instance_of(__obj, "io/prebindgen/covertest/model/Hold$Indefinite")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        concat!("Hold", ": instanceof ",
                        "io/prebindgen/covertest/model/Hold$Indefinite", ": {}"), e
                    ),
                ))?
            {
                return ::core::result::Result::Ok(perftest_flat::Hold::Indefinite);
            }
            if env
                .is_instance_of(__obj, "io/prebindgen/covertest/model/Hold$For")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        concat!("Hold", ": instanceof ",
                        "io/prebindgen/covertest/model/Hold$For", ": {}"), e
                    ),
                ))?
            {
                let __p_v0_raw: jni::sys::jlong = env
                    .get_field(__obj, "v0", "J")
                    .and_then(|val| val.j())
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Hold.For.v0: {}", e)))? as _;
                let __p_v0 = {
                    let __p_v0_s0 = jlong_to_u64_4384a5d6(env, &__p_v0_raw)?;
                    let __p_v0_s1 = u64_to_Duration_7c0845f9(env, __p_v0_s0)
                        .map_err(|__e| <__JniErr as ::core::convert::From<
                            String,
                        >>::from(__e.to_string()))?;
                    __p_v0_s1
                };
                return ::core::result::Result::Ok(perftest_flat::Hold::For(__p_v0));
            }
            ::core::result::Result::Err(
                <__JniErr as ::core::convert::From<
                    String,
                >>::from("Hold: value is not one of its declared variants".to_string()),
            )
        })()?
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Holder_c36a9705<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Holder, __JniErr> {
    Ok({
        let __tag_raw: jni::sys::jlong = env
            .get_field(v, "tag", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Holder.tag: {}", e)))? as _;
        let tag = jlong_to_i64_fbf9a9bc(env, &__tag_raw)?;
        let __summary_jobj: jni::objects::JObject = env
            .get_field(v, "summary", "Lio/prebindgen/covertest/analytics/Summary;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Holder.summary: {}", e)))?;
        let __summary_raw: jni::sys::jlong = if __summary_jobj.is_null() {
            0
        } else {
            env.call_method(&__summary_jobj, "peek", "()J", &[])
                .and_then(|val| val.j())
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Holder.summary: {}", e)))?
        };
        if __summary_raw == 0 || (__summary_raw & 1) == 1 {
            return ::core::result::Result::Err(
                <__JniErr as ::core::convert::From<
                    String,
                >>::from("Operation on a closed native handle.".to_string()),
            );
        }
        let summary: perftest_flat::Summary = unsafe {
            *std::boxed::Box::from_raw(__summary_raw as *mut perftest_flat::Summary)
        };
        perftest_flat::Holder {
            tag,
            summary,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Lookup_94ada15e<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Lookup, __JniErr> {
    Ok({
        let __obj = v;
        (|| -> ::core::result::Result<perftest_flat::Lookup, __JniErr> {
            if __obj.is_null() {
                return ::core::result::Result::Err(
                    <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        "Lookup: null value where a variant was required".to_string(),
                    ),
                );
            }
            if env
                .is_instance_of(__obj, "io/prebindgen/covertest/model/Lookup$Absent")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        concat!("Lookup", ": instanceof ",
                        "io/prebindgen/covertest/model/Lookup$Absent", ": {}"), e
                    ),
                ))?
            {
                return ::core::result::Result::Ok(perftest_flat::Lookup::Absent);
            }
            if env
                .is_instance_of(__obj, "io/prebindgen/covertest/model/Lookup$Found")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        concat!("Lookup", ": instanceof ",
                        "io/prebindgen/covertest/model/Lookup$Found", ": {}"), e
                    ),
                ))?
            {
                let __p_v0_obj: jni::objects::JObject = env
                    .get_field(
                        __obj,
                        "v0",
                        "Lio/prebindgen/covertest/analytics/Summary;",
                    )
                    .and_then(|val| val.l())
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Lookup.Found.v0: {}", e)))?;
                let __p_v0_raw: jni::sys::jlong = if __p_v0_obj.is_null() {
                    0
                } else {
                    env.call_method(&__p_v0_obj, "peek", "()J", &[])
                        .and_then(|val| val.j())
                        .map_err(|e| <__JniErr as ::core::convert::From<
                            String,
                        >>::from(format!("Lookup.Found.v0: {}", e)))?
                };
                if __p_v0_raw == 0 || (__p_v0_raw & 1) == 1 {
                    return ::core::result::Result::Err(
                        <__JniErr as ::core::convert::From<
                            String,
                        >>::from("Operation on a closed native handle.".to_string()),
                    );
                }
                let __p_v0: perftest_flat::Summary = unsafe {
                    *std::boxed::Box::from_raw(__p_v0_raw as *mut perftest_flat::Summary)
                };
                return ::core::result::Result::Ok(perftest_flat::Lookup::Found(__p_v0));
            }
            if env
                .is_instance_of(__obj, "io/prebindgen/covertest/model/Lookup$Failed")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        concat!("Lookup", ": instanceof ",
                        "io/prebindgen/covertest/model/Lookup$Failed", ": {}"), e
                    ),
                ))?
            {
                let __p_v0_obj: jni::objects::JObject = env
                    .get_field(__obj, "v0", "Ljava/lang/String;")
                    .and_then(|val| val.l())
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Lookup.Failed.v0: {}", e)))?;
                let __p_v0_raw: jni::objects::JString = __p_v0_obj.into();
                let __p_v0 = JString_to_String_c7f3ca43(env, &__p_v0_raw)?;
                return ::core::result::Result::Ok(perftest_flat::Lookup::Failed(__p_v0));
            }
            ::core::result::Result::Err(
                <__JniErr as ::core::convert::From<
                    String,
                >>::from("Lookup: value is not one of its declared variants".to_string()),
            )
        })()?
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Marker_3dc81334<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Marker, __JniErr> {
    Ok({
        let __obj = v;
        (|| -> ::core::result::Result<perftest_flat::Marker, __JniErr> {
            if __obj.is_null() {
                return ::core::result::Result::Err(
                    <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        "Marker: null value where a variant was required".to_string(),
                    ),
                );
            }
            if env
                .is_instance_of(__obj, "io/prebindgen/covertest/model/Marker$None_")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        concat!("Marker", ": instanceof ",
                        "io/prebindgen/covertest/model/Marker$None_", ": {}"), e
                    ),
                ))?
            {
                return ::core::result::Result::Ok(perftest_flat::Marker::None_);
            }
            if env
                .is_instance_of(__obj, "io/prebindgen/covertest/model/Marker$Ranked")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        concat!("Marker", ": instanceof ",
                        "io/prebindgen/covertest/model/Marker$Ranked", ": {}"), e
                    ),
                ))?
            {
                let __p_v0_obj: jni::objects::JObject = env
                    .get_field(__obj, "v0", "Lio/prebindgen/covertest/model/Priority;")
                    .and_then(|val| val.l())
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Marker.Ranked.v0: {}", e)))?;
                let __p_v0 = if __p_v0_obj.is_null() {
                    ::core::option::Option::None
                } else {
                    let __p_v0_raw: jni::sys::jint = env
                        .call_method(&__p_v0_obj, "getValue", "()I", &[])
                        .and_then(|val| val.i())
                        .map_err(|e| <__JniErr as ::core::convert::From<
                            String,
                        >>::from(format!("Marker.Ranked.v0: {}", e)))?;
                    ::core::option::Option::Some(
                        jint_to_Priority_447102d2(env, &__p_v0_raw)?,
                    )
                };
                return ::core::result::Result::Ok(perftest_flat::Marker::Ranked(__p_v0));
            }
            ::core::result::Result::Err(
                <__JniErr as ::core::convert::From<
                    String,
                >>::from("Marker: value is not one of its declared variants".to_string()),
            )
        })()?
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_MaybeHolder_1c68fbac<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::MaybeHolder, __JniErr> {
    Ok({
        let __tag_raw: jni::sys::jlong = env
            .get_field(v, "tag", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("MaybeHolder.tag: {}", e)))? as _;
        let tag = jlong_to_i64_fbf9a9bc(env, &__tag_raw)?;
        let __summary_jobj: jni::objects::JObject = env
            .get_field(v, "summary", "Lio/prebindgen/covertest/analytics/Summary;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("MaybeHolder.summary: {}", e)))?;
        let __summary_raw: jni::sys::jlong = if __summary_jobj.is_null() {
            0
        } else {
            env.call_method(&__summary_jobj, "peek", "()J", &[])
                .and_then(|val| val.j())
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("MaybeHolder.summary: {}", e)))?
        };
        let summary = jlong_to_Option_Summary_252ef2ba(env, &__summary_raw)?;
        perftest_flat::MaybeHolder {
            tag,
            summary,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_ObjectBoundary16_e9d41606<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary16, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/covertest/model/ObjectBoundary8;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary16.left: {}", e)))?;
        let left = JObject_to_ObjectBoundary8_55b82b02(env, &__left_raw)?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/covertest/model/ObjectBoundary8;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary16.right: {}", e)))?;
        let right = JObject_to_ObjectBoundary8_55b82b02(env, &__right_raw)?;
        perftest_flat::ObjectBoundary16 {
            left,
            right,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_ObjectBoundary2_a8f288cc<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary2, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/covertest/model/ObjectBoundaryLeaf;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary2.left: {}", e)))?;
        let left = JObject_to_ObjectBoundaryLeaf_93531764(env, &__left_raw)?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/covertest/model/ObjectBoundaryLeaf;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary2.right: {}", e)))?;
        let right = JObject_to_ObjectBoundaryLeaf_93531764(env, &__right_raw)?;
        perftest_flat::ObjectBoundary2 {
            left,
            right,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_ObjectBoundary32_ed80fac3<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary32, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/covertest/model/ObjectBoundary16;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary32.left: {}", e)))?;
        let left = JObject_to_ObjectBoundary16_e9d41606(env, &__left_raw)?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/covertest/model/ObjectBoundary16;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary32.right: {}", e)))?;
        let right = JObject_to_ObjectBoundary16_e9d41606(env, &__right_raw)?;
        perftest_flat::ObjectBoundary32 {
            left,
            right,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_ObjectBoundary4_ea3fd497<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary4, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/covertest/model/ObjectBoundary2;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary4.left: {}", e)))?;
        let left = JObject_to_ObjectBoundary2_a8f288cc(env, &__left_raw)?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/covertest/model/ObjectBoundary2;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary4.right: {}", e)))?;
        let right = JObject_to_ObjectBoundary2_a8f288cc(env, &__right_raw)?;
        perftest_flat::ObjectBoundary4 {
            left,
            right,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_ObjectBoundary63_29aa82ff<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary63, __JniErr> {
    Ok({
        let __leaves32_raw: jni::objects::JObject = env
            .get_field(v, "leaves32", "Lio/prebindgen/covertest/model/ObjectBoundary32;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary63.leaves32: {}", e)))?;
        let leaves32 = JObject_to_ObjectBoundary32_ed80fac3(env, &__leaves32_raw)?;
        let __leaves16_raw: jni::objects::JObject = env
            .get_field(v, "leaves16", "Lio/prebindgen/covertest/model/ObjectBoundary16;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary63.leaves16: {}", e)))?;
        let leaves16 = JObject_to_ObjectBoundary16_e9d41606(env, &__leaves16_raw)?;
        let __leaves8_raw: jni::objects::JObject = env
            .get_field(v, "leaves8", "Lio/prebindgen/covertest/model/ObjectBoundary8;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary63.leaves8: {}", e)))?;
        let leaves8 = JObject_to_ObjectBoundary8_55b82b02(env, &__leaves8_raw)?;
        let __leaves4_raw: jni::objects::JObject = env
            .get_field(v, "leaves4", "Lio/prebindgen/covertest/model/ObjectBoundary4;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary63.leaves4: {}", e)))?;
        let leaves4 = JObject_to_ObjectBoundary4_ea3fd497(env, &__leaves4_raw)?;
        let __leaves2_raw: jni::objects::JObject = env
            .get_field(v, "leaves2", "Lio/prebindgen/covertest/model/ObjectBoundary2;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary63.leaves2: {}", e)))?;
        let leaves2 = JObject_to_ObjectBoundary2_a8f288cc(env, &__leaves2_raw)?;
        let __leaf_raw: jni::objects::JObject = env
            .get_field(v, "leaf", "Lio/prebindgen/covertest/model/ObjectBoundaryLeaf;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary63.leaf: {}", e)))?;
        let leaf = JObject_to_ObjectBoundaryLeaf_93531764(env, &__leaf_raw)?;
        perftest_flat::ObjectBoundary63 {
            leaves32,
            leaves16,
            leaves8,
            leaves4,
            leaves2,
            leaf,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_ObjectBoundary64_b2751ca5<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary64, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/covertest/model/ObjectBoundary32;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary64.left: {}", e)))?;
        let left = JObject_to_ObjectBoundary32_ed80fac3(env, &__left_raw)?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/covertest/model/ObjectBoundary32;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary64.right: {}", e)))?;
        let right = JObject_to_ObjectBoundary32_ed80fac3(env, &__right_raw)?;
        perftest_flat::ObjectBoundary64 {
            left,
            right,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_ObjectBoundary8_55b82b02<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary8, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/covertest/model/ObjectBoundary4;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary8.left: {}", e)))?;
        let left = JObject_to_ObjectBoundary4_ea3fd497(env, &__left_raw)?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/covertest/model/ObjectBoundary4;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary8.right: {}", e)))?;
        let right = JObject_to_ObjectBoundary4_ea3fd497(env, &__right_raw)?;
        perftest_flat::ObjectBoundary8 {
            left,
            right,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_ObjectBoundaryLeaf_93531764<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundaryLeaf, __JniErr> {
    Ok({
        let __value_raw: jni::sys::jlong = env
            .get_field(v, "value", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundaryLeaf.value: {}", e)))? as _;
        let value = jlong_to_i64_fbf9a9bc(env, &__value_raw)?;
        perftest_flat::ObjectBoundaryLeaf {
            value,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_ObjectBoundary_dc5ac22b<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/covertest/model/ObjectBoundary64;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary.left: {}", e)))?;
        let left = JObject_to_ObjectBoundary64_b2751ca5(env, &__left_raw)?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/covertest/model/ObjectBoundary63;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary.right: {}", e)))?;
        let right = JObject_to_ObjectBoundary63_29aa82ff(env, &__right_raw)?;
        perftest_flat::ObjectBoundary {
            left,
            right,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Observation_435b0724<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Observation, __JniErr> {
    Ok({
        let __id_raw: jni::sys::jlong = env
            .get_field(v, "id", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Observation.id: {}", e)))? as _;
        let id = jlong_to_i64_fbf9a9bc(env, &__id_raw)?;
        let __reading_raw: jni::objects::JObject = env
            .get_field(v, "reading", "Lio/prebindgen/covertest/model/Reading;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Observation.reading: {}", e)))?;
        let reading = JObject_to_Reading_2261050f(env, &__reading_raw)?;
        let __fallback_raw: jni::objects::JObject = env
            .get_field(v, "fallback", "Lio/prebindgen/covertest/model/Reading;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Observation.fallback: {}", e)))?;
        let fallback = JObject_to_Option_Reading_80df84a9(env, &__fallback_raw)?;
        let __note_jobj: jni::objects::JObject = env
            .get_field(v, "note", "Ljava/lang/String;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Observation.note: {}", e)))?;
        let __note_raw: jni::objects::JString = __note_jobj.into();
        let note = JString_to_String_c7f3ca43(env, &__note_raw)?;
        perftest_flat::Observation {
            id,
            reading,
            fallback,
            note,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Option_CacheConfig_a6be794d<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<perftest_flat::CacheConfig>, __JniErr> {
    ::core::result::Result::Ok({
        if v.is_null() {
            ::core::option::Option::None
        } else {
            let __present = v;
            ::core::option::Option::Some(
                JObject_to_CacheConfig_db89a97c(env, __present)?,
            )
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Option_Hold_230d7f9b<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<perftest_flat::Hold>, __JniErr> {
    ::core::result::Result::Ok({
        if v.is_null() {
            ::core::option::Option::None
        } else {
            let __present = v;
            ::core::option::Option::Some(JObject_to_Hold_5f85caaf(env, __present)?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Option_Holder_ca758c1f<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<perftest_flat::Holder>, __JniErr> {
    ::core::result::Result::Ok({
        if v.is_null() {
            ::core::option::Option::None
        } else {
            let __present = v;
            ::core::option::Option::Some(JObject_to_Holder_c36a9705(env, __present)?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Option_Payload_97036642<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<perftest_flat::Payload>, __JniErr> {
    ::core::result::Result::Ok({
        if v.is_null() {
            ::core::option::Option::None
        } else {
            let __present = v;
            ::core::option::Option::Some(JObject_to_Payload_98f64326(env, __present)?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Option_Percent_544dd364<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<perftest_flat::Percent>, __JniErr> {
    ::core::result::Result::Ok({
        if (v).is_null() {
            ::core::option::Option::None
        } else {
            let __present = {
                env.call_method(&v, "intValue", "()I", &[])
                    .and_then(|__value| __value.i())
                    .map(|__value| __value as jni::sys::jint)
                    .map_err(|__error| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option unbox: {}", __error)))?
            };
            ::core::option::Option::Some(
                {
                    let __chain_s0 = jint_to_i32_a3e3b6ef(env, &(__present))?;
                    let __chain_s1 = i32_to_Percent_db3641cc(env, __chain_s0)
                        .map_err(|__e| <__JniErr as ::core::convert::From<
                            String,
                        >>::from(__e.to_string()))?;
                    ::core::result::Result::<_, __JniErr>::Ok(__chain_s1)
                }?,
            )
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Option_Priority_ad5cbb32<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<perftest_flat::Priority>, __JniErr> {
    ::core::result::Result::Ok({
        if (v).is_null() {
            ::core::option::Option::None
        } else {
            let __present = {
                env.call_method(&v, "intValue", "()I", &[])
                    .and_then(|__value| __value.i())
                    .map(|__value| __value as jni::sys::jint)
                    .map_err(|__error| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option unbox: {}", __error)))?
            };
            ::core::option::Option::Some(jint_to_Priority_447102d2(env, &(__present))?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Option_Reading_80df84a9<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<perftest_flat::Reading>, __JniErr> {
    ::core::result::Result::Ok({
        if v.is_null() {
            ::core::option::Option::None
        } else {
            let __present = v;
            ::core::option::Option::Some(JObject_to_Reading_2261050f(env, __present)?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Option_i64_2ba9a5ed<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<i64>, __JniErr> {
    ::core::result::Result::Ok({
        if (v).is_null() {
            ::core::option::Option::None
        } else {
            let __present = {
                env.call_method(&v, "longValue", "()J", &[])
                    .and_then(|__value| __value.j())
                    .map(|__value| __value as jni::sys::jlong)
                    .map_err(|__error| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option unbox: {}", __error)))?
            };
            ::core::option::Option::Some(jlong_to_i64_fbf9a9bc(env, &(__present))?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Option_u64_32be16a2<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Option<u64>, __JniErr> {
    ::core::result::Result::Ok({
        if (v).is_null() {
            ::core::option::Option::None
        } else {
            let __present = {
                env.call_method(&v, "longValue", "()J", &[])
                    .and_then(|__value| __value.j())
                    .map(|__value| __value as jni::sys::jlong)
                    .map_err(|__error| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option unbox: {}", __error)))?
            };
            ::core::option::Option::Some(jlong_to_u64_4384a5d6(env, &(__present))?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
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
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Reading_2261050f<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Reading, __JniErr> {
    Ok({
        let __obj = v;
        (|| -> ::core::result::Result<perftest_flat::Reading, __JniErr> {
            if __obj.is_null() {
                return ::core::result::Result::Err(
                    <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        "Reading: null value where a variant was required".to_string(),
                    ),
                );
            }
            if env
                .is_instance_of(__obj, "io/prebindgen/covertest/model/Reading$Missing")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        concat!("Reading", ": instanceof ",
                        "io/prebindgen/covertest/model/Reading$Missing", ": {}"), e
                    ),
                ))?
            {
                return ::core::result::Result::Ok(perftest_flat::Reading::Missing);
            }
            if env
                .is_instance_of(__obj, "io/prebindgen/covertest/model/Reading$Exact")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        concat!("Reading", ": instanceof ",
                        "io/prebindgen/covertest/model/Reading$Exact", ": {}"), e
                    ),
                ))?
            {
                let __p_v0_raw: jni::sys::jlong = env
                    .get_field(__obj, "v0", "J")
                    .and_then(|val| val.j())
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Reading.Exact.v0: {}", e)))? as _;
                let __p_v0 = jlong_to_i64_fbf9a9bc(env, &__p_v0_raw)?;
                return ::core::result::Result::Ok(perftest_flat::Reading::Exact(__p_v0));
            }
            if env
                .is_instance_of(__obj, "io/prebindgen/covertest/model/Reading$Range")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        concat!("Reading", ": instanceof ",
                        "io/prebindgen/covertest/model/Reading$Range", ": {}"), e
                    ),
                ))?
            {
                let __p_low_raw: jni::sys::jlong = env
                    .get_field(__obj, "low", "J")
                    .and_then(|val| val.j())
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Reading.Range.low: {}", e)))? as _;
                let __p_low = jlong_to_i64_fbf9a9bc(env, &__p_low_raw)?;
                let __p_high_raw: jni::sys::jlong = env
                    .get_field(__obj, "high", "J")
                    .and_then(|val| val.j())
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Reading.Range.high: {}", e)))? as _;
                let __p_high = jlong_to_i64_fbf9a9bc(env, &__p_high_raw)?;
                return ::core::result::Result::Ok(perftest_flat::Reading::Range {
                    low: __p_low,
                    high: __p_high,
                });
            }
            if env
                .is_instance_of(__obj, "io/prebindgen/covertest/model/Reading$Tagged")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        concat!("Reading", ": instanceof ",
                        "io/prebindgen/covertest/model/Reading$Tagged", ": {}"), e
                    ),
                ))?
            {
                let __p_v0_obj: jni::objects::JObject = env
                    .get_field(__obj, "v0", "Ljava/lang/String;")
                    .and_then(|val| val.l())
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Reading.Tagged.v0: {}", e)))?;
                let __p_v0_raw: jni::objects::JString = __p_v0_obj.into();
                let __p_v0 = JString_to_String_c7f3ca43(env, &__p_v0_raw)?;
                let __p_v1_obj: jni::objects::JObject = env
                    .get_field(__obj, "v1", "Lio/prebindgen/covertest/model/Priority;")
                    .and_then(|val| val.l())
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Reading.Tagged.v1: {}", e)))?;
                let __p_v1_raw: jni::sys::jint = env
                    .call_method(&__p_v1_obj, "getValue", "()I", &[])
                    .and_then(|val| val.i())
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Reading.Tagged.v1: {}", e)))?;
                let __p_v1 = jint_to_Priority_447102d2(env, &__p_v1_raw)?;
                return ::core::result::Result::Ok(
                    perftest_flat::Reading::Labeled(__p_v0, __p_v1),
                );
            }
            if env
                .is_instance_of(__obj, "io/prebindgen/covertest/model/Reading$Companion")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        concat!("Reading", ": instanceof ",
                        "io/prebindgen/covertest/model/Reading$Companion", ": {}"), e
                    ),
                ))?
            {
                let __p_v0_raw: jni::sys::jlong = env
                    .get_field(__obj, "v0", "J")
                    .and_then(|val| val.j())
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Reading.Companion.v0: {}", e)))? as _;
                let __p_v0 = jlong_to_i64_fbf9a9bc(env, &__p_v0_raw)?;
                return ::core::result::Result::Ok(
                    perftest_flat::Reading::Companion(__p_v0),
                );
            }
            ::core::result::Result::Err(
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    "Reading: value is not one of its declared variants".to_string(),
                ),
            )
        })()?
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_RepliesConfig_eb8e9079<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::RepliesConfig, __JniErr> {
    Ok({
        let __priority_jobj: jni::objects::JObject = env
            .get_field(v, "priority", "Lio/prebindgen/covertest/model/Priority;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("RepliesConfig.priority: {}", e)))?;
        let __priority_raw: jni::sys::jint = env
            .call_method(&__priority_jobj, "getValue", "()I", &[])
            .and_then(|val| val.i())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("RepliesConfig.priority: {}", e)))?;
        let priority = jint_to_Priority_447102d2(env, &__priority_raw)?;
        let __max_samples_raw: jni::sys::jlong = env
            .get_field(v, "maxSamples", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("RepliesConfig.maxSamples: {}", e)))? as _;
        let max_samples = jlong_to_i64_fbf9a9bc(env, &__max_samples_raw)?;
        perftest_flat::RepliesConfig {
            priority,
            max_samples,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Stamp_f6b1e942<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Stamp, __JniErr> {
    Ok({
        let __secs_raw: jni::sys::jlong = env
            .get_field(v, "secs", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Stamp.secs: {}", e)))? as _;
        let secs = jlong_to_i64_fbf9a9bc(env, &__secs_raw)?;
        let __nanos_raw: jni::sys::jlong = env
            .get_field(v, "nanos", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Stamp.nanos: {}", e)))? as _;
        let nanos = jlong_to_i64_fbf9a9bc(env, &__nanos_raw)?;
        perftest_flat::Stamp {
            secs,
            nanos,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Tagged_641b984c<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Tagged, __JniErr> {
    Ok({
        let __id_raw: jni::sys::jlong = env
            .get_field(v, "id", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Tagged.id: {}", e)))? as _;
        let id = jlong_to_i64_fbf9a9bc(env, &__id_raw)?;
        let __marker_raw: jni::objects::JObject = env
            .get_field(v, "marker", "Lio/prebindgen/covertest/model/Marker;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Tagged.marker: {}", e)))?;
        let marker = JObject_to_Marker_3dc81334(env, &__marker_raw)?;
        perftest_flat::Tagged {
            id,
            marker,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Unsigned_7e3cc618<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Unsigned, __JniErr> {
    Ok({
        let __byte_raw: jni::sys::jint = env
            .get_field(v, "byte", "I")
            .and_then(|val| val.i())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unsigned.byte: {}", e)))? as _;
        let byte = jint_to_u8_553cf6ec(env, &__byte_raw)?;
        let __short_raw: jni::sys::jint = env
            .get_field(v, "short", "I")
            .and_then(|val| val.i())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unsigned.short: {}", e)))? as _;
        let short = jint_to_u16_28edf527(env, &__short_raw)?;
        let __int_raw: jni::sys::jlong = env
            .get_field(v, "int", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unsigned.int: {}", e)))? as _;
        let int = jlong_to_u32_9594a230(env, &__int_raw)?;
        let __long_raw: jni::sys::jlong = env
            .get_field(v, "long", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unsigned.long: {}", e)))?;
        let long = jlong_to_u64_4384a5d6(env, &__long_raw)?;
        let __maybe_long_jobj: jni::objects::JObject = env
            .get_field(v, "maybeLong", "Lkotlin/ULong;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unsigned.maybeLong: {}", e)))?;
        let maybe_long = if __maybe_long_jobj.is_null() {
            ::core::option::Option::None
        } else {
            let __maybe_long_raw: jni::sys::jlong = env
                .call_method(&__maybe_long_jobj, "unbox-impl", "()J", &[])
                .and_then(|val| val.j())
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Unsigned.maybeLong: {}", e)))?;
            ::core::option::Option::Some(jlong_to_u64_4384a5d6(env, &__maybe_long_raw)?)
        };
        perftest_flat::Unsigned {
            byte,
            short,
            int,
            long,
            maybe_long,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Vec_Label_3fdf860d<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'a>,
) -> ::core::result::Result<Vec<perftest_flat::Label>, __JniErr> {
    ::core::result::Result::Ok({
        let __sequence_list = jni::objects::JList::from_env(env, v)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-from-env: {}", e)))?;
        let mut __sequence_iter = __sequence_list
            .iter(env)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-iter: {}", e)))?;
        let mut __sequence_values: ::std::vec::Vec<perftest_flat::Label> = ::std::vec::Vec::new();
        while let ::core::option::Option::Some(__sequence_part) = match __sequence_iter
            .next(env)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-next: {}", e)))?
        {
            ::core::option::Option::Some(__sequence_object) => {
                let __sequence_part: jni::objects::JString = __sequence_object.into();
                ::core::option::Option::Some(__sequence_part)
            }
            ::core::option::Option::None => ::core::option::Option::None,
        } {
            __sequence_values
                .push(
                    {
                        let __chain_s0 = JString_to_String_c7f3ca43(
                            env,
                            &(__sequence_part),
                        )?;
                        let __chain_s1 = String_to_Label_c1a79668(env, __chain_s0)
                            .map_err(|__e| <__JniErr as ::core::convert::From<
                                String,
                            >>::from(__e.to_string()))?;
                        ::core::result::Result::<_, __JniErr>::Ok(__chain_s1)
                    }?,
                );
        }
        __sequence_values
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Vec_Option_u64_a34190e7<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<Vec<Option<u64>>, __JniErr> {
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
        let mut __out: Vec<Option<u64>> = Vec::new();
        while let Some(__obj) = __it
            .next(env)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-next: {}", e)))?
        {
            let __elem_wire: jni::objects::JObject = __obj.into();
            let __elem: Option<u64> = JObject_to_Option_u64_32be16a2(env, &__elem_wire)?;
            __out.push(__elem);
        }
        __out
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
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
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Vec_Vec_u8_43404875<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'a>,
) -> ::core::result::Result<Vec<Vec<u8>>, __JniErr> {
    ::core::result::Result::Ok({
        let __sequence_list = jni::objects::JList::from_env(env, v)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-from-env: {}", e)))?;
        let mut __sequence_iter = __sequence_list
            .iter(env)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-iter: {}", e)))?;
        let mut __sequence_values: ::std::vec::Vec<Vec<u8>> = ::std::vec::Vec::new();
        while let ::core::option::Option::Some(__sequence_part) = match __sequence_iter
            .next(env)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-next: {}", e)))?
        {
            ::core::option::Option::Some(__sequence_object) => {
                let __sequence_part: jni::objects::JByteArray = __sequence_object.into();
                ::core::option::Option::Some(__sequence_part)
            }
            ::core::option::Option::None => ::core::option::Option::None,
        } {
            __sequence_values
                .push(JByteArray_to_Vec_u8_7936d5de(env, &(__sequence_part))?);
        }
        __sequence_values
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_Verdict_a94c1ffd<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::Verdict, __JniErr> {
    Ok({
        let __id_raw: jni::sys::jlong = env
            .get_field(v, "id", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Verdict.id: {}", e)))? as _;
        let id = jlong_to_i64_fbf9a9bc(env, &__id_raw)?;
        let __outcome_raw: jni::objects::JObject = env
            .get_field(v, "outcome", "Lio/prebindgen/covertest/model/Lookup;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Verdict.outcome: {}", e)))?;
        let outcome = JObject_to_Lookup_94ada15e(env, &__outcome_raw)?;
        perftest_flat::Verdict {
            id,
            outcome,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_WrappedFields_f14f08c1<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::WrappedFields, __JniErr> {
    Ok({
        let __id_raw: jni::sys::jlong = env
            .get_field(v, "id", "J")
            .and_then(|val| val.j())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("WrappedFields.id: {}", e)))? as _;
        let id = jlong_to_i64_fbf9a9bc(env, &__id_raw)?;
        let __boxed_raw: jni::objects::JObject = env
            .get_field(v, "boxed", "Ljava/lang/Long;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("WrappedFields.boxed: {}", e)))?;
        let boxed = JObject_to_Box_Option_i64_cf5a3724(env, &__boxed_raw)?;
        let __plain_raw: jni::objects::JObject = env
            .get_field(v, "plain", "Ljava/lang/Long;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("WrappedFields.plain: {}", e)))?;
        let plain = JObject_to_Option_i64_2ba9a5ed(env, &__plain_raw)?;
        let __boxed_enum_jobj: jni::objects::JObject = env
            .get_field(v, "boxedEnum", "Lio/prebindgen/covertest/model/Priority;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("WrappedFields.boxedEnum: {}", e)))?;
        let __boxed_enum_raw: jni::sys::jint = env
            .call_method(&__boxed_enum_jobj, "getValue", "()I", &[])
            .and_then(|val| val.i())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("WrappedFields.boxedEnum: {}", e)))?;
        let boxed_enum = jint_to_Box_Priority_a16653ae(env, &__boxed_enum_raw)?;
        let __plain_enum_jobj: jni::objects::JObject = env
            .get_field(v, "plainEnum", "Lio/prebindgen/covertest/model/Priority;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("WrappedFields.plainEnum: {}", e)))?;
        let __plain_enum_raw: jni::sys::jint = env
            .call_method(&__plain_enum_jobj, "getValue", "()I", &[])
            .and_then(|val| val.i())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("WrappedFields.plainEnum: {}", e)))?;
        let plain_enum = jint_to_Priority_447102d2(env, &__plain_enum_raw)?;
        perftest_flat::WrappedFields {
            id,
            boxed,
            plain,
            boxed_enum,
            plain_enum,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_impl_Fn_Duration_Send_Sync_static_98c9f460<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<
    impl Fn(perftest_flat::Duration) + Send + Sync + 'static,
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
                format!("Unable to get callback class for {}: {}", "Fn(Duration)", e),
            ))?;
        let __invoke_id = env
            .get_method_id(&__invoke_class, "run", "(J)V")
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to resolve run for {}: {}", "Fn(Duration)", e)))?;
        Box::new(move |__cb_arg0: perftest_flat::Duration| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Attach thread for {}: {}", "Fn(Duration)", e)))?;
                env.push_local_frame(16)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        format!("push local frame for {}: {}", "Fn(Duration)", e),
                    ))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let __cb0_enc = {
                        let __cb0_s0 = Duration_to_u64_e3980876(&mut env, __cb_arg0)
                            .map_err(|__e| <__JniErr as ::core::convert::From<
                                String,
                            >>::from(__e.to_string()))?;
                        u64_to_jlong_4384a5d6(&mut env, __cb0_s0)?
                    };
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
                .map_err(|e| tracing::error!("{} callback error: {e}", "Fn(Duration)"));
        })
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_impl_Fn_Ledger_Send_Sync_static_c76008cc<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<
    impl Fn(perftest_flat::Ledger) + Send + Sync + 'static,
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
                format!("Unable to get callback class for {}: {}", "Fn(Ledger)", e),
            ))?;
        let __invoke_id = env
            .get_method_id(
                &__invoke_class,
                "run",
                "(Ljava/lang/Long;Ljava/lang/Double;Lio/prebindgen/covertest/model/Stamp;Ljava/lang/Long;Ljava/lang/Long;Ljava/lang/Integer;Ljava/lang/Long;Ljava/lang/String;Ljava/lang/String;Ljava/lang/Long;Ljava/lang/Double;Lio/prebindgen/covertest/model/Stamp;Ljava/lang/Long;Ljava/lang/Long;Ljava/lang/Integer;Ljava/lang/Long;Ljava/lang/String;Ljava/lang/String;)V",
            )
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to resolve run for {}: {}", "Fn(Ledger)", e)))?;
        Box::new(move |__cb_arg0: perftest_flat::Ledger| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Attach thread for {}: {}", "Fn(Ledger)", e)))?;
                env.push_local_frame(42)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("push local frame for {}: {}", "Fn(Ledger)", e)))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let __vf0 = perftest_flat::ledger_filed(&__cb_arg0)
                        .map(|__hb0| perftest_flat::report_into_struct((__hb0).clone()));
                    let __vf1 = perftest_flat::ledger_archived(&__cb_arg0)
                        .map(|__hb0| perftest_flat::report_into_struct(__hb0));
                    let (
                        __cb0_obj0,
                        __cb0_obj1,
                        __cb0_obj2,
                        __cb0_obj3,
                        __cb0_obj4,
                        __cb0_obj5,
                        __cb0_obj6,
                        __cb0_obj7,
                        __cb0_obj8,
                    ): (
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                    ) = match __vf0 {
                        ::core::option::Option::Some(__u0) => {
                            let __cb0_obj5: jni::objects::JObject;
                            let __cb0_obj6: jni::objects::JObject;
                            let __cb0_obj7: jni::objects::JObject;
                            match &__u0.outcome {
                                perftest_flat::Lookup::Absent => {
                                    __cb0_obj5 = match ::prebindgen_jni_runtime::box_jint(
                                        &mut env,
                                        0,
                                    ) {
                                        ::core::result::Result::Ok(__o) => __o,
                                        ::core::result::Result::Err(__e) => {
                                            return ::core::result::Result::Err(
                                                <__JniErr as ::core::convert::From<String>>::from(__e),
                                            );
                                        }
                                    };
                                    __cb0_obj6 = jni::objects::JObject::null();
                                    __cb0_obj7 = jni::objects::JObject::null();
                                }
                                perftest_flat::Lookup::Found(__sv0) => {
                                    let __enc___cb0_obj6 = match Summary_to_jlong_3cb103b9(
                                        &mut env,
                                        __sv0.clone(),
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
                                    __cb0_obj6 = match ::prebindgen_jni_runtime::box_jlong(
                                        &mut env,
                                        __enc___cb0_obj6,
                                    ) {
                                        ::core::result::Result::Ok(__o) => __o,
                                        ::core::result::Result::Err(__e) => {
                                            return ::core::result::Result::Err(
                                                <__JniErr as ::core::convert::From<String>>::from(__e),
                                            );
                                        }
                                    };
                                    __cb0_obj5 = match ::prebindgen_jni_runtime::box_jint(
                                        &mut env,
                                        1,
                                    ) {
                                        ::core::result::Result::Ok(__o) => __o,
                                        ::core::result::Result::Err(__e) => {
                                            return ::core::result::Result::Err(
                                                <__JniErr as ::core::convert::From<String>>::from(__e),
                                            );
                                        }
                                    };
                                    __cb0_obj7 = jni::objects::JObject::null();
                                }
                                perftest_flat::Lookup::Failed(__sv0) => {
                                    let __enc___cb0_obj7 = match String_to_JString_c7f3ca43(
                                        &mut env,
                                        __sv0.clone(),
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
                                    __cb0_obj7 = __enc___cb0_obj7.into();
                                    __cb0_obj5 = match ::prebindgen_jni_runtime::box_jint(
                                        &mut env,
                                        2,
                                    ) {
                                        ::core::result::Result::Ok(__o) => __o,
                                        ::core::result::Result::Err(__e) => {
                                            return ::core::result::Result::Err(
                                                <__JniErr as ::core::convert::From<String>>::from(__e),
                                            );
                                        }
                                    };
                                    __cb0_obj6 = jni::objects::JObject::null();
                                }
                            }
                            let __cb0_obj0: jni::objects::JObject = {
                                let __enc0 = match i64_to_jlong_fbf9a9bc(
                                    &mut env,
                                    perftest_flat::summary_count(&__u0.summary),
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
                                match ::prebindgen_jni_runtime::box_jlong(
                                    &mut env,
                                    __enc0,
                                ) {
                                    ::core::result::Result::Ok(__o) => __o,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<String>>::from(__e),
                                        );
                                    }
                                }
                            };
                            let __cb0_obj1: jni::objects::JObject = {
                                let __enc1 = match f64_to_jdouble_9e4a8f70(
                                    &mut env,
                                    perftest_flat::summary_total(&__u0.summary),
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
                                match ::prebindgen_jni_runtime::box_jdouble(
                                    &mut env,
                                    __enc1,
                                ) {
                                    ::core::result::Result::Ok(__o) => __o,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<String>>::from(__e),
                                        );
                                    }
                                }
                            };
                            let __cb0_obj2: jni::objects::JObject = {
                                let __enc2 = match Option_Stamp_to_JObject_6375b503(
                                    &mut env,
                                    __u0.taken,
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
                                __enc2
                            };
                            let __cb0_obj3: jni::objects::JObject = {
                                let __enc3 = match i64_to_jlong_fbf9a9bc(
                                    &mut env,
                                    __u0.origin.secs,
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
                                match ::prebindgen_jni_runtime::box_jlong(
                                    &mut env,
                                    __enc3,
                                ) {
                                    ::core::result::Result::Ok(__o) => __o,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<String>>::from(__e),
                                        );
                                    }
                                }
                            };
                            let __cb0_obj4: jni::objects::JObject = {
                                let __enc4 = match i64_to_jlong_fbf9a9bc(
                                    &mut env,
                                    __u0.origin.nanos,
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
                                match ::prebindgen_jni_runtime::box_jlong(
                                    &mut env,
                                    __enc4,
                                ) {
                                    ::core::result::Result::Ok(__o) => __o,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<String>>::from(__e),
                                        );
                                    }
                                }
                            };
                            let __cb0_obj8: jni::objects::JObject = {
                                let __enc8 = match String_to_JString_c7f3ca43(
                                    &mut env,
                                    __u0.label,
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
                                __enc8.into()
                            };
                            (
                                __cb0_obj0,
                                __cb0_obj1,
                                __cb0_obj2,
                                __cb0_obj3,
                                __cb0_obj4,
                                __cb0_obj5,
                                __cb0_obj6,
                                __cb0_obj7,
                                __cb0_obj8,
                            )
                        }
                        ::core::option::Option::None => {
                            (
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                            )
                        }
                    };
                    let (
                        __cb0_obj9,
                        __cb0_obj10,
                        __cb0_obj11,
                        __cb0_obj12,
                        __cb0_obj13,
                        __cb0_obj14,
                        __cb0_obj15,
                        __cb0_obj16,
                        __cb0_obj17,
                    ): (
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                    ) = match __vf1 {
                        ::core::option::Option::Some(__u1) => {
                            let __cb0_obj14: jni::objects::JObject;
                            let __cb0_obj15: jni::objects::JObject;
                            let __cb0_obj16: jni::objects::JObject;
                            match &__u1.outcome {
                                perftest_flat::Lookup::Absent => {
                                    __cb0_obj14 = match ::prebindgen_jni_runtime::box_jint(
                                        &mut env,
                                        0,
                                    ) {
                                        ::core::result::Result::Ok(__o) => __o,
                                        ::core::result::Result::Err(__e) => {
                                            return ::core::result::Result::Err(
                                                <__JniErr as ::core::convert::From<String>>::from(__e),
                                            );
                                        }
                                    };
                                    __cb0_obj15 = jni::objects::JObject::null();
                                    __cb0_obj16 = jni::objects::JObject::null();
                                }
                                perftest_flat::Lookup::Found(__sv0) => {
                                    let __enc___cb0_obj15 = match Summary_to_jlong_3cb103b9(
                                        &mut env,
                                        __sv0.clone(),
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
                                    __cb0_obj15 = match ::prebindgen_jni_runtime::box_jlong(
                                        &mut env,
                                        __enc___cb0_obj15,
                                    ) {
                                        ::core::result::Result::Ok(__o) => __o,
                                        ::core::result::Result::Err(__e) => {
                                            return ::core::result::Result::Err(
                                                <__JniErr as ::core::convert::From<String>>::from(__e),
                                            );
                                        }
                                    };
                                    __cb0_obj14 = match ::prebindgen_jni_runtime::box_jint(
                                        &mut env,
                                        1,
                                    ) {
                                        ::core::result::Result::Ok(__o) => __o,
                                        ::core::result::Result::Err(__e) => {
                                            return ::core::result::Result::Err(
                                                <__JniErr as ::core::convert::From<String>>::from(__e),
                                            );
                                        }
                                    };
                                    __cb0_obj16 = jni::objects::JObject::null();
                                }
                                perftest_flat::Lookup::Failed(__sv0) => {
                                    let __enc___cb0_obj16 = match String_to_JString_c7f3ca43(
                                        &mut env,
                                        __sv0.clone(),
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
                                    __cb0_obj16 = __enc___cb0_obj16.into();
                                    __cb0_obj14 = match ::prebindgen_jni_runtime::box_jint(
                                        &mut env,
                                        2,
                                    ) {
                                        ::core::result::Result::Ok(__o) => __o,
                                        ::core::result::Result::Err(__e) => {
                                            return ::core::result::Result::Err(
                                                <__JniErr as ::core::convert::From<String>>::from(__e),
                                            );
                                        }
                                    };
                                    __cb0_obj15 = jni::objects::JObject::null();
                                }
                            }
                            let __cb0_obj9: jni::objects::JObject = {
                                let __enc9 = match i64_to_jlong_fbf9a9bc(
                                    &mut env,
                                    perftest_flat::summary_count(&__u1.summary),
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
                                match ::prebindgen_jni_runtime::box_jlong(
                                    &mut env,
                                    __enc9,
                                ) {
                                    ::core::result::Result::Ok(__o) => __o,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<String>>::from(__e),
                                        );
                                    }
                                }
                            };
                            let __cb0_obj10: jni::objects::JObject = {
                                let __enc10 = match f64_to_jdouble_9e4a8f70(
                                    &mut env,
                                    perftest_flat::summary_total(&__u1.summary),
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
                                match ::prebindgen_jni_runtime::box_jdouble(
                                    &mut env,
                                    __enc10,
                                ) {
                                    ::core::result::Result::Ok(__o) => __o,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<String>>::from(__e),
                                        );
                                    }
                                }
                            };
                            let __cb0_obj11: jni::objects::JObject = {
                                let __enc11 = match Option_Stamp_to_JObject_6375b503(
                                    &mut env,
                                    __u1.taken,
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
                                __enc11
                            };
                            let __cb0_obj12: jni::objects::JObject = {
                                let __enc12 = match i64_to_jlong_fbf9a9bc(
                                    &mut env,
                                    __u1.origin.secs,
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
                                match ::prebindgen_jni_runtime::box_jlong(
                                    &mut env,
                                    __enc12,
                                ) {
                                    ::core::result::Result::Ok(__o) => __o,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<String>>::from(__e),
                                        );
                                    }
                                }
                            };
                            let __cb0_obj13: jni::objects::JObject = {
                                let __enc13 = match i64_to_jlong_fbf9a9bc(
                                    &mut env,
                                    __u1.origin.nanos,
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
                                match ::prebindgen_jni_runtime::box_jlong(
                                    &mut env,
                                    __enc13,
                                ) {
                                    ::core::result::Result::Ok(__o) => __o,
                                    ::core::result::Result::Err(__e) => {
                                        return ::core::result::Result::Err(
                                            <__JniErr as ::core::convert::From<String>>::from(__e),
                                        );
                                    }
                                }
                            };
                            let __cb0_obj17: jni::objects::JObject = {
                                let __enc17 = match String_to_JString_c7f3ca43(
                                    &mut env,
                                    __u1.label,
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
                                __enc17.into()
                            };
                            (
                                __cb0_obj9,
                                __cb0_obj10,
                                __cb0_obj11,
                                __cb0_obj12,
                                __cb0_obj13,
                                __cb0_obj14,
                                __cb0_obj15,
                                __cb0_obj16,
                                __cb0_obj17,
                            )
                        }
                        ::core::option::Option::None => {
                            (
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                                jni::objects::JObject::null(),
                            )
                        }
                    };
                    let __call_res: ::core::result::Result<(), __JniErr> = unsafe {
                        env.call_method_unchecked(
                            &callback_global_ref,
                            __invoke_id,
                            jni::signature::ReturnType::Primitive(
                                jni::signature::Primitive::Void,
                            ),
                            &[
                                jni::sys::jvalue {
                                    l: __cb0_obj0.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj1.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj2.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj3.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj4.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj5.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj6.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj7.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj8.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj9.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj10.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj11.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj12.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj13.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj14.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj15.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj16.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj17.as_raw(),
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
                .map_err(|e| tracing::error!("{} callback error: {e}", "Fn(Ledger)"));
        })
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_impl_Fn_Lookup_Send_Sync_static_4a65bc23<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<
    impl Fn(perftest_flat::Lookup) + Send + Sync + 'static,
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
                format!("Unable to get callback class for {}: {}", "Fn(Lookup)", e),
            ))?;
        let __invoke_id = env
            .get_method_id(&__invoke_class, "run", "(IJLjava/lang/String;)V")
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to resolve run for {}: {}", "Fn(Lookup)", e)))?;
        Box::new(move |__cb_arg0: perftest_flat::Lookup| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Attach thread for {}: {}", "Fn(Lookup)", e)))?;
                env.push_local_frame(16)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("push local frame for {}: {}", "Fn(Lookup)", e)))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let (__chain_wire0, (), (__chain_wire1,), (__chain_wire2,)) = match Lookup_to_tuple4_68d54df3(
                        &mut env,
                        __cb_arg0,
                    ) {
                        ::core::result::Result::Ok(__intermediate) => __intermediate,
                        ::core::result::Result::Err(__chain_error) => {
                            return ::core::result::Result::Err(
                                <__JniErr as ::core::convert::From<
                                    String,
                                >>::from(__chain_error.to_string()),
                            );
                        }
                    };
                    let __cb0_obj0 = jni::sys::jvalue {
                        i: __chain_wire0,
                    };
                    let __cb0_obj1 = jni::sys::jvalue {
                        j: __chain_wire1,
                    };
                    let __cb0_obj2: jni::objects::JObject = __chain_wire2.into();
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
                                jni::sys::jvalue {
                                    l: __cb0_obj2.as_raw(),
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
                .map_err(|e| tracing::error!("{} callback error: {e}", "Fn(Lookup)"));
        })
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_impl_Fn_Option_CallbackHolder_Send_Sync_static_b3ec3f73<
    'env,
    'v,
>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<
    impl Fn(Option<perftest_flat::CallbackHolder>) + Send + Sync + 'static,
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
                format!(
                    "Unable to get callback class for {}: {}",
                    "Fn(Option < CallbackHolder >)", e
                ),
            ))?;
        let __invoke_id = env
            .get_method_id(&__invoke_class, "run", "(ZJJ)V")
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(
                format!(
                    "Unable to resolve run for {}: {}", "Fn(Option < CallbackHolder >)",
                    e
                ),
            ))?;
        Box::new(move |__cb_arg0: Option<perftest_flat::CallbackHolder>| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        format!(
                            "Attach thread for {}: {}", "Fn(Option < CallbackHolder >)",
                            e
                        ),
                    ))?;
                env.push_local_frame(16)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        format!(
                            "push local frame for {}: {}",
                            "Fn(Option < CallbackHolder >)", e
                        ),
                    ))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let (__chain_present, (__chain_wire0, __chain_wire1)) = match Option_CallbackHolder_to_tuple2_6762df4a(
                        &mut env,
                        __cb_arg0,
                    ) {
                        ::core::result::Result::Ok(__intermediate) => __intermediate,
                        ::core::result::Result::Err(__chain_error) => {
                            return ::core::result::Result::Err(
                                <__JniErr as ::core::convert::From<
                                    String,
                                >>::from(__chain_error.to_string()),
                            );
                        }
                    };
                    let __cb0_obj0 = jni::sys::jvalue {
                        j: __chain_wire0,
                    };
                    let __cb0_obj1 = jni::sys::jvalue {
                        j: __chain_wire1,
                    };
                    let __call_res: ::core::result::Result<(), __JniErr> = unsafe {
                        env.call_method_unchecked(
                            &callback_global_ref,
                            __invoke_id,
                            jni::signature::ReturnType::Primitive(
                                jni::signature::Primitive::Void,
                            ),
                            &[
                                jni::sys::jvalue {
                                    z: __chain_present,
                                },
                                __cb0_obj0,
                                __cb0_obj1,
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
                    tracing::error!(
                        "{} callback error: {e}", "Fn(Option < CallbackHolder >)"
                    )
                });
        })
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_impl_Fn_Option_Payload_Send_Sync_static_b308aaa4<
    'env,
    'v,
>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<
    impl Fn(Option<perftest_flat::Payload>) + Send + Sync + 'static,
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
                format!(
                    "Unable to get callback class for {}: {}", "Fn(Option < Payload >)",
                    e
                ),
            ))?;
        let __invoke_id = env
            .get_method_id(&__invoke_class, "run", "(ZJIDZLjava/lang/String;)V")
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(
                format!("Unable to resolve run for {}: {}", "Fn(Option < Payload >)", e),
            ))?;
        Box::new(move |__cb_arg0: Option<perftest_flat::Payload>| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        format!("Attach thread for {}: {}", "Fn(Option < Payload >)", e),
                    ))?;
                env.push_local_frame(18)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        format!(
                            "push local frame for {}: {}", "Fn(Option < Payload >)", e
                        ),
                    ))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let (
                        __chain_present,
                        (
                            __chain_wire0,
                            __chain_wire1,
                            __chain_wire2,
                            __chain_wire3,
                            __chain_wire4,
                        ),
                    ) = match Option_Payload_to_tuple2_af2bd54b(&mut env, __cb_arg0) {
                        ::core::result::Result::Ok(__intermediate) => __intermediate,
                        ::core::result::Result::Err(__chain_error) => {
                            return ::core::result::Result::Err(
                                <__JniErr as ::core::convert::From<
                                    String,
                                >>::from(__chain_error.to_string()),
                            );
                        }
                    };
                    let __cb0_obj0 = jni::sys::jvalue {
                        j: __chain_wire0,
                    };
                    let __cb0_obj1 = jni::sys::jvalue {
                        i: __chain_wire1,
                    };
                    let __cb0_obj2 = jni::sys::jvalue {
                        d: __chain_wire2,
                    };
                    let __cb0_obj3 = jni::sys::jvalue {
                        z: __chain_wire3,
                    };
                    let __cb0_obj4: jni::objects::JObject = __chain_wire4.into();
                    let __call_res: ::core::result::Result<(), __JniErr> = unsafe {
                        env.call_method_unchecked(
                            &callback_global_ref,
                            __invoke_id,
                            jni::signature::ReturnType::Primitive(
                                jni::signature::Primitive::Void,
                            ),
                            &[
                                jni::sys::jvalue {
                                    z: __chain_present,
                                },
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
                .map_err(|e| {
                    tracing::error!("{} callback error: {e}", "Fn(Option < Payload >)")
                });
        })
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
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
                            let (
                                __chain_wire0,
                                __chain_wire1,
                                __chain_wire2,
                                __chain_wire3,
                                __chain_wire4,
                            ) = match Payload_to_tuple5_2ea1d0c2(&mut env, __cb_elem) {
                                ::core::result::Result::Ok(__intermediate) => __intermediate,
                                ::core::result::Result::Err(__chain_error) => {
                                    return ::core::result::Result::Err(
                                        <__JniErr as ::core::convert::From<
                                            String,
                                        >>::from(__chain_error.to_string()),
                                    );
                                }
                            };
                            let __cbfold0_obj0 = jni::sys::jvalue {
                                j: __chain_wire0,
                            };
                            let __cbfold0_obj1 = jni::sys::jvalue {
                                i: __chain_wire1,
                            };
                            let __cbfold0_obj2 = jni::sys::jvalue {
                                d: __chain_wire2,
                            };
                            let __cbfold0_obj3 = jni::sys::jvalue {
                                z: __chain_wire3,
                            };
                            let __cbfold0_obj4: jni::objects::JObject = __chain_wire4
                                .into();
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
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
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
                    let (
                        __chain_wire0,
                        __chain_wire1,
                        __chain_wire2,
                        __chain_wire3,
                        __chain_wire4,
                    ) = match Payload_to_tuple5_2ea1d0c2(&mut env, __cb_arg0) {
                        ::core::result::Result::Ok(__intermediate) => __intermediate,
                        ::core::result::Result::Err(__chain_error) => {
                            return ::core::result::Result::Err(
                                <__JniErr as ::core::convert::From<
                                    String,
                                >>::from(__chain_error.to_string()),
                            );
                        }
                    };
                    let __cb0_obj0 = jni::sys::jvalue {
                        j: __chain_wire0,
                    };
                    let __cb0_obj1 = jni::sys::jvalue {
                        i: __chain_wire1,
                    };
                    let __cb0_obj2 = jni::sys::jvalue {
                        d: __chain_wire2,
                    };
                    let __cb0_obj3 = jni::sys::jvalue {
                        z: __chain_wire3,
                    };
                    let __cb0_obj4: jni::objects::JObject = __chain_wire4.into();
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
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_impl_Fn_Probe_Send_Sync_static_b0418db6<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<
    impl Fn(perftest_flat::Probe) + Send + Sync + 'static,
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
                format!("Unable to get callback class for {}: {}", "Fn(Probe)", e),
            ))?;
        let __invoke_id = env
            .get_method_id(
                &__invoke_class,
                "run",
                "(JLjava/lang/Integer;Ljava/lang/Long;Ljava/lang/String;)V",
            )
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to resolve run for {}: {}", "Fn(Probe)", e)))?;
        Box::new(move |__cb_arg0: perftest_flat::Probe| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Attach thread for {}: {}", "Fn(Probe)", e)))?;
                env.push_local_frame(16)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("push local frame for {}: {}", "Fn(Probe)", e)))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let __vf0 = perftest_flat::probe_to_struct(&__cb_arg0);
                    let (
                        __cb0_obj1,
                        __cb0_obj2,
                        __cb0_obj3,
                    ): (
                        jni::objects::JObject,
                        jni::objects::JObject,
                        jni::objects::JObject,
                    ) = {
                        let __so1: &::core::option::Option<_> = &(&__vf0).outcome;
                        match __so1 {
                            ::core::option::Option::Some(__sg1) => {
                                let __cb0_obj1: jni::objects::JObject;
                                let __cb0_obj2: jni::objects::JObject;
                                let __cb0_obj3: jni::objects::JObject;
                                match __sg1 {
                                    perftest_flat::Lookup::Absent => {
                                        __cb0_obj1 = match ::prebindgen_jni_runtime::box_jint(
                                            &mut env,
                                            0,
                                        ) {
                                            ::core::result::Result::Ok(__o) => __o,
                                            ::core::result::Result::Err(__e) => {
                                                return ::core::result::Result::Err(
                                                    <__JniErr as ::core::convert::From<String>>::from(__e),
                                                );
                                            }
                                        };
                                        __cb0_obj2 = jni::objects::JObject::null();
                                        __cb0_obj3 = jni::objects::JObject::null();
                                    }
                                    perftest_flat::Lookup::Found(__sv0) => {
                                        let __enc___cb0_obj2 = match Summary_to_jlong_3cb103b9(
                                            &mut env,
                                            __sv0.clone(),
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
                                        __cb0_obj2 = match ::prebindgen_jni_runtime::box_jlong(
                                            &mut env,
                                            __enc___cb0_obj2,
                                        ) {
                                            ::core::result::Result::Ok(__o) => __o,
                                            ::core::result::Result::Err(__e) => {
                                                return ::core::result::Result::Err(
                                                    <__JniErr as ::core::convert::From<String>>::from(__e),
                                                );
                                            }
                                        };
                                        __cb0_obj1 = match ::prebindgen_jni_runtime::box_jint(
                                            &mut env,
                                            1,
                                        ) {
                                            ::core::result::Result::Ok(__o) => __o,
                                            ::core::result::Result::Err(__e) => {
                                                return ::core::result::Result::Err(
                                                    <__JniErr as ::core::convert::From<String>>::from(__e),
                                                );
                                            }
                                        };
                                        __cb0_obj3 = jni::objects::JObject::null();
                                    }
                                    perftest_flat::Lookup::Failed(__sv0) => {
                                        let __enc___cb0_obj3 = match String_to_JString_c7f3ca43(
                                            &mut env,
                                            __sv0.clone(),
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
                                        __cb0_obj3 = __enc___cb0_obj3.into();
                                        __cb0_obj1 = match ::prebindgen_jni_runtime::box_jint(
                                            &mut env,
                                            2,
                                        ) {
                                            ::core::result::Result::Ok(__o) => __o,
                                            ::core::result::Result::Err(__e) => {
                                                return ::core::result::Result::Err(
                                                    <__JniErr as ::core::convert::From<String>>::from(__e),
                                                );
                                            }
                                        };
                                        __cb0_obj2 = jni::objects::JObject::null();
                                    }
                                }
                                (__cb0_obj1, __cb0_obj2, __cb0_obj3)
                            }
                            ::core::option::Option::None => {
                                (
                                    jni::objects::JObject::null(),
                                    jni::objects::JObject::null(),
                                    jni::objects::JObject::null(),
                                )
                            }
                        }
                    };
                    let __cb0_obj0: jni::sys::jvalue = {
                        let __enc0 = match i64_to_jlong_fbf9a9bc(
                            &mut env,
                            __vf0.seq.clone(),
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
                    let __call_res: ::core::result::Result<(), __JniErr> = unsafe {
                        env.call_method_unchecked(
                            &callback_global_ref,
                            __invoke_id,
                            jni::signature::ReturnType::Primitive(
                                jni::signature::Primitive::Void,
                            ),
                            &[
                                __cb0_obj0,
                                jni::sys::jvalue {
                                    l: __cb0_obj1.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj2.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj3.as_raw(),
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
                .map_err(|e| tracing::error!("{} callback error: {e}", "Fn(Probe)"));
        })
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_impl_Fn_Reading_Send_Sync_static_5964f1fc<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<
    impl Fn(perftest_flat::Reading) + Send + Sync + 'static,
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
                format!("Unable to get callback class for {}: {}", "Fn(Reading)", e),
            ))?;
        let __invoke_id = env
            .get_method_id(&__invoke_class, "run", "(IJJJLjava/lang/String;IJ)V")
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to resolve run for {}: {}", "Fn(Reading)", e)))?;
        Box::new(move |__cb_arg0: perftest_flat::Reading| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Attach thread for {}: {}", "Fn(Reading)", e)))?;
                env.push_local_frame(20)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("push local frame for {}: {}", "Fn(Reading)", e)))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let (
                        __chain_wire0,
                        (),
                        (__chain_wire1,),
                        (__chain_wire2, __chain_wire3),
                        (__chain_wire4, __chain_wire5),
                        (__chain_wire6,),
                    ) = match Reading_to_tuple6_69702d1f(&mut env, __cb_arg0) {
                        ::core::result::Result::Ok(__intermediate) => __intermediate,
                        ::core::result::Result::Err(__chain_error) => {
                            return ::core::result::Result::Err(
                                <__JniErr as ::core::convert::From<
                                    String,
                                >>::from(__chain_error.to_string()),
                            );
                        }
                    };
                    let __cb0_obj0 = jni::sys::jvalue {
                        i: __chain_wire0,
                    };
                    let __cb0_obj1 = jni::sys::jvalue {
                        j: __chain_wire1,
                    };
                    let __cb0_obj2 = jni::sys::jvalue {
                        j: __chain_wire2,
                    };
                    let __cb0_obj3 = jni::sys::jvalue {
                        j: __chain_wire3,
                    };
                    let __cb0_obj4: jni::objects::JObject = __chain_wire4.into();
                    let __cb0_obj5 = jni::sys::jvalue {
                        i: __chain_wire5,
                    };
                    let __cb0_obj6 = jni::sys::jvalue {
                        j: __chain_wire6,
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
                                __cb0_obj5,
                                __cb0_obj6,
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
                .map_err(|e| tracing::error!("{} callback error: {e}", "Fn(Reading)"));
        })
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_impl_Fn_Report_Send_Sync_static_eb5ca515<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<
    impl Fn(perftest_flat::Report) + Send + Sync + 'static,
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
                format!("Unable to get callback class for {}: {}", "Fn(Report)", e),
            ))?;
        let __invoke_id = env
            .get_method_id(
                &__invoke_class,
                "run",
                "(JDLio/prebindgen/covertest/model/Stamp;JJIJLjava/lang/String;Ljava/lang/String;)V",
            )
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to resolve run for {}: {}", "Fn(Report)", e)))?;
        Box::new(move |__cb_arg0: perftest_flat::Report| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Attach thread for {}: {}", "Fn(Report)", e)))?;
                env.push_local_frame(24)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("push local frame for {}: {}", "Fn(Report)", e)))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let __vf0 = perftest_flat::report_into_struct(__cb_arg0);
                    let __cb0_obj5: jni::sys::jvalue;
                    let __cb0_obj6: jni::sys::jvalue;
                    let __cb0_obj7: jni::objects::JObject;
                    match &__vf0.outcome {
                        perftest_flat::Lookup::Absent => {
                            __cb0_obj5 = jni::sys::jvalue { i: 0 };
                            __cb0_obj6 = jni::sys::jvalue { j: 0i64 };
                            __cb0_obj7 = jni::objects::JObject::null();
                        }
                        perftest_flat::Lookup::Found(__sv0) => {
                            let __enc___cb0_obj6 = match Summary_to_jlong_3cb103b9(
                                &mut env,
                                __sv0.clone(),
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
                            __cb0_obj6 = jni::sys::jvalue {
                                j: __enc___cb0_obj6,
                            };
                            __cb0_obj5 = jni::sys::jvalue { i: 1 };
                            __cb0_obj7 = jni::objects::JObject::null();
                        }
                        perftest_flat::Lookup::Failed(__sv0) => {
                            let __enc___cb0_obj7 = match String_to_JString_c7f3ca43(
                                &mut env,
                                __sv0.clone(),
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
                            __cb0_obj7 = __enc___cb0_obj7.into();
                            __cb0_obj5 = jni::sys::jvalue { i: 2 };
                            __cb0_obj6 = jni::sys::jvalue { j: 0i64 };
                        }
                    }
                    let __cb0_obj0: jni::sys::jvalue = {
                        let __enc0 = match i64_to_jlong_fbf9a9bc(
                            &mut env,
                            perftest_flat::summary_count(&__vf0.summary),
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
                        let __enc1 = match f64_to_jdouble_9e4a8f70(
                            &mut env,
                            perftest_flat::summary_total(&__vf0.summary),
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
                        jni::sys::jvalue { d: __enc1 }
                    };
                    let __cb0_obj2: jni::objects::JObject = {
                        let __enc2 = match Option_Stamp_to_JObject_6375b503(
                            &mut env,
                            __vf0.taken,
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
                        __enc2
                    };
                    let __cb0_obj3: jni::sys::jvalue = {
                        let __enc3 = match i64_to_jlong_fbf9a9bc(
                            &mut env,
                            __vf0.origin.secs,
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
                        jni::sys::jvalue { j: __enc3 }
                    };
                    let __cb0_obj4: jni::sys::jvalue = {
                        let __enc4 = match i64_to_jlong_fbf9a9bc(
                            &mut env,
                            __vf0.origin.nanos,
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
                        jni::sys::jvalue { j: __enc4 }
                    };
                    let __cb0_obj8: jni::objects::JObject = {
                        let __enc8 = match String_to_JString_c7f3ca43(
                            &mut env,
                            __vf0.label,
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
                        __enc8.into()
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
                                jni::sys::jvalue {
                                    l: __cb0_obj2.as_raw(),
                                },
                                __cb0_obj3,
                                __cb0_obj4,
                                __cb0_obj5,
                                __cb0_obj6,
                                jni::sys::jvalue {
                                    l: __cb0_obj7.as_raw(),
                                },
                                jni::sys::jvalue {
                                    l: __cb0_obj8.as_raw(),
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
                .map_err(|e| tracing::error!("{} callback error: {e}", "Fn(Report)"));
        })
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
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
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_impl_Fn_Vec_Option_Ticks_Send_Sync_static_26c17cf0<
    'env,
    'v,
>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<
    impl Fn(Vec<Option<perftest_flat::Ticks>>) + Send + Sync + 'static,
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
                format!(
                    "Unable to get callback class for {}: {}",
                    "Fn(Vec < Option < Ticks > >)", e
                ),
            ))?;
        let __invoke_id = env
            .get_method_id(&__invoke_class, "run", "(Ljava/util/List;)V")
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(
                format!(
                    "Unable to resolve run for {}: {}", "Fn(Vec < Option < Ticks > >)", e
                ),
            ))?;
        Box::new(move |__cb_arg0: Vec<Option<perftest_flat::Ticks>>| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        format!(
                            "Attach thread for {}: {}", "Fn(Vec < Option < Ticks > >)", e
                        ),
                    ))?;
                env.push_local_frame(16)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(
                        format!(
                            "push local frame for {}: {}",
                            "Fn(Vec < Option < Ticks > >)", e
                        ),
                    ))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let __cb0_enc = Vec_Option_Ticks_to_JObject_2f4b03da(
                        &mut env,
                        __cb_arg0,
                    )?;
                    let __cb0_obj: jni::objects::JObject = __cb0_enc;
                    let __call_res: ::core::result::Result<(), __JniErr> = unsafe {
                        env.call_method_unchecked(
                            &callback_global_ref,
                            __invoke_id,
                            jni::signature::ReturnType::Primitive(
                                jni::signature::Primitive::Void,
                            ),
                            &[
                                jni::sys::jvalue {
                                    l: __cb0_obj.as_raw(),
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
                    tracing::error!(
                        "{} callback error: {e}", "Fn(Vec < Option < Ticks > >)"
                    )
                });
        })
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JObject_to_impl_Fn_u64_Send_Sync_static_c7830b57<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<impl Fn(u64) + Send + Sync + 'static, __JniErr> {
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
            >>::from(format!("Unable to get callback class for {}: {}", "Fn(u64)", e)))?;
        let __invoke_id = env
            .get_method_id(&__invoke_class, "run", "(J)V")
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to resolve run for {}: {}", "Fn(u64)", e)))?;
        Box::new(move |__cb_arg0: u64| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Attach thread for {}: {}", "Fn(u64)", e)))?;
                env.push_local_frame(16)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("push local frame for {}: {}", "Fn(u64)", e)))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let __cb0_enc = u64_to_jlong_4384a5d6(&mut env, __cb_arg0)?;
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
                .map_err(|e| tracing::error!("{} callback error: {e}", "Fn(u64)"));
        })
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JShortArray_to_i16_2_098f4ad5<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JShortArray<'v>,
) -> ::core::result::Result<[i16; 2], __JniErr> {
    Ok({
        let __len = env
            .get_array_length(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })? as usize;
        let mut __buf: ::std::vec::Vec<jni::sys::jshort> = ::std::vec![
            0 as jni::sys::jshort; __len
        ];
        env.get_short_array_region(v, 0, &mut __buf)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array decode: {}", e))
            })?;
        let __vals: ::std::vec::Vec<i16> = __buf.iter().map(|__x| *__x as i16).collect();
        let __arr: [i16; 2] = __vals
            .as_slice()
            .try_into()
            .map_err(|_| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    "fixed-size array decode: `[i16 ; 2]` expects a different length"
                        .to_string(),
                )
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JString_to_Box_Option_String_caeff346<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JString<'v>,
) -> ::core::result::Result<Box<Option<String>>, __JniErr> {
    ::core::result::Result::Ok(
        ::std::boxed::Box::new({
            if v.is_null() {
                ::core::option::Option::None
            } else {
                let __present = v;
                ::core::option::Option::Some(JString_to_String_c7f3ca43(env, __present)?)
            }
        }),
    )
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JString_to_Box_String_027f6250<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JString<'v>,
) -> ::core::result::Result<Box<String>, __JniErr> {
    Ok({
        let s = env
            .get_string(v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("decode_string: {}", e))
            })?;
        ::std::string::String::from(s).into()
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JString_to_Option_Box_String_071e4c8c<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JString<'v>,
) -> ::core::result::Result<Option<Box<String>>, __JniErr> {
    ::core::result::Result::Ok({
        if v.is_null() {
            ::core::option::Option::None
        } else {
            let __present = v;
            ::core::option::Option::Some(JString_to_Box_String_027f6250(env, __present)?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn JString_to_Option_String_56d5e304<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JString<'v>,
) -> ::core::result::Result<Option<String>, __JniErr> {
    ::core::result::Result::Ok({
        if v.is_null() {
            ::core::option::Option::None
        } else {
            let __present = v;
            ::core::option::Option::Some(JString_to_String_c7f3ca43(env, __present)?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
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
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Label_to_String_63dec766<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Label,
) -> ::core::result::Result<String, __JniErr> {
    Ok(crate::label_out(v))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn Layered_to_tuple8_4f5948ba<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Layered,
) -> ::core::result::Result<
    (
        jni::sys::jint,
        (jni::objects::JObject<'a>,),
        (jni::sys::jlong,),
        (jni::objects::JObject<'a>,),
        (jni::objects::JObject<'a>,),
        (jni::objects::JObject<'a>,),
        (jni::objects::JByteArray<'a>,),
        (jni::sys::jlong,),
    ),
    __JniErr,
> {
    ::core::result::Result::Ok({
        match v {
            perftest_flat::Layered::Count(__part0) => {
                (
                    0i32,
                    (Option_u64_to_JObject_32be16a2(env, __part0)?,),
                    (0 as jni::sys::jlong,),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (0 as jni::sys::jlong,),
                )
            }
            perftest_flat::Layered::Held(__part0) => {
                (
                    1i32,
                    (jni::objects::JObject::null().into(),),
                    (Option_Summary_to_jlong_252ef2ba(env, __part0)?,),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (0 as jni::sys::jlong,),
                )
            }
            perftest_flat::Layered::Many(__part0) => {
                (
                    2i32,
                    (jni::objects::JObject::null().into(),),
                    (0 as jni::sys::jlong,),
                    (Vec_Option_u64_to_JObject_a34190e7(env, __part0)?,),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (0 as jni::sys::jlong,),
                )
            }
            perftest_flat::Layered::Values(__part0) => {
                (
                    3i32,
                    (jni::objects::JObject::null().into(),),
                    (0 as jni::sys::jlong,),
                    (jni::objects::JObject::null().into(),),
                    (Option_Vec_Option_u64_to_JObject_006312b6(env, __part0)?,),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (0 as jni::sys::jlong,),
                )
            }
            perftest_flat::Layered::Nested(__part0) => {
                (
                    4i32,
                    (jni::objects::JObject::null().into(),),
                    (0 as jni::sys::jlong,),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (Vec_Vec_Option_u64_to_JObject_342a76c6(env, __part0)?,),
                    (jni::objects::JObject::null().into(),),
                    (0 as jni::sys::jlong,),
                )
            }
            perftest_flat::Layered::Blob(__part0) => {
                (
                    5i32,
                    (jni::objects::JObject::null().into(),),
                    (0 as jni::sys::jlong,),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (Vec_u8_to_JByteArray_7936d5de(env, __part0)?,),
                    (0 as jni::sys::jlong,),
                )
            }
            perftest_flat::Layered::Plain(__part0) => {
                (
                    6i32,
                    (jni::objects::JObject::null().into(),),
                    (0 as jni::sys::jlong,),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (jni::objects::JObject::null().into(),),
                    (i64_to_jlong_fbf9a9bc(env, __part0)?,),
                )
            }
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn Lookup_to_tuple4_68d54df3<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Lookup,
) -> ::core::result::Result<
    (jni::sys::jint, (), (jni::sys::jlong,), (jni::objects::JString<'a>,)),
    __JniErr,
> {
    ::core::result::Result::Ok({
        match v {
            perftest_flat::Lookup::Absent => {
                (
                    0i32,
                    (),
                    (0 as jni::sys::jlong,),
                    (jni::objects::JObject::null().into(),),
                )
            }
            perftest_flat::Lookup::Found(__part0) => {
                (
                    1i32,
                    (),
                    (Summary_to_jlong_3cb103b9(env, __part0)?,),
                    (jni::objects::JObject::null().into(),),
                )
            }
            perftest_flat::Lookup::Failed(__part0) => {
                (
                    2i32,
                    (),
                    (0 as jni::sys::jlong,),
                    (String_to_JString_c7f3ca43(env, __part0)?,),
                )
            }
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn Marker_to_tuple3_8b7f3646<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Marker,
) -> ::core::result::Result<
    (jni::sys::jint, (), (jni::objects::JObject<'a>,)),
    __JniErr,
> {
    ::core::result::Result::Ok({
        match v {
            perftest_flat::Marker::None_ => {
                (0i32, (), (jni::objects::JObject::null().into(),))
            }
            perftest_flat::Marker::Ranked(__part0) => {
                (1i32, (), (Option_Priority_to_JObject_ad5cbb32(env, __part0)?,))
            }
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn MaybeHolder_to_JObject_1c68fbac<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::MaybeHolder,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___tag: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.tag.clone())?;
        let ___summary: jni::sys::jlong = Option_Summary_to_jlong_252ef2ba(
            env,
            v.summary.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/MaybeHolder",
                "fromParts",
                "(JJ)Lio/prebindgen/covertest/MaybeHolder;",
                &[
                    jni::objects::JValue::from(___tag),
                    jni::objects::JValue::from(___summary),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Millis_to_i64_61ecf054<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Millis,
) -> ::core::result::Result<i64, __JniErr> {
    Ok(cov_helpers::millis_value(&v))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn ObjectBoundary16_to_JObject_e9d41606<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary16,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.value.clone(),
        )?;
        let ___left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.value.clone(),
        )?;
        let ___left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.value.clone(),
        )?;
        let ___left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.value.clone(),
        )?;
        let ___left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.value.clone(),
        )?;
        let ___left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.value.clone(),
        )?;
        let ___left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.value.clone(),
        )?;
        let ___left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.value.clone(),
        )?;
        let ___right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.left.value.clone(),
        )?;
        let ___right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.right.value.clone(),
        )?;
        let ___right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.left.value.clone(),
        )?;
        let ___right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.right.value.clone(),
        )?;
        let ___right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.left.value.clone(),
        )?;
        let ___right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.right.value.clone(),
        )?;
        let ___right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.left.value.clone(),
        )?;
        let ___right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.right.value.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/ObjectBoundary16",
                "fromParts",
                "(JJJJJJJJJJJJJJJJ)Lio/prebindgen/covertest/model/ObjectBoundary16;",
                &[
                    jni::objects::JValue::from(___left_left_left_left_value),
                    jni::objects::JValue::from(___left_left_left_right_value),
                    jni::objects::JValue::from(___left_left_right_left_value),
                    jni::objects::JValue::from(___left_left_right_right_value),
                    jni::objects::JValue::from(___left_right_left_left_value),
                    jni::objects::JValue::from(___left_right_left_right_value),
                    jni::objects::JValue::from(___left_right_right_left_value),
                    jni::objects::JValue::from(___left_right_right_right_value),
                    jni::objects::JValue::from(___right_left_left_left_value),
                    jni::objects::JValue::from(___right_left_left_right_value),
                    jni::objects::JValue::from(___right_left_right_left_value),
                    jni::objects::JValue::from(___right_left_right_right_value),
                    jni::objects::JValue::from(___right_right_left_left_value),
                    jni::objects::JValue::from(___right_right_left_right_value),
                    jni::objects::JValue::from(___right_right_right_left_value),
                    jni::objects::JValue::from(___right_right_right_right_value),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn ObjectBoundary2_to_JObject_a8f288cc<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary2,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.value.clone(),
        )?;
        let ___right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.value.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/ObjectBoundary2",
                "fromParts",
                "(JJ)Lio/prebindgen/covertest/model/ObjectBoundary2;",
                &[
                    jni::objects::JValue::from(___left_value),
                    jni::objects::JValue::from(___right_value),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn ObjectBoundary32_to_JObject_ed80fac3<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary32,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___left_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.left.value.clone(),
        )?;
        let ___left_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.right.value.clone(),
        )?;
        let ___left_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.left.value.clone(),
        )?;
        let ___left_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.right.value.clone(),
        )?;
        let ___left_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.left.value.clone(),
        )?;
        let ___left_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.right.value.clone(),
        )?;
        let ___left_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.left.value.clone(),
        )?;
        let ___left_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.right.value.clone(),
        )?;
        let ___left_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.left.value.clone(),
        )?;
        let ___left_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.right.value.clone(),
        )?;
        let ___left_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.left.value.clone(),
        )?;
        let ___left_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.right.value.clone(),
        )?;
        let ___left_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.left.value.clone(),
        )?;
        let ___left_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.right.value.clone(),
        )?;
        let ___left_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.left.value.clone(),
        )?;
        let ___left_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.right.value.clone(),
        )?;
        let ___right_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.left.left.value.clone(),
        )?;
        let ___right_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.left.right.value.clone(),
        )?;
        let ___right_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.right.left.value.clone(),
        )?;
        let ___right_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.right.right.value.clone(),
        )?;
        let ___right_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.left.left.value.clone(),
        )?;
        let ___right_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.left.right.value.clone(),
        )?;
        let ___right_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.right.left.value.clone(),
        )?;
        let ___right_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.right.right.value.clone(),
        )?;
        let ___right_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.left.left.value.clone(),
        )?;
        let ___right_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.left.right.value.clone(),
        )?;
        let ___right_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.right.left.value.clone(),
        )?;
        let ___right_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.right.right.value.clone(),
        )?;
        let ___right_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.left.left.value.clone(),
        )?;
        let ___right_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.left.right.value.clone(),
        )?;
        let ___right_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.right.left.value.clone(),
        )?;
        let ___right_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.right.right.value.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/ObjectBoundary32",
                "fromParts",
                "(JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ)Lio/prebindgen/covertest/model/ObjectBoundary32;",
                &[
                    jni::objects::JValue::from(___left_left_left_left_left_value),
                    jni::objects::JValue::from(___left_left_left_left_right_value),
                    jni::objects::JValue::from(___left_left_left_right_left_value),
                    jni::objects::JValue::from(___left_left_left_right_right_value),
                    jni::objects::JValue::from(___left_left_right_left_left_value),
                    jni::objects::JValue::from(___left_left_right_left_right_value),
                    jni::objects::JValue::from(___left_left_right_right_left_value),
                    jni::objects::JValue::from(___left_left_right_right_right_value),
                    jni::objects::JValue::from(___left_right_left_left_left_value),
                    jni::objects::JValue::from(___left_right_left_left_right_value),
                    jni::objects::JValue::from(___left_right_left_right_left_value),
                    jni::objects::JValue::from(___left_right_left_right_right_value),
                    jni::objects::JValue::from(___left_right_right_left_left_value),
                    jni::objects::JValue::from(___left_right_right_left_right_value),
                    jni::objects::JValue::from(___left_right_right_right_left_value),
                    jni::objects::JValue::from(___left_right_right_right_right_value),
                    jni::objects::JValue::from(___right_left_left_left_left_value),
                    jni::objects::JValue::from(___right_left_left_left_right_value),
                    jni::objects::JValue::from(___right_left_left_right_left_value),
                    jni::objects::JValue::from(___right_left_left_right_right_value),
                    jni::objects::JValue::from(___right_left_right_left_left_value),
                    jni::objects::JValue::from(___right_left_right_left_right_value),
                    jni::objects::JValue::from(___right_left_right_right_left_value),
                    jni::objects::JValue::from(___right_left_right_right_right_value),
                    jni::objects::JValue::from(___right_right_left_left_left_value),
                    jni::objects::JValue::from(___right_right_left_left_right_value),
                    jni::objects::JValue::from(___right_right_left_right_left_value),
                    jni::objects::JValue::from(___right_right_left_right_right_value),
                    jni::objects::JValue::from(___right_right_right_left_left_value),
                    jni::objects::JValue::from(___right_right_right_left_right_value),
                    jni::objects::JValue::from(___right_right_right_right_left_value),
                    jni::objects::JValue::from(___right_right_right_right_right_value),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn ObjectBoundary4_to_JObject_ea3fd497<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary4,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.value.clone(),
        )?;
        let ___left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.value.clone(),
        )?;
        let ___right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.value.clone(),
        )?;
        let ___right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.value.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/ObjectBoundary4",
                "fromParts",
                "(JJJJ)Lio/prebindgen/covertest/model/ObjectBoundary4;",
                &[
                    jni::objects::JValue::from(___left_left_value),
                    jni::objects::JValue::from(___left_right_value),
                    jni::objects::JValue::from(___right_left_value),
                    jni::objects::JValue::from(___right_right_value),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn ObjectBoundary63_to_JObject_29aa82ff<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary63,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___leaves32_left_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.left.left.left.left.value.clone(),
        )?;
        let ___leaves32_left_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.left.left.left.right.value.clone(),
        )?;
        let ___leaves32_left_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.left.left.right.left.value.clone(),
        )?;
        let ___leaves32_left_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.left.left.right.right.value.clone(),
        )?;
        let ___leaves32_left_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.left.right.left.left.value.clone(),
        )?;
        let ___leaves32_left_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.left.right.left.right.value.clone(),
        )?;
        let ___leaves32_left_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.left.right.right.left.value.clone(),
        )?;
        let ___leaves32_left_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.left.right.right.right.value.clone(),
        )?;
        let ___leaves32_left_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.right.left.left.left.value.clone(),
        )?;
        let ___leaves32_left_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.right.left.left.right.value.clone(),
        )?;
        let ___leaves32_left_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.right.left.right.left.value.clone(),
        )?;
        let ___leaves32_left_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.right.left.right.right.value.clone(),
        )?;
        let ___leaves32_left_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.right.right.left.left.value.clone(),
        )?;
        let ___leaves32_left_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.right.right.left.right.value.clone(),
        )?;
        let ___leaves32_left_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.right.right.right.left.value.clone(),
        )?;
        let ___leaves32_left_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.left.right.right.right.right.value.clone(),
        )?;
        let ___leaves32_right_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.left.left.left.left.value.clone(),
        )?;
        let ___leaves32_right_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.left.left.left.right.value.clone(),
        )?;
        let ___leaves32_right_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.left.left.right.left.value.clone(),
        )?;
        let ___leaves32_right_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.left.left.right.right.value.clone(),
        )?;
        let ___leaves32_right_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.left.right.left.left.value.clone(),
        )?;
        let ___leaves32_right_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.left.right.left.right.value.clone(),
        )?;
        let ___leaves32_right_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.left.right.right.left.value.clone(),
        )?;
        let ___leaves32_right_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.left.right.right.right.value.clone(),
        )?;
        let ___leaves32_right_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.right.left.left.left.value.clone(),
        )?;
        let ___leaves32_right_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.right.left.left.right.value.clone(),
        )?;
        let ___leaves32_right_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.right.left.right.left.value.clone(),
        )?;
        let ___leaves32_right_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.right.left.right.right.value.clone(),
        )?;
        let ___leaves32_right_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.right.right.left.left.value.clone(),
        )?;
        let ___leaves32_right_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.right.right.left.right.value.clone(),
        )?;
        let ___leaves32_right_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.right.right.right.left.value.clone(),
        )?;
        let ___leaves32_right_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves32.right.right.right.right.right.value.clone(),
        )?;
        let ___leaves16_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.left.left.left.left.value.clone(),
        )?;
        let ___leaves16_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.left.left.left.right.value.clone(),
        )?;
        let ___leaves16_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.left.left.right.left.value.clone(),
        )?;
        let ___leaves16_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.left.left.right.right.value.clone(),
        )?;
        let ___leaves16_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.left.right.left.left.value.clone(),
        )?;
        let ___leaves16_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.left.right.left.right.value.clone(),
        )?;
        let ___leaves16_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.left.right.right.left.value.clone(),
        )?;
        let ___leaves16_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.left.right.right.right.value.clone(),
        )?;
        let ___leaves16_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.right.left.left.left.value.clone(),
        )?;
        let ___leaves16_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.right.left.left.right.value.clone(),
        )?;
        let ___leaves16_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.right.left.right.left.value.clone(),
        )?;
        let ___leaves16_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.right.left.right.right.value.clone(),
        )?;
        let ___leaves16_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.right.right.left.left.value.clone(),
        )?;
        let ___leaves16_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.right.right.left.right.value.clone(),
        )?;
        let ___leaves16_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.right.right.right.left.value.clone(),
        )?;
        let ___leaves16_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves16.right.right.right.right.value.clone(),
        )?;
        let ___leaves8_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves8.left.left.left.value.clone(),
        )?;
        let ___leaves8_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves8.left.left.right.value.clone(),
        )?;
        let ___leaves8_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves8.left.right.left.value.clone(),
        )?;
        let ___leaves8_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves8.left.right.right.value.clone(),
        )?;
        let ___leaves8_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves8.right.left.left.value.clone(),
        )?;
        let ___leaves8_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves8.right.left.right.value.clone(),
        )?;
        let ___leaves8_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves8.right.right.left.value.clone(),
        )?;
        let ___leaves8_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves8.right.right.right.value.clone(),
        )?;
        let ___leaves4_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves4.left.left.value.clone(),
        )?;
        let ___leaves4_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves4.left.right.value.clone(),
        )?;
        let ___leaves4_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves4.right.left.value.clone(),
        )?;
        let ___leaves4_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves4.right.right.value.clone(),
        )?;
        let ___leaves2_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves2.left.value.clone(),
        )?;
        let ___leaves2_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaves2.right.value.clone(),
        )?;
        let ___leaf_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.leaf.value.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/ObjectBoundary63",
                "fromParts",
                "(JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ)Lio/prebindgen/covertest/model/ObjectBoundary63;",
                &[
                    jni::objects::JValue::from(
                        ___leaves32_left_left_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_left_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_left_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_left_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_left_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_left_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_left_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_right_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_right_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_right_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_right_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_left_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_left_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_left_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_left_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_left_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_left_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_left_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_left_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_right_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_right_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_right_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_right_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___leaves32_right_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(___leaves16_left_left_left_left_value),
                    jni::objects::JValue::from(___leaves16_left_left_left_right_value),
                    jni::objects::JValue::from(___leaves16_left_left_right_left_value),
                    jni::objects::JValue::from(___leaves16_left_left_right_right_value),
                    jni::objects::JValue::from(___leaves16_left_right_left_left_value),
                    jni::objects::JValue::from(___leaves16_left_right_left_right_value),
                    jni::objects::JValue::from(___leaves16_left_right_right_left_value),
                    jni::objects::JValue::from(___leaves16_left_right_right_right_value),
                    jni::objects::JValue::from(___leaves16_right_left_left_left_value),
                    jni::objects::JValue::from(___leaves16_right_left_left_right_value),
                    jni::objects::JValue::from(___leaves16_right_left_right_left_value),
                    jni::objects::JValue::from(___leaves16_right_left_right_right_value),
                    jni::objects::JValue::from(___leaves16_right_right_left_left_value),
                    jni::objects::JValue::from(___leaves16_right_right_left_right_value),
                    jni::objects::JValue::from(___leaves16_right_right_right_left_value),
                    jni::objects::JValue::from(
                        ___leaves16_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(___leaves8_left_left_left_value),
                    jni::objects::JValue::from(___leaves8_left_left_right_value),
                    jni::objects::JValue::from(___leaves8_left_right_left_value),
                    jni::objects::JValue::from(___leaves8_left_right_right_value),
                    jni::objects::JValue::from(___leaves8_right_left_left_value),
                    jni::objects::JValue::from(___leaves8_right_left_right_value),
                    jni::objects::JValue::from(___leaves8_right_right_left_value),
                    jni::objects::JValue::from(___leaves8_right_right_right_value),
                    jni::objects::JValue::from(___leaves4_left_left_value),
                    jni::objects::JValue::from(___leaves4_left_right_value),
                    jni::objects::JValue::from(___leaves4_right_left_value),
                    jni::objects::JValue::from(___leaves4_right_right_value),
                    jni::objects::JValue::from(___leaves2_left_value),
                    jni::objects::JValue::from(___leaves2_right_value),
                    jni::objects::JValue::from(___leaf_value),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn ObjectBoundary64_to_JObject_b2751ca5<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary64,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___left_left_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.left.left.value.clone(),
        )?;
        let ___left_left_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.left.right.value.clone(),
        )?;
        let ___left_left_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.right.left.value.clone(),
        )?;
        let ___left_left_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.right.right.value.clone(),
        )?;
        let ___left_left_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.left.left.value.clone(),
        )?;
        let ___left_left_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.left.right.value.clone(),
        )?;
        let ___left_left_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.right.left.value.clone(),
        )?;
        let ___left_left_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.right.right.value.clone(),
        )?;
        let ___left_left_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.left.left.value.clone(),
        )?;
        let ___left_left_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.left.right.value.clone(),
        )?;
        let ___left_left_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.right.left.value.clone(),
        )?;
        let ___left_left_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.right.right.value.clone(),
        )?;
        let ___left_left_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.left.left.value.clone(),
        )?;
        let ___left_left_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.left.right.value.clone(),
        )?;
        let ___left_left_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.right.left.value.clone(),
        )?;
        let ___left_left_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.right.right.value.clone(),
        )?;
        let ___left_right_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.left.left.value.clone(),
        )?;
        let ___left_right_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.left.right.value.clone(),
        )?;
        let ___left_right_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.right.left.value.clone(),
        )?;
        let ___left_right_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.right.right.value.clone(),
        )?;
        let ___left_right_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.left.left.value.clone(),
        )?;
        let ___left_right_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.left.right.value.clone(),
        )?;
        let ___left_right_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.right.left.value.clone(),
        )?;
        let ___left_right_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.right.right.value.clone(),
        )?;
        let ___left_right_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.left.left.value.clone(),
        )?;
        let ___left_right_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.left.right.value.clone(),
        )?;
        let ___left_right_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.right.left.value.clone(),
        )?;
        let ___left_right_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.right.right.value.clone(),
        )?;
        let ___left_right_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.left.left.value.clone(),
        )?;
        let ___left_right_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.left.right.value.clone(),
        )?;
        let ___left_right_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.right.left.value.clone(),
        )?;
        let ___left_right_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.right.right.value.clone(),
        )?;
        let ___right_left_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.left.left.left.value.clone(),
        )?;
        let ___right_left_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.left.left.right.value.clone(),
        )?;
        let ___right_left_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.left.right.left.value.clone(),
        )?;
        let ___right_left_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.left.right.right.value.clone(),
        )?;
        let ___right_left_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.right.left.left.value.clone(),
        )?;
        let ___right_left_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.right.left.right.value.clone(),
        )?;
        let ___right_left_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.right.right.left.value.clone(),
        )?;
        let ___right_left_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.right.right.right.value.clone(),
        )?;
        let ___right_left_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.left.left.left.value.clone(),
        )?;
        let ___right_left_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.left.left.right.value.clone(),
        )?;
        let ___right_left_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.left.right.left.value.clone(),
        )?;
        let ___right_left_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.left.right.right.value.clone(),
        )?;
        let ___right_left_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.right.left.left.value.clone(),
        )?;
        let ___right_left_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.right.left.right.value.clone(),
        )?;
        let ___right_left_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.right.right.left.value.clone(),
        )?;
        let ___right_left_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.right.right.right.value.clone(),
        )?;
        let ___right_right_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.left.left.left.value.clone(),
        )?;
        let ___right_right_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.left.left.right.value.clone(),
        )?;
        let ___right_right_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.left.right.left.value.clone(),
        )?;
        let ___right_right_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.left.right.right.value.clone(),
        )?;
        let ___right_right_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.right.left.left.value.clone(),
        )?;
        let ___right_right_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.right.left.right.value.clone(),
        )?;
        let ___right_right_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.right.right.left.value.clone(),
        )?;
        let ___right_right_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.right.right.right.value.clone(),
        )?;
        let ___right_right_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.left.left.left.value.clone(),
        )?;
        let ___right_right_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.left.left.right.value.clone(),
        )?;
        let ___right_right_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.left.right.left.value.clone(),
        )?;
        let ___right_right_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.left.right.right.value.clone(),
        )?;
        let ___right_right_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.right.left.left.value.clone(),
        )?;
        let ___right_right_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.right.left.right.value.clone(),
        )?;
        let ___right_right_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.right.right.left.value.clone(),
        )?;
        let ___right_right_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.right.right.right.value.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/ObjectBoundary64",
                "fromParts",
                "(JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ)Lio/prebindgen/covertest/model/ObjectBoundary64;",
                &[
                    jni::objects::JValue::from(___left_left_left_left_left_left_value),
                    jni::objects::JValue::from(___left_left_left_left_left_right_value),
                    jni::objects::JValue::from(___left_left_left_left_right_left_value),
                    jni::objects::JValue::from(___left_left_left_left_right_right_value),
                    jni::objects::JValue::from(___left_left_left_right_left_left_value),
                    jni::objects::JValue::from(___left_left_left_right_left_right_value),
                    jni::objects::JValue::from(___left_left_left_right_right_left_value),
                    jni::objects::JValue::from(
                        ___left_left_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(___left_left_right_left_left_left_value),
                    jni::objects::JValue::from(___left_left_right_left_left_right_value),
                    jni::objects::JValue::from(___left_left_right_left_right_left_value),
                    jni::objects::JValue::from(
                        ___left_left_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(___left_left_right_right_left_left_value),
                    jni::objects::JValue::from(
                        ___left_left_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(___left_right_left_left_left_left_value),
                    jni::objects::JValue::from(___left_right_left_left_left_right_value),
                    jni::objects::JValue::from(___left_right_left_left_right_left_value),
                    jni::objects::JValue::from(
                        ___left_right_left_left_right_right_value,
                    ),
                    jni::objects::JValue::from(___left_right_left_right_left_left_value),
                    jni::objects::JValue::from(
                        ___left_right_left_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(___left_right_right_left_left_left_value),
                    jni::objects::JValue::from(
                        ___left_right_right_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(___right_left_left_left_left_left_value),
                    jni::objects::JValue::from(___right_left_left_left_left_right_value),
                    jni::objects::JValue::from(___right_left_left_left_right_left_value),
                    jni::objects::JValue::from(
                        ___right_left_left_left_right_right_value,
                    ),
                    jni::objects::JValue::from(___right_left_left_right_left_left_value),
                    jni::objects::JValue::from(
                        ___right_left_left_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_left_left_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_left_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(___right_left_right_left_left_left_value),
                    jni::objects::JValue::from(
                        ___right_left_right_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_left_right_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_left_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_left_right_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_left_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_left_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_left_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(___right_right_left_left_left_left_value),
                    jni::objects::JValue::from(
                        ___right_right_left_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_left_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_left_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_left_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_left_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_left_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_right_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_right_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_right_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_right_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_right_right_right_right_right_value,
                    ),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn ObjectBoundary8_to_JObject_55b82b02<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary8,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.value.clone(),
        )?;
        let ___left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.value.clone(),
        )?;
        let ___left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.value.clone(),
        )?;
        let ___left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.value.clone(),
        )?;
        let ___right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.left.value.clone(),
        )?;
        let ___right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.left.right.value.clone(),
        )?;
        let ___right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.left.value.clone(),
        )?;
        let ___right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.right.right.value.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/ObjectBoundary8",
                "fromParts",
                "(JJJJJJJJ)Lio/prebindgen/covertest/model/ObjectBoundary8;",
                &[
                    jni::objects::JValue::from(___left_left_left_value),
                    jni::objects::JValue::from(___left_left_right_value),
                    jni::objects::JValue::from(___left_right_left_value),
                    jni::objects::JValue::from(___left_right_right_value),
                    jni::objects::JValue::from(___right_left_left_value),
                    jni::objects::JValue::from(___right_left_right_value),
                    jni::objects::JValue::from(___right_right_left_value),
                    jni::objects::JValue::from(___right_right_right_value),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn ObjectBoundaryLeaf_to_JObject_93531764<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundaryLeaf,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.value.clone())?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/ObjectBoundaryLeaf",
                "fromParts",
                "(J)Lio/prebindgen/covertest/model/ObjectBoundaryLeaf;",
                &[jni::objects::JValue::from(___value)],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn ObjectBoundary_to_JObject_dc5ac22b<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___left_left_left_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.left.left.left.value.clone(),
        )?;
        let ___left_left_left_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.left.left.right.value.clone(),
        )?;
        let ___left_left_left_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.left.right.left.value.clone(),
        )?;
        let ___left_left_left_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.left.right.right.value.clone(),
        )?;
        let ___left_left_left_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.right.left.left.value.clone(),
        )?;
        let ___left_left_left_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.right.left.right.value.clone(),
        )?;
        let ___left_left_left_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.right.right.left.value.clone(),
        )?;
        let ___left_left_left_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.left.right.right.right.value.clone(),
        )?;
        let ___left_left_left_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.left.left.left.value.clone(),
        )?;
        let ___left_left_left_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.left.left.right.value.clone(),
        )?;
        let ___left_left_left_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.left.right.left.value.clone(),
        )?;
        let ___left_left_left_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.left.right.right.value.clone(),
        )?;
        let ___left_left_left_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.right.left.left.value.clone(),
        )?;
        let ___left_left_left_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.right.left.right.value.clone(),
        )?;
        let ___left_left_left_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.right.right.left.value.clone(),
        )?;
        let ___left_left_left_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.left.right.right.right.right.value.clone(),
        )?;
        let ___left_left_right_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.left.left.left.value.clone(),
        )?;
        let ___left_left_right_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.left.left.right.value.clone(),
        )?;
        let ___left_left_right_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.left.right.left.value.clone(),
        )?;
        let ___left_left_right_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.left.right.right.value.clone(),
        )?;
        let ___left_left_right_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.right.left.left.value.clone(),
        )?;
        let ___left_left_right_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.right.left.right.value.clone(),
        )?;
        let ___left_left_right_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.right.right.left.value.clone(),
        )?;
        let ___left_left_right_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.left.right.right.right.value.clone(),
        )?;
        let ___left_left_right_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.left.left.left.value.clone(),
        )?;
        let ___left_left_right_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.left.left.right.value.clone(),
        )?;
        let ___left_left_right_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.left.right.left.value.clone(),
        )?;
        let ___left_left_right_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.left.right.right.value.clone(),
        )?;
        let ___left_left_right_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.right.left.left.value.clone(),
        )?;
        let ___left_left_right_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.right.left.right.value.clone(),
        )?;
        let ___left_left_right_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.right.right.left.value.clone(),
        )?;
        let ___left_left_right_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.left.right.right.right.right.right.value.clone(),
        )?;
        let ___left_right_left_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.left.left.left.value.clone(),
        )?;
        let ___left_right_left_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.left.left.right.value.clone(),
        )?;
        let ___left_right_left_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.left.right.left.value.clone(),
        )?;
        let ___left_right_left_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.left.right.right.value.clone(),
        )?;
        let ___left_right_left_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.right.left.left.value.clone(),
        )?;
        let ___left_right_left_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.right.left.right.value.clone(),
        )?;
        let ___left_right_left_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.right.right.left.value.clone(),
        )?;
        let ___left_right_left_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.left.right.right.right.value.clone(),
        )?;
        let ___left_right_left_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.left.left.left.value.clone(),
        )?;
        let ___left_right_left_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.left.left.right.value.clone(),
        )?;
        let ___left_right_left_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.left.right.left.value.clone(),
        )?;
        let ___left_right_left_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.left.right.right.value.clone(),
        )?;
        let ___left_right_left_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.right.left.left.value.clone(),
        )?;
        let ___left_right_left_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.right.left.right.value.clone(),
        )?;
        let ___left_right_left_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.right.right.left.value.clone(),
        )?;
        let ___left_right_left_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.left.right.right.right.right.value.clone(),
        )?;
        let ___left_right_right_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.left.left.left.value.clone(),
        )?;
        let ___left_right_right_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.left.left.right.value.clone(),
        )?;
        let ___left_right_right_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.left.right.left.value.clone(),
        )?;
        let ___left_right_right_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.left.right.right.value.clone(),
        )?;
        let ___left_right_right_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.right.left.left.value.clone(),
        )?;
        let ___left_right_right_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.right.left.right.value.clone(),
        )?;
        let ___left_right_right_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.right.right.left.value.clone(),
        )?;
        let ___left_right_right_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.left.right.right.right.value.clone(),
        )?;
        let ___left_right_right_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.left.left.left.value.clone(),
        )?;
        let ___left_right_right_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.left.left.right.value.clone(),
        )?;
        let ___left_right_right_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.left.right.left.value.clone(),
        )?;
        let ___left_right_right_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.left.right.right.value.clone(),
        )?;
        let ___left_right_right_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.right.left.left.value.clone(),
        )?;
        let ___left_right_right_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.right.left.right.value.clone(),
        )?;
        let ___left_right_right_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.right.right.left.value.clone(),
        )?;
        let ___left_right_right_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.left.right.right.right.right.right.right.value.clone(),
        )?;
        let ___right_leaves32_left_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.left.left.left.left.value.clone(),
        )?;
        let ___right_leaves32_left_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.left.left.left.right.value.clone(),
        )?;
        let ___right_leaves32_left_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.left.left.right.left.value.clone(),
        )?;
        let ___right_leaves32_left_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.left.left.right.right.value.clone(),
        )?;
        let ___right_leaves32_left_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.left.right.left.left.value.clone(),
        )?;
        let ___right_leaves32_left_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.left.right.left.right.value.clone(),
        )?;
        let ___right_leaves32_left_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.left.right.right.left.value.clone(),
        )?;
        let ___right_leaves32_left_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.left.right.right.right.value.clone(),
        )?;
        let ___right_leaves32_left_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.right.left.left.left.value.clone(),
        )?;
        let ___right_leaves32_left_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.right.left.left.right.value.clone(),
        )?;
        let ___right_leaves32_left_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.right.left.right.left.value.clone(),
        )?;
        let ___right_leaves32_left_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.right.left.right.right.value.clone(),
        )?;
        let ___right_leaves32_left_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.right.right.left.left.value.clone(),
        )?;
        let ___right_leaves32_left_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.right.right.left.right.value.clone(),
        )?;
        let ___right_leaves32_left_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.right.right.right.left.value.clone(),
        )?;
        let ___right_leaves32_left_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.left.right.right.right.right.value.clone(),
        )?;
        let ___right_leaves32_right_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.left.left.left.left.value.clone(),
        )?;
        let ___right_leaves32_right_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.left.left.left.right.value.clone(),
        )?;
        let ___right_leaves32_right_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.left.left.right.left.value.clone(),
        )?;
        let ___right_leaves32_right_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.left.left.right.right.value.clone(),
        )?;
        let ___right_leaves32_right_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.left.right.left.left.value.clone(),
        )?;
        let ___right_leaves32_right_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.left.right.left.right.value.clone(),
        )?;
        let ___right_leaves32_right_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.left.right.right.left.value.clone(),
        )?;
        let ___right_leaves32_right_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.left.right.right.right.value.clone(),
        )?;
        let ___right_leaves32_right_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.right.left.left.left.value.clone(),
        )?;
        let ___right_leaves32_right_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.right.left.left.right.value.clone(),
        )?;
        let ___right_leaves32_right_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.right.left.right.left.value.clone(),
        )?;
        let ___right_leaves32_right_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.right.left.right.right.value.clone(),
        )?;
        let ___right_leaves32_right_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.right.right.left.left.value.clone(),
        )?;
        let ___right_leaves32_right_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.right.right.left.right.value.clone(),
        )?;
        let ___right_leaves32_right_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.right.right.right.left.value.clone(),
        )?;
        let ___right_leaves32_right_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves32.right.right.right.right.right.value.clone(),
        )?;
        let ___right_leaves16_left_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.left.left.left.left.value.clone(),
        )?;
        let ___right_leaves16_left_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.left.left.left.right.value.clone(),
        )?;
        let ___right_leaves16_left_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.left.left.right.left.value.clone(),
        )?;
        let ___right_leaves16_left_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.left.left.right.right.value.clone(),
        )?;
        let ___right_leaves16_left_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.left.right.left.left.value.clone(),
        )?;
        let ___right_leaves16_left_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.left.right.left.right.value.clone(),
        )?;
        let ___right_leaves16_left_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.left.right.right.left.value.clone(),
        )?;
        let ___right_leaves16_left_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.left.right.right.right.value.clone(),
        )?;
        let ___right_leaves16_right_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.right.left.left.left.value.clone(),
        )?;
        let ___right_leaves16_right_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.right.left.left.right.value.clone(),
        )?;
        let ___right_leaves16_right_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.right.left.right.left.value.clone(),
        )?;
        let ___right_leaves16_right_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.right.left.right.right.value.clone(),
        )?;
        let ___right_leaves16_right_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.right.right.left.left.value.clone(),
        )?;
        let ___right_leaves16_right_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.right.right.left.right.value.clone(),
        )?;
        let ___right_leaves16_right_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.right.right.right.left.value.clone(),
        )?;
        let ___right_leaves16_right_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves16.right.right.right.right.value.clone(),
        )?;
        let ___right_leaves8_left_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves8.left.left.left.value.clone(),
        )?;
        let ___right_leaves8_left_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves8.left.left.right.value.clone(),
        )?;
        let ___right_leaves8_left_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves8.left.right.left.value.clone(),
        )?;
        let ___right_leaves8_left_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves8.left.right.right.value.clone(),
        )?;
        let ___right_leaves8_right_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves8.right.left.left.value.clone(),
        )?;
        let ___right_leaves8_right_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves8.right.left.right.value.clone(),
        )?;
        let ___right_leaves8_right_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves8.right.right.left.value.clone(),
        )?;
        let ___right_leaves8_right_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves8.right.right.right.value.clone(),
        )?;
        let ___right_leaves4_left_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves4.left.left.value.clone(),
        )?;
        let ___right_leaves4_left_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves4.left.right.value.clone(),
        )?;
        let ___right_leaves4_right_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves4.right.left.value.clone(),
        )?;
        let ___right_leaves4_right_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves4.right.right.value.clone(),
        )?;
        let ___right_leaves2_left_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves2.left.value.clone(),
        )?;
        let ___right_leaves2_right_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaves2.right.value.clone(),
        )?;
        let ___right_leaf_value: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.right.leaf.value.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/ObjectBoundary",
                "fromParts",
                "(JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ)Lio/prebindgen/covertest/model/ObjectBoundary;",
                &[
                    jni::objects::JValue::from(
                        ___left_left_left_left_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_left_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_left_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_left_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_left_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_left_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_left_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_right_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_right_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_right_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_right_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_left_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_left_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_left_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_left_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_left_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_left_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_left_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_left_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_right_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_right_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_right_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_right_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_left_right_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_left_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_left_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_left_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_left_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_left_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_left_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_left_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_right_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_right_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_right_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_right_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_left_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_left_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_left_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_left_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_left_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_left_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_left_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_left_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_right_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_right_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_right_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_right_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___left_right_right_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_left_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_left_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_left_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_left_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_left_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_left_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_left_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_right_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_right_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_right_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_right_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_left_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_left_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_left_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_left_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_left_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_left_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_left_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_left_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_right_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_right_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_right_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_right_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves32_right_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_left_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_left_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_left_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_left_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_left_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_left_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_left_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_left_right_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_right_left_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_right_left_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_right_left_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_right_left_right_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_right_right_left_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_right_right_left_right_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_right_right_right_left_value,
                    ),
                    jni::objects::JValue::from(
                        ___right_leaves16_right_right_right_right_value,
                    ),
                    jni::objects::JValue::from(___right_leaves8_left_left_left_value),
                    jni::objects::JValue::from(___right_leaves8_left_left_right_value),
                    jni::objects::JValue::from(___right_leaves8_left_right_left_value),
                    jni::objects::JValue::from(___right_leaves8_left_right_right_value),
                    jni::objects::JValue::from(___right_leaves8_right_left_left_value),
                    jni::objects::JValue::from(___right_leaves8_right_left_right_value),
                    jni::objects::JValue::from(___right_leaves8_right_right_left_value),
                    jni::objects::JValue::from(___right_leaves8_right_right_right_value),
                    jni::objects::JValue::from(___right_leaves4_left_left_value),
                    jni::objects::JValue::from(___right_leaves4_left_right_value),
                    jni::objects::JValue::from(___right_leaves4_right_left_value),
                    jni::objects::JValue::from(___right_leaves4_right_right_value),
                    jni::objects::JValue::from(___right_leaves2_left_value),
                    jni::objects::JValue::from(___right_leaves2_right_value),
                    jni::objects::JValue::from(___right_leaf_value),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Observation_to_JObject_435b0724<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Observation,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___id: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.id.clone())?;
        let ___reading__tag: jni::sys::jint;
        let ___reading_g0: jni::sys::jlong;
        let ___reading_g1: jni::sys::jlong;
        let ___reading_g2: jni::sys::jlong;
        let ___reading_g3: jni::objects::JObject;
        let ___reading_g4: jni::sys::jint;
        let ___reading_g5: jni::sys::jlong;
        match &v.reading {
            perftest_flat::Reading::Missing => {
                ___reading__tag = 0;
                ___reading_g0 = 0i64;
                ___reading_g1 = 0i64;
                ___reading_g2 = 0i64;
                ___reading_g3 = jni::objects::JObject::null();
                ___reading_g4 = 0i32;
                ___reading_g5 = 0i64;
            }
            perftest_flat::Reading::Exact(__s0_0) => {
                let ___reading_exact_v0: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
                    env,
                    __s0_0.clone(),
                )?;
                ___reading__tag = 1;
                ___reading_g0 = ___reading_exact_v0;
                ___reading_g1 = 0i64;
                ___reading_g2 = 0i64;
                ___reading_g3 = jni::objects::JObject::null();
                ___reading_g4 = 0i32;
                ___reading_g5 = 0i64;
            }
            perftest_flat::Reading::Range { low: __s0_0, high: __s0_1 } => {
                let ___reading_range_low: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
                    env,
                    __s0_0.clone(),
                )?;
                let ___reading_range_high: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
                    env,
                    __s0_1.clone(),
                )?;
                ___reading__tag = 2;
                ___reading_g1 = ___reading_range_low;
                ___reading_g2 = ___reading_range_high;
                ___reading_g0 = 0i64;
                ___reading_g3 = jni::objects::JObject::null();
                ___reading_g4 = 0i32;
                ___reading_g5 = 0i64;
            }
            perftest_flat::Reading::Labeled(__s0_0, __s0_1) => {
                let ___reading_tagged_v0: jni::objects::JObject = String_to_JString_c7f3ca43(
                        env,
                        __s0_0.clone(),
                    )?
                    .into();
                let ___reading_tagged_v1: jni::sys::jint = Priority_to_jint_447102d2(
                    env,
                    __s0_1.clone(),
                )?;
                ___reading__tag = 3;
                ___reading_g3 = ___reading_tagged_v0;
                ___reading_g4 = ___reading_tagged_v1;
                ___reading_g0 = 0i64;
                ___reading_g1 = 0i64;
                ___reading_g2 = 0i64;
                ___reading_g5 = 0i64;
            }
            perftest_flat::Reading::Companion(__s0_0) => {
                let ___reading_companion_v0: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
                    env,
                    __s0_0.clone(),
                )?;
                ___reading__tag = 4;
                ___reading_g5 = ___reading_companion_v0;
                ___reading_g0 = 0i64;
                ___reading_g1 = 0i64;
                ___reading_g2 = 0i64;
                ___reading_g3 = jni::objects::JObject::null();
                ___reading_g4 = 0i32;
            }
        }
        let ___fallback_present: jni::sys::jboolean;
        let ___fallback__tag: jni::sys::jint;
        let ___fallback_g0: jni::sys::jlong;
        let ___fallback_g1: jni::sys::jlong;
        let ___fallback_g2: jni::sys::jlong;
        let ___fallback_g3: jni::objects::JObject;
        let ___fallback_g4: jni::sys::jint;
        let ___fallback_g5: jni::sys::jlong;
        let __oc0: &::core::option::Option<_> = &v.fallback;
        match __oc0 {
            ::core::option::Option::Some(__o0) => {
                ___fallback_present = 1u8;
                match __o0 {
                    perftest_flat::Reading::Missing => {
                        ___fallback__tag = 0;
                        ___fallback_g0 = 0i64;
                        ___fallback_g1 = 0i64;
                        ___fallback_g2 = 0i64;
                        ___fallback_g3 = jni::objects::JObject::null();
                        ___fallback_g4 = 0i32;
                        ___fallback_g5 = 0i64;
                    }
                    perftest_flat::Reading::Exact(__s0_0) => {
                        let ___fallback_exact_v0: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
                            env,
                            __s0_0.clone(),
                        )?;
                        ___fallback__tag = 1;
                        ___fallback_g0 = ___fallback_exact_v0;
                        ___fallback_g1 = 0i64;
                        ___fallback_g2 = 0i64;
                        ___fallback_g3 = jni::objects::JObject::null();
                        ___fallback_g4 = 0i32;
                        ___fallback_g5 = 0i64;
                    }
                    perftest_flat::Reading::Range { low: __s0_0, high: __s0_1 } => {
                        let ___fallback_range_low: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
                            env,
                            __s0_0.clone(),
                        )?;
                        let ___fallback_range_high: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
                            env,
                            __s0_1.clone(),
                        )?;
                        ___fallback__tag = 2;
                        ___fallback_g1 = ___fallback_range_low;
                        ___fallback_g2 = ___fallback_range_high;
                        ___fallback_g0 = 0i64;
                        ___fallback_g3 = jni::objects::JObject::null();
                        ___fallback_g4 = 0i32;
                        ___fallback_g5 = 0i64;
                    }
                    perftest_flat::Reading::Labeled(__s0_0, __s0_1) => {
                        let ___fallback_tagged_v0: jni::objects::JObject = String_to_JString_c7f3ca43(
                                env,
                                __s0_0.clone(),
                            )?
                            .into();
                        let ___fallback_tagged_v1: jni::sys::jint = Priority_to_jint_447102d2(
                            env,
                            __s0_1.clone(),
                        )?;
                        ___fallback__tag = 3;
                        ___fallback_g3 = ___fallback_tagged_v0;
                        ___fallback_g4 = ___fallback_tagged_v1;
                        ___fallback_g0 = 0i64;
                        ___fallback_g1 = 0i64;
                        ___fallback_g2 = 0i64;
                        ___fallback_g5 = 0i64;
                    }
                    perftest_flat::Reading::Companion(__s0_0) => {
                        let ___fallback_companion_v0: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
                            env,
                            __s0_0.clone(),
                        )?;
                        ___fallback__tag = 4;
                        ___fallback_g5 = ___fallback_companion_v0;
                        ___fallback_g0 = 0i64;
                        ___fallback_g1 = 0i64;
                        ___fallback_g2 = 0i64;
                        ___fallback_g3 = jni::objects::JObject::null();
                        ___fallback_g4 = 0i32;
                    }
                }
            }
            ::core::option::Option::None => {
                ___fallback_present = 0u8;
                ___fallback__tag = 0i32;
                ___fallback_g0 = 0i64;
                ___fallback_g1 = 0i64;
                ___fallback_g2 = 0i64;
                ___fallback_g3 = jni::objects::JObject::null();
                ___fallback_g4 = 0i32;
                ___fallback_g5 = 0i64;
            }
        }
        let ___note: jni::objects::JObject = String_to_JString_c7f3ca43(
                env,
                v.note.clone(),
            )?
            .into();
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/Observation",
                "fromParts",
                "(JIJJJLjava/lang/String;IJZIJJJLjava/lang/String;IJLjava/lang/String;)Lio/prebindgen/covertest/model/Observation;",
                &[
                    jni::objects::JValue::from(___id),
                    jni::objects::JValue::from(___reading__tag),
                    jni::objects::JValue::from(___reading_g0),
                    jni::objects::JValue::from(___reading_g1),
                    jni::objects::JValue::from(___reading_g2),
                    jni::objects::JValue::Object(&___reading_g3),
                    jni::objects::JValue::from(___reading_g4),
                    jni::objects::JValue::from(___reading_g5),
                    jni::objects::JValue::from(___fallback_present),
                    jni::objects::JValue::from(___fallback__tag),
                    jni::objects::JValue::from(___fallback_g0),
                    jni::objects::JValue::from(___fallback_g1),
                    jni::objects::JValue::from(___fallback_g2),
                    jni::objects::JValue::Object(&___fallback_g3),
                    jni::objects::JValue::from(___fallback_g4),
                    jni::objects::JValue::from(___fallback_g5),
                    jni::objects::JValue::Object(&___note),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_Box_String_to_JString_071e4c8c<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<Box<String>>,
) -> ::core::result::Result<jni::objects::JString<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                Box_String_to_JString_027f6250(env, __value)?
            }
            ::core::option::Option::None => jni::objects::JObject::null().into(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_CallbackHolder_to_tuple2_6762df4a<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<perftest_flat::CallbackHolder>,
) -> ::core::result::Result<
    (jni::sys::jboolean, (jni::sys::jlong, jni::sys::jlong)),
    __JniErr,
> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                (1u8, CallbackHolder_to_tuple2_14aebb91(env, __value)?)
            }
            ::core::option::Option::None => {
                (0u8, (0 as jni::sys::jlong, 0 as jni::sys::jlong))
            }
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_Duration_to_jlong_1cfa4d44<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<perftest_flat::Duration>,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                {
                    let __chain_s0 = Duration_to_u64_e3980876(env, __value)
                        .map_err(|__e| <__JniErr as ::core::convert::From<
                            String,
                        >>::from(__e.to_string()))?;
                    u64_to_jlong_4384a5d6(env, __chain_s0)
                }?
            }
            ::core::option::Option::None => -1i64,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_Ingot_to_jlong_a76a8f2f<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<perftest_flat::Ingot>,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                Ingot_to_jlong_020c3a86(env, __value)?
            }
            ::core::option::Option::None => 0i64,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_Payload_to_JObject_97036642<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<perftest_flat::Payload>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                Payload_to_JObject_98f64326(env, __value)?
            }
            ::core::option::Option::None => jni::objects::JObject::null().into(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_Payload_to_tuple2_af2bd54b<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<perftest_flat::Payload>,
) -> ::core::result::Result<
    (
        jni::sys::jboolean,
        (
            jni::sys::jlong,
            jni::sys::jint,
            jni::sys::jdouble,
            jni::sys::jboolean,
            jni::objects::JString<'a>,
        ),
    ),
    __JniErr,
> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                (1u8, Payload_to_tuple5_bbb055bc(env, __value)?)
            }
            ::core::option::Option::None => {
                (
                    0u8,
                    (
                        0 as jni::sys::jlong,
                        0 as jni::sys::jint,
                        0.0 as jni::sys::jdouble,
                        0 as jni::sys::jboolean,
                        jni::objects::JObject::null().into(),
                    ),
                )
            }
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_Percent_to_JObject_544dd364<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<perftest_flat::Percent>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                let __raw: jni::sys::jint = {
                    let __chain_s0 = Percent_to_i32_01484801(env, __value)
                        .map_err(|__e| <__JniErr as ::core::convert::From<
                            String,
                        >>::from(__e.to_string()))?;
                    i32_to_jint_a3e3b6ef(env, __chain_s0)
                }?;
                ::prebindgen_jni_runtime::box_jint(env, __raw)
                    .map_err(|__error| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option box: {}", __error)))?
            }
            ::core::option::Option::None => jni::objects::JObject::null(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_Priority_to_JObject_ad5cbb32<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<perftest_flat::Priority>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                let __raw: jni::sys::jint = Priority_to_jint_447102d2(env, __value)?;
                ::prebindgen_jni_runtime::box_jint(env, __raw)
                    .map_err(|__error| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option box: {}", __error)))?
            }
            ::core::option::Option::None => jni::objects::JObject::null(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_Stamp_to_JObject_6375b503<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<perftest_flat::Stamp>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                Stamp_to_JObject_f6b1e942(env, __value)?
            }
            ::core::option::Option::None => jni::objects::JObject::null().into(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_String_to_JString_56d5e304<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<String>,
) -> ::core::result::Result<jni::objects::JString<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                String_to_JString_c7f3ca43(env, __value)?
            }
            ::core::option::Option::None => jni::objects::JObject::null().into(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_Summary_to_jlong_252ef2ba<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<perftest_flat::Summary>,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                Summary_to_jlong_3cb103b9(env, __value)?
            }
            ::core::option::Option::None => 0i64,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_Summary_to_jlong_828826f3<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<&perftest_flat::Summary>,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                Summary_to_jlong_ccacdeac(env, __value)?
            }
            ::core::option::Option::None => 0i64,
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_Ticks_to_JObject_95efad57<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<perftest_flat::Ticks>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                let __raw: jni::sys::jlong = {
                    let __chain_s0 = Ticks_to_u64_78fe2120(env, __value)
                        .map_err(|__e| <__JniErr as ::core::convert::From<
                            String,
                        >>::from(__e.to_string()))?;
                    u64_to_jlong_4384a5d6(env, __chain_s0)
                }?;
                ::prebindgen_jni_runtime::box_jlong(env, __raw)
                    .map_err(|__error| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option box: {}", __error)))?
            }
            ::core::option::Option::None => jni::objects::JObject::null(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_Vec_Option_u64_to_JObject_006312b6<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<Vec<Option<u64>>>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                Vec_Option_u64_to_JObject_a34190e7(env, __value)?
            }
            ::core::option::Option::None => jni::objects::JObject::null().into(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_f64_to_JObject_b3f3e9a9<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<f64>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                let __raw: jni::sys::jdouble = f64_to_jdouble_9e4a8f70(env, __value)?;
                ::prebindgen_jni_runtime::box_jdouble(env, __raw)
                    .map_err(|__error| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option box: {}", __error)))?
            }
            ::core::option::Option::None => jni::objects::JObject::null(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_i64_to_JObject_2ba9a5ed<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<i64>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                let __raw: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, __value)?;
                ::prebindgen_jni_runtime::box_jlong(env, __raw)
                    .map_err(|__error| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option box: {}", __error)))?
            }
            ::core::option::Option::None => jni::objects::JObject::null(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Option_u64_to_JObject_32be16a2<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Option<u64>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                let __raw: jni::sys::jlong = u64_to_jlong_4384a5d6(env, __value)?;
                ::prebindgen_jni_runtime::box_jlong(env, __raw)
                    .map_err(|__error| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Option box: {}", __error)))?
            }
            ::core::option::Option::None => jni::objects::JObject::null(),
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn PayloadHandler_to_jlong_d61fd890<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::PayloadHandler,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn PayloadVecHandler_to_jlong_b32d2812<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::PayloadVecHandler,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
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
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
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
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn Payload_to_tuple5_2ea1d0c2<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: &perftest_flat::Payload,
) -> ::core::result::Result<
    (
        jni::sys::jlong,
        jni::sys::jint,
        jni::sys::jdouble,
        jni::sys::jboolean,
        jni::objects::JString<'a>,
    ),
    __JniErr,
> {
    ::core::result::Result::Ok((
        i64_to_jlong_fbf9a9bc(env, (*&(v.id)).clone())?,
        i32_to_jint_a3e3b6ef(env, (*&(v.seq)).clone())?,
        f64_to_jdouble_9e4a8f70(env, (*&(v.value)).clone())?,
        bool_to_jboolean_31306d98(env, (*&(v.flag)).clone())?,
        Option_Box_String_to_JString_071e4c8c(env, (*&(v.label)).clone())?,
    ))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn Payload_to_tuple5_bbb055bc<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Payload,
) -> ::core::result::Result<
    (
        jni::sys::jlong,
        jni::sys::jint,
        jni::sys::jdouble,
        jni::sys::jboolean,
        jni::objects::JString<'a>,
    ),
    __JniErr,
> {
    ::core::result::Result::Ok((
        i64_to_jlong_fbf9a9bc(env, v.id)?,
        i32_to_jint_a3e3b6ef(env, v.seq)?,
        f64_to_jdouble_9e4a8f70(env, v.value)?,
        bool_to_jboolean_31306d98(env, v.flag)?,
        Option_Box_String_to_JString_071e4c8c(env, v.label)?,
    ))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Percent_to_i32_01484801<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Percent,
) -> ::core::result::Result<i32, String> {
    crate::percent_out(v)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Priority_to_jint_447102d2<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Priority,
) -> ::core::result::Result<jni::sys::jint, __JniErr> {
    Ok({ v as jni::sys::jint })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Probe_to_jlong_76f3d10e<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Probe,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn Reading_to_tuple6_69702d1f<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Reading,
) -> ::core::result::Result<
    (
        jni::sys::jint,
        (),
        (jni::sys::jlong,),
        (jni::sys::jlong, jni::sys::jlong),
        (jni::objects::JString<'a>, jni::sys::jint),
        (jni::sys::jlong,),
    ),
    __JniErr,
> {
    ::core::result::Result::Ok({
        match v {
            perftest_flat::Reading::Missing => {
                (
                    0i32,
                    (),
                    (0 as jni::sys::jlong,),
                    (0 as jni::sys::jlong, 0 as jni::sys::jlong),
                    (jni::objects::JObject::null().into(), 0 as jni::sys::jint),
                    (0 as jni::sys::jlong,),
                )
            }
            perftest_flat::Reading::Exact(__part0) => {
                (
                    1i32,
                    (),
                    (i64_to_jlong_fbf9a9bc(env, __part0)?,),
                    (0 as jni::sys::jlong, 0 as jni::sys::jlong),
                    (jni::objects::JObject::null().into(), 0 as jni::sys::jint),
                    (0 as jni::sys::jlong,),
                )
            }
            perftest_flat::Reading::Range { low: __part0, high: __part1 } => {
                (
                    2i32,
                    (),
                    (0 as jni::sys::jlong,),
                    (
                        i64_to_jlong_fbf9a9bc(env, __part0)?,
                        i64_to_jlong_fbf9a9bc(env, __part1)?,
                    ),
                    (jni::objects::JObject::null().into(), 0 as jni::sys::jint),
                    (0 as jni::sys::jlong,),
                )
            }
            perftest_flat::Reading::Labeled(__part0, __part1) => {
                (
                    3i32,
                    (),
                    (0 as jni::sys::jlong,),
                    (0 as jni::sys::jlong, 0 as jni::sys::jlong),
                    (
                        String_to_JString_c7f3ca43(env, __part0)?,
                        Priority_to_jint_447102d2(env, __part1)?,
                    ),
                    (0 as jni::sys::jlong,),
                )
            }
            perftest_flat::Reading::Companion(__part0) => {
                (
                    4i32,
                    (),
                    (0 as jni::sys::jlong,),
                    (0 as jni::sys::jlong, 0 as jni::sys::jlong),
                    (jni::objects::JObject::null().into(), 0 as jni::sys::jint),
                    (i64_to_jlong_fbf9a9bc(env, __part0)?,),
                )
            }
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn RepliesConfig_to_JObject_eb8e9079<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::RepliesConfig,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___priority: jni::sys::jint = Priority_to_jint_447102d2(
            env,
            v.priority.clone(),
        )?;
        let ___max_samples: jni::sys::jlong = i64_to_jlong_fbf9a9bc(
            env,
            v.max_samples.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/RepliesConfig",
                "fromParts",
                "(IJ)Lio/prebindgen/covertest/model/RepliesConfig;",
                &[
                    jni::objects::JValue::from(___priority),
                    jni::objects::JValue::from(___max_samples),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Report_to_jlong_eaed4ba1<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Report,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Result_Storage_StorageError_to_Storage_7ccce404<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Result<perftest_flat::Storage, perftest_flat::StorageError>,
) -> ::core::result::Result<perftest_flat::Storage, perftest_flat::StorageError> {
    v
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Result_Summary_String_to_Summary_dfdf7f9e<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Result<perftest_flat::Summary, String>,
) -> ::core::result::Result<perftest_flat::Summary, String> {
    v
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn SpanHolder_to_jlong_7ffe9314<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::SpanHolder,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Span_to_jlong_6d59d587<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Span,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Stamp_to_JObject_f6b1e942<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Stamp,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___secs: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.secs.clone())?;
        let ___nanos: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.nanos.clone())?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/Stamp",
                "fromParts",
                "(JJ)Lio/prebindgen/covertest/model/Stamp;",
                &[
                    jni::objects::JValue::from(___secs),
                    jni::objects::JValue::from(___nanos),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn Stamp_to_tuple2_8d33d015<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Stamp,
) -> ::core::result::Result<(jni::sys::jlong, jni::sys::jlong), __JniErr> {
    ::core::result::Result::Ok((
        i64_to_jlong_fbf9a9bc(env, v.secs)?,
        i64_to_jlong_fbf9a9bc(env, v.nanos)?,
    ))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn StorageError_to_jlong_26b2d298<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::StorageError,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn StorageHandler_to_jlong_3b4d3ed3<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::StorageHandler,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Storage_to_jlong_1b233abd<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Storage,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn String_to_JString_c7f3ca43<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: String,
) -> ::core::result::Result<jni::objects::JString<'a>, __JniErr> {
    Ok({
        env.new_string(&*v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("encode_str: {}", e))
            })?
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn String_to_Label_c1a79668<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: String,
) -> ::core::result::Result<perftest_flat::Label, String> {
    crate::label_in(v)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Summary_to_jlong_3cb103b9<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Summary,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Summary_to_jlong_ccacdeac<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: &perftest_flat::Summary,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v.clone())) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Tagged_to_JObject_641b984c<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Tagged,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___id: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.id.clone())?;
        let ___marker__tag: jni::sys::jint;
        let ___marker_g0: jni::objects::JObject;
        match &v.marker {
            perftest_flat::Marker::None_ => {
                ___marker__tag = 0;
                ___marker_g0 = jni::objects::JObject::null();
            }
            perftest_flat::Marker::Ranked(__s0_0) => {
                let ___marker_ranked_v0: jni::objects::JObject = Option_Priority_to_JObject_ad5cbb32(
                    env,
                    __s0_0.clone(),
                )?;
                ___marker__tag = 1;
                ___marker_g0 = ___marker_ranked_v0;
            }
        }
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/Tagged",
                "fromParts",
                "(JILjava/lang/Integer;)Lio/prebindgen/covertest/model/Tagged;",
                &[
                    jni::objects::JValue::from(___id),
                    jni::objects::JValue::from(___marker__tag),
                    jni::objects::JValue::Object(&___marker_g0),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Ticks_to_u64_78fe2120<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Ticks,
) -> ::core::result::Result<u64, __JniErr> {
    Ok(perftest_flat::ticks_value(&v))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Unsigned_to_JObject_7e3cc618<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Unsigned,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___byte: jni::sys::jint = u8_to_jint_553cf6ec(env, v.byte.clone())?;
        let ___short: jni::sys::jint = u16_to_jint_28edf527(env, v.short.clone())?;
        let ___int: jni::sys::jlong = u32_to_jlong_9594a230(env, v.int.clone())?;
        let ___long: jni::sys::jlong = u64_to_jlong_4384a5d6(env, v.long.clone())?;
        let ___maybe_long: jni::objects::JObject = Option_u64_to_JObject_32be16a2(
            env,
            v.maybe_long.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/Unsigned",
                "fromParts",
                "(IIJJLjava/lang/Long;)Lio/prebindgen/covertest/model/Unsigned;",
                &[
                    jni::objects::JValue::from(___byte),
                    jni::objects::JValue::from(___short),
                    jni::objects::JValue::from(___int),
                    jni::objects::JValue::from(___long),
                    jni::objects::JValue::Object(&___maybe_long),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn Unsigned_to_tuple5_371b0950<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Unsigned,
) -> ::core::result::Result<
    (
        jni::sys::jint,
        jni::sys::jint,
        jni::sys::jlong,
        jni::sys::jlong,
        jni::objects::JObject<'a>,
    ),
    __JniErr,
> {
    ::core::result::Result::Ok((
        u8_to_jint_553cf6ec(env, v.byte)?,
        u16_to_jint_28edf527(env, v.short)?,
        u32_to_jlong_9594a230(env, v.int)?,
        u64_to_jlong_4384a5d6(env, v.long)?,
        Option_u64_to_JObject_32be16a2(env, v.maybe_long)?,
    ))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn VaultHolder_to_jlong_1de3f656<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::VaultHolder,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Vault_to_jlong_4a33ea23<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Vault,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(std::boxed::Box::into_raw(std::boxed::Box::new(v)) as i64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Vec_Label_to_JObject_3fdf860d<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Vec<perftest_flat::Label>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        let __sequence_source = v;
        let __sequence_output = env
            .new_object("java/util/ArrayList", "()V", &[])
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: new ArrayList: {}", e)))?;
        let __sequence_list = jni::objects::JList::from_env(env, &__sequence_output)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-from-env: {}", e)))?;
        for __sequence_element in __sequence_source.into_iter() {
            let __sequence_part = {
                let __chain_s0 = Label_to_String_63dec766(env, __sequence_element)
                    .map_err(|__e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(__e.to_string()))?;
                String_to_JString_c7f3ca43(env, __chain_s0)
            }?;
            let __sequence_object: jni::objects::JObject = __sequence_part.into();
            __sequence_list
                .add(env, &__sequence_object)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Vec<_>: list-add: {}", e)))?;
        }
        __sequence_output
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Vec_Option_Ticks_to_JObject_2f4b03da<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Vec<Option<perftest_flat::Ticks>>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        let __sequence_source = v;
        let __sequence_output = env
            .new_object("java/util/ArrayList", "()V", &[])
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: new ArrayList: {}", e)))?;
        let __sequence_list = jni::objects::JList::from_env(env, &__sequence_output)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-from-env: {}", e)))?;
        for __sequence_element in __sequence_source.into_iter() {
            let __sequence_part = Option_Ticks_to_JObject_95efad57(
                env,
                __sequence_element,
            )?;
            let __sequence_object: jni::objects::JObject = __sequence_part.into();
            __sequence_list
                .add(env, &__sequence_object)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Vec<_>: list-add: {}", e)))?;
        }
        __sequence_output
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Vec_Option_u64_to_JObject_a34190e7<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Vec<Option<u64>>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        let __sequence_source = v;
        let __sequence_output = env
            .new_object("java/util/ArrayList", "()V", &[])
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: new ArrayList: {}", e)))?;
        let __sequence_list = jni::objects::JList::from_env(env, &__sequence_output)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-from-env: {}", e)))?;
        for __sequence_element in __sequence_source.into_iter() {
            let __sequence_part = Option_u64_to_JObject_32be16a2(
                env,
                __sequence_element,
            )?;
            let __sequence_object: jni::objects::JObject = __sequence_part.into();
            __sequence_list
                .add(env, &__sequence_object)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Vec<_>: list-add: {}", e)))?;
        }
        __sequence_output
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Vec_Vec_Option_u64_to_JObject_342a76c6<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Vec<Vec<Option<u64>>>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        let __sequence_source = v;
        let __sequence_output = env
            .new_object("java/util/ArrayList", "()V", &[])
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: new ArrayList: {}", e)))?;
        let __sequence_list = jni::objects::JList::from_env(env, &__sequence_output)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-from-env: {}", e)))?;
        for __sequence_element in __sequence_source.into_iter() {
            let __sequence_part = Vec_Option_u64_to_JObject_a34190e7(
                env,
                __sequence_element,
            )?;
            let __sequence_object: jni::objects::JObject = __sequence_part.into();
            __sequence_list
                .add(env, &__sequence_object)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Vec<_>: list-add: {}", e)))?;
        }
        __sequence_output
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Vec_Vec_u8_to_JObject_43404875<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Vec<Vec<u8>>,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    ::core::result::Result::Ok({
        let __sequence_source = v;
        let __sequence_output = env
            .new_object("java/util/ArrayList", "()V", &[])
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: new ArrayList: {}", e)))?;
        let __sequence_list = jni::objects::JList::from_env(env, &__sequence_output)
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Vec<_>: list-from-env: {}", e)))?;
        for __sequence_element in __sequence_source.into_iter() {
            let __sequence_part = Vec_u8_to_JByteArray_7936d5de(
                env,
                __sequence_element,
            )?;
            let __sequence_object: jni::objects::JObject = __sequence_part.into();
            __sequence_list
                .add(env, &__sequence_object)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("Vec<_>: list-add: {}", e)))?;
        }
        __sequence_output
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Vec_u8_to_JByteArray_7936d5de<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: Vec<u8>,
) -> ::core::result::Result<jni::objects::JByteArray<'a>, __JniErr> {
    Ok({
        env.byte_array_from_slice(v.as_slice())
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("encode_byte_array: {}", e))
            })?
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn Verdict_to_JObject_a94c1ffd<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Verdict,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___id: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.id.clone())?;
        let ___outcome__tag: jni::sys::jint;
        let ___outcome_g0: jni::sys::jlong;
        let ___outcome_g1: jni::objects::JObject;
        match &v.outcome {
            perftest_flat::Lookup::Absent => {
                ___outcome__tag = 0;
                ___outcome_g0 = 0i64;
                ___outcome_g1 = jni::objects::JObject::null();
            }
            perftest_flat::Lookup::Found(__s0_0) => {
                let ___outcome_found_v0: jni::sys::jlong = Summary_to_jlong_3cb103b9(
                    env,
                    __s0_0.clone(),
                )?;
                ___outcome__tag = 1;
                ___outcome_g0 = ___outcome_found_v0;
                ___outcome_g1 = jni::objects::JObject::null();
            }
            perftest_flat::Lookup::Failed(__s0_0) => {
                let ___outcome_failed_v0: jni::objects::JObject = String_to_JString_c7f3ca43(
                        env,
                        __s0_0.clone(),
                    )?
                    .into();
                ___outcome__tag = 2;
                ___outcome_g1 = ___outcome_failed_v0;
                ___outcome_g0 = 0i64;
            }
        }
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/model/Verdict",
                "fromParts",
                "(JIJLjava/lang/String;)Lio/prebindgen/covertest/model/Verdict;",
                &[
                    jni::objects::JValue::from(___id),
                    jni::objects::JValue::from(___outcome__tag),
                    jni::objects::JValue::from(___outcome_g0),
                    jni::objects::JValue::Object(&___outcome_g1),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn WrappedFields_to_JObject_f14f08c1<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::WrappedFields,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let ___id: jni::sys::jlong = i64_to_jlong_fbf9a9bc(env, v.id.clone())?;
        let ___boxed: jni::objects::JObject = Box_Option_i64_to_JObject_cf5a3724(
            env,
            v.boxed.clone(),
        )?;
        let ___plain: jni::objects::JObject = Option_i64_to_JObject_2ba9a5ed(
            env,
            v.plain.clone(),
        )?;
        let ___boxed_enum: jni::sys::jint = Box_Priority_to_jint_a16653ae(
            env,
            v.boxed_enum.clone(),
        )?;
        let ___plain_enum: jni::sys::jint = Priority_to_jint_447102d2(
            env,
            v.plain_enum.clone(),
        )?;
        let __obj = env
            .call_static_method(
                "io/prebindgen/covertest/WrappedFields",
                "fromParts",
                "(JLjava/lang/Long;Ljava/lang/Long;II)Lio/prebindgen/covertest/WrappedFields;",
                &[
                    jni::objects::JValue::from(___id),
                    jni::objects::JValue::Object(&___boxed),
                    jni::objects::JValue::Object(&___plain),
                    jni::objects::JValue::from(___boxed_enum),
                    jni::objects::JValue::from(___plain_enum),
                ],
            )
            .and_then(|__v| __v.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("encode struct via fromParts: {}", e)))?;
        __obj
    })
}
#[allow(dead_code)]
fn __jni_parts() {}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn bool_3_to_JBooleanArray_3f960c58<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: [bool; 3],
) -> ::core::result::Result<jni::objects::JBooleanArray<'a>, __JniErr> {
    Ok({
        let __buf: ::std::vec::Vec<jni::sys::jboolean> = v
            .iter()
            .map(|__x| *__x as jni::sys::jboolean)
            .collect();
        let __arr = env
            .new_boolean_array(__buf.len() as jni::sys::jsize)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?;
        env.set_boolean_array_region(&__arr, 0, &__buf)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn bool_to_jboolean_31306d98<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: bool,
) -> ::core::result::Result<jni::sys::jboolean, __JniErr> {
    Ok(v as jni::sys::jboolean)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn f64_2_to_JDoubleArray_dc30d1f9<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: [f64; 2],
) -> ::core::result::Result<jni::objects::JDoubleArray<'a>, __JniErr> {
    Ok({
        let __buf: ::std::vec::Vec<jni::sys::jdouble> = v
            .iter()
            .map(|__x| *__x as jni::sys::jdouble)
            .collect();
        let __arr = env
            .new_double_array(__buf.len() as jni::sys::jsize)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?;
        env.set_double_array_region(&__arr, 0, &__buf)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn f64_to_jdouble_9e4a8f70<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: f64,
) -> ::core::result::Result<jni::sys::jdouble, __JniErr> {
    Ok(v as jni::sys::jdouble)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn i16_2_to_JShortArray_098f4ad5<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: [i16; 2],
) -> ::core::result::Result<jni::objects::JShortArray<'a>, __JniErr> {
    Ok({
        let __buf: ::std::vec::Vec<jni::sys::jshort> = v
            .iter()
            .map(|__x| *__x as jni::sys::jshort)
            .collect();
        let __arr = env
            .new_short_array(__buf.len() as jni::sys::jsize)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?;
        env.set_short_array_region(&__arr, 0, &__buf)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn i32_3_to_JIntArray_60e5e35a<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: [i32; 3],
) -> ::core::result::Result<jni::objects::JIntArray<'a>, __JniErr> {
    Ok({
        let __buf: ::std::vec::Vec<jni::sys::jint> = v
            .iter()
            .map(|__x| *__x as jni::sys::jint)
            .collect();
        let __arr = env
            .new_int_array(__buf.len() as jni::sys::jsize)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?;
        env.set_int_array_region(&__arr, 0, &__buf)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn i32_to_Celsius_8c363100<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: i32,
) -> ::core::result::Result<perftest_flat::Celsius, __JniErr> {
    Ok(<i32 as ::core::convert::Into<perftest_flat::Celsius>>::into(v))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn i32_to_Percent_db3641cc<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: i32,
) -> ::core::result::Result<
    perftest_flat::Percent,
    <i32 as ::core::convert::TryInto<perftest_flat::Percent>>::Error,
> {
    <i32 as ::core::convert::TryInto<perftest_flat::Percent>>::try_into(v)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn i32_to_jint_a3e3b6ef<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: i32,
) -> ::core::result::Result<jni::sys::jint, __JniErr> {
    Ok(v as jni::sys::jint)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn i64_2_to_JLongArray_73596912<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: [i64; 2],
) -> ::core::result::Result<jni::objects::JLongArray<'a>, __JniErr> {
    Ok({
        let __buf: ::std::vec::Vec<jni::sys::jlong> = v
            .iter()
            .map(|__x| *__x as jni::sys::jlong)
            .collect();
        let __arr = env
            .new_long_array(__buf.len() as jni::sys::jsize)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?;
        env.set_long_array_region(&__arr, 0, &__buf)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn i64_to_Millis_bb88777a<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: i64,
) -> ::core::result::Result<perftest_flat::Millis, __JniErr> {
    Ok(cov_helpers::millis_from_long(v))
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn i64_to_jlong_fbf9a9bc<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: i64,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(v as jni::sys::jlong)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jboolean_to_bool_31306d98<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jboolean,
) -> ::core::result::Result<bool, __JniErr> {
    Ok(*v != 0)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jdouble_to_f64_9e4a8f70<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jdouble,
) -> ::core::result::Result<f64, __JniErr> {
    Ok(*v)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jint_to_Box_Priority_a16653ae<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jint,
) -> ::core::result::Result<Box<perftest_flat::Priority>, __JniErr> {
    Ok({
        let __inner = jint_to_Priority_447102d2(env, v)?;
        ::std::boxed::Box::new(__inner)
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
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
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jint_to_i32_a3e3b6ef<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jint,
) -> ::core::result::Result<i32, __JniErr> {
    Ok(*v)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jint_to_u16_28edf527<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jint,
) -> ::core::result::Result<u16, __JniErr> {
    Ok(
        ::core::primitive::u16::try_from(*v)
            .map_err(|_| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("u16 input out of range: {}", * v))
            })?,
    )
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jint_to_u8_553cf6ec<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jint,
) -> ::core::result::Result<u8, __JniErr> {
    Ok(
        ::core::primitive::u8::try_from(*v)
            .map_err(|_| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("u8 input out of range: {}", * v))
            })?,
    )
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_Archive_cd73502c<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::Archive>, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::Archive) })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_Box_Duration_0776c1ca<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<Box<perftest_flat::Duration>, __JniErr> {
    Ok({
        let __inner = {
            let __inner_s0 = jlong_to_u64_4384a5d6(env, v)?;
            let __inner_s1 = u64_to_Duration_7c0845f9(env, __inner_s0)
                .map_err(|__e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string()))?;
            __inner_s1
        };
        ::std::boxed::Box::new(__inner)
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_EscapeProbe_416aab42<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::EscapeProbe>, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::EscapeProbe) })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_Ingot_020c3a86<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::Ingot>, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::Ingot) })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_Ingot_020c3a86_owned<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<perftest_flat::Ingot, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    ::core::result::Result::Ok(unsafe {
        *::std::boxed::Box::from_raw(*v as *mut perftest_flat::Ingot)
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_Option_Duration_1cfa4d44<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<Option<perftest_flat::Duration>, __JniErr> {
    ::core::result::Result::Ok({
        if *v == -1i64 {
            ::core::option::Option::None
        } else {
            let __present = v;
            ::core::option::Option::Some(
                {
                    let __chain_s0 = jlong_to_u64_4384a5d6(env, __present)?;
                    let __chain_s1 = u64_to_Duration_7c0845f9(env, __chain_s0)
                        .map_err(|__e| <__JniErr as ::core::convert::From<
                            String,
                        >>::from(__e.to_string()))?;
                    ::core::result::Result::<_, __JniErr>::Ok(__chain_s1)
                }?,
            )
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_Option_Summary_252ef2ba<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<Option<perftest_flat::Summary>, __JniErr> {
    Ok({
        let __v: ::core::option::Option<perftest_flat::Summary> = if *v == 0 {
            None
        } else if (*v & 1) == 1 {
            return ::core::result::Result::Err(
                <__JniErr as ::core::convert::From<
                    String,
                >>::from("Operation on a closed native handle.".to_string()),
            );
        } else {
            Some(*std::boxed::Box::from_raw(*v as *mut perftest_flat::Summary))
        };
        __v
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_Option_Summary_828826f3<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<Option<OwnedObject<perftest_flat::Summary>>, __JniErr> {
    if *v == 0 {
        Ok(None)
    } else if (*v & 1) == 1 {
        Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        )
    } else {
        Ok(Some(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::Summary) }))
    }
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_PayloadHandler_d61fd890<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::PayloadHandler>, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::PayloadHandler) })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_PayloadVecHandler_b32d2812<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::PayloadVecHandler>, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::PayloadVecHandler) })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_StorageError_26b2d298<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::StorageError>, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::StorageError) })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_StorageHandler_3b4d3ed3<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::StorageHandler>, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::StorageHandler) })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_Storage_1b233abd<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::Storage>, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::Storage) })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_Summary_3cb103b9<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::Summary>, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::Summary) })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_Summary_3cb103b9_owned<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<perftest_flat::Summary, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    ::core::result::Result::Ok(unsafe {
        *::std::boxed::Box::from_raw(*v as *mut perftest_flat::Summary)
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_i64_fbf9a9bc<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<i64, __JniErr> {
    Ok(*v)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_u32_9594a230<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<u32, __JniErr> {
    Ok(
        ::core::primitive::u32::try_from(*v)
            .map_err(|_| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("u32 input out of range: {}", * v))
            })?,
    )
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn jlong_to_u64_4384a5d6<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<u64, __JniErr> {
    Ok(*v as ::core::primitive::u64)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
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
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn tuple2_to_Box_Option_Payload_0aa97b23<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        jni::sys::jboolean,
        (
            jni::sys::jlong,
            jni::sys::jint,
            jni::sys::jdouble,
            jni::sys::jboolean,
            jni::objects::JString<'v>,
        ),
    ),
) -> ::core::result::Result<Box<Option<perftest_flat::Payload>>, __JniErr> {
    ::core::result::Result::Ok(
        ::std::boxed::Box::new({
            if (v).0 == 0u8 {
                ::core::option::Option::None
            } else {
                let __present = (v).1;
                ::core::option::Option::Some(tuple5_to_Payload_bbb055bc(env, __present)?)
            }
        }),
    )
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple2_to_CacheConfig_fcb43e74<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: ((jni::sys::jint, jni::sys::jlong), jni::sys::jlong),
) -> ::core::result::Result<perftest_flat::CacheConfig, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::CacheConfig {
        replies: tuple2_to_RepliesConfig_e72c0bc9(env, (v).0)?,
        ttl: jlong_to_i64_fbf9a9bc(env, &((v).1))?,
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple2_to_HoldPolicy_c9df7bfe<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: (jni::objects::JObject<'a>, jni::objects::JObject<'a>),
) -> ::core::result::Result<perftest_flat::HoldPolicy, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::HoldPolicy {
        hold: JObject_to_Hold_5f85caaf(env, &((v).0))?,
        grace: JObject_to_Option_Hold_230d7f9b(env, &((v).1))?,
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple2_to_Holder_e920eaad<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: (jni::sys::jlong, jni::sys::jlong),
) -> ::core::result::Result<perftest_flat::Holder, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::Holder {
        tag: jlong_to_i64_fbf9a9bc(env, &((v).0))?,
        summary: jlong_to_Summary_3cb103b9_owned(env, &((v).1))?,
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn tuple2_to_Option_CacheConfig_c580ddce<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: (jni::sys::jboolean, ((jni::sys::jint, jni::sys::jlong), jni::sys::jlong)),
) -> ::core::result::Result<Option<perftest_flat::CacheConfig>, __JniErr> {
    ::core::result::Result::Ok({
        if (v).0 == 0u8 {
            ::core::option::Option::None
        } else {
            let __present = (v).1;
            ::core::option::Option::Some(tuple2_to_CacheConfig_fcb43e74(env, __present)?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn tuple2_to_Option_Holder_d4c4f3c3<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: (jni::sys::jboolean, (jni::sys::jlong, jni::sys::jlong)),
) -> ::core::result::Result<Option<perftest_flat::Holder>, __JniErr> {
    ::core::result::Result::Ok({
        if (v).0 == 0u8 {
            ::core::option::Option::None
        } else {
            let __present = (v).1;
            ::core::option::Option::Some(tuple2_to_Holder_e920eaad(env, __present)?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn tuple2_to_Option_Payload_af2bd54b<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        jni::sys::jboolean,
        (
            jni::sys::jlong,
            jni::sys::jint,
            jni::sys::jdouble,
            jni::sys::jboolean,
            jni::objects::JString<'v>,
        ),
    ),
) -> ::core::result::Result<Option<perftest_flat::Payload>, __JniErr> {
    ::core::result::Result::Ok({
        if (v).0 == 0u8 {
            ::core::option::Option::None
        } else {
            let __present = (v).1;
            ::core::option::Option::Some(tuple5_to_Payload_bbb055bc(env, __present)?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn tuple2_to_Option_Reading_550e9a70<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        jni::sys::jboolean,
        (
            jni::sys::jint,
            (),
            (jni::sys::jlong,),
            (jni::sys::jlong, jni::sys::jlong),
            (jni::objects::JString<'v>, jni::sys::jint),
            (jni::sys::jlong,),
        ),
    ),
) -> ::core::result::Result<Option<perftest_flat::Reading>, __JniErr> {
    ::core::result::Result::Ok({
        if (v).0 == 0u8 {
            ::core::option::Option::None
        } else {
            let __present = (v).1;
            ::core::option::Option::Some(tuple6_to_Reading_69702d1f(env, __present)?)
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple2_to_RepliesConfig_e72c0bc9<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: (jni::sys::jint, jni::sys::jlong),
) -> ::core::result::Result<perftest_flat::RepliesConfig, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::RepliesConfig {
        priority: jint_to_Priority_447102d2(env, &((v).0))?,
        max_samples: jlong_to_i64_fbf9a9bc(env, &((v).1))?,
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple2_to_Stamp_43c8d6ce<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: (jni::sys::jlong, jni::sys::jlong),
) -> ::core::result::Result<perftest_flat::Stamp, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::Stamp {
        secs: jlong_to_i64_fbf9a9bc(env, &((v).0))?,
        nanos: jlong_to_i64_fbf9a9bc(env, &((v).1))?,
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple2_to_Stamp_8d33d015<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: (jni::sys::jlong, jni::sys::jlong),
) -> ::core::result::Result<perftest_flat::Stamp, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::Stamp {
        secs: jlong_to_i64_fbf9a9bc(env, &((v).0))?,
        nanos: jlong_to_i64_fbf9a9bc(env, &((v).1))?,
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple2_to_Tagged_b68a6969<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: (jni::sys::jlong, jni::objects::JObject<'a>),
) -> ::core::result::Result<perftest_flat::Tagged, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::Tagged {
        id: jlong_to_i64_fbf9a9bc(env, &((v).0))?,
        marker: JObject_to_Marker_3dc81334(env, &((v).1))?,
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple4_to_Observation_438d4015<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        jni::sys::jlong,
        (
            jni::sys::jint,
            (),
            (jni::sys::jlong,),
            (jni::sys::jlong, jni::sys::jlong),
            (jni::objects::JString<'a>, jni::sys::jint),
            (jni::sys::jlong,),
        ),
        (
            jni::sys::jboolean,
            (
                jni::sys::jint,
                (),
                (jni::sys::jlong,),
                (jni::sys::jlong, jni::sys::jlong),
                (jni::objects::JString<'a>, jni::sys::jint),
                (jni::sys::jlong,),
            ),
        ),
        jni::objects::JString<'a>,
    ),
) -> ::core::result::Result<perftest_flat::Observation, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::Observation {
        id: jlong_to_i64_fbf9a9bc(env, &((v).0))?,
        reading: tuple6_to_Reading_69702d1f(env, (v).1)?,
        fallback: tuple2_to_Option_Reading_550e9a70(env, (v).2)?,
        note: JString_to_String_c7f3ca43(env, &((v).3))?,
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple5_to_Box_Payload_697caed4<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        jni::sys::jlong,
        jni::sys::jint,
        jni::sys::jdouble,
        jni::sys::jboolean,
        jni::objects::JString<'a>,
    ),
) -> ::core::result::Result<Box<perftest_flat::Payload>, __JniErr> {
    ::core::result::Result::Ok(
        ::std::boxed::Box::new(perftest_flat::Payload {
            id: jlong_to_i64_fbf9a9bc(env, &((v).0))?,
            seq: jint_to_i32_a3e3b6ef(env, &((v).1))?,
            value: jdouble_to_f64_9e4a8f70(env, &((v).2))?,
            flag: jboolean_to_bool_31306d98(env, &((v).3))?,
            label: JString_to_Option_Box_String_071e4c8c(env, &((v).4))?,
        }),
    )
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple5_to_Payload_2ea1d0c2<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        jni::sys::jlong,
        jni::sys::jint,
        jni::sys::jdouble,
        jni::sys::jboolean,
        jni::objects::JString<'a>,
    ),
) -> ::core::result::Result<perftest_flat::Payload, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::Payload {
        id: jlong_to_i64_fbf9a9bc(env, &((v).0))?,
        seq: jint_to_i32_a3e3b6ef(env, &((v).1))?,
        value: jdouble_to_f64_9e4a8f70(env, &((v).2))?,
        flag: jboolean_to_bool_31306d98(env, &((v).3))?,
        label: JString_to_Option_Box_String_071e4c8c(env, &((v).4))?,
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple5_to_Payload_bbb055bc<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        jni::sys::jlong,
        jni::sys::jint,
        jni::sys::jdouble,
        jni::sys::jboolean,
        jni::objects::JString<'a>,
    ),
) -> ::core::result::Result<perftest_flat::Payload, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::Payload {
        id: jlong_to_i64_fbf9a9bc(env, &((v).0))?,
        seq: jint_to_i32_a3e3b6ef(env, &((v).1))?,
        value: jdouble_to_f64_9e4a8f70(env, &((v).2))?,
        flag: jboolean_to_bool_31306d98(env, &((v).3))?,
        label: JString_to_Option_Box_String_071e4c8c(env, &((v).4))?,
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple6_to_Reading_69702d1f<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        jni::sys::jint,
        (),
        (jni::sys::jlong,),
        (jni::sys::jlong, jni::sys::jlong),
        (jni::objects::JString<'a>, jni::sys::jint),
        (jni::sys::jlong,),
    ),
) -> ::core::result::Result<perftest_flat::Reading, __JniErr> {
    ::core::result::Result::Ok({
        let __tag = (v).0;
        match __tag {
            0i32 => perftest_flat::Reading::Missing,
            1i32 => {
                let __choice = v;
                let __arm = (__choice).2;
                perftest_flat::Reading::Exact(jlong_to_i64_fbf9a9bc(env, &((__arm).0))?)
            }
            2i32 => {
                let __choice = v;
                let __arm = (__choice).3;
                perftest_flat::Reading::Range {
                    low: jlong_to_i64_fbf9a9bc(env, &((__arm).0))?,
                    high: jlong_to_i64_fbf9a9bc(env, &((__arm).1))?,
                }
            }
            3i32 => {
                let __choice = v;
                let __arm = (__choice).4;
                perftest_flat::Reading::Labeled(
                    JString_to_String_c7f3ca43(env, &((__arm).0))?,
                    jint_to_Priority_447102d2(env, &((__arm).1))?,
                )
            }
            4i32 => {
                let __choice = v;
                let __arm = (__choice).5;
                perftest_flat::Reading::Companion(
                    jlong_to_i64_fbf9a9bc(env, &((__arm).0))?,
                )
            }
            _ => {
                return ::core::result::Result::Err({
                    let __invalid_tag = __tag;
                    <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("{}: invalid tag {}", "Reading", __invalid_tag,))
                });
            }
        }
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
#[inline(always)]
pub(crate) unsafe fn tuple7_to_Arrays_c0fbd13f<'env, 'a>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        jni::objects::JByteArray<'a>,
        jni::objects::JShortArray<'a>,
        jni::objects::JIntArray<'a>,
        jni::objects::JLongArray<'a>,
        jni::objects::JDoubleArray<'a>,
        jni::objects::JBooleanArray<'a>,
        jni::objects::JLongArray<'a>,
    ),
) -> ::core::result::Result<perftest_flat::Arrays, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::Arrays {
        bytes: JByteArray_to_u8_4_39abedfa(env, &((v).0))?,
        shorts: JShortArray_to_i16_2_098f4ad5(env, &((v).1))?,
        ints: JIntArray_to_i32_3_60e5e35a(env, &((v).2))?,
        longs: JLongArray_to_i64_2_73596912(env, &((v).3))?,
        doubles: JDoubleArray_to_f64_2_dc30d1f9(env, &((v).4))?,
        flags: JBooleanArray_to_bool_3_3f960c58(env, &((v).5))?,
        raw: JLongArray_to_u64_2_60bcc6a5(env, &((v).6))?,
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn u16_to_jint_28edf527<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: u16,
) -> ::core::result::Result<jni::sys::jint, __JniErr> {
    Ok(v as jni::sys::jint)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn u32_to_jlong_9594a230<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: u32,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(v as jni::sys::jlong)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn u64_2_to_JLongArray_60bcc6a5<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: [u64; 2],
) -> ::core::result::Result<jni::objects::JLongArray<'a>, __JniErr> {
    Ok({
        let __buf: ::std::vec::Vec<jni::sys::jlong> = v
            .iter()
            .map(|__x| *__x as jni::sys::jlong)
            .collect();
        let __arr = env
            .new_long_array(__buf.len() as jni::sys::jsize)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?;
        env.set_long_array_region(&__arr, 0, &__buf)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?;
        __arr
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn u64_to_Duration_7c0845f9<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: u64,
) -> ::core::result::Result<perftest_flat::Duration, __JniErr> {
    {
        if (true && true && (v) <= 86400000u64) && !(false) {
            ::core::result::Result::Ok(crate::duration_from_millis(v))
        } else {
            ::core::result::Result::Err(
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        "{} representation is outside its declared domain", "Duration"
                    ),
                ),
            )
        }
    }
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn u64_to_jlong_4384a5d6<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: u64,
) -> ::core::result::Result<jni::sys::jlong, __JniErr> {
    Ok(v as jni::sys::jlong)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn u8_4_to_JByteArray_39abedfa<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: [u8; 4],
) -> ::core::result::Result<jni::objects::JByteArray<'a>, __JniErr> {
    Ok({
        env.byte_array_from_slice(&v)
            .map_err(|e| {
                <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("fixed-size array encode: {}", e))
            })?
    })
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn u8_to_jint_553cf6ec<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: u8,
) -> ::core::result::Result<jni::sys::jint, __JniErr> {
    Ok(v as jni::sys::jint)
}
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn unit_to_unit_9ecccf8e<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: (),
) -> ::core::result::Result<(), __JniErr> {
    Ok(v)
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_annotatedAlternateValue<
    'a,
>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a_payload_id: jni::sys::jlong,
    a_payload_seq: jni::sys::jint,
    a_payload_value: jni::sys::jdouble,
    a_payload_flag: jni::sys::jboolean,
    a_payload_label: jni::objects::JString<'a>,
    a_alternate_present: jni::sys::jboolean,
    a_alternate_id: jni::sys::jlong,
    a_alternate_seq: jni::sys::jint,
    a_alternate_value: jni::sys::jdouble,
    a_alternate_flag: jni::sys::jboolean,
    a_alternate_label: jni::objects::JString<'a>,
    a_ttl_present: jni::sys::jboolean,
    a_ttl_value: jni::sys::jlong,
    a_priority_present: jni::sys::jboolean,
    a_priority_value: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __flat_a_payload_id = match jlong_to_i64_fbf9a9bc(&mut env, &a_payload_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload_seq = match jint_to_i32_a3e3b6ef(&mut env, &a_payload_seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload_value = match jdouble_to_f64_9e4a8f70(
        &mut env,
        &a_payload_value,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload_flag = match jboolean_to_bool_31306d98(
        &mut env,
        &a_payload_flag,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload_label = match JString_to_Option_Box_String_071e4c8c(
        &mut env,
        &a_payload_label,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload = perftest_flat::Payload {
        id: __flat_a_payload_id,
        seq: __flat_a_payload_seq,
        value: __flat_a_payload_value,
        flag: __flat_a_payload_flag,
        label: __flat_a_payload_label,
    };
    let __flat_a_alternate = if a_alternate_present != 0u8 {
        let __flat_a_alternate_id = match jlong_to_i64_fbf9a9bc(
            &mut env,
            &a_alternate_id,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __flat_a_alternate_seq = match jint_to_i32_a3e3b6ef(
            &mut env,
            &a_alternate_seq,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __flat_a_alternate_value = match jdouble_to_f64_9e4a8f70(
            &mut env,
            &a_alternate_value,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __flat_a_alternate_flag = match jboolean_to_bool_31306d98(
            &mut env,
            &a_alternate_flag,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __flat_a_alternate_label = match JString_to_Option_Box_String_071e4c8c(
            &mut env,
            &a_alternate_label,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(perftest_flat::Payload {
            id: __flat_a_alternate_id,
            seq: __flat_a_alternate_seq,
            value: __flat_a_alternate_value,
            flag: __flat_a_alternate_flag,
            label: __flat_a_alternate_label,
        })
    } else {
        ::core::option::Option::None
    };
    let __flat_a_ttl = if a_ttl_present != 0u8 {
        let __flat_a_ttl_value = match jlong_to_i64_fbf9a9bc(&mut env, &a_ttl_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__flat_a_ttl_value)
    } else {
        ::core::option::Option::None
    };
    let __flat_a_priority = if a_priority_present != 0u8 {
        let __flat_a_priority_value = match jint_to_Priority_447102d2(
            &mut env,
            &a_priority_value,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__flat_a_priority_value)
    } else {
        ::core::option::Option::None
    };
    let __flat_a = perftest_flat::Annotated {
        payload: __flat_a_payload,
        alternate: __flat_a_alternate,
        ttl: __flat_a_ttl,
        priority: __flat_a_priority,
    };
    let a = __flat_a;
    let __out = perftest_flat::annotated_alternate_value(&a);
    match Option_f64_to_JObject_b3f3e9a9(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let payload = match tuple5_to_Payload_bbb055bc(
        &mut env,
        (payload_id, payload_seq, payload_value, payload_flag, payload_label),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let ttl = if ttl_present != 0u8 {
        let __ttl_val = match jlong_to_i64_fbf9a9bc(&mut env, &ttl_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    a_payload_id: jni::sys::jlong,
    a_payload_seq: jni::sys::jint,
    a_payload_value: jni::sys::jdouble,
    a_payload_flag: jni::sys::jboolean,
    a_payload_label: jni::objects::JString<'a>,
    a_alternate_present: jni::sys::jboolean,
    a_alternate_id: jni::sys::jlong,
    a_alternate_seq: jni::sys::jint,
    a_alternate_value: jni::sys::jdouble,
    a_alternate_flag: jni::sys::jboolean,
    a_alternate_label: jni::objects::JString<'a>,
    a_ttl_present: jni::sys::jboolean,
    a_ttl_value: jni::sys::jlong,
    a_priority_present: jni::sys::jboolean,
    a_priority_value: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jdouble {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __flat_a_payload_id = match jlong_to_i64_fbf9a9bc(&mut env, &a_payload_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __flat_a_payload_seq = match jint_to_i32_a3e3b6ef(&mut env, &a_payload_seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __flat_a_payload_value = match jdouble_to_f64_9e4a8f70(
        &mut env,
        &a_payload_value,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __flat_a_payload_flag = match jboolean_to_bool_31306d98(
        &mut env,
        &a_payload_flag,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __flat_a_payload_label = match JString_to_Option_Box_String_071e4c8c(
        &mut env,
        &a_payload_label,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __flat_a_payload = perftest_flat::Payload {
        id: __flat_a_payload_id,
        seq: __flat_a_payload_seq,
        value: __flat_a_payload_value,
        flag: __flat_a_payload_flag,
        label: __flat_a_payload_label,
    };
    let __flat_a_alternate = if a_alternate_present != 0u8 {
        let __flat_a_alternate_id = match jlong_to_i64_fbf9a9bc(
            &mut env,
            &a_alternate_id,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0.0 as jni::sys::jdouble;
            }
        };
        let __flat_a_alternate_seq = match jint_to_i32_a3e3b6ef(
            &mut env,
            &a_alternate_seq,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0.0 as jni::sys::jdouble;
            }
        };
        let __flat_a_alternate_value = match jdouble_to_f64_9e4a8f70(
            &mut env,
            &a_alternate_value,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0.0 as jni::sys::jdouble;
            }
        };
        let __flat_a_alternate_flag = match jboolean_to_bool_31306d98(
            &mut env,
            &a_alternate_flag,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0.0 as jni::sys::jdouble;
            }
        };
        let __flat_a_alternate_label = match JString_to_Option_Box_String_071e4c8c(
            &mut env,
            &a_alternate_label,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0.0 as jni::sys::jdouble;
            }
        };
        ::core::option::Option::Some(perftest_flat::Payload {
            id: __flat_a_alternate_id,
            seq: __flat_a_alternate_seq,
            value: __flat_a_alternate_value,
            flag: __flat_a_alternate_flag,
            label: __flat_a_alternate_label,
        })
    } else {
        ::core::option::Option::None
    };
    let __flat_a_ttl = if a_ttl_present != 0u8 {
        let __flat_a_ttl_value = match jlong_to_i64_fbf9a9bc(&mut env, &a_ttl_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0.0 as jni::sys::jdouble;
            }
        };
        ::core::option::Option::Some(__flat_a_ttl_value)
    } else {
        ::core::option::Option::None
    };
    let __flat_a_priority = if a_priority_present != 0u8 {
        let __flat_a_priority_value = match jint_to_Priority_447102d2(
            &mut env,
            &a_priority_value,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0.0 as jni::sys::jdouble;
            }
        };
        ::core::option::Option::Some(__flat_a_priority_value)
    } else {
        ::core::option::Option::None
    };
    let __flat_a = perftest_flat::Annotated {
        payload: __flat_a_payload,
        alternate: __flat_a_alternate,
        ttl: __flat_a_ttl,
        priority: __flat_a_priority,
    };
    let a = __flat_a;
    let __out = perftest_flat::annotated_payload_value(&a);
    match f64_to_jdouble_9e4a8f70(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    a_payload_id: jni::sys::jlong,
    a_payload_seq: jni::sys::jint,
    a_payload_value: jni::sys::jdouble,
    a_payload_flag: jni::sys::jboolean,
    a_payload_label: jni::objects::JString<'a>,
    a_alternate_present: jni::sys::jboolean,
    a_alternate_id: jni::sys::jlong,
    a_alternate_seq: jni::sys::jint,
    a_alternate_value: jni::sys::jdouble,
    a_alternate_flag: jni::sys::jboolean,
    a_alternate_label: jni::objects::JString<'a>,
    a_ttl_present: jni::sys::jboolean,
    a_ttl_value: jni::sys::jlong,
    a_priority_present: jni::sys::jboolean,
    a_priority_value: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __flat_a_payload_id = match jlong_to_i64_fbf9a9bc(&mut env, &a_payload_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload_seq = match jint_to_i32_a3e3b6ef(&mut env, &a_payload_seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload_value = match jdouble_to_f64_9e4a8f70(
        &mut env,
        &a_payload_value,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload_flag = match jboolean_to_bool_31306d98(
        &mut env,
        &a_payload_flag,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload_label = match JString_to_Option_Box_String_071e4c8c(
        &mut env,
        &a_payload_label,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload = perftest_flat::Payload {
        id: __flat_a_payload_id,
        seq: __flat_a_payload_seq,
        value: __flat_a_payload_value,
        flag: __flat_a_payload_flag,
        label: __flat_a_payload_label,
    };
    let __flat_a_alternate = if a_alternate_present != 0u8 {
        let __flat_a_alternate_id = match jlong_to_i64_fbf9a9bc(
            &mut env,
            &a_alternate_id,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __flat_a_alternate_seq = match jint_to_i32_a3e3b6ef(
            &mut env,
            &a_alternate_seq,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __flat_a_alternate_value = match jdouble_to_f64_9e4a8f70(
            &mut env,
            &a_alternate_value,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __flat_a_alternate_flag = match jboolean_to_bool_31306d98(
            &mut env,
            &a_alternate_flag,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __flat_a_alternate_label = match JString_to_Option_Box_String_071e4c8c(
            &mut env,
            &a_alternate_label,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(perftest_flat::Payload {
            id: __flat_a_alternate_id,
            seq: __flat_a_alternate_seq,
            value: __flat_a_alternate_value,
            flag: __flat_a_alternate_flag,
            label: __flat_a_alternate_label,
        })
    } else {
        ::core::option::Option::None
    };
    let __flat_a_ttl = if a_ttl_present != 0u8 {
        let __flat_a_ttl_value = match jlong_to_i64_fbf9a9bc(&mut env, &a_ttl_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__flat_a_ttl_value)
    } else {
        ::core::option::Option::None
    };
    let __flat_a_priority = if a_priority_present != 0u8 {
        let __flat_a_priority_value = match jint_to_Priority_447102d2(
            &mut env,
            &a_priority_value,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__flat_a_priority_value)
    } else {
        ::core::option::Option::None
    };
    let __flat_a = perftest_flat::Annotated {
        payload: __flat_a_payload,
        alternate: __flat_a_alternate,
        ttl: __flat_a_ttl,
        priority: __flat_a_priority,
    };
    let a = __flat_a;
    let __out = perftest_flat::annotated_priority(&a);
    match Option_Priority_to_JObject_ad5cbb32(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    a_payload_id: jni::sys::jlong,
    a_payload_seq: jni::sys::jint,
    a_payload_value: jni::sys::jdouble,
    a_payload_flag: jni::sys::jboolean,
    a_payload_label: jni::objects::JString<'a>,
    a_alternate_present: jni::sys::jboolean,
    a_alternate_id: jni::sys::jlong,
    a_alternate_seq: jni::sys::jint,
    a_alternate_value: jni::sys::jdouble,
    a_alternate_flag: jni::sys::jboolean,
    a_alternate_label: jni::objects::JString<'a>,
    a_ttl_present: jni::sys::jboolean,
    a_ttl_value: jni::sys::jlong,
    a_priority_present: jni::sys::jboolean,
    a_priority_value: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __flat_a_payload_id = match jlong_to_i64_fbf9a9bc(&mut env, &a_payload_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload_seq = match jint_to_i32_a3e3b6ef(&mut env, &a_payload_seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload_value = match jdouble_to_f64_9e4a8f70(
        &mut env,
        &a_payload_value,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload_flag = match jboolean_to_bool_31306d98(
        &mut env,
        &a_payload_flag,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload_label = match JString_to_Option_Box_String_071e4c8c(
        &mut env,
        &a_payload_label,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_a_payload = perftest_flat::Payload {
        id: __flat_a_payload_id,
        seq: __flat_a_payload_seq,
        value: __flat_a_payload_value,
        flag: __flat_a_payload_flag,
        label: __flat_a_payload_label,
    };
    let __flat_a_alternate = if a_alternate_present != 0u8 {
        let __flat_a_alternate_id = match jlong_to_i64_fbf9a9bc(
            &mut env,
            &a_alternate_id,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __flat_a_alternate_seq = match jint_to_i32_a3e3b6ef(
            &mut env,
            &a_alternate_seq,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __flat_a_alternate_value = match jdouble_to_f64_9e4a8f70(
            &mut env,
            &a_alternate_value,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __flat_a_alternate_flag = match jboolean_to_bool_31306d98(
            &mut env,
            &a_alternate_flag,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __flat_a_alternate_label = match JString_to_Option_Box_String_071e4c8c(
            &mut env,
            &a_alternate_label,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(perftest_flat::Payload {
            id: __flat_a_alternate_id,
            seq: __flat_a_alternate_seq,
            value: __flat_a_alternate_value,
            flag: __flat_a_alternate_flag,
            label: __flat_a_alternate_label,
        })
    } else {
        ::core::option::Option::None
    };
    let __flat_a_ttl = if a_ttl_present != 0u8 {
        let __flat_a_ttl_value = match jlong_to_i64_fbf9a9bc(&mut env, &a_ttl_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__flat_a_ttl_value)
    } else {
        ::core::option::Option::None
    };
    let __flat_a_priority = if a_priority_present != 0u8 {
        let __flat_a_priority_value = match jint_to_Priority_447102d2(
            &mut env,
            &a_priority_value,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__flat_a_priority_value)
    } else {
        ::core::option::Option::None
    };
    let __flat_a = perftest_flat::Annotated {
        payload: __flat_a_payload,
        alternate: __flat_a_alternate,
        ttl: __flat_a_ttl,
        priority: __flat_a_priority,
    };
    let a = __flat_a;
    let __out = perftest_flat::annotated_ttl(&a);
    match Option_i64_to_JObject_2ba9a5ed(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let a = match jlong_to_Archive_cd73502c(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::archive_latest(&a);
    match Option_Summary_to_jlong_828826f3(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::archive_new();
    match Archive_to_jlong_cd73502c(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_archiveReading<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a: jni::sys::jlong,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let a = match jlong_to_Archive_cd73502c(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/ReadingBuilder";
    const __CB_DESCR: &str = "(IJJJLjava/lang/String;IJ)Ljava/lang/Object;";
    let __out = perftest_flat::archive_reading(&a);
    let __obj0: jni::sys::jvalue;
    let __obj1: jni::sys::jvalue;
    let __obj2: jni::sys::jvalue;
    let __obj3: jni::sys::jvalue;
    let __obj4: jni::objects::JObject;
    let __obj5: jni::sys::jvalue;
    let __obj6: jni::sys::jvalue;
    match __out {
        perftest_flat::Reading::Missing => {
            __obj0 = jni::sys::jvalue { i: 0 };
            __obj1 = jni::sys::jvalue { j: 0i64 };
            __obj2 = jni::sys::jvalue { j: 0i64 };
            __obj3 = jni::sys::jvalue { j: 0i64 };
            __obj4 = jni::objects::JObject::null();
            __obj5 = jni::sys::jvalue { i: 0i32 };
            __obj6 = jni::sys::jvalue { j: 0i64 };
        }
        perftest_flat::Reading::Exact(__sv0) => {
            let __enc___obj1 = match i64_to_jlong_fbf9a9bc(&mut env, __sv0.clone()) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
            };
            __obj1 = jni::sys::jvalue {
                j: __enc___obj1,
            };
            __obj0 = jni::sys::jvalue { i: 1 };
            __obj2 = jni::sys::jvalue { j: 0i64 };
            __obj3 = jni::sys::jvalue { j: 0i64 };
            __obj4 = jni::objects::JObject::null();
            __obj5 = jni::sys::jvalue { i: 0i32 };
            __obj6 = jni::sys::jvalue { j: 0i64 };
        }
        perftest_flat::Reading::Range { low: __sv0, high: __sv1 } => {
            let __enc___obj2 = match i64_to_jlong_fbf9a9bc(&mut env, __sv0.clone()) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
            };
            __obj2 = jni::sys::jvalue {
                j: __enc___obj2,
            };
            let __enc___obj3 = match i64_to_jlong_fbf9a9bc(&mut env, __sv1.clone()) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
            };
            __obj3 = jni::sys::jvalue {
                j: __enc___obj3,
            };
            __obj0 = jni::sys::jvalue { i: 2 };
            __obj1 = jni::sys::jvalue { j: 0i64 };
            __obj4 = jni::objects::JObject::null();
            __obj5 = jni::sys::jvalue { i: 0i32 };
            __obj6 = jni::sys::jvalue { j: 0i64 };
        }
        perftest_flat::Reading::Labeled(__sv0, __sv1) => {
            let __enc___obj4 = match String_to_JString_c7f3ca43(
                &mut env,
                __sv0.clone(),
            ) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
            };
            __obj4 = __enc___obj4.into();
            let __enc___obj5 = match Priority_to_jint_447102d2(&mut env, __sv1.clone()) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
            };
            __obj5 = jni::sys::jvalue {
                i: __enc___obj5,
            };
            __obj0 = jni::sys::jvalue { i: 3 };
            __obj1 = jni::sys::jvalue { j: 0i64 };
            __obj2 = jni::sys::jvalue { j: 0i64 };
            __obj3 = jni::sys::jvalue { j: 0i64 };
            __obj6 = jni::sys::jvalue { j: 0i64 };
        }
        perftest_flat::Reading::Companion(__sv0) => {
            let __enc___obj6 = match i64_to_jlong_fbf9a9bc(&mut env, __sv0.clone()) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
            };
            __obj6 = jni::sys::jvalue {
                j: __enc___obj6,
            };
            __obj0 = jni::sys::jvalue { i: 4 };
            __obj1 = jni::sys::jvalue { j: 0i64 };
            __obj2 = jni::sys::jvalue { j: 0i64 };
            __obj3 = jni::sys::jvalue { j: 0i64 };
            __obj4 = jni::objects::JObject::null();
            __obj5 = jni::sys::jvalue { i: 0i32 };
        }
    }
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
                __obj5,
                __obj6,
            ],
        )
    {
        ::core::result::Result::Ok(__o) => __o,
        ::core::result::Result::Err(__e) => {
            let _ = env.exception_describe();
            let __e2 = <__JniErr as ::core::convert::From<
                String,
            >>::from(__e.to_string());
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_archiveReadingMaybe<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a: jni::sys::jlong,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let a = match jlong_to_Archive_cd73502c(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/ReadingBuilder";
    const __CB_DESCR: &str = "(IJJJLjava/lang/String;IJ)Ljava/lang/Object;";
    let __out = perftest_flat::archive_reading_maybe(&a);
    match __out {
        ::core::option::Option::Some(__inner) => {
            let __obj0: jni::sys::jvalue;
            let __obj1: jni::sys::jvalue;
            let __obj2: jni::sys::jvalue;
            let __obj3: jni::sys::jvalue;
            let __obj4: jni::objects::JObject;
            let __obj5: jni::sys::jvalue;
            let __obj6: jni::sys::jvalue;
            match __inner {
                perftest_flat::Reading::Missing => {
                    __obj0 = jni::sys::jvalue { i: 0 };
                    __obj1 = jni::sys::jvalue { j: 0i64 };
                    __obj2 = jni::sys::jvalue { j: 0i64 };
                    __obj3 = jni::sys::jvalue { j: 0i64 };
                    __obj4 = jni::objects::JObject::null();
                    __obj5 = jni::sys::jvalue { i: 0i32 };
                    __obj6 = jni::sys::jvalue { j: 0i64 };
                }
                perftest_flat::Reading::Exact(__sv0) => {
                    let __enc___obj1 = match i64_to_jlong_fbf9a9bc(
                        &mut env,
                        __sv0.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj1 = jni::sys::jvalue {
                        j: __enc___obj1,
                    };
                    __obj0 = jni::sys::jvalue { i: 1 };
                    __obj2 = jni::sys::jvalue { j: 0i64 };
                    __obj3 = jni::sys::jvalue { j: 0i64 };
                    __obj4 = jni::objects::JObject::null();
                    __obj5 = jni::sys::jvalue { i: 0i32 };
                    __obj6 = jni::sys::jvalue { j: 0i64 };
                }
                perftest_flat::Reading::Range { low: __sv0, high: __sv1 } => {
                    let __enc___obj2 = match i64_to_jlong_fbf9a9bc(
                        &mut env,
                        __sv0.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj2 = jni::sys::jvalue {
                        j: __enc___obj2,
                    };
                    let __enc___obj3 = match i64_to_jlong_fbf9a9bc(
                        &mut env,
                        __sv1.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj3 = jni::sys::jvalue {
                        j: __enc___obj3,
                    };
                    __obj0 = jni::sys::jvalue { i: 2 };
                    __obj1 = jni::sys::jvalue { j: 0i64 };
                    __obj4 = jni::objects::JObject::null();
                    __obj5 = jni::sys::jvalue { i: 0i32 };
                    __obj6 = jni::sys::jvalue { j: 0i64 };
                }
                perftest_flat::Reading::Labeled(__sv0, __sv1) => {
                    let __enc___obj4 = match String_to_JString_c7f3ca43(
                        &mut env,
                        __sv0.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj4 = __enc___obj4.into();
                    let __enc___obj5 = match Priority_to_jint_447102d2(
                        &mut env,
                        __sv1.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj5 = jni::sys::jvalue {
                        i: __enc___obj5,
                    };
                    __obj0 = jni::sys::jvalue { i: 3 };
                    __obj1 = jni::sys::jvalue { j: 0i64 };
                    __obj2 = jni::sys::jvalue { j: 0i64 };
                    __obj3 = jni::sys::jvalue { j: 0i64 };
                    __obj6 = jni::sys::jvalue { j: 0i64 };
                }
                perftest_flat::Reading::Companion(__sv0) => {
                    let __enc___obj6 = match i64_to_jlong_fbf9a9bc(
                        &mut env,
                        __sv0.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj6 = jni::sys::jvalue {
                        j: __enc___obj6,
                    };
                    __obj0 = jni::sys::jvalue { i: 4 };
                    __obj1 = jni::sys::jvalue { j: 0i64 };
                    __obj2 = jni::sys::jvalue { j: 0i64 };
                    __obj3 = jni::sys::jvalue { j: 0i64 };
                    __obj4 = jni::objects::JObject::null();
                    __obj5 = jni::sys::jvalue { i: 0i32 };
                }
            }
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
                        __obj5,
                        __obj6,
                    ],
                )
            {
                ::core::result::Result::Ok(__o) => __o,
                ::core::result::Result::Err(__e) => {
                    let _ = env.exception_describe();
                    let __e2 = <__JniErr as ::core::convert::From<
                        String,
                    >>::from(__e.to_string());
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e2.to_string(),
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
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_archiveSetReading<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a: jni::sys::jlong,
    which: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut a = match jlong_to_Archive_cd73502c(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let which = match jint_to_i32_a3e3b6ef(&mut env, &which) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::archive_set_reading(&mut a, which);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            ()
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut a = match jlong_to_Archive_cd73502c(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __exp_s_sel = match jint_to_i32_a3e3b6ef(&mut env, &s_sel) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __exp_s_0_0: Option<i64> = if s_0_0_present != 0u8 {
        let __v = match jlong_to_i64_fbf9a9bc(&mut env, &s_0_0_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__je.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::archive_store(&mut a, __folded_s);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_arraysEcho<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a_bytes: jni::objects::JByteArray<'a>,
    a_shorts: jni::objects::JShortArray<'a>,
    a_ints: jni::objects::JIntArray<'a>,
    a_longs: jni::objects::JLongArray<'a>,
    a_doubles: jni::objects::JDoubleArray<'a>,
    a_flags: jni::objects::JBooleanArray<'a>,
    a_raw: jni::objects::JLongArray<'a>,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let a = match tuple7_to_Arrays_c0fbd13f(
        &mut env,
        (a_bytes, a_shorts, a_ints, a_longs, a_doubles, a_flags, a_raw),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/ArraysBuilder";
    const __CB_DESCR: &str = "([B[S[I[J[D[Z[J)Ljava/lang/Object;";
    let __out = perftest_flat::arrays_echo(a);
    let (
        __chain_wire0,
        __chain_wire1,
        __chain_wire2,
        __chain_wire3,
        __chain_wire4,
        __chain_wire5,
        __chain_wire6,
    ) = match Arrays_to_tuple7_c0fbd13f(&mut env, __out) {
        ::core::result::Result::Ok(__intermediate) => __intermediate,
        ::core::result::Result::Err(__chain_error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__chain_error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __obj0: jni::objects::JObject = __chain_wire0.into();
    let __obj1: jni::objects::JObject = __chain_wire1.into();
    let __obj2: jni::objects::JObject = __chain_wire2.into();
    let __obj3: jni::objects::JObject = __chain_wire3.into();
    let __obj4: jni::objects::JObject = __chain_wire4.into();
    let __obj5: jni::objects::JObject = __chain_wire5.into();
    let __obj6: jni::objects::JObject = __chain_wire6.into();
    match __CB_MID
        .call_object(
            &mut env,
            __CB_FQN,
            "run",
            __CB_DESCR,
            &__builder,
            &[
                jni::sys::jvalue {
                    l: __obj0.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj1.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj2.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj3.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj4.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj5.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj6.as_raw(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_blobValueEcho<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value: jni::objects::JObject<'a>,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let value = match JObject_to_BlobValue_89b5dab7(&mut env, &value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/BlobValueBuilder";
    const __CB_DESCR: &str = "(JJ[BLjava/util/List;)Ljava/lang/Object;";
    let __out = perftest_flat::blob_value_echo(value);
    let ((__chain_wire0, __chain_wire1), __chain_wire2, __chain_wire3) = match BlobValue_to_tuple3_2c75fc67(
        &mut env,
        __out,
    ) {
        ::core::result::Result::Ok(__intermediate) => __intermediate,
        ::core::result::Result::Err(__chain_error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__chain_error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __obj0 = jni::sys::jvalue {
        j: __chain_wire0,
    };
    let __obj1 = jni::sys::jvalue {
        j: __chain_wire1,
    };
    let __obj2: jni::objects::JObject = __chain_wire2.into();
    let __obj3: jni::objects::JObject = __chain_wire3;
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
                jni::sys::jvalue {
                    l: __obj2.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj3.as_raw(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_blobValueNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    secs: jni::sys::jlong,
    id: jni::objects::JByteArray<'a>,
    chunks: jni::objects::JObject<'a>,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let secs = match jlong_to_i64_fbf9a9bc(&mut env, &secs) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let id = match JByteArray_to_Vec_u8_7936d5de(&mut env, &id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let chunks = match JObject_to_Vec_Vec_u8_43404875(&mut env, &chunks) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/BlobValueBuilder";
    const __CB_DESCR: &str = "(JJ[BLjava/util/List;)Ljava/lang/Object;";
    let __out = perftest_flat::blob_value_new(secs, id, chunks);
    let ((__chain_wire0, __chain_wire1), __chain_wire2, __chain_wire3) = match BlobValue_to_tuple3_2c75fc67(
        &mut env,
        __out,
    ) {
        ::core::result::Result::Ok(__intermediate) => __intermediate,
        ::core::result::Result::Err(__chain_error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__chain_error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __obj0 = jni::sys::jvalue {
        j: __chain_wire0,
    };
    let __obj1 = jni::sys::jvalue {
        j: __chain_wire1,
    };
    let __obj2: jni::objects::JObject = __chain_wire2.into();
    let __obj3: jni::objects::JObject = __chain_wire3;
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
                jni::sys::jvalue {
                    l: __obj2.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj3.as_raw(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_boxedDurationEcho<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let value = match jlong_to_Box_Duration_0776c1ca(&mut env, &value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::boxed_duration_echo(value);
    match Box_Duration_to_jlong_0776c1ca(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_boxedElemIdSum<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    ps_handle: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let ps = unsafe {
        ::core::mem::take(&mut *(ps_handle as *mut Vec<perftest_flat::Payload>))
    }
        .into_iter()
        .map(|__e| ::std::boxed::Box::new(__e))
        .collect::<Vec<_>>();
    let __out = perftest_flat::boxed_elem_id_sum(ps);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_boxedLatest<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    a: jni::sys::jlong,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let a = match jlong_to_Archive_cd73502c(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/analytics/SummaryBuilder";
    const __CB_DESCR: &str = "(JD)Ljava/lang/Object;";
    let __out = *perftest_flat::boxed_latest(&a);
    match __out {
        ::core::option::Option::Some(__inner) => {
            let __obj0: jni::sys::jvalue = {
                let __enc0 = match i64_to_jlong_fbf9a9bc(
                    &mut env,
                    perftest_flat::summary_count(&__inner),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                jni::sys::jvalue { j: __enc0 }
            };
            let __obj1: jni::sys::jvalue = {
                let __enc1 = match f64_to_jdouble_9e4a8f70(
                    &mut env,
                    perftest_flat::summary_total(&__inner),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
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
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e2.to_string(),
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
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_boxedNoteEcho<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    note: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JString<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let note = match JString_to_Box_Option_String_caeff346(&mut env, &note) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::boxed_note_echo(note);
    match Box_Box_Option_String_to_JString_299999e0(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_boxedOptPayloadId<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    p_present: jni::sys::jboolean,
    p_id: jni::sys::jlong,
    p_seq: jni::sys::jint,
    p_value: jni::sys::jdouble,
    p_flag: jni::sys::jboolean,
    p_label: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let p = match tuple2_to_Box_Option_Payload_0aa97b23(
        &mut env,
        (p_present, (p_id, p_seq, p_value, p_flag, p_label)),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::boxed_opt_payload_id(p);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_boxedOptPriorityWeight<
    'a,
>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    p_present: jni::sys::jboolean,
    p_value: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let p = ::std::boxed::Box::new(
        if p_present != 0u8 {
            let __p_val = match jint_to_Priority_447102d2(&mut env, &p_value) {
                ::core::result::Result::Ok(__v) => __v,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return 0 as jni::sys::jlong;
                }
            };
            ::core::option::Option::Some(__p_val)
        } else {
            ::core::option::Option::None
        },
    );
    let __out = perftest_flat::boxed_opt_priority_weight(p);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_boxedPayloadId<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    p_id: jni::sys::jlong,
    p_seq: jni::sys::jint,
    p_value: jni::sys::jdouble,
    p_flag: jni::sys::jboolean,
    p_label: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let p = match tuple5_to_Box_Payload_697caed4(
        &mut env,
        (p_id, p_seq, p_value, p_flag, p_label),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::boxed_payload_id(p);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_boxedRunIdSum<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    ps_handle: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let ps = ::std::boxed::Box::new(unsafe {
        ::core::mem::take(&mut *(ps_handle as *mut Vec<perftest_flat::Payload>))
    });
    let __out = perftest_flat::boxed_run_id_sum(ps);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_cacheConfigWeight<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    cache_present: jni::sys::jboolean,
    cache_replies_priority: jni::sys::jint,
    cache_replies_max_samples: jni::sys::jlong,
    cache_ttl: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jint {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let cache = match tuple2_to_Option_CacheConfig_c580ddce(
        &mut env,
        (cache_present, ((cache_replies_priority, cache_replies_max_samples), cache_ttl)),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    let __out = perftest_flat::cache_config_weight(cache);
    match i32_to_jint_a3e3b6ef(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jint
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_callbackHolderOptionalEmit<
    'a,
>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    present: jni::sys::jboolean,
    f: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let present = match jboolean_to_bool_31306d98(&mut env, &present) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let f = match JObject_to_impl_Fn_Option_CallbackHolder_Send_Sync_static_b3ec3f73(
        &mut env,
        &f,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::callback_holder_optional_emit(present, f);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __c_s0 = match jint_to_i32_a3e3b6ef(&mut env, &c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    let c = match i32_to_Celsius_8c363100(&mut env, __c_s0) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    let __out = perftest_flat::celsius_double(c);
    let __out_s0 = match Celsius_to_i32_88c8e884(&mut env, __out) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    match i32_to_jint_a3e3b6ef(&mut env, __out_s0) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::cover_tag_runtime();
    match String_to_JString_c7f3ca43(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_dossierNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    note: jni::sys::jlong,
    tag: jni::sys::jlong,
    count: jni::sys::jlong,
    total: jni::sys::jdouble,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let note = match jlong_to_i64_fbf9a9bc(&mut env, &note) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let tag = match jlong_to_i64_fbf9a9bc(&mut env, &tag) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let total = match jdouble_to_f64_9e4a8f70(&mut env, &total) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::dossier_new(note, tag, count, total);
    match Dossier_to_JObject_eabbdbfa(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_durationBoundaryEcho<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value: jni::objects::JObject<'a>,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let value = match JObject_to_DurationBoundary_9c5bf9bc(&mut env, &value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/DurationBoundaryBuilderRaw";
    const __CB_DESCR: &str = "(JJ)Ljava/lang/Object;";
    let __out = perftest_flat::duration_boundary_echo(&value);
    let (__chain_wire0, __chain_wire1) = match DurationBoundary_to_tuple2_3834b601(
        &mut env,
        __out,
    ) {
        ::core::result::Result::Ok(__intermediate) => __intermediate,
        ::core::result::Result::Err(__chain_error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__chain_error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __obj0 = jni::sys::jvalue {
        j: __chain_wire0,
    };
    let __obj1 = jni::sys::jvalue {
        j: __chain_wire1,
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_durationEmit<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value: jni::sys::jlong,
    f: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __value_s0 = match jlong_to_u64_4384a5d6(&mut env, &value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let value = match u64_to_Duration_7c0845f9(&mut env, __value_s0) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let f = match JObject_to_impl_Fn_Duration_Send_Sync_static_98c9f460(&mut env, &f) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::duration_emit(value, f);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_durationOptional<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let value = match jlong_to_Option_Duration_1cfa4d44(&mut env, &value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::duration_optional(value);
    match Option_Duration_to_jlong_1cfa4d44(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_durationOutOfRange<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::duration_out_of_range();
    match Option_Duration_to_jlong_1cfa4d44(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_escapeProbeNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let value = match jlong_to_i64_fbf9a9bc(&mut env, &value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::escape_probe_new(value);
    match EscapeProbe_to_jlong_416aab42(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_escape_1probe_1value<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    p: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let p = match jlong_to_EscapeProbe_416aab42(&mut env, &p) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::escape_probe_value(&p);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_holdEcho<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    h: jni::objects::JObject<'a>,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let h = match JObject_to_Hold_5f85caaf(&mut env, &h) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/HoldBuilderRaw";
    const __CB_DESCR: &str = "(IJ)Ljava/lang/Object;";
    let __out = perftest_flat::hold_echo(h);
    let (__chain_wire0, (), (__chain_wire1,)) = match Hold_to_tuple3_bf18c116(
        &mut env,
        __out,
    ) {
        ::core::result::Result::Ok(__intermediate) => __intermediate,
        ::core::result::Result::Err(__chain_error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__chain_error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __obj0 = jni::sys::jvalue {
        i: __chain_wire0,
    };
    let __obj1 = jni::sys::jvalue {
        j: __chain_wire1,
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_holdPolicyEcho<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    p_hold: jni::objects::JObject<'a>,
    p_grace: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let p = match tuple2_to_HoldPolicy_c9df7bfe(&mut env, (p_hold, p_grace)) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::hold_policy_echo(p);
    match HoldPolicy_to_JObject_d2a5bcc4(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_holderTagOr<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    h_present: jni::sys::jboolean,
    h_tag: jni::sys::jlong,
    h_summary: jni::sys::jlong,
    fallback: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let h = match tuple2_to_Option_Holder_d4c4f3c3(
        &mut env,
        (h_present, (h_tag, h_summary)),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let fallback = match jlong_to_i64_fbf9a9bc(&mut env, &fallback) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::holder_tag_or(h, fallback);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_ingotGrams<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    i: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let i = match jlong_to_Ingot_020c3a86(&mut env, &i) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::ingot_grams(&i);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __l_s0 = match JString_to_String_c7f3ca43(&mut env, &l) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let l = match String_to_Label_c1a79668(&mut env, __l_s0) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::label_reverse(l);
    let __out_s0 = match Label_to_String_63dec766(&mut env, __out) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    match String_to_JString_c7f3ca43(&mut env, __out_s0) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_labelSeriesEcho<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    labels: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let labels = match JObject_to_Vec_Label_3fdf860d(&mut env, &labels) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::label_series_echo(labels);
    match Vec_Label_to_JObject_3fdf860d(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_layeredOf<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    which: jni::sys::jint,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let which = match jint_to_i32_a3e3b6ef(&mut env, &which) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/LayeredBuilderRaw";
    const __CB_DESCR: &str = "(ILjava/lang/Long;JLjava/util/List;Ljava/util/List;Ljava/util/List;[BJ)Ljava/lang/Object;";
    let __out = perftest_flat::layered_of(which);
    let (
        __chain_wire0,
        (__chain_wire1,),
        (__chain_wire2,),
        (__chain_wire3,),
        (__chain_wire4,),
        (__chain_wire5,),
        (__chain_wire6,),
        (__chain_wire7,),
    ) = match Layered_to_tuple8_4f5948ba(&mut env, __out) {
        ::core::result::Result::Ok(__intermediate) => __intermediate,
        ::core::result::Result::Err(__chain_error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__chain_error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __obj0 = jni::sys::jvalue {
        i: __chain_wire0,
    };
    let __obj1: jni::objects::JObject = __chain_wire1;
    let __obj2 = jni::sys::jvalue {
        j: __chain_wire2,
    };
    let __obj3: jni::objects::JObject = __chain_wire3;
    let __obj4: jni::objects::JObject = __chain_wire4;
    let __obj5: jni::objects::JObject = __chain_wire5;
    let __obj6: jni::objects::JObject = __chain_wire6.into();
    let __obj7 = jni::sys::jvalue {
        j: __chain_wire7,
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
                jni::sys::jvalue {
                    l: __obj1.as_raw(),
                },
                __obj2,
                jni::sys::jvalue {
                    l: __obj3.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj4.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj5.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj6.as_raw(),
                },
                __obj7,
            ],
        )
    {
        ::core::result::Result::Ok(__o) => __o,
        ::core::result::Result::Err(__e) => {
            let _ = env.exception_describe();
            let __e2 = <__JniErr as ::core::convert::From<
                String,
            >>::from(__e.to_string());
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_ledgerEach<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    n: jni::sys::jlong,
    sink: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let n = match jlong_to_i64_fbf9a9bc(&mut env, &n) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let sink = match JObject_to_impl_Fn_Ledger_Send_Sync_static_c76008cc(
        &mut env,
        &sink,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::ledger_each(n, sink);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_ledgerNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    n: jni::sys::jlong,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let n = match jlong_to_i64_fbf9a9bc(&mut env, &n) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/LedgerBuilderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Long;Ljava/lang/Double;Lio/prebindgen/covertest/model/Stamp;Ljava/lang/Long;Ljava/lang/Long;Ljava/lang/Integer;Ljava/lang/Long;Ljava/lang/String;Ljava/lang/String;Ljava/lang/Long;Ljava/lang/Double;Lio/prebindgen/covertest/model/Stamp;Ljava/lang/Long;Ljava/lang/Long;Ljava/lang/Integer;Ljava/lang/Long;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::ledger_new(n);
    let __vf0 = perftest_flat::ledger_filed(&__out)
        .map(|__hb0| perftest_flat::report_into_struct((__hb0).clone()));
    let __vf1 = perftest_flat::ledger_archived(&__out)
        .map(|__hb0| perftest_flat::report_into_struct(__hb0));
    let (
        __obj0,
        __obj1,
        __obj2,
        __obj3,
        __obj4,
        __obj5,
        __obj6,
        __obj7,
        __obj8,
    ): (
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
    ) = match __vf0 {
        ::core::option::Option::Some(__u0) => {
            let __obj5: jni::objects::JObject;
            let __obj6: jni::objects::JObject;
            let __obj7: jni::objects::JObject;
            match &__u0.outcome {
                perftest_flat::Lookup::Absent => {
                    __obj5 = match ::prebindgen_jni_runtime::box_jint(&mut env, 0) {
                        ::core::result::Result::Ok(__o) => __o,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj6 = jni::objects::JObject::null();
                    __obj7 = jni::objects::JObject::null();
                }
                perftest_flat::Lookup::Found(__sv0) => {
                    let __enc___obj6 = match Summary_to_jlong_3cb103b9(
                        &mut env,
                        __sv0.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj6 = match ::prebindgen_jni_runtime::box_jlong(
                        &mut env,
                        __enc___obj6,
                    ) {
                        ::core::result::Result::Ok(__o) => __o,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj5 = match ::prebindgen_jni_runtime::box_jint(&mut env, 1) {
                        ::core::result::Result::Ok(__o) => __o,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj7 = jni::objects::JObject::null();
                }
                perftest_flat::Lookup::Failed(__sv0) => {
                    let __enc___obj7 = match String_to_JString_c7f3ca43(
                        &mut env,
                        __sv0.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj7 = __enc___obj7.into();
                    __obj5 = match ::prebindgen_jni_runtime::box_jint(&mut env, 2) {
                        ::core::result::Result::Ok(__o) => __o,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj6 = jni::objects::JObject::null();
                }
            }
            let __obj0: jni::objects::JObject = {
                let __enc0 = match i64_to_jlong_fbf9a9bc(
                    &mut env,
                    perftest_flat::summary_count(&__u0.summary),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                match ::prebindgen_jni_runtime::box_jlong(&mut env, __enc0) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e,
                        );
                        return jni::objects::JObject::null().into();
                    }
                }
            };
            let __obj1: jni::objects::JObject = {
                let __enc1 = match f64_to_jdouble_9e4a8f70(
                    &mut env,
                    perftest_flat::summary_total(&__u0.summary),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                match ::prebindgen_jni_runtime::box_jdouble(&mut env, __enc1) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e,
                        );
                        return jni::objects::JObject::null().into();
                    }
                }
            };
            let __obj2: jni::objects::JObject = {
                let __enc2 = match Option_Stamp_to_JObject_6375b503(
                    &mut env,
                    __u0.taken,
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                __enc2
            };
            let __obj3: jni::objects::JObject = {
                let __enc3 = match i64_to_jlong_fbf9a9bc(&mut env, __u0.origin.secs) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                match ::prebindgen_jni_runtime::box_jlong(&mut env, __enc3) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e,
                        );
                        return jni::objects::JObject::null().into();
                    }
                }
            };
            let __obj4: jni::objects::JObject = {
                let __enc4 = match i64_to_jlong_fbf9a9bc(&mut env, __u0.origin.nanos) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                match ::prebindgen_jni_runtime::box_jlong(&mut env, __enc4) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e,
                        );
                        return jni::objects::JObject::null().into();
                    }
                }
            };
            let __obj8: jni::objects::JObject = {
                let __enc8 = match String_to_JString_c7f3ca43(&mut env, __u0.label) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                __enc8.into()
            };
            (__obj0, __obj1, __obj2, __obj3, __obj4, __obj5, __obj6, __obj7, __obj8)
        }
        ::core::option::Option::None => {
            (
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
            )
        }
    };
    let (
        __obj9,
        __obj10,
        __obj11,
        __obj12,
        __obj13,
        __obj14,
        __obj15,
        __obj16,
        __obj17,
    ): (
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
        jni::objects::JObject,
    ) = match __vf1 {
        ::core::option::Option::Some(__u1) => {
            let __obj14: jni::objects::JObject;
            let __obj15: jni::objects::JObject;
            let __obj16: jni::objects::JObject;
            match &__u1.outcome {
                perftest_flat::Lookup::Absent => {
                    __obj14 = match ::prebindgen_jni_runtime::box_jint(&mut env, 0) {
                        ::core::result::Result::Ok(__o) => __o,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj15 = jni::objects::JObject::null();
                    __obj16 = jni::objects::JObject::null();
                }
                perftest_flat::Lookup::Found(__sv0) => {
                    let __enc___obj15 = match Summary_to_jlong_3cb103b9(
                        &mut env,
                        __sv0.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj15 = match ::prebindgen_jni_runtime::box_jlong(
                        &mut env,
                        __enc___obj15,
                    ) {
                        ::core::result::Result::Ok(__o) => __o,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj14 = match ::prebindgen_jni_runtime::box_jint(&mut env, 1) {
                        ::core::result::Result::Ok(__o) => __o,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj16 = jni::objects::JObject::null();
                }
                perftest_flat::Lookup::Failed(__sv0) => {
                    let __enc___obj16 = match String_to_JString_c7f3ca43(
                        &mut env,
                        __sv0.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj16 = __enc___obj16.into();
                    __obj14 = match ::prebindgen_jni_runtime::box_jint(&mut env, 2) {
                        ::core::result::Result::Ok(__o) => __o,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e,
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    __obj15 = jni::objects::JObject::null();
                }
            }
            let __obj9: jni::objects::JObject = {
                let __enc9 = match i64_to_jlong_fbf9a9bc(
                    &mut env,
                    perftest_flat::summary_count(&__u1.summary),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                match ::prebindgen_jni_runtime::box_jlong(&mut env, __enc9) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e,
                        );
                        return jni::objects::JObject::null().into();
                    }
                }
            };
            let __obj10: jni::objects::JObject = {
                let __enc10 = match f64_to_jdouble_9e4a8f70(
                    &mut env,
                    perftest_flat::summary_total(&__u1.summary),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                match ::prebindgen_jni_runtime::box_jdouble(&mut env, __enc10) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e,
                        );
                        return jni::objects::JObject::null().into();
                    }
                }
            };
            let __obj11: jni::objects::JObject = {
                let __enc11 = match Option_Stamp_to_JObject_6375b503(
                    &mut env,
                    __u1.taken,
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                __enc11
            };
            let __obj12: jni::objects::JObject = {
                let __enc12 = match i64_to_jlong_fbf9a9bc(&mut env, __u1.origin.secs) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                match ::prebindgen_jni_runtime::box_jlong(&mut env, __enc12) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e,
                        );
                        return jni::objects::JObject::null().into();
                    }
                }
            };
            let __obj13: jni::objects::JObject = {
                let __enc13 = match i64_to_jlong_fbf9a9bc(&mut env, __u1.origin.nanos) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                match ::prebindgen_jni_runtime::box_jlong(&mut env, __enc13) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e,
                        );
                        return jni::objects::JObject::null().into();
                    }
                }
            };
            let __obj17: jni::objects::JObject = {
                let __enc17 = match String_to_JString_c7f3ca43(&mut env, __u1.label) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                __enc17.into()
            };
            (
                __obj9,
                __obj10,
                __obj11,
                __obj12,
                __obj13,
                __obj14,
                __obj15,
                __obj16,
                __obj17,
            )
        }
        ::core::option::Option::None => {
            (
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
                jni::objects::JObject::null(),
            )
        }
    };
    match __CB_MID
        .call_object(
            &mut env,
            __CB_FQN,
            "run",
            __CB_DESCR,
            &__builder,
            &[
                jni::sys::jvalue {
                    l: __obj0.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj1.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj2.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj3.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj4.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj5.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj6.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj7.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj8.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj9.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj10.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj11.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj12.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj13.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj14.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj15.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj16.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj17.as_raw(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_lookupEach<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    n: jni::sys::jlong,
    total: jni::sys::jdouble,
    sink: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let n = match jlong_to_i64_fbf9a9bc(&mut env, &n) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let total = match jdouble_to_f64_9e4a8f70(&mut env, &total) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let sink = match JObject_to_impl_Fn_Lookup_Send_Sync_static_4a65bc23(
        &mut env,
        &sink,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::lookup_each(n, total, sink);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_lookupOf<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    count: jni::sys::jlong,
    total: jni::sys::jdouble,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let total = match jdouble_to_f64_9e4a8f70(&mut env, &total) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/LookupBuilderRaw";
    const __CB_DESCR: &str = "(IJLjava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::lookup_of(count, total);
    let (__chain_wire0, (), (__chain_wire1,), (__chain_wire2,)) = match Lookup_to_tuple4_68d54df3(
        &mut env,
        __out,
    ) {
        ::core::result::Result::Ok(__intermediate) => __intermediate,
        ::core::result::Result::Err(__chain_error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__chain_error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __obj0 = jni::sys::jvalue {
        i: __chain_wire0,
    };
    let __obj1 = jni::sys::jvalue {
        j: __chain_wire1,
    };
    let __obj2: jni::objects::JObject = __chain_wire2.into();
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
                jni::sys::jvalue {
                    l: __obj2.as_raw(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_markerOf<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    which: jni::sys::jint,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let which = match jint_to_i32_a3e3b6ef(&mut env, &which) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/MarkerBuilder";
    const __CB_DESCR: &str = "(ILjava/lang/Integer;)Ljava/lang/Object;";
    let __out = perftest_flat::marker_of(which);
    let (__chain_wire0, (), (__chain_wire1,)) = match Marker_to_tuple3_8b7f3646(
        &mut env,
        __out,
    ) {
        ::core::result::Result::Ok(__intermediate) => __intermediate,
        ::core::result::Result::Err(__chain_error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__chain_error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __obj0 = jni::sys::jvalue {
        i: __chain_wire0,
    };
    let __obj1: jni::objects::JObject = __chain_wire1;
    match __CB_MID
        .call_object(
            &mut env,
            __CB_FQN,
            "run",
            __CB_DESCR,
            &__builder,
            &[
                __obj0,
                jni::sys::jvalue {
                    l: __obj1.as_raw(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_maybeHolderNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    tag: jni::sys::jlong,
    count: jni::sys::jlong,
    total: jni::sys::jdouble,
    present: jni::sys::jboolean,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let tag = match jlong_to_i64_fbf9a9bc(&mut env, &tag) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let total = match jdouble_to_f64_9e4a8f70(&mut env, &total) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let present = match jboolean_to_bool_31306d98(&mut env, &present) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::maybe_holder_new(tag, count, total, present);
    match MaybeHolder_to_JObject_1c68fbac(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __a_s0 = match jlong_to_i64_fbf9a9bc(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let a = match i64_to_Millis_bb88777a(&mut env, __a_s0) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __b_s0 = match jlong_to_i64_fbf9a9bc(&mut env, &b) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let b = match i64_to_Millis_bb88777a(&mut env, __b_s0) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::millis_add(a, b);
    let __out_s0 = match Millis_to_i64_61ecf054(&mut env, __out) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    match i64_to_jlong_fbf9a9bc(&mut env, __out_s0) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_objectBoundaryValue<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let value = match JObject_to_ObjectBoundary_dc5ac22b(&mut env, &value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::object_boundary_value(&value);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_observationNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    which: jni::sys::jint,
    with_fallback: jni::sys::jboolean,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let which = match jint_to_i32_a3e3b6ef(&mut env, &which) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let with_fallback = match jboolean_to_bool_31306d98(&mut env, &with_fallback) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::observation_new(which, with_fallback);
    match Observation_to_JObject_435b0724(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_observationWhich<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    o_id: jni::sys::jlong,
    o_reading__tag: jni::sys::jint,
    o_reading_exact_v0: jni::sys::jlong,
    o_reading_range_low: jni::sys::jlong,
    o_reading_range_high: jni::sys::jlong,
    o_reading_tagged_v0: jni::objects::JString<'a>,
    o_reading_tagged_v1: jni::sys::jint,
    o_reading_companion_v0: jni::sys::jlong,
    o_fallback_present: jni::sys::jboolean,
    o_fallback__tag: jni::sys::jint,
    o_fallback_exact_v0: jni::sys::jlong,
    o_fallback_range_low: jni::sys::jlong,
    o_fallback_range_high: jni::sys::jlong,
    o_fallback_tagged_v0: jni::objects::JString<'a>,
    o_fallback_tagged_v1: jni::sys::jint,
    o_fallback_companion_v0: jni::sys::jlong,
    o_note: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jint {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let o = match tuple4_to_Observation_438d4015(
        &mut env,
        (
            o_id,
            (
                o_reading__tag,
                (),
                (o_reading_exact_v0,),
                (o_reading_range_low, o_reading_range_high),
                (o_reading_tagged_v0, o_reading_tagged_v1),
                (o_reading_companion_v0,),
            ),
            (
                o_fallback_present,
                (
                    o_fallback__tag,
                    (),
                    (o_fallback_exact_v0,),
                    (o_fallback_range_low, o_fallback_range_high),
                    (o_fallback_tagged_v0, o_fallback_tagged_v1),
                    (o_fallback_companion_v0,),
                ),
            ),
            o_note,
        ),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    let __out = perftest_flat::observation_which(o);
    match i32_to_jint_a3e3b6ef(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jint
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let f = match JObject_to_impl_Fn_Payload_Send_Sync_static_96d50906(&mut env, &f) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::payload_handler_new(f);
    match PayloadHandler_to_jlong_d61fd890(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let p = match tuple5_to_Payload_2ea1d0c2(
        &mut env,
        (p_id, p_seq, p_value, p_flag, p_label),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::payload_label_len(&p);
    match Option_i64_to_JObject_2ba9a5ed(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_payloadOptionalEmit<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    present: jni::sys::jboolean,
    f: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let present = match jboolean_to_bool_31306d98(&mut env, &present) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let f = match JObject_to_impl_Fn_Option_Payload_Send_Sync_static_b308aaa4(
        &mut env,
        &f,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::payload_optional_emit(present, f);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            ()
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let p = match tuple5_to_Payload_2ea1d0c2(
        &mut env,
        (p_id, p_seq, p_value, p_flag, p_label),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    let __out = perftest_flat::payload_priority(&p);
    match Priority_to_jint_447102d2(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let f = match JObject_to_impl_Fn_Payload_Send_Sync_static_95073668(&mut env, &f) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::payload_vec_handler_new(f);
    match PayloadVecHandler_to_jlong_b32d2812(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_percentInvalidOutput<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::percent_invalid_output();
    match Option_Percent_to_JObject_544dd364(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_percentOptional<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    p: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let p = match JObject_to_Option_Percent_544dd364(&mut env, &p) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::percent_optional(p);
    match Option_Percent_to_JObject_544dd364(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __p_s0 = match jint_to_i32_a3e3b6ef(&mut env, &p) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    let p = match i32_to_Percent_db3641cc(&mut env, __p_s0) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    let factor = match jint_to_i32_a3e3b6ef(&mut env, &factor) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    let __out = perftest_flat::percent_scale(p, factor);
    let __out_s0 = match Percent_to_i32_01484801(&mut env, __out) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    match i32_to_jint_a3e3b6ef(&mut env, __out_s0) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jint
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_plainNoteEcho<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    note: jni::objects::JString<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JString<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let note = match JString_to_Option_String_56d5e304(&mut env, &note) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::plain_note_echo(note);
    match Option_String_to_JString_56d5e304(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let p = if p_present != 0u8 {
        let __p_val = match jint_to_Priority_447102d2(&mut env, &p_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    let __out = perftest_flat::priority_or(p, fallback);
    match Priority_to_jint_447102d2(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let p = match jint_to_Priority_447102d2(&mut env, &p) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    let __out = perftest_flat::priority_weight(p);
    match i32_to_jint_a3e3b6ef(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jint
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_probeEach<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    n: jni::sys::jlong,
    total: jni::sys::jdouble,
    sink: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let n = match jlong_to_i64_fbf9a9bc(&mut env, &n) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let total = match jdouble_to_f64_9e4a8f70(&mut env, &total) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let sink = match JObject_to_impl_Fn_Probe_Send_Sync_static_b0418db6(
        &mut env,
        &sink,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::probe_each(n, total, sink);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_probeNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    seq: jni::sys::jlong,
    count: jni::sys::jlong,
    total: jni::sys::jdouble,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let seq = match jlong_to_i64_fbf9a9bc(&mut env, &seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let total = match jdouble_to_f64_9e4a8f70(&mut env, &total) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/ProbeBuilderRaw";
    const __CB_DESCR: &str = "(JLjava/lang/Integer;Ljava/lang/Long;Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::probe_new(seq, count, total);
    let __vf0 = perftest_flat::probe_to_struct(&__out);
    let (
        __obj1,
        __obj2,
        __obj3,
    ): (jni::objects::JObject, jni::objects::JObject, jni::objects::JObject) = {
        let __so1: &::core::option::Option<_> = &(&__vf0).outcome;
        match __so1 {
            ::core::option::Option::Some(__sg1) => {
                let __obj1: jni::objects::JObject;
                let __obj2: jni::objects::JObject;
                let __obj3: jni::objects::JObject;
                match __sg1 {
                    perftest_flat::Lookup::Absent => {
                        __obj1 = match ::prebindgen_jni_runtime::box_jint(&mut env, 0) {
                            ::core::result::Result::Ok(__o) => __o,
                            ::core::result::Result::Err(__e) => {
                                signal_binding_error(
                                    &mut env,
                                    &__error_sink,
                                    &__SINK_MID,
                                    __SINK_FQN,
                                    __SINK_DESCR,
                                    &__e,
                                );
                                return jni::objects::JObject::null().into();
                            }
                        };
                        __obj2 = jni::objects::JObject::null();
                        __obj3 = jni::objects::JObject::null();
                    }
                    perftest_flat::Lookup::Found(__sv0) => {
                        let __enc___obj2 = match Summary_to_jlong_3cb103b9(
                            &mut env,
                            __sv0.clone(),
                        ) {
                            ::core::result::Result::Ok(__w) => __w,
                            ::core::result::Result::Err(__e) => {
                                signal_binding_error(
                                    &mut env,
                                    &__error_sink,
                                    &__SINK_MID,
                                    __SINK_FQN,
                                    __SINK_DESCR,
                                    &__e.to_string(),
                                );
                                return jni::objects::JObject::null().into();
                            }
                        };
                        __obj2 = match ::prebindgen_jni_runtime::box_jlong(
                            &mut env,
                            __enc___obj2,
                        ) {
                            ::core::result::Result::Ok(__o) => __o,
                            ::core::result::Result::Err(__e) => {
                                signal_binding_error(
                                    &mut env,
                                    &__error_sink,
                                    &__SINK_MID,
                                    __SINK_FQN,
                                    __SINK_DESCR,
                                    &__e,
                                );
                                return jni::objects::JObject::null().into();
                            }
                        };
                        __obj1 = match ::prebindgen_jni_runtime::box_jint(&mut env, 1) {
                            ::core::result::Result::Ok(__o) => __o,
                            ::core::result::Result::Err(__e) => {
                                signal_binding_error(
                                    &mut env,
                                    &__error_sink,
                                    &__SINK_MID,
                                    __SINK_FQN,
                                    __SINK_DESCR,
                                    &__e,
                                );
                                return jni::objects::JObject::null().into();
                            }
                        };
                        __obj3 = jni::objects::JObject::null();
                    }
                    perftest_flat::Lookup::Failed(__sv0) => {
                        let __enc___obj3 = match String_to_JString_c7f3ca43(
                            &mut env,
                            __sv0.clone(),
                        ) {
                            ::core::result::Result::Ok(__w) => __w,
                            ::core::result::Result::Err(__e) => {
                                signal_binding_error(
                                    &mut env,
                                    &__error_sink,
                                    &__SINK_MID,
                                    __SINK_FQN,
                                    __SINK_DESCR,
                                    &__e.to_string(),
                                );
                                return jni::objects::JObject::null().into();
                            }
                        };
                        __obj3 = __enc___obj3.into();
                        __obj1 = match ::prebindgen_jni_runtime::box_jint(&mut env, 2) {
                            ::core::result::Result::Ok(__o) => __o,
                            ::core::result::Result::Err(__e) => {
                                signal_binding_error(
                                    &mut env,
                                    &__error_sink,
                                    &__SINK_MID,
                                    __SINK_FQN,
                                    __SINK_DESCR,
                                    &__e,
                                );
                                return jni::objects::JObject::null().into();
                            }
                        };
                        __obj2 = jni::objects::JObject::null();
                    }
                }
                (__obj1, __obj2, __obj3)
            }
            ::core::option::Option::None => {
                (
                    jni::objects::JObject::null(),
                    jni::objects::JObject::null(),
                    jni::objects::JObject::null(),
                )
            }
        }
    };
    let __obj0: jni::sys::jvalue = {
        let __enc0 = match i64_to_jlong_fbf9a9bc(&mut env, __vf0.seq.clone()) {
            ::core::result::Result::Ok(__w) => __w,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        jni::sys::jvalue { j: __enc0 }
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
                jni::sys::jvalue {
                    l: __obj1.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj2.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj3.as_raw(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_readingEach<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    n: jni::sys::jint,
    sink: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let n = match jint_to_i32_a3e3b6ef(&mut env, &n) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let sink = match JObject_to_impl_Fn_Reading_Send_Sync_static_5964f1fc(
        &mut env,
        &sink,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::reading_each(n, sink);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_readingMaybe<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    which: jni::sys::jint,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let which = match jint_to_i32_a3e3b6ef(&mut env, &which) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/ReadingBuilder";
    const __CB_DESCR: &str = "(IJJJLjava/lang/String;IJ)Ljava/lang/Object;";
    let __out = perftest_flat::reading_maybe(which);
    match __out {
        ::core::option::Option::Some(__inner) => {
            let (
                __chain_wire0,
                (),
                (__chain_wire1,),
                (__chain_wire2, __chain_wire3),
                (__chain_wire4, __chain_wire5),
                (__chain_wire6,),
            ) = match Reading_to_tuple6_69702d1f(&mut env, __inner) {
                ::core::result::Result::Ok(__intermediate) => __intermediate,
                ::core::result::Result::Err(__chain_error) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__chain_error.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
            };
            let __obj0 = jni::sys::jvalue {
                i: __chain_wire0,
            };
            let __obj1 = jni::sys::jvalue {
                j: __chain_wire1,
            };
            let __obj2 = jni::sys::jvalue {
                j: __chain_wire2,
            };
            let __obj3 = jni::sys::jvalue {
                j: __chain_wire3,
            };
            let __obj4: jni::objects::JObject = __chain_wire4.into();
            let __obj5 = jni::sys::jvalue {
                i: __chain_wire5,
            };
            let __obj6 = jni::sys::jvalue {
                j: __chain_wire6,
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
                        __obj5,
                        __obj6,
                    ],
                )
            {
                ::core::result::Result::Ok(__o) => __o,
                ::core::result::Result::Err(__e) => {
                    let _ = env.exception_describe();
                    let __e2 = <__JniErr as ::core::convert::From<
                        String,
                    >>::from(__e.to_string());
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e2.to_string(),
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
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_readingOf<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    which: jni::sys::jint,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let which = match jint_to_i32_a3e3b6ef(&mut env, &which) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/ReadingBuilder";
    const __CB_DESCR: &str = "(IJJJLjava/lang/String;IJ)Ljava/lang/Object;";
    let __out = perftest_flat::reading_of(which);
    let (
        __chain_wire0,
        (),
        (__chain_wire1,),
        (__chain_wire2, __chain_wire3),
        (__chain_wire4, __chain_wire5),
        (__chain_wire6,),
    ) = match Reading_to_tuple6_69702d1f(&mut env, __out) {
        ::core::result::Result::Ok(__intermediate) => __intermediate,
        ::core::result::Result::Err(__chain_error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__chain_error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __obj0 = jni::sys::jvalue {
        i: __chain_wire0,
    };
    let __obj1 = jni::sys::jvalue {
        j: __chain_wire1,
    };
    let __obj2 = jni::sys::jvalue {
        j: __chain_wire2,
    };
    let __obj3 = jni::sys::jvalue {
        j: __chain_wire3,
    };
    let __obj4: jni::objects::JObject = __chain_wire4.into();
    let __obj5 = jni::sys::jvalue {
        i: __chain_wire5,
    };
    let __obj6 = jni::sys::jvalue {
        j: __chain_wire6,
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
                __obj5,
                __obj6,
            ],
        )
    {
        ::core::result::Result::Ok(__o) => __o,
        ::core::result::Result::Err(__e) => {
            let _ = env.exception_describe();
            let __e2 = <__JniErr as ::core::convert::From<
                String,
            >>::from(__e.to_string());
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_readingSeries<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    n: jni::sys::jint,
    __acc: jni::objects::JObject<'a>,
    __fold: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let n = match jint_to_i32_a3e3b6ef(&mut env, &n) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/ReadingFolderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Object;IJJJLjava/lang/String;IJ)Ljava/lang/Object;";
    let __vec = perftest_flat::reading_series(n);
    let mut __acc = __acc;
    for __elem in __vec.into_iter() {
        let (
            __chain_wire0,
            (),
            (__chain_wire1,),
            (__chain_wire2, __chain_wire3),
            (__chain_wire4, __chain_wire5),
            (__chain_wire6,),
        ) = match Reading_to_tuple6_69702d1f(&mut env, __elem) {
            ::core::result::Result::Ok(__intermediate) => __intermediate,
            ::core::result::Result::Err(__chain_error) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__chain_error.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __obj0 = jni::sys::jvalue {
            i: __chain_wire0,
        };
        let __obj1 = jni::sys::jvalue {
            j: __chain_wire1,
        };
        let __obj2 = jni::sys::jvalue {
            j: __chain_wire2,
        };
        let __obj3 = jni::sys::jvalue {
            j: __chain_wire3,
        };
        let __obj4: jni::objects::JObject = __chain_wire4.into();
        let __obj5 = jni::sys::jvalue {
            i: __chain_wire5,
        };
        let __obj6 = jni::sys::jvalue {
            j: __chain_wire6,
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
                    __obj5,
                    __obj6,
                ],
            )
        {
            ::core::result::Result::Ok(__o) => __o,
            ::core::result::Result::Err(__e) => {
                let _ = env.exception_describe();
                let __e2 = <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string());
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e2.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
    }
    __acc
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_refVecIdSum<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    ps_handle: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let ps = unsafe { &*(ps_handle as *const Vec<perftest_flat::Payload>) };
    let __out = perftest_flat::ref_vec_id_sum(ps);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_reportEach<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    n: jni::sys::jlong,
    sink: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let n = match jlong_to_i64_fbf9a9bc(&mut env, &n) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let sink = match JObject_to_impl_Fn_Report_Send_Sync_static_eb5ca515(
        &mut env,
        &sink,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::report_each(n, sink);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_sliceIdSum<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    ps_handle: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let ps = unsafe { &*(ps_handle as *const Vec<perftest_flat::Payload>) };
    let __out = perftest_flat::slice_id_sum(ps);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_spanHolderNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    seq: jni::sys::jlong,
    required_ms: jni::sys::jlong,
    delay_ms: jni::sys::jlong,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let seq = match jlong_to_i64_fbf9a9bc(&mut env, &seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let required_ms = match jlong_to_u64_4384a5d6(&mut env, &required_ms) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let delay_ms = match jlong_to_i64_fbf9a9bc(&mut env, &delay_ms) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/SpanHolderBuilderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Long;Ljava/lang/Long;)Ljava/lang/Object;";
    let __out = perftest_flat::span_holder_new(seq, required_ms, delay_ms);
    let __vf0 = perftest_flat::span_holder_span(&__out)
        .map(|__hb0| perftest_flat::span_to_struct(__hb0));
    let (__obj0, __obj1): (jni::objects::JObject, jni::objects::JObject) = match __vf0 {
        ::core::option::Option::Some(__u0) => {
            let __obj0: jni::objects::JObject = {
                let __enc0 = {
                    let __cs0_0 = match Duration_to_u64_e3980876(
                        &mut env,
                        __u0.required.clone(),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    match u64_to_jlong_4384a5d6(&mut env, __cs0_0) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    }
                };
                match ::prebindgen_jni_runtime::box_jlong(&mut env, __enc0) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e,
                        );
                        return jni::objects::JObject::null().into();
                    }
                }
            };
            let __obj1: jni::objects::JObject = {
                let __enc1 = match Option_Duration_to_jlong_1cfa4d44(
                    &mut env,
                    __u0.delay.clone(),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                match ::prebindgen_jni_runtime::box_jlong(&mut env, __enc1) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e,
                        );
                        return jni::objects::JObject::null().into();
                    }
                }
            };
            (__obj0, __obj1)
        }
        ::core::option::Option::None => {
            (jni::objects::JObject::null(), jni::objects::JObject::null())
        }
    };
    match __CB_MID
        .call_object(
            &mut env,
            __CB_FQN,
            "run",
            __CB_DESCR,
            &__builder,
            &[
                jni::sys::jvalue {
                    l: __obj0.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj1.as_raw(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_stampNanos<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s_secs: jni::sys::jlong,
    s_nanos: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match tuple2_to_Stamp_43c8d6ce(&mut env, (s_secs, s_nanos)) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::stamp_nanos(&s);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let secs = match jlong_to_i64_fbf9a9bc(&mut env, &secs) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let nanos = match jlong_to_i64_fbf9a9bc(&mut env, &nanos) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/StampBuilder";
    const __CB_DESCR: &str = "(JJ)Ljava/lang/Object;";
    let __out = perftest_flat::stamp_new(secs, nanos);
    let (__chain_wire0, __chain_wire1) = match Stamp_to_tuple2_8d33d015(
        &mut env,
        __out,
    ) {
        ::core::result::Result::Ok(__intermediate) => __intermediate,
        ::core::result::Result::Err(__chain_error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__chain_error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __obj0 = jni::sys::jvalue {
        j: __chain_wire0,
    };
    let __obj1 = jni::sys::jvalue {
        j: __chain_wire1,
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
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
    s_secs: jni::sys::jlong,
    s_nanos: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match tuple2_to_Stamp_43c8d6ce(&mut env, (s_secs, s_nanos)) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::stamp_secs(&s);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/StampFolderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Object;JJ)Ljava/lang/Object;";
    let __vec = perftest_flat::stamp_series(count);
    let mut __acc = __acc;
    for __elem in __vec.into_iter() {
        let (__chain_wire0, __chain_wire1) = match Stamp_to_tuple2_8d33d015(
            &mut env,
            __elem,
        ) {
            ::core::result::Result::Ok(__intermediate) => __intermediate,
            ::core::result::Result::Err(__chain_error) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__chain_error.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        let __obj0 = jni::sys::jvalue {
            j: __chain_wire0,
        };
        let __obj1 = jni::sys::jvalue {
            j: __chain_wire1,
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
                ],
            )
        {
            ::core::result::Result::Ok(__o) => __o,
            ::core::result::Result::Err(__e) => {
                let _ = env.exception_describe();
                let __e2 = <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string());
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e2.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let handler = match jlong_to_PayloadHandler_d61fd890(&mut env, &handler) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::storage_callback(&s, &handler);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let handler = match jlong_to_PayloadVecHandler_b32d2812(&mut env, &handler) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::storage_callback_vec(&s, &handler);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let id = match jlong_to_i64_fbf9a9bc(&mut env, &id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __out = perftest_flat::storage_contains(&s, id);
    match bool_to_jboolean_31306d98(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let n = match jlong_to_i64_fbf9a9bc(&mut env, &n) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let h = match jlong_to_StorageHandler_3b4d3ed3(&mut env, &h) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::storage_emit(n, &h);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let e = match jlong_to_StorageError_26b2d298(&mut env, &e) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::storage_error_message(&e);
    match String_to_JString_c7f3ca43(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __exp_expected_sel = match jint_to_i32_a3e3b6ef(&mut env, &expected_sel) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __exp_expected_0_0: Option<i64> = if expected_0_0_present != 0u8 {
        let __v = match jlong_to_i64_fbf9a9bc(&mut env, &expected_0_0_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__je.to_string(),
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __out = perftest_flat::storage_expect_summary(&mut s, __folded_expected);
    match bool_to_jboolean_31306d98(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/PayloadBuilder";
    const __CB_DESCR: &str = "(JIDZLjava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::storage_get(&s);
    let (
        __chain_present,
        (__chain_wire0, __chain_wire1, __chain_wire2, __chain_wire3, __chain_wire4),
    ) = match Option_Payload_to_tuple2_af2bd54b(&mut env, __out) {
        ::core::result::Result::Ok(__intermediate) => __intermediate,
        ::core::result::Result::Err(__chain_error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__chain_error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __obj0 = jni::sys::jvalue {
        j: __chain_wire0,
    };
    let __obj1 = jni::sys::jvalue {
        i: __chain_wire1,
    };
    let __obj2 = jni::sys::jvalue {
        d: __chain_wire2,
    };
    let __obj3 = jni::sys::jvalue {
        z: __chain_wire3,
    };
    let __obj4: jni::objects::JObject = __chain_wire4.into();
    if __chain_present != 0 {
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e2.to_string(),
                );
                jni::objects::JObject::null().into()
            }
        }
    } else {
        jni::objects::JObject::null().into()
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/PayloadFolderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Object;JIDZLjava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::storage_get_vec(&s);
    match __out {
        ::core::option::Option::Some(__vec) => {
            let mut __acc = __acc;
            for __elem in __vec.into_iter() {
                let (
                    __chain_wire0,
                    __chain_wire1,
                    __chain_wire2,
                    __chain_wire3,
                    __chain_wire4,
                ) = match Payload_to_tuple5_bbb055bc(&mut env, __elem) {
                    ::core::result::Result::Ok(__intermediate) => __intermediate,
                    ::core::result::Result::Err(__chain_error) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__chain_error.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                let __obj0 = jni::sys::jvalue {
                    j: __chain_wire0,
                };
                let __obj1 = jni::sys::jvalue {
                    i: __chain_wire1,
                };
                let __obj2 = jni::sys::jvalue {
                    d: __chain_wire2,
                };
                let __obj3 = jni::sys::jvalue {
                    z: __chain_wire3,
                };
                let __obj4: jni::objects::JObject = __chain_wire4.into();
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
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e2.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let f = match JObject_to_impl_Fn_Storage_Send_Sync_static_2f26edcf(&mut env, &f) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::storage_handler_new(f);
    match StorageHandler_to_jlong_3b4d3ed3(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/StringFolder";
    const __CB_DESCR: &str = "(Ljava/lang/Object;Ljava/lang/String;)Ljava/lang/Object;";
    let __vec = perftest_flat::storage_labels(&s);
    let mut __acc = __acc;
    for __elem in __vec.into_iter() {
        let __enc = {
            match String_to_JString_c7f3ca43(&mut env, __elem) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e2.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::storage_len(&s);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __exp_expected_sel = match jint_to_i32_a3e3b6ef(&mut env, &expected_sel) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __exp_expected_0_0: Option<i64> = if expected_0_0_present != 0u8 {
        let __v = match jlong_to_i64_fbf9a9bc(&mut env, &expected_0_0_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__je.to_string(),
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __out = perftest_flat::storage_matches_summary(&s, __folded_expected);
    match bool_to_jboolean_31306d98(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::storage_new();
    match Storage_to_jlong_1b233abd(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let payload = match tuple5_to_Payload_2ea1d0c2(
        &mut env,
        (payload_id, payload_seq, payload_value, payload_flag, payload_label),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::storage_put_by_read(&mut s, &payload);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let payload = match tuple5_to_Payload_bbb055bc(
        &mut env,
        (payload_id, payload_seq, payload_value, payload_flag, payload_label),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::storage_put_by_take(&mut s, payload);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let p = match tuple2_to_Option_Payload_af2bd54b(
        &mut env,
        (p_present, (p_id, p_seq, p_value, p_flag, p_label)),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return 0 as jni::sys::jboolean;
        }
    };
    let __out = perftest_flat::storage_put_opt(&mut s, p);
    match bool_to_jboolean_31306d98(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let payloads = unsafe { &*(payloads_handle as *const Vec<perftest_flat::Payload>) };
    let __out = perftest_flat::storage_put_slice(&mut s, payloads);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let each = match jlong_to_i64_fbf9a9bc(&mut env, &each) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/StorageFolderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Object;J)Ljava/lang/Object;";
    let __vec = perftest_flat::storage_shards(count, each);
    let mut __acc = __acc;
    for __elem in __vec.into_iter() {
        let __enc = {
            match Storage_to_jlong_1b233abd(&mut env, __elem) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e2.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let each = match jlong_to_i64_fbf9a9bc(&mut env, &each) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/StorageFolderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Object;J)Ljava/lang/Object;";
    let __out = perftest_flat::storage_shards_opt(count, each);
    match __out {
        ::core::option::Option::Some(__vec) => {
            let mut __acc = __acc;
            for __elem in __vec.into_iter() {
                let __enc = {
                    match Storage_to_jlong_1b233abd(&mut env, __elem) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
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
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e2.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::storage_summary_handle(&s);
    match Summary_to_jlong_3cb103b9(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageSummaryProbe<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Storage_1b233abd(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/analytics/SummaryStorageSummaryProbeBuilderRaw";
    const __CB_DESCR: &str = "(JDLjava/lang/Long;)Ljava/lang/Object;";
    let __out = perftest_flat::storage_summary_probe(&s);
    let __obj0: jni::sys::jvalue = {
        let __enc0 = match i64_to_jlong_fbf9a9bc(
            &mut env,
            perftest_flat::summary_count(&__out),
        ) {
            ::core::result::Result::Ok(__w) => __w,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        jni::sys::jvalue { d: __enc1 }
    };
    let __obj2: jni::objects::JObject = match crate::summary_if_nonempty(&__out) {
        ::core::option::Option::Some(__n0) => {
            let __h2: jni::sys::jlong = match Summary_to_jlong_ccacdeac(&mut env, __n0) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
            };
            match ::prebindgen_jni_runtime::box_jlong(&mut env, __h2) {
                ::core::result::Result::Ok(__o) => __o,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
            }
        }
        ::core::option::Option::None => jni::objects::JObject::null(),
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
                jni::sys::jvalue {
                    l: __obj2.as_raw(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let a = match jlong_to_Storage_1b233abd(&mut env, &a) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let b = match jlong_to_Storage_1b233abd(&mut env, &b) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let c = match jlong_to_Storage_1b233abd(&mut env, &c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::storage_total_len(&a, &b, &c);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_storageTryFromStamp<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s_secs: jni::sys::jlong,
    s_nanos: jni::sys::jlong,
    tag: jni::objects::JByteArray<'a>,
    __error_sink: jni::objects::JObject<'a>,
    __domain_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    #[allow(non_upper_case_globals)]
    static __DSINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __DSINK_FQN: &str = "io/prebindgen/covertest/errors/StorageErrorHandlerRaw";
    const __DSINK_DESCR: &str = "(Ljava/lang/String;J)Ljava/lang/Object;";
    let s = match tuple2_to_Stamp_8d33d015(&mut env, (s_secs, s_nanos)) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let tag = match JByteArray_to_u8_2_9ca14e44(&mut env, &tag) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = match perftest_flat::storage_try_from_stamp(s, tag) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__de) => {
            let __eze0: jni::objects::JObject = {
                let __enc0 = match String_to_JString_c7f3ca43(
                    &mut env,
                    perftest_flat::storage_error_message(&__de),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
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
            signal_domain_error(
                &mut env,
                &__domain_sink,
                &__DSINK_MID,
                __DSINK_FQN,
                __DSINK_DESCR,
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    __domain_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    #[allow(non_upper_case_globals)]
    static __DSINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __DSINK_FQN: &str = "io/prebindgen/covertest/errors/StorageErrorHandlerRaw";
    const __DSINK_DESCR: &str = "(Ljava/lang/String;J)Ljava/lang/Object;";
    let label = match JString_to_String_c7f3ca43(&mut env, &label) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
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
            signal_domain_error(
                &mut env,
                &__domain_sink,
                &__DSINK_MID,
                __DSINK_FQN,
                __DSINK_DESCR,
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let payload = match tuple5_to_Payload_bbb055bc(
        &mut env,
        (payload_id, payload_seq, payload_value, payload_flag, payload_label),
    ) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::storage_with_payload(payload);
    match Storage_to_jlong_1b233abd(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match JString_to_String_c7f3ca43(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::string_new(&s);
    match String_to_JString_c7f3ca43(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Summary_3cb103b9(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::summary_count(&s);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summaryDescribe<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s_sel: jni::sys::jint,
    s_0_0_present: jni::sys::jboolean,
    s_0_0_value: jni::sys::jlong,
    s_0_1_present: jni::sys::jboolean,
    s_0_1_value: jni::sys::jdouble,
    s_1: jni::sys::jlong,
    verbose: jni::sys::jboolean,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JString<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __exp_s_sel = match jint_to_i32_a3e3b6ef(&mut env, &s_sel) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __exp_s_0_0: Option<i64> = if s_0_0_present != 0u8 {
        let __v = match jlong_to_i64_fbf9a9bc(&mut env, &s_0_0_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_s_1 = match jlong_to_Option_Summary_828826f3(&mut env, &s_1) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
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
                    ::core::option::Option::Some(__v) => {
                        ::core::result::Result::Ok(::core::clone::Clone::clone(&*__v))
                    }
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__je.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let verbose = match jboolean_to_bool_31306d98(&mut env, &verbose) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = crate::summary_describe(&__folded_s, verbose);
    match String_to_JString_c7f3ca43(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summaryFromMean<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    count: jni::sys::jlong,
    mean: jni::sys::jdouble,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let mean = match jdouble_to_f64_9e4a8f70(&mut env, &mean) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = crate::summary_from_mean(count, mean);
    let __out_s0 = match Result_Summary_String_to_Summary_dfdf7f9e(&mut env, __out) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    match Summary_to_jlong_3cb103b9(&mut env, __out_s0) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summaryMean<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jdouble {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Summary_3cb103b9(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __out = crate::summary_mean(&s);
    match f64_to_jdouble_9e4a8f70(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0.0 as jni::sys::jdouble
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summaryMerge<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    primary_sel: jni::sys::jint,
    primary_0_0_present: jni::sys::jboolean,
    primary_0_0_value: jni::sys::jlong,
    primary_0_1_present: jni::sys::jboolean,
    primary_0_1_value: jni::sys::jdouble,
    primary_1: jni::sys::jlong,
    fallback_sel: jni::sys::jint,
    fallback_0_0_present: jni::sys::jboolean,
    fallback_0_0_value: jni::sys::jlong,
    fallback_0_1_present: jni::sys::jboolean,
    fallback_0_1_value: jni::sys::jdouble,
    fallback_1: jni::sys::jlong,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __exp_primary_sel = match jint_to_i32_a3e3b6ef(&mut env, &primary_sel) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __exp_primary_0_0: Option<i64> = if primary_0_0_present != 0u8 {
        let __v = match jlong_to_i64_fbf9a9bc(&mut env, &primary_0_0_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_primary_0_1: Option<f64> = if primary_0_1_present != 0u8 {
        let __v = match jdouble_to_f64_9e4a8f70(&mut env, &primary_0_1_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_primary_1 = match jlong_to_Option_Summary_252ef2ba(&mut env, &primary_1) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __folded_primary = match {
        match __exp_primary_sel {
            0i32 => {
                match (__exp_primary_0_0, __exp_primary_0_1) {
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
                match __exp_primary_1 {
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__je.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __exp_fallback_sel = match jint_to_i32_a3e3b6ef(&mut env, &fallback_sel) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __exp_fallback_0_0: Option<i64> = if fallback_0_0_present != 0u8 {
        let __v = match jlong_to_i64_fbf9a9bc(&mut env, &fallback_0_0_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_fallback_0_1: Option<f64> = if fallback_0_1_present != 0u8 {
        let __v = match jdouble_to_f64_9e4a8f70(&mut env, &fallback_0_1_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_fallback_1 = match jlong_to_Option_Summary_252ef2ba(
        &mut env,
        &fallback_1,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __folded_fallback = match {
        match __exp_fallback_sel {
            0i32 => {
                match (__exp_fallback_0_0, __exp_fallback_0_1) {
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
                match __exp_fallback_1 {
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__je.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/analytics/SummaryBuilder";
    const __CB_DESCR: &str = "(JD)Ljava/lang/Object;";
    let __out = perftest_flat::summary_merge(__folded_primary, __folded_fallback);
    let __obj0: jni::sys::jvalue = {
        let __enc0 = match i64_to_jlong_fbf9a9bc(
            &mut env,
            perftest_flat::summary_count(&__out),
        ) {
            ::core::result::Result::Ok(__w) => __w,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let total = match jdouble_to_f64_9e4a8f70(&mut env, &total) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::summary_new(count, total);
    match Summary_to_jlong_3cb103b9(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summaryPrefer<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    primary_sel: jni::sys::jint,
    primary_0_0_present: jni::sys::jboolean,
    primary_0_0_value: jni::sys::jlong,
    primary_0_1_present: jni::sys::jboolean,
    primary_0_1_value: jni::sys::jdouble,
    primary_1: jni::sys::jlong,
    fallback_sel: jni::sys::jint,
    fallback_0_0_present: jni::sys::jboolean,
    fallback_0_0_value: jni::sys::jlong,
    fallback_0_1_present: jni::sys::jboolean,
    fallback_0_1_value: jni::sys::jdouble,
    fallback_1: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __exp_primary_sel = match jint_to_i32_a3e3b6ef(&mut env, &primary_sel) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __exp_primary_0_0: Option<i64> = if primary_0_0_present != 0u8 {
        let __v = match jlong_to_i64_fbf9a9bc(&mut env, &primary_0_0_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0 as jni::sys::jlong;
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_primary_0_1: Option<f64> = if primary_0_1_present != 0u8 {
        let __v = match jdouble_to_f64_9e4a8f70(&mut env, &primary_0_1_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0 as jni::sys::jlong;
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_primary_1 = match jlong_to_Option_Summary_252ef2ba(&mut env, &primary_1) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __folded_primary = match {
        match __exp_primary_sel {
            0i32 => {
                match (__exp_primary_0_0, __exp_primary_0_1) {
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
                match __exp_primary_1 {
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__je.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __exp_fallback_sel = match jint_to_i32_a3e3b6ef(&mut env, &fallback_sel) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __exp_fallback_0_0: Option<i64> = if fallback_0_0_present != 0u8 {
        let __v = match jlong_to_i64_fbf9a9bc(&mut env, &fallback_0_0_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0 as jni::sys::jlong;
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_fallback_0_1: Option<f64> = if fallback_0_1_present != 0u8 {
        let __v = match jdouble_to_f64_9e4a8f70(&mut env, &fallback_0_1_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0 as jni::sys::jlong;
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_fallback_1 = match jlong_to_Option_Summary_252ef2ba(
        &mut env,
        &fallback_1,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __folded_fallback = match {
        match __exp_fallback_sel {
            0i32 => {
                match (__exp_fallback_0_0, __exp_fallback_0_1) {
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
                match __exp_fallback_1 {
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__je.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __out = cov_helpers::summary_prefer(__folded_primary, __folded_fallback);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Summary_3cb103b9(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let factor = match jdouble_to_f64_9e4a8f70(&mut env, &factor) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __out = perftest_flat::summary_scaled(&s, factor);
    match f64_to_jdouble_9e4a8f70(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0.0 as jni::sys::jdouble
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summarySeries<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    count: jni::sys::jlong,
    start: jni::sys::jlong,
    __acc: jni::objects::JObject<'a>,
    __fold: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let start = match jlong_to_i64_fbf9a9bc(&mut env, &start) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/analytics/SummaryFolder";
    const __CB_DESCR: &str = "(Ljava/lang/Object;JD)Ljava/lang/Object;";
    let __vec = perftest_flat::summary_series(count, start);
    let mut __acc = __acc;
    for __elem in __vec.into_iter() {
        let __obj0: jni::sys::jvalue = {
            let __enc0 = match i64_to_jlong_fbf9a9bc(
                &mut env,
                perftest_flat::summary_count(&__elem),
            ) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
            };
            jni::sys::jvalue { j: __enc0 }
        };
        let __obj1: jni::sys::jvalue = {
            let __enc1 = match f64_to_jdouble_9e4a8f70(
                &mut env,
                perftest_flat::summary_total(&__elem),
            ) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
            };
            jni::sys::jvalue { d: __enc1 }
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
                ],
            )
        {
            ::core::result::Result::Ok(__o) => __o,
            ::core::result::Result::Err(__e) => {
                let _ = env.exception_describe();
                let __e2 = <__JniErr as ::core::convert::From<
                    String,
                >>::from(__e.to_string());
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e2.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
    }
    __acc
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summarySeriesOpt<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    count: jni::sys::jlong,
    start: jni::sys::jlong,
    __acc: jni::objects::JObject<'a>,
    __fold: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let start = match jlong_to_i64_fbf9a9bc(&mut env, &start) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/analytics/SummaryFolder";
    const __CB_DESCR: &str = "(Ljava/lang/Object;JD)Ljava/lang/Object;";
    let __out = perftest_flat::summary_series_opt(count, start);
    match __out {
        ::core::option::Option::Some(__vec) => {
            let mut __acc = __acc;
            for __elem in __vec.into_iter() {
                let __obj0: jni::sys::jvalue = {
                    let __enc0 = match i64_to_jlong_fbf9a9bc(
                        &mut env,
                        perftest_flat::summary_count(&__elem),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    jni::sys::jvalue { j: __enc0 }
                };
                let __obj1: jni::sys::jvalue = {
                    let __enc1 = match f64_to_jdouble_9e4a8f70(
                        &mut env,
                        perftest_flat::summary_total(&__elem),
                    ) {
                        ::core::result::Result::Ok(__w) => __w,
                        ::core::result::Result::Err(__e) => {
                            signal_binding_error(
                                &mut env,
                                &__error_sink,
                                &__SINK_MID,
                                __SINK_FQN,
                                __SINK_DESCR,
                                &__e.to_string(),
                            );
                            return jni::objects::JObject::null().into();
                        }
                    };
                    jni::sys::jvalue { d: __enc1 }
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
                        ],
                    )
                {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        let _ = env.exception_describe();
                        let __e2 = <__JniErr as ::core::convert::From<
                            String,
                        >>::from(__e.to_string());
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e2.to_string(),
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
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summaryTotal<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jdouble {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match jlong_to_Summary_3cb103b9(&mut env, &s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __out = perftest_flat::summary_total(&s);
    match f64_to_jdouble_9e4a8f70(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0.0 as jni::sys::jdouble
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_summaryTotalOpt<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s_sel: jni::sys::jint,
    s_0_0_present: jni::sys::jboolean,
    s_0_0_value: jni::sys::jlong,
    s_0_1_present: jni::sys::jboolean,
    s_0_1_value: jni::sys::jdouble,
    s_1: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jdouble {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __exp_s_sel = match jint_to_i32_a3e3b6ef(&mut env, &s_sel) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __exp_s_0_0: Option<i64> = if s_0_0_present != 0u8 {
        let __v = match jlong_to_i64_fbf9a9bc(&mut env, &s_0_0_value) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0.0 as jni::sys::jdouble;
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0.0 as jni::sys::jdouble;
            }
        };
        ::core::option::Option::Some(__v)
    } else {
        ::core::option::Option::None
    };
    let __exp_s_1 = match jlong_to_Option_Summary_828826f3(&mut env, &s_1) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __folded_s = match if __exp_s_sel < 0 {
        ::core::result::Result::Ok(::core::option::Option::None)
    } else {
        ({
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
                        ::core::option::Option::Some(__v) => {
                            ::core::result::Result::Ok(
                                ::core::clone::Clone::clone(&*__v),
                            )
                        }
                        ::core::option::Option::None => {
                            ::core::result::Result::Err(
                                ::std::string::String::from(
                                    "identity variant value missing",
                                ),
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
        })
            .map(::core::option::Option::Some)
    } {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            let __je = <__JniErr as ::core::convert::From<
                ::std::string::String,
            >>::from(__e);
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__je.to_string(),
            );
            return 0.0 as jni::sys::jdouble;
        }
    };
    let __out = cov_helpers::summary_total_opt(__folded_s.as_ref());
    match f64_to_jdouble_9e4a8f70(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    if s == 0 || (s & 1) == 1 {
        signal_binding_error(
            &mut env,
            &__error_sink,
            &__SINK_MID,
            __SINK_FQN,
            __SINK_DESCR,
            "Operation on a closed native handle.",
        );
        return 0.0 as jni::sys::jdouble;
    }
    let s: perftest_flat::Summary = unsafe {
        *std::boxed::Box::from_raw(s as *mut perftest_flat::Summary)
    };
    let __out = perftest_flat::summary_total_raw(s);
    match f64_to_jdouble_9e4a8f70(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0.0 as jni::sys::jdouble
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_taggedNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    which: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let which = match jint_to_i32_a3e3b6ef(&mut env, &which) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::tagged_new(which);
    match Tagged_to_JObject_641b984c(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_taggedRank<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    t_id: jni::sys::jlong,
    t_marker: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jint {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let t = match tuple2_to_Tagged_b68a6969(&mut env, (t_id, t_marker)) {
        ::core::result::Result::Ok(__value) => __value,
        ::core::result::Result::Err(__error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__error.to_string(),
            );
            return 0 as jni::sys::jint;
        }
    };
    let __out = perftest_flat::tagged_rank(t);
    match i32_to_jint_a3e3b6ef(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jint
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_ticksEmit<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    f: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let f = match JObject_to_impl_Fn_Vec_Option_Ticks_Send_Sync_static_26c17cf0(
        &mut env,
        &f,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::ticks_emit(f);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_unsignedDataMaybe<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value_byte: jni::sys::jint,
    value_short: jni::sys::jint,
    value_int: jni::sys::jlong,
    value_long: jni::sys::jlong,
    value_maybe_long_present: jni::sys::jboolean,
    value_maybe_long_value: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __flat_value_byte = match jint_to_u8_553cf6ec(&mut env, &value_byte) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_value_short = match jint_to_u16_28edf527(&mut env, &value_short) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_value_int = match jlong_to_u32_9594a230(&mut env, &value_int) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_value_long = match jlong_to_u64_4384a5d6(&mut env, &value_long) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __flat_value_maybe_long = if value_maybe_long_present != 0u8 {
        let __flat_value_maybe_long_value = match jlong_to_u64_4384a5d6(
            &mut env,
            &value_maybe_long_value,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
        ::core::option::Option::Some(__flat_value_maybe_long_value)
    } else {
        ::core::option::Option::None
    };
    let __flat_value = perftest_flat::Unsigned {
        byte: __flat_value_byte,
        short: __flat_value_short,
        int: __flat_value_int,
        long: __flat_value_long,
        maybe_long: __flat_value_maybe_long,
    };
    let value = __flat_value;
    let __out = perftest_flat::unsigned_data_maybe(&value);
    match Option_u64_to_JObject_32be16a2(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_unsignedEmit<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value: jni::sys::jlong,
    f: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let value = match jlong_to_u64_4384a5d6(&mut env, &value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let f = match JObject_to_impl_Fn_u64_Send_Sync_static_c7830b57(&mut env, &f) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return ();
        }
    };
    let __out = perftest_flat::unsigned_emit(value, f);
    match unit_to_unit_9ecccf8e(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            ()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_unsignedOptional<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let value = match JObject_to_Option_u64_32be16a2(&mut env, &value) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::unsigned_optional(value);
    match Option_u64_to_JObject_32be16a2(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_unsignedRoundTrip<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    byte: jni::sys::jint,
    short: jni::sys::jint,
    int: jni::sys::jlong,
    long: jni::sys::jlong,
    maybe_long: jni::objects::JObject<'a>,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let byte = match jint_to_u8_553cf6ec(&mut env, &byte) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let short = match jint_to_u16_28edf527(&mut env, &short) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let int = match jlong_to_u32_9594a230(&mut env, &int) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let long = match jlong_to_u64_4384a5d6(&mut env, &long) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let maybe_long = match JObject_to_Option_u64_32be16a2(&mut env, &maybe_long) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/UnsignedBuilderRaw";
    const __CB_DESCR: &str = "(IIJJLjava/lang/Long;)Ljava/lang/Object;";
    let __out = perftest_flat::unsigned_round_trip(byte, short, int, long, maybe_long);
    let (__chain_wire0, __chain_wire1, __chain_wire2, __chain_wire3, __chain_wire4) = match Unsigned_to_tuple5_371b0950(
        &mut env,
        __out,
    ) {
        ::core::result::Result::Ok(__intermediate) => __intermediate,
        ::core::result::Result::Err(__chain_error) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__chain_error.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __obj0 = jni::sys::jvalue {
        i: __chain_wire0,
    };
    let __obj1 = jni::sys::jvalue {
        i: __chain_wire1,
    };
    let __obj2 = jni::sys::jvalue {
        j: __chain_wire2,
    };
    let __obj3 = jni::sys::jvalue {
        j: __chain_wire3,
    };
    let __obj4: jni::objects::JObject = __chain_wire4;
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_unsignedSeries<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    __acc: jni::objects::JObject<'a>,
    __fold: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/u64FolderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Object;J)Ljava/lang/Object;";
    let __vec = perftest_flat::unsigned_series();
    let mut __acc = __acc;
    for __elem in __vec.into_iter() {
        let __enc = {
            match u64_to_jlong_4384a5d6(&mut env, __elem) {
                ::core::result::Result::Ok(__w) => __w,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return jni::objects::JObject::null().into();
                }
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
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e2.to_string(),
                );
                return jni::objects::JObject::null().into();
            }
        };
    }
    __acc
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_vaultHolderNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    seq: jni::sys::jlong,
    count: jni::sys::jlong,
    maybe_count: jni::sys::jlong,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let seq = match jlong_to_i64_fbf9a9bc(&mut env, &seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let maybe_count = match jlong_to_i64_fbf9a9bc(&mut env, &maybe_count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/covertest/model/VaultHolderBuilderRaw";
    const __CB_DESCR: &str = "(Ljava/lang/Long;Ljava/lang/Long;)Ljava/lang/Object;";
    let __out = perftest_flat::vault_holder_new(seq, count, maybe_count);
    let __vf0 = perftest_flat::vault_holder_vault(&__out)
        .map(|__hb0| perftest_flat::vault_to_struct(__hb0));
    let (__obj0, __obj1): (jni::objects::JObject, jni::objects::JObject) = match __vf0 {
        ::core::option::Option::Some(__u0) => {
            let __obj0: jni::objects::JObject = {
                let __enc0 = match Ingot_to_jlong_020c3a86(
                    &mut env,
                    __u0.always.clone(),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                match ::prebindgen_jni_runtime::box_jlong(&mut env, __enc0) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e,
                        );
                        return jni::objects::JObject::null().into();
                    }
                }
            };
            let __obj1: jni::objects::JObject = {
                let __enc1 = match Option_Ingot_to_jlong_a76a8f2f(
                    &mut env,
                    __u0.maybe.clone(),
                ) {
                    ::core::result::Result::Ok(__w) => __w,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e.to_string(),
                        );
                        return jni::objects::JObject::null().into();
                    }
                };
                match ::prebindgen_jni_runtime::box_jlong(&mut env, __enc1) {
                    ::core::result::Result::Ok(__o) => __o,
                    ::core::result::Result::Err(__e) => {
                        signal_binding_error(
                            &mut env,
                            &__error_sink,
                            &__SINK_MID,
                            __SINK_FQN,
                            __SINK_DESCR,
                            &__e,
                        );
                        return jni::objects::JObject::null().into();
                    }
                }
            };
            (__obj0, __obj1)
        }
        ::core::option::Option::None => {
            (jni::objects::JObject::null(), jni::objects::JObject::null())
        }
    };
    match __CB_MID
        .call_object(
            &mut env,
            __CB_FQN,
            "run",
            __CB_DESCR,
            &__builder,
            &[
                jni::sys::jvalue {
                    l: __obj0.as_raw(),
                },
                jni::sys::jvalue {
                    l: __obj1.as_raw(),
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
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e2.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_verdictNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    id: jni::sys::jlong,
    count: jni::sys::jlong,
    total: jni::sys::jdouble,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let id = match jlong_to_i64_fbf9a9bc(&mut env, &id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let count = match jlong_to_i64_fbf9a9bc(&mut env, &count) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let total = match jdouble_to_f64_9e4a8f70(&mut env, &total) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return jni::objects::JObject::null().into();
        }
    };
    let __out = perftest_flat::verdict_new(id, count, total);
    match Verdict_to_JObject_a94c1ffd(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            jni::objects::JObject::null().into()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_covertest_CovNative_wrappedFieldsSum<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    w_id: jni::sys::jlong,
    w_boxed_present: jni::sys::jboolean,
    w_boxed_value: jni::sys::jlong,
    w_plain_present: jni::sys::jboolean,
    w_plain_value: jni::sys::jlong,
    w_boxed_enum: jni::sys::jint,
    w_plain_enum: jni::sys::jint,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __flat_w_id = match jlong_to_i64_fbf9a9bc(&mut env, &w_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __flat_w_boxed = ::std::boxed::Box::new(
        if w_boxed_present != 0u8 {
            let __flat_w_boxed_value = match jlong_to_i64_fbf9a9bc(
                &mut env,
                &w_boxed_value,
            ) {
                ::core::result::Result::Ok(__v) => __v,
                ::core::result::Result::Err(__e) => {
                    signal_binding_error(
                        &mut env,
                        &__error_sink,
                        &__SINK_MID,
                        __SINK_FQN,
                        __SINK_DESCR,
                        &__e.to_string(),
                    );
                    return 0 as jni::sys::jlong;
                }
            };
            ::core::option::Option::Some(__flat_w_boxed_value)
        } else {
            ::core::option::Option::None
        },
    );
    let __flat_w_plain = if w_plain_present != 0u8 {
        let __flat_w_plain_value = match jlong_to_i64_fbf9a9bc(
            &mut env,
            &w_plain_value,
        ) {
            ::core::result::Result::Ok(__v) => __v,
            ::core::result::Result::Err(__e) => {
                signal_binding_error(
                    &mut env,
                    &__error_sink,
                    &__SINK_MID,
                    __SINK_FQN,
                    __SINK_DESCR,
                    &__e.to_string(),
                );
                return 0 as jni::sys::jlong;
            }
        };
        ::core::option::Option::Some(__flat_w_plain_value)
    } else {
        ::core::option::Option::None
    };
    let __flat_w_boxed_enum = match jint_to_Box_Priority_a16653ae(
        &mut env,
        &w_boxed_enum,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __flat_w_plain_enum = match jint_to_Priority_447102d2(&mut env, &w_plain_enum) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            return 0 as jni::sys::jlong;
        }
    };
    let __flat_w = perftest_flat::WrappedFields {
        id: __flat_w_id,
        boxed: __flat_w_boxed,
        plain: __flat_w_plain,
        boxed_enum: __flat_w_boxed_enum,
        plain_enum: __flat_w_plain_enum,
    };
    let w = __flat_w;
    let __out = perftest_flat::wrapped_fields_sum(w);
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
            );
            0 as jni::sys::jlong
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::COVER_MAGIC;
    match i64_to_jlong_fbf9a9bc(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/covertest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::COVER_TAG;
    match str_to_JString_7b77dc67(&mut env, __out) {
        ::core::result::Result::Ok(__w) => __w,
        ::core::result::Result::Err(__e) => {
            signal_binding_error(
                &mut env,
                &__error_sink,
                &__SINK_MID,
                __SINK_FQN,
                __SINK_DESCR,
                &__e.to_string(),
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
        cov_helpers::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
