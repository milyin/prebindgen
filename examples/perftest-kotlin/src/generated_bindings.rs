#[allow(dead_code)]
pub(crate) type __JniErr = ::prebindgen_jni_runtime::JniBindingError<()>;
/// See module-level docs at [`owned_object_prerequisite_items`].
#[allow(dead_code)]
pub(crate) struct OwnedObject<T: ?Sized> {
    ptr: *const T,
}
impl<T: ?Sized> std::ops::Deref for OwnedObject<T> {
    type Target = T;
    #[inline]
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
pub(crate) unsafe extern "C" fn Java_io_prebindgen_perftest_PayloadHandler_freePtr(
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
pub(crate) unsafe extern "C" fn Java_io_prebindgen_perftest_PayloadVecHandler_freePtr(
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
pub(crate) unsafe extern "C" fn Java_io_prebindgen_perftest_Storage_freePtr(
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
pub(crate) unsafe extern "C" fn Java_io_prebindgen_perftest_TokenGc_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::TokenGc));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::TokenGc>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_perftest_Token_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut perftest_flat::Token));
    }
}
const _: () = {
    if ::core::mem::align_of::<perftest_flat::Token>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_payloadVecFree(
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
pub(crate) unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_payloadVecNew(
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
pub(crate) unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_payloadVecPush<'a>(
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
    let __e_id = match __jni_in_convert_wire_to_i64_da07d745d9e26f71(&mut env, &e_id) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            tracing::error!("vecPush: decoding `{}`: {}", stringify!(id), __e);
            return;
        }
    };
    let __e_seq = match __jni_in_convert_wire_to_i32_83b133e23cc76fc5(&mut env, &e_seq) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            tracing::error!("vecPush: decoding `{}`: {}", stringify!(seq), __e);
            return;
        }
    };
    let __e_value = match __jni_in_convert_wire_to_f64_b312e1b95182cdfd(
        &mut env,
        &e_value,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            tracing::error!("vecPush: decoding `{}`: {}", stringify!(value), __e);
            return;
        }
    };
    let __e_flag = match __jni_in_convert_wire_to_bool_1be2f6c32f925207(
        &mut env,
        &e_flag,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__e) => {
            tracing::error!("vecPush: decoding `{}`: {}", stringify!(flag), __e);
            return;
        }
    };
    let __e_label = match __jni_in_convert_wire_to_Option_Box_String_jni_optional_intermediate_input_niche_87b03b4201168b29(
        &mut env,
        &e_label,
    ) {
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary64_jni_product_intermediate_tuple_0a10a5cc1f8d19f9<
    'env,
    'a,
>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        (
            (
                (
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                ),
                (
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                ),
            ),
            (
                (
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                ),
                (
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                ),
            ),
        ),
        (
            (
                (
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                ),
                (
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                ),
            ),
            (
                (
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                ),
                (
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                    (
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                        ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ),
                ),
            ),
        ),
    ),
) -> ::core::result::Result<perftest_flat::ObjectBoundary64, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::ObjectBoundary64 {
        left: __jni_in_convert_wire_to_ObjectBoundary32_jni_product_intermediate_tuple_fa270ebfc88aa9e2(
            env,
            (v).0,
        )?,
        right: __jni_in_convert_wire_to_ObjectBoundary32_jni_product_intermediate_tuple_fa270ebfc88aa9e2(
            env,
            (v).1,
        )?,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary64_9d95f2f133e7e0ab<
    'env,
    'v,
>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary64, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/perftest/ObjectBoundary32;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary64.left: {}", e)))?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/perftest/ObjectBoundary32;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary64.right: {}", e)))?;
        perftest_flat::ObjectBoundary64 {
            left: __jni_in_convert_wire_to_ObjectBoundary32_79abada03cc29a21(
                env,
                &__left_raw,
            )?,
            right: __jni_in_convert_wire_to_ObjectBoundary32_79abada03cc29a21(
                env,
                &__right_raw,
            )?,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary64Object_3c7cf869ff4a3b45<
    'env,
    'v,
>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary64Object, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/perftest/ObjectBoundary32;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary64Object.left: {}", e)))?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/perftest/ObjectBoundary32;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary64Object.right: {}", e)))?;
        perftest_flat::ObjectBoundary64Object {
            left: __jni_in_convert_wire_to_ObjectBoundary32_79abada03cc29a21(
                env,
                &__left_raw,
            )?,
            right: __jni_in_convert_wire_to_ObjectBoundary32_79abada03cc29a21(
                env,
                &__right_raw,
            )?,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_Payload_jni_product_intermediate_tuple_f379fad611b26734<
    'env,
    'a,
>(
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
        id: __jni_in_convert_wire_to_i64_da07d745d9e26f71(env, &((v).0))?,
        seq: __jni_in_convert_wire_to_i32_83b133e23cc76fc5(env, &((v).1))?,
        value: __jni_in_convert_wire_to_f64_b312e1b95182cdfd(env, &((v).2))?,
        flag: __jni_in_convert_wire_to_bool_1be2f6c32f925207(env, &((v).3))?,
        label: __jni_in_convert_wire_to_Option_Box_String_jni_optional_intermediate_input_niche_87b03b4201168b29(
            env,
            &((v).4),
        )?,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_Payload_7e701167233e784f<'env, 'v>(
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
        let __seq_raw: jni::sys::jint = env
            .get_field(v, "seq", "I")
            .and_then(|val| val.i())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Payload.seq: {}", e)))? as _;
        let __value_raw: jni::sys::jdouble = env
            .get_field(v, "value", "D")
            .and_then(|val| val.d())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Payload.value: {}", e)))? as _;
        let __flag_raw: jni::sys::jboolean = env
            .get_field(v, "flag", "Z")
            .and_then(|val| val.z())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Payload.flag: {}", e)))? as _;
        let __label_jobj: jni::objects::JObject = env
            .get_field(v, "label", "Ljava/lang/String;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Payload.label: {}", e)))?;
        let __label_raw: jni::objects::JString = __label_jobj.into();
        perftest_flat::Payload {
            id: __jni_in_convert_wire_to_i64_da07d745d9e26f71(env, &__id_raw)?,
            seq: __jni_in_convert_wire_to_i32_83b133e23cc76fc5(env, &__seq_raw)?,
            value: __jni_in_convert_wire_to_f64_b312e1b95182cdfd(env, &__value_raw)?,
            flag: __jni_in_convert_wire_to_bool_1be2f6c32f925207(env, &__flag_raw)?,
            label: __jni_in_convert_wire_to_Option_Box_String_jni_optional_intermediate_input_niche_87b03b4201168b29(
                env,
                &__label_raw,
            )?,
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
pub(crate) unsafe fn __jni_out_convert_Payload_jni_product_intermediate_tuple_to_wire_c5a1c01cf2cbf49b<
    'a,
>(
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
        __jni_out_convert_i64_to_wire_15d458bf28dc9c80(env, (*&(v.id)).clone())?,
        __jni_out_convert_i32_to_wire_67173b19ae5a9348(env, (*&(v.seq)).clone())?,
        __jni_out_convert_f64_to_wire_61461de12ea6bc04(env, (*&(v.value)).clone())?,
        __jni_out_convert_bool_to_wire_3ee62077915d5228(env, (*&(v.flag)).clone())?,
        __jni_out_convert_Option_Box_String_jni_optional_intermediate_output_niche_to_wire_57342b1f497b4507(
            env,
            (*&(v.label)).clone(),
        )?,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_PayloadHandler_jni_handle_codec_borrow_input_f89cfeecbb4e240b<
    'env,
    'v,
>(
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
pub(crate) unsafe fn __jni_in_convert_wire_to_PayloadVecHandler_jni_handle_codec_borrow_input_1b365539726eca03<
    'env,
    'v,
>(
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
pub(crate) unsafe fn __jni_in_convert_wire_to_Storage_jni_handle_codec_borrow_input_697104a332693566<
    'env,
    'v,
>(
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
pub(crate) unsafe fn __jni_in_convert_wire_to_Token_jni_handle_codec_borrow_input_52a62575119c2292<
    'env,
    'v,
>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::Token>, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::Token) })
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
pub(crate) unsafe fn __jni_in_convert_wire_to_TokenGc_jni_handle_codec_borrow_input_7a9be906e3827840<
    'env,
    'v,
>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::TokenGc>, __JniErr> {
    if *v == 0 || (*v & 1) == 1 {
        return ::core::result::Result::Err(
            <__JniErr as ::core::convert::From<
                String,
            >>::from("Operation on a closed native handle.".to_string()),
        );
    }
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::TokenGc) })
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
pub(crate) unsafe fn __jni_out_convert_unit_to_wire_9e1510fd173c1fd6<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: (),
) -> ::core::result::Result<(), __JniErr> {
    Ok(v)
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
pub(crate) unsafe fn __jni_in_convert_wire_to_Box_String_7bf3c88ef26eb8e6<'env, 'v>(
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
pub(crate) unsafe fn __jni_out_convert_Box_String_to_wire_445d29257759cad9<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: ::std::boxed::Box<::std::string::String>,
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
#[inline(always)]
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary16_jni_product_intermediate_tuple_e8ffc41ce78124d4<
    'env,
    'a,
>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        (
            (
                ((jni::sys::jlong,), (jni::sys::jlong,)),
                ((jni::sys::jlong,), (jni::sys::jlong,)),
            ),
            (
                ((jni::sys::jlong,), (jni::sys::jlong,)),
                ((jni::sys::jlong,), (jni::sys::jlong,)),
            ),
        ),
        (
            (
                ((jni::sys::jlong,), (jni::sys::jlong,)),
                ((jni::sys::jlong,), (jni::sys::jlong,)),
            ),
            (
                ((jni::sys::jlong,), (jni::sys::jlong,)),
                ((jni::sys::jlong,), (jni::sys::jlong,)),
            ),
        ),
    ),
) -> ::core::result::Result<perftest_flat::ObjectBoundary16, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::ObjectBoundary16 {
        left: __jni_in_convert_wire_to_ObjectBoundary8_jni_product_intermediate_tuple_4e665030ef655cf9(
            env,
            (v).0,
        )?,
        right: __jni_in_convert_wire_to_ObjectBoundary8_jni_product_intermediate_tuple_4e665030ef655cf9(
            env,
            (v).1,
        )?,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary16_3944a3a904c1510d<
    'env,
    'v,
>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary16, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/perftest/ObjectBoundary8;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary16.left: {}", e)))?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/perftest/ObjectBoundary8;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary16.right: {}", e)))?;
        perftest_flat::ObjectBoundary16 {
            left: __jni_in_convert_wire_to_ObjectBoundary8_cca4b20df6695267(
                env,
                &__left_raw,
            )?,
            right: __jni_in_convert_wire_to_ObjectBoundary8_cca4b20df6695267(
                env,
                &__right_raw,
            )?,
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
pub(crate) unsafe fn __jni_out_convert_ObjectBoundary16_to_wire_f1c84cc9740f51b2<'a>(
    mut env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary16,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __obj0: jni::sys::jlong = {
            let __enc0 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).left).left).left).left).value).clone(),
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
            __enc0
        };
        let __obj1: jni::sys::jlong = {
            let __enc1 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).left).left).left).right).value).clone(),
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
            __enc1
        };
        let __obj2: jni::sys::jlong = {
            let __enc2 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).left).left).right).left).value).clone(),
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
        let __obj3: jni::sys::jlong = {
            let __enc3 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).left).left).right).right).value).clone(),
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
            __enc3
        };
        let __obj4: jni::sys::jlong = {
            let __enc4 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).left).right).left).left).value).clone(),
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
            __enc4
        };
        let __obj5: jni::sys::jlong = {
            let __enc5 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).left).right).left).right).value).clone(),
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
            __enc5
        };
        let __obj6: jni::sys::jlong = {
            let __enc6 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).left).right).right).left).value).clone(),
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
            __enc6
        };
        let __obj7: jni::sys::jlong = {
            let __enc7 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).left).right).right).right).value).clone(),
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
            __enc7
        };
        let __obj8: jni::sys::jlong = {
            let __enc8 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).right).left).left).left).value).clone(),
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
            __enc8
        };
        let __obj9: jni::sys::jlong = {
            let __enc9 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).right).left).left).right).value).clone(),
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
            __enc9
        };
        let __obj10: jni::sys::jlong = {
            let __enc10 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).right).left).right).left).value).clone(),
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
            __enc10
        };
        let __obj11: jni::sys::jlong = {
            let __enc11 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).right).left).right).right).value).clone(),
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
        let __obj12: jni::sys::jlong = {
            let __enc12 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).right).right).left).left).value).clone(),
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
            __enc12
        };
        let __obj13: jni::sys::jlong = {
            let __enc13 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).right).right).left).right).value).clone(),
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
            __enc13
        };
        let __obj14: jni::sys::jlong = {
            let __enc14 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).right).right).right).left).value).clone(),
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
            __enc14
        };
        let __obj15: jni::sys::jlong = {
            let __enc15 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&v).right).right).right).right).value).clone(),
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
            __enc15
        };
        let __obj = env
            .call_static_method(
                "io/prebindgen/perftest/ObjectBoundary16",
                "fromParts",
                "(JJJJJJJJJJJJJJJJ)Lio/prebindgen/perftest/ObjectBoundary16;",
                &[
                    jni::objects::JValue::from(__obj0),
                    jni::objects::JValue::from(__obj1),
                    jni::objects::JValue::from(__obj2),
                    jni::objects::JValue::from(__obj3),
                    jni::objects::JValue::from(__obj4),
                    jni::objects::JValue::from(__obj5),
                    jni::objects::JValue::from(__obj6),
                    jni::objects::JValue::from(__obj7),
                    jni::objects::JValue::from(__obj8),
                    jni::objects::JValue::from(__obj9),
                    jni::objects::JValue::from(__obj10),
                    jni::objects::JValue::from(__obj11),
                    jni::objects::JValue::from(__obj12),
                    jni::objects::JValue::from(__obj13),
                    jni::objects::JValue::from(__obj14),
                    jni::objects::JValue::from(__obj15),
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary2_jni_product_intermediate_tuple_37d2083cec1ab44b<
    'env,
    'a,
>(
    env: &mut jni::JNIEnv<'env>,
    v: ((jni::sys::jlong,), (jni::sys::jlong,)),
) -> ::core::result::Result<perftest_flat::ObjectBoundary2, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::ObjectBoundary2 {
        left: __jni_in_convert_wire_to_ObjectBoundaryLeaf_jni_product_intermediate_tuple_1a6620ad58b014cf(
            env,
            (v).0,
        )?,
        right: __jni_in_convert_wire_to_ObjectBoundaryLeaf_jni_product_intermediate_tuple_1a6620ad58b014cf(
            env,
            (v).1,
        )?,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary2_a3195430eb031edb<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary2, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/perftest/ObjectBoundaryLeaf;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary2.left: {}", e)))?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/perftest/ObjectBoundaryLeaf;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary2.right: {}", e)))?;
        perftest_flat::ObjectBoundary2 {
            left: __jni_in_convert_wire_to_ObjectBoundaryLeaf_3132a7517e2ab837(
                env,
                &__left_raw,
            )?,
            right: __jni_in_convert_wire_to_ObjectBoundaryLeaf_3132a7517e2ab837(
                env,
                &__right_raw,
            )?,
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
pub(crate) unsafe fn __jni_out_convert_ObjectBoundary2_to_wire_86a291e77dd72646<'a>(
    mut env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary2,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __obj0: jni::sys::jlong = {
            let __enc0 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&v).left).value).clone(),
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
            __enc0
        };
        let __obj1: jni::sys::jlong = {
            let __enc1 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&v).right).value).clone(),
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
            __enc1
        };
        let __obj = env
            .call_static_method(
                "io/prebindgen/perftest/ObjectBoundary2",
                "fromParts",
                "(JJ)Lio/prebindgen/perftest/ObjectBoundary2;",
                &[jni::objects::JValue::from(__obj0), jni::objects::JValue::from(__obj1)],
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary32_jni_product_intermediate_tuple_fa270ebfc88aa9e2<
    'env,
    'a,
>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        (
            (
                (
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                ),
                (
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                ),
            ),
            (
                (
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                ),
                (
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                ),
            ),
        ),
        (
            (
                (
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                ),
                (
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                ),
            ),
            (
                (
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                ),
                (
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                    ((jni::sys::jlong,), (jni::sys::jlong,)),
                ),
            ),
        ),
    ),
) -> ::core::result::Result<perftest_flat::ObjectBoundary32, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::ObjectBoundary32 {
        left: __jni_in_convert_wire_to_ObjectBoundary16_jni_product_intermediate_tuple_e8ffc41ce78124d4(
            env,
            (v).0,
        )?,
        right: __jni_in_convert_wire_to_ObjectBoundary16_jni_product_intermediate_tuple_e8ffc41ce78124d4(
            env,
            (v).1,
        )?,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary32_79abada03cc29a21<
    'env,
    'v,
>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary32, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/perftest/ObjectBoundary16;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary32.left: {}", e)))?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/perftest/ObjectBoundary16;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary32.right: {}", e)))?;
        perftest_flat::ObjectBoundary32 {
            left: __jni_in_convert_wire_to_ObjectBoundary16_3944a3a904c1510d(
                env,
                &__left_raw,
            )?,
            right: __jni_in_convert_wire_to_ObjectBoundary16_3944a3a904c1510d(
                env,
                &__right_raw,
            )?,
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
pub(crate) unsafe fn __jni_out_convert_ObjectBoundary32_to_wire_2004ebb99e1975a2<'a>(
    mut env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary32,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __obj0: jni::sys::jlong = {
            let __enc0 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).left).left).left).left).value).clone(),
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
            __enc0
        };
        let __obj1: jni::sys::jlong = {
            let __enc1 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).left).left).left).right).value).clone(),
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
            __enc1
        };
        let __obj2: jni::sys::jlong = {
            let __enc2 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).left).left).right).left).value).clone(),
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
        let __obj3: jni::sys::jlong = {
            let __enc3 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).left).left).right).right).value).clone(),
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
            __enc3
        };
        let __obj4: jni::sys::jlong = {
            let __enc4 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).left).right).left).left).value).clone(),
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
            __enc4
        };
        let __obj5: jni::sys::jlong = {
            let __enc5 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).left).right).left).right).value).clone(),
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
            __enc5
        };
        let __obj6: jni::sys::jlong = {
            let __enc6 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).left).right).right).left).value).clone(),
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
            __enc6
        };
        let __obj7: jni::sys::jlong = {
            let __enc7 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).left).right).right).right).value).clone(),
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
            __enc7
        };
        let __obj8: jni::sys::jlong = {
            let __enc8 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).right).left).left).left).value).clone(),
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
            __enc8
        };
        let __obj9: jni::sys::jlong = {
            let __enc9 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).right).left).left).right).value).clone(),
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
            __enc9
        };
        let __obj10: jni::sys::jlong = {
            let __enc10 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).right).left).right).left).value).clone(),
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
            __enc10
        };
        let __obj11: jni::sys::jlong = {
            let __enc11 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).right).left).right).right).value).clone(),
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
        let __obj12: jni::sys::jlong = {
            let __enc12 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).right).right).left).left).value).clone(),
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
            __enc12
        };
        let __obj13: jni::sys::jlong = {
            let __enc13 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).right).right).left).right).value).clone(),
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
            __enc13
        };
        let __obj14: jni::sys::jlong = {
            let __enc14 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).right).right).right).left).value).clone(),
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
            __enc14
        };
        let __obj15: jni::sys::jlong = {
            let __enc15 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).left).right).right).right).right).value).clone(),
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
            __enc15
        };
        let __obj16: jni::sys::jlong = {
            let __enc16 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).left).left).left).left).value).clone(),
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
            __enc16
        };
        let __obj17: jni::sys::jlong = {
            let __enc17 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).left).left).left).right).value).clone(),
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
            __enc17
        };
        let __obj18: jni::sys::jlong = {
            let __enc18 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).left).left).right).left).value).clone(),
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
            __enc18
        };
        let __obj19: jni::sys::jlong = {
            let __enc19 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).left).left).right).right).value).clone(),
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
            __enc19
        };
        let __obj20: jni::sys::jlong = {
            let __enc20 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).left).right).left).left).value).clone(),
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
            __enc20
        };
        let __obj21: jni::sys::jlong = {
            let __enc21 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).left).right).left).right).value).clone(),
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
            __enc21
        };
        let __obj22: jni::sys::jlong = {
            let __enc22 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).left).right).right).left).value).clone(),
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
            __enc22
        };
        let __obj23: jni::sys::jlong = {
            let __enc23 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).left).right).right).right).value).clone(),
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
            __enc23
        };
        let __obj24: jni::sys::jlong = {
            let __enc24 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).right).left).left).left).value).clone(),
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
            __enc24
        };
        let __obj25: jni::sys::jlong = {
            let __enc25 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).right).left).left).right).value).clone(),
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
            __enc25
        };
        let __obj26: jni::sys::jlong = {
            let __enc26 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).right).left).right).left).value).clone(),
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
            __enc26
        };
        let __obj27: jni::sys::jlong = {
            let __enc27 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).right).left).right).right).value).clone(),
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
            __enc27
        };
        let __obj28: jni::sys::jlong = {
            let __enc28 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).right).right).left).left).value).clone(),
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
            __enc28
        };
        let __obj29: jni::sys::jlong = {
            let __enc29 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).right).right).left).right).value).clone(),
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
            __enc29
        };
        let __obj30: jni::sys::jlong = {
            let __enc30 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).right).right).right).left).value).clone(),
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
            __enc30
        };
        let __obj31: jni::sys::jlong = {
            let __enc31 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&v).right).right).right).right).right).value).clone(),
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
            __enc31
        };
        let __obj = env
            .call_static_method(
                "io/prebindgen/perftest/ObjectBoundary32",
                "fromParts",
                "(JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ)Lio/prebindgen/perftest/ObjectBoundary32;",
                &[
                    jni::objects::JValue::from(__obj0),
                    jni::objects::JValue::from(__obj1),
                    jni::objects::JValue::from(__obj2),
                    jni::objects::JValue::from(__obj3),
                    jni::objects::JValue::from(__obj4),
                    jni::objects::JValue::from(__obj5),
                    jni::objects::JValue::from(__obj6),
                    jni::objects::JValue::from(__obj7),
                    jni::objects::JValue::from(__obj8),
                    jni::objects::JValue::from(__obj9),
                    jni::objects::JValue::from(__obj10),
                    jni::objects::JValue::from(__obj11),
                    jni::objects::JValue::from(__obj12),
                    jni::objects::JValue::from(__obj13),
                    jni::objects::JValue::from(__obj14),
                    jni::objects::JValue::from(__obj15),
                    jni::objects::JValue::from(__obj16),
                    jni::objects::JValue::from(__obj17),
                    jni::objects::JValue::from(__obj18),
                    jni::objects::JValue::from(__obj19),
                    jni::objects::JValue::from(__obj20),
                    jni::objects::JValue::from(__obj21),
                    jni::objects::JValue::from(__obj22),
                    jni::objects::JValue::from(__obj23),
                    jni::objects::JValue::from(__obj24),
                    jni::objects::JValue::from(__obj25),
                    jni::objects::JValue::from(__obj26),
                    jni::objects::JValue::from(__obj27),
                    jni::objects::JValue::from(__obj28),
                    jni::objects::JValue::from(__obj29),
                    jni::objects::JValue::from(__obj30),
                    jni::objects::JValue::from(__obj31),
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary4_jni_product_intermediate_tuple_e940f3772c7beded<
    'env,
    'a,
>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        ((jni::sys::jlong,), (jni::sys::jlong,)),
        ((jni::sys::jlong,), (jni::sys::jlong,)),
    ),
) -> ::core::result::Result<perftest_flat::ObjectBoundary4, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::ObjectBoundary4 {
        left: __jni_in_convert_wire_to_ObjectBoundary2_jni_product_intermediate_tuple_37d2083cec1ab44b(
            env,
            (v).0,
        )?,
        right: __jni_in_convert_wire_to_ObjectBoundary2_jni_product_intermediate_tuple_37d2083cec1ab44b(
            env,
            (v).1,
        )?,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary4_feb5834d3248e52f<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary4, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/perftest/ObjectBoundary2;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary4.left: {}", e)))?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/perftest/ObjectBoundary2;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary4.right: {}", e)))?;
        perftest_flat::ObjectBoundary4 {
            left: __jni_in_convert_wire_to_ObjectBoundary2_a3195430eb031edb(
                env,
                &__left_raw,
            )?,
            right: __jni_in_convert_wire_to_ObjectBoundary2_a3195430eb031edb(
                env,
                &__right_raw,
            )?,
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
pub(crate) unsafe fn __jni_out_convert_ObjectBoundary4_to_wire_ad4aabb9343a25e6<'a>(
    mut env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary4,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __obj0: jni::sys::jlong = {
            let __enc0 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&v).left).left).value).clone(),
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
            __enc0
        };
        let __obj1: jni::sys::jlong = {
            let __enc1 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&v).left).right).value).clone(),
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
            __enc1
        };
        let __obj2: jni::sys::jlong = {
            let __enc2 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&v).right).left).value).clone(),
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
        let __obj3: jni::sys::jlong = {
            let __enc3 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&v).right).right).value).clone(),
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
            __enc3
        };
        let __obj = env
            .call_static_method(
                "io/prebindgen/perftest/ObjectBoundary4",
                "fromParts",
                "(JJJJ)Lio/prebindgen/perftest/ObjectBoundary4;",
                &[
                    jni::objects::JValue::from(__obj0),
                    jni::objects::JValue::from(__obj1),
                    jni::objects::JValue::from(__obj2),
                    jni::objects::JValue::from(__obj3),
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
pub(crate) unsafe fn __jni_out_convert_ObjectBoundary64_to_wire_93ea03fd8ed503d0<'a>(
    mut env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary64,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __obj0: jni::sys::jlong = {
            let __enc0 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).left).left).left).value).clone(),
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
            __enc0
        };
        let __obj1: jni::sys::jlong = {
            let __enc1 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).left).left).right).value).clone(),
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
            __enc1
        };
        let __obj2: jni::sys::jlong = {
            let __enc2 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).left).right).left).value).clone(),
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
        let __obj3: jni::sys::jlong = {
            let __enc3 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).left).right).right).value).clone(),
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
            __enc3
        };
        let __obj4: jni::sys::jlong = {
            let __enc4 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).right).left).left).value).clone(),
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
            __enc4
        };
        let __obj5: jni::sys::jlong = {
            let __enc5 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).right).left).right).value).clone(),
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
            __enc5
        };
        let __obj6: jni::sys::jlong = {
            let __enc6 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).right).right).left).value).clone(),
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
            __enc6
        };
        let __obj7: jni::sys::jlong = {
            let __enc7 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).right).right).right).value).clone(),
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
            __enc7
        };
        let __obj8: jni::sys::jlong = {
            let __enc8 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).left).left).left).value).clone(),
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
            __enc8
        };
        let __obj9: jni::sys::jlong = {
            let __enc9 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).left).left).right).value).clone(),
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
            __enc9
        };
        let __obj10: jni::sys::jlong = {
            let __enc10 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).left).right).left).value).clone(),
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
            __enc10
        };
        let __obj11: jni::sys::jlong = {
            let __enc11 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).left).right).right).value).clone(),
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
        let __obj12: jni::sys::jlong = {
            let __enc12 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).right).left).left).value).clone(),
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
            __enc12
        };
        let __obj13: jni::sys::jlong = {
            let __enc13 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).right).left).right).value).clone(),
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
            __enc13
        };
        let __obj14: jni::sys::jlong = {
            let __enc14 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).right).right).left).value).clone(),
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
            __enc14
        };
        let __obj15: jni::sys::jlong = {
            let __enc15 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).right).right).right).value).clone(),
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
            __enc15
        };
        let __obj16: jni::sys::jlong = {
            let __enc16 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).left).left).left).value).clone(),
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
            __enc16
        };
        let __obj17: jni::sys::jlong = {
            let __enc17 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).left).left).right).value).clone(),
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
            __enc17
        };
        let __obj18: jni::sys::jlong = {
            let __enc18 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).left).right).left).value).clone(),
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
            __enc18
        };
        let __obj19: jni::sys::jlong = {
            let __enc19 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).left).right).right).value).clone(),
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
            __enc19
        };
        let __obj20: jni::sys::jlong = {
            let __enc20 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).right).left).left).value).clone(),
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
            __enc20
        };
        let __obj21: jni::sys::jlong = {
            let __enc21 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).right).left).right).value).clone(),
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
            __enc21
        };
        let __obj22: jni::sys::jlong = {
            let __enc22 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).right).right).left).value).clone(),
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
            __enc22
        };
        let __obj23: jni::sys::jlong = {
            let __enc23 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).right).right).right).value).clone(),
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
            __enc23
        };
        let __obj24: jni::sys::jlong = {
            let __enc24 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).left).left).left).value).clone(),
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
            __enc24
        };
        let __obj25: jni::sys::jlong = {
            let __enc25 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).left).left).right).value).clone(),
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
            __enc25
        };
        let __obj26: jni::sys::jlong = {
            let __enc26 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).left).right).left).value).clone(),
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
            __enc26
        };
        let __obj27: jni::sys::jlong = {
            let __enc27 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).left).right).right).value).clone(),
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
            __enc27
        };
        let __obj28: jni::sys::jlong = {
            let __enc28 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).right).left).left).value).clone(),
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
            __enc28
        };
        let __obj29: jni::sys::jlong = {
            let __enc29 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).right).left).right).value).clone(),
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
            __enc29
        };
        let __obj30: jni::sys::jlong = {
            let __enc30 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).right).right).left).value).clone(),
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
            __enc30
        };
        let __obj31: jni::sys::jlong = {
            let __enc31 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).right).right).right).value)
                    .clone(),
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
            __enc31
        };
        let __obj32: jni::sys::jlong = {
            let __enc32 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).left).left).left).value).clone(),
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
            __enc32
        };
        let __obj33: jni::sys::jlong = {
            let __enc33 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).left).left).right).value).clone(),
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
            __enc33
        };
        let __obj34: jni::sys::jlong = {
            let __enc34 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).left).right).left).value).clone(),
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
            __enc34
        };
        let __obj35: jni::sys::jlong = {
            let __enc35 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).left).right).right).value).clone(),
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
            __enc35
        };
        let __obj36: jni::sys::jlong = {
            let __enc36 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).right).left).left).value).clone(),
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
            __enc36
        };
        let __obj37: jni::sys::jlong = {
            let __enc37 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).right).left).right).value).clone(),
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
            __enc37
        };
        let __obj38: jni::sys::jlong = {
            let __enc38 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).right).right).left).value).clone(),
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
            __enc38
        };
        let __obj39: jni::sys::jlong = {
            let __enc39 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).right).right).right).value).clone(),
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
            __enc39
        };
        let __obj40: jni::sys::jlong = {
            let __enc40 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).left).left).left).value).clone(),
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
            __enc40
        };
        let __obj41: jni::sys::jlong = {
            let __enc41 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).left).left).right).value).clone(),
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
            __enc41
        };
        let __obj42: jni::sys::jlong = {
            let __enc42 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).left).right).left).value).clone(),
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
            __enc42
        };
        let __obj43: jni::sys::jlong = {
            let __enc43 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).left).right).right).value).clone(),
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
            __enc43
        };
        let __obj44: jni::sys::jlong = {
            let __enc44 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).right).left).left).value).clone(),
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
            __enc44
        };
        let __obj45: jni::sys::jlong = {
            let __enc45 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).right).left).right).value).clone(),
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
            __enc45
        };
        let __obj46: jni::sys::jlong = {
            let __enc46 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).right).right).left).value).clone(),
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
            __enc46
        };
        let __obj47: jni::sys::jlong = {
            let __enc47 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).right).right).right).value)
                    .clone(),
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
            __enc47
        };
        let __obj48: jni::sys::jlong = {
            let __enc48 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).left).left).left).value).clone(),
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
            __enc48
        };
        let __obj49: jni::sys::jlong = {
            let __enc49 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).left).left).right).value).clone(),
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
            __enc49
        };
        let __obj50: jni::sys::jlong = {
            let __enc50 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).left).right).left).value).clone(),
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
            __enc50
        };
        let __obj51: jni::sys::jlong = {
            let __enc51 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).left).right).right).value).clone(),
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
            __enc51
        };
        let __obj52: jni::sys::jlong = {
            let __enc52 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).right).left).left).value).clone(),
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
            __enc52
        };
        let __obj53: jni::sys::jlong = {
            let __enc53 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).right).left).right).value).clone(),
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
            __enc53
        };
        let __obj54: jni::sys::jlong = {
            let __enc54 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).right).right).left).value).clone(),
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
            __enc54
        };
        let __obj55: jni::sys::jlong = {
            let __enc55 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).right).right).right).value)
                    .clone(),
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
            __enc55
        };
        let __obj56: jni::sys::jlong = {
            let __enc56 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).left).left).left).value).clone(),
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
            __enc56
        };
        let __obj57: jni::sys::jlong = {
            let __enc57 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).left).left).right).value).clone(),
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
            __enc57
        };
        let __obj58: jni::sys::jlong = {
            let __enc58 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).left).right).left).value).clone(),
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
            __enc58
        };
        let __obj59: jni::sys::jlong = {
            let __enc59 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).left).right).right).value)
                    .clone(),
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
            __enc59
        };
        let __obj60: jni::sys::jlong = {
            let __enc60 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).right).left).left).value).clone(),
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
            __enc60
        };
        let __obj61: jni::sys::jlong = {
            let __enc61 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).right).left).right).value)
                    .clone(),
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
            __enc61
        };
        let __obj62: jni::sys::jlong = {
            let __enc62 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).right).right).left).value)
                    .clone(),
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
            __enc62
        };
        let __obj63: jni::sys::jlong = {
            let __enc63 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).right).right).right).value)
                    .clone(),
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
            __enc63
        };
        let __obj = env
            .call_static_method(
                "io/prebindgen/perftest/ObjectBoundary64",
                "fromParts",
                "(JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ)Lio/prebindgen/perftest/ObjectBoundary64;",
                &[
                    jni::objects::JValue::from(__obj0),
                    jni::objects::JValue::from(__obj1),
                    jni::objects::JValue::from(__obj2),
                    jni::objects::JValue::from(__obj3),
                    jni::objects::JValue::from(__obj4),
                    jni::objects::JValue::from(__obj5),
                    jni::objects::JValue::from(__obj6),
                    jni::objects::JValue::from(__obj7),
                    jni::objects::JValue::from(__obj8),
                    jni::objects::JValue::from(__obj9),
                    jni::objects::JValue::from(__obj10),
                    jni::objects::JValue::from(__obj11),
                    jni::objects::JValue::from(__obj12),
                    jni::objects::JValue::from(__obj13),
                    jni::objects::JValue::from(__obj14),
                    jni::objects::JValue::from(__obj15),
                    jni::objects::JValue::from(__obj16),
                    jni::objects::JValue::from(__obj17),
                    jni::objects::JValue::from(__obj18),
                    jni::objects::JValue::from(__obj19),
                    jni::objects::JValue::from(__obj20),
                    jni::objects::JValue::from(__obj21),
                    jni::objects::JValue::from(__obj22),
                    jni::objects::JValue::from(__obj23),
                    jni::objects::JValue::from(__obj24),
                    jni::objects::JValue::from(__obj25),
                    jni::objects::JValue::from(__obj26),
                    jni::objects::JValue::from(__obj27),
                    jni::objects::JValue::from(__obj28),
                    jni::objects::JValue::from(__obj29),
                    jni::objects::JValue::from(__obj30),
                    jni::objects::JValue::from(__obj31),
                    jni::objects::JValue::from(__obj32),
                    jni::objects::JValue::from(__obj33),
                    jni::objects::JValue::from(__obj34),
                    jni::objects::JValue::from(__obj35),
                    jni::objects::JValue::from(__obj36),
                    jni::objects::JValue::from(__obj37),
                    jni::objects::JValue::from(__obj38),
                    jni::objects::JValue::from(__obj39),
                    jni::objects::JValue::from(__obj40),
                    jni::objects::JValue::from(__obj41),
                    jni::objects::JValue::from(__obj42),
                    jni::objects::JValue::from(__obj43),
                    jni::objects::JValue::from(__obj44),
                    jni::objects::JValue::from(__obj45),
                    jni::objects::JValue::from(__obj46),
                    jni::objects::JValue::from(__obj47),
                    jni::objects::JValue::from(__obj48),
                    jni::objects::JValue::from(__obj49),
                    jni::objects::JValue::from(__obj50),
                    jni::objects::JValue::from(__obj51),
                    jni::objects::JValue::from(__obj52),
                    jni::objects::JValue::from(__obj53),
                    jni::objects::JValue::from(__obj54),
                    jni::objects::JValue::from(__obj55),
                    jni::objects::JValue::from(__obj56),
                    jni::objects::JValue::from(__obj57),
                    jni::objects::JValue::from(__obj58),
                    jni::objects::JValue::from(__obj59),
                    jni::objects::JValue::from(__obj60),
                    jni::objects::JValue::from(__obj61),
                    jni::objects::JValue::from(__obj62),
                    jni::objects::JValue::from(__obj63),
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
pub(crate) unsafe fn __jni_out_convert_ObjectBoundary64Object_to_wire_0894385ef68841d6<
    'a,
>(
    mut env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary64Object,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __obj0: jni::sys::jlong = {
            let __enc0 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).left).left).left).value).clone(),
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
            __enc0
        };
        let __obj1: jni::sys::jlong = {
            let __enc1 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).left).left).right).value).clone(),
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
            __enc1
        };
        let __obj2: jni::sys::jlong = {
            let __enc2 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).left).right).left).value).clone(),
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
        let __obj3: jni::sys::jlong = {
            let __enc3 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).left).right).right).value).clone(),
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
            __enc3
        };
        let __obj4: jni::sys::jlong = {
            let __enc4 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).right).left).left).value).clone(),
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
            __enc4
        };
        let __obj5: jni::sys::jlong = {
            let __enc5 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).right).left).right).value).clone(),
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
            __enc5
        };
        let __obj6: jni::sys::jlong = {
            let __enc6 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).right).right).left).value).clone(),
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
            __enc6
        };
        let __obj7: jni::sys::jlong = {
            let __enc7 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).left).right).right).right).value).clone(),
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
            __enc7
        };
        let __obj8: jni::sys::jlong = {
            let __enc8 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).left).left).left).value).clone(),
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
            __enc8
        };
        let __obj9: jni::sys::jlong = {
            let __enc9 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).left).left).right).value).clone(),
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
            __enc9
        };
        let __obj10: jni::sys::jlong = {
            let __enc10 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).left).right).left).value).clone(),
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
            __enc10
        };
        let __obj11: jni::sys::jlong = {
            let __enc11 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).left).right).right).value).clone(),
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
        let __obj12: jni::sys::jlong = {
            let __enc12 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).right).left).left).value).clone(),
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
            __enc12
        };
        let __obj13: jni::sys::jlong = {
            let __enc13 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).right).left).right).value).clone(),
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
            __enc13
        };
        let __obj14: jni::sys::jlong = {
            let __enc14 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).right).right).left).value).clone(),
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
            __enc14
        };
        let __obj15: jni::sys::jlong = {
            let __enc15 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).left).right).right).right).right).value).clone(),
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
            __enc15
        };
        let __obj16: jni::sys::jlong = {
            let __enc16 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).left).left).left).value).clone(),
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
            __enc16
        };
        let __obj17: jni::sys::jlong = {
            let __enc17 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).left).left).right).value).clone(),
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
            __enc17
        };
        let __obj18: jni::sys::jlong = {
            let __enc18 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).left).right).left).value).clone(),
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
            __enc18
        };
        let __obj19: jni::sys::jlong = {
            let __enc19 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).left).right).right).value).clone(),
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
            __enc19
        };
        let __obj20: jni::sys::jlong = {
            let __enc20 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).right).left).left).value).clone(),
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
            __enc20
        };
        let __obj21: jni::sys::jlong = {
            let __enc21 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).right).left).right).value).clone(),
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
            __enc21
        };
        let __obj22: jni::sys::jlong = {
            let __enc22 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).right).right).left).value).clone(),
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
            __enc22
        };
        let __obj23: jni::sys::jlong = {
            let __enc23 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).left).right).right).right).value).clone(),
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
            __enc23
        };
        let __obj24: jni::sys::jlong = {
            let __enc24 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).left).left).left).value).clone(),
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
            __enc24
        };
        let __obj25: jni::sys::jlong = {
            let __enc25 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).left).left).right).value).clone(),
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
            __enc25
        };
        let __obj26: jni::sys::jlong = {
            let __enc26 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).left).right).left).value).clone(),
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
            __enc26
        };
        let __obj27: jni::sys::jlong = {
            let __enc27 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).left).right).right).value).clone(),
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
            __enc27
        };
        let __obj28: jni::sys::jlong = {
            let __enc28 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).right).left).left).value).clone(),
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
            __enc28
        };
        let __obj29: jni::sys::jlong = {
            let __enc29 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).right).left).right).value).clone(),
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
            __enc29
        };
        let __obj30: jni::sys::jlong = {
            let __enc30 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).right).right).left).value).clone(),
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
            __enc30
        };
        let __obj31: jni::sys::jlong = {
            let __enc31 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).left).right).right).right).right).right).value)
                    .clone(),
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
            __enc31
        };
        let __obj32: jni::sys::jlong = {
            let __enc32 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).left).left).left).value).clone(),
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
            __enc32
        };
        let __obj33: jni::sys::jlong = {
            let __enc33 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).left).left).right).value).clone(),
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
            __enc33
        };
        let __obj34: jni::sys::jlong = {
            let __enc34 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).left).right).left).value).clone(),
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
            __enc34
        };
        let __obj35: jni::sys::jlong = {
            let __enc35 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).left).right).right).value).clone(),
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
            __enc35
        };
        let __obj36: jni::sys::jlong = {
            let __enc36 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).right).left).left).value).clone(),
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
            __enc36
        };
        let __obj37: jni::sys::jlong = {
            let __enc37 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).right).left).right).value).clone(),
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
            __enc37
        };
        let __obj38: jni::sys::jlong = {
            let __enc38 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).right).right).left).value).clone(),
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
            __enc38
        };
        let __obj39: jni::sys::jlong = {
            let __enc39 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).left).right).right).right).value).clone(),
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
            __enc39
        };
        let __obj40: jni::sys::jlong = {
            let __enc40 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).left).left).left).value).clone(),
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
            __enc40
        };
        let __obj41: jni::sys::jlong = {
            let __enc41 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).left).left).right).value).clone(),
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
            __enc41
        };
        let __obj42: jni::sys::jlong = {
            let __enc42 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).left).right).left).value).clone(),
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
            __enc42
        };
        let __obj43: jni::sys::jlong = {
            let __enc43 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).left).right).right).value).clone(),
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
            __enc43
        };
        let __obj44: jni::sys::jlong = {
            let __enc44 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).right).left).left).value).clone(),
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
            __enc44
        };
        let __obj45: jni::sys::jlong = {
            let __enc45 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).right).left).right).value).clone(),
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
            __enc45
        };
        let __obj46: jni::sys::jlong = {
            let __enc46 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).right).right).left).value).clone(),
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
            __enc46
        };
        let __obj47: jni::sys::jlong = {
            let __enc47 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).left).right).right).right).right).value)
                    .clone(),
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
            __enc47
        };
        let __obj48: jni::sys::jlong = {
            let __enc48 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).left).left).left).value).clone(),
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
            __enc48
        };
        let __obj49: jni::sys::jlong = {
            let __enc49 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).left).left).right).value).clone(),
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
            __enc49
        };
        let __obj50: jni::sys::jlong = {
            let __enc50 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).left).right).left).value).clone(),
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
            __enc50
        };
        let __obj51: jni::sys::jlong = {
            let __enc51 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).left).right).right).value).clone(),
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
            __enc51
        };
        let __obj52: jni::sys::jlong = {
            let __enc52 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).right).left).left).value).clone(),
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
            __enc52
        };
        let __obj53: jni::sys::jlong = {
            let __enc53 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).right).left).right).value).clone(),
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
            __enc53
        };
        let __obj54: jni::sys::jlong = {
            let __enc54 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).right).right).left).value).clone(),
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
            __enc54
        };
        let __obj55: jni::sys::jlong = {
            let __enc55 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).left).right).right).right).value)
                    .clone(),
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
            __enc55
        };
        let __obj56: jni::sys::jlong = {
            let __enc56 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).left).left).left).value).clone(),
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
            __enc56
        };
        let __obj57: jni::sys::jlong = {
            let __enc57 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).left).left).right).value).clone(),
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
            __enc57
        };
        let __obj58: jni::sys::jlong = {
            let __enc58 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).left).right).left).value).clone(),
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
            __enc58
        };
        let __obj59: jni::sys::jlong = {
            let __enc59 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).left).right).right).value)
                    .clone(),
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
            __enc59
        };
        let __obj60: jni::sys::jlong = {
            let __enc60 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).right).left).left).value).clone(),
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
            __enc60
        };
        let __obj61: jni::sys::jlong = {
            let __enc61 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).right).left).right).value)
                    .clone(),
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
            __enc61
        };
        let __obj62: jni::sys::jlong = {
            let __enc62 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).right).right).left).value)
                    .clone(),
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
            __enc62
        };
        let __obj63: jni::sys::jlong = {
            let __enc63 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&(&(&(&v).right).right).right).right).right).right).value)
                    .clone(),
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
            __enc63
        };
        let __obj = env
            .call_static_method(
                "io/prebindgen/perftest/ObjectBoundary64Object",
                "fromParts",
                "(JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ)Lio/prebindgen/perftest/ObjectBoundary64Object;",
                &[
                    jni::objects::JValue::from(__obj0),
                    jni::objects::JValue::from(__obj1),
                    jni::objects::JValue::from(__obj2),
                    jni::objects::JValue::from(__obj3),
                    jni::objects::JValue::from(__obj4),
                    jni::objects::JValue::from(__obj5),
                    jni::objects::JValue::from(__obj6),
                    jni::objects::JValue::from(__obj7),
                    jni::objects::JValue::from(__obj8),
                    jni::objects::JValue::from(__obj9),
                    jni::objects::JValue::from(__obj10),
                    jni::objects::JValue::from(__obj11),
                    jni::objects::JValue::from(__obj12),
                    jni::objects::JValue::from(__obj13),
                    jni::objects::JValue::from(__obj14),
                    jni::objects::JValue::from(__obj15),
                    jni::objects::JValue::from(__obj16),
                    jni::objects::JValue::from(__obj17),
                    jni::objects::JValue::from(__obj18),
                    jni::objects::JValue::from(__obj19),
                    jni::objects::JValue::from(__obj20),
                    jni::objects::JValue::from(__obj21),
                    jni::objects::JValue::from(__obj22),
                    jni::objects::JValue::from(__obj23),
                    jni::objects::JValue::from(__obj24),
                    jni::objects::JValue::from(__obj25),
                    jni::objects::JValue::from(__obj26),
                    jni::objects::JValue::from(__obj27),
                    jni::objects::JValue::from(__obj28),
                    jni::objects::JValue::from(__obj29),
                    jni::objects::JValue::from(__obj30),
                    jni::objects::JValue::from(__obj31),
                    jni::objects::JValue::from(__obj32),
                    jni::objects::JValue::from(__obj33),
                    jni::objects::JValue::from(__obj34),
                    jni::objects::JValue::from(__obj35),
                    jni::objects::JValue::from(__obj36),
                    jni::objects::JValue::from(__obj37),
                    jni::objects::JValue::from(__obj38),
                    jni::objects::JValue::from(__obj39),
                    jni::objects::JValue::from(__obj40),
                    jni::objects::JValue::from(__obj41),
                    jni::objects::JValue::from(__obj42),
                    jni::objects::JValue::from(__obj43),
                    jni::objects::JValue::from(__obj44),
                    jni::objects::JValue::from(__obj45),
                    jni::objects::JValue::from(__obj46),
                    jni::objects::JValue::from(__obj47),
                    jni::objects::JValue::from(__obj48),
                    jni::objects::JValue::from(__obj49),
                    jni::objects::JValue::from(__obj50),
                    jni::objects::JValue::from(__obj51),
                    jni::objects::JValue::from(__obj52),
                    jni::objects::JValue::from(__obj53),
                    jni::objects::JValue::from(__obj54),
                    jni::objects::JValue::from(__obj55),
                    jni::objects::JValue::from(__obj56),
                    jni::objects::JValue::from(__obj57),
                    jni::objects::JValue::from(__obj58),
                    jni::objects::JValue::from(__obj59),
                    jni::objects::JValue::from(__obj60),
                    jni::objects::JValue::from(__obj61),
                    jni::objects::JValue::from(__obj62),
                    jni::objects::JValue::from(__obj63),
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary8_jni_product_intermediate_tuple_4e665030ef655cf9<
    'env,
    'a,
>(
    env: &mut jni::JNIEnv<'env>,
    v: (
        (
            ((jni::sys::jlong,), (jni::sys::jlong,)),
            ((jni::sys::jlong,), (jni::sys::jlong,)),
        ),
        (
            ((jni::sys::jlong,), (jni::sys::jlong,)),
            ((jni::sys::jlong,), (jni::sys::jlong,)),
        ),
    ),
) -> ::core::result::Result<perftest_flat::ObjectBoundary8, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::ObjectBoundary8 {
        left: __jni_in_convert_wire_to_ObjectBoundary4_jni_product_intermediate_tuple_e940f3772c7beded(
            env,
            (v).0,
        )?,
        right: __jni_in_convert_wire_to_ObjectBoundary4_jni_product_intermediate_tuple_e940f3772c7beded(
            env,
            (v).1,
        )?,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundary8_cca4b20df6695267<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<perftest_flat::ObjectBoundary8, __JniErr> {
    Ok({
        let __left_raw: jni::objects::JObject = env
            .get_field(v, "left", "Lio/prebindgen/perftest/ObjectBoundary4;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary8.left: {}", e)))?;
        let __right_raw: jni::objects::JObject = env
            .get_field(v, "right", "Lio/prebindgen/perftest/ObjectBoundary4;")
            .and_then(|val| val.l())
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("ObjectBoundary8.right: {}", e)))?;
        perftest_flat::ObjectBoundary8 {
            left: __jni_in_convert_wire_to_ObjectBoundary4_feb5834d3248e52f(
                env,
                &__left_raw,
            )?,
            right: __jni_in_convert_wire_to_ObjectBoundary4_feb5834d3248e52f(
                env,
                &__right_raw,
            )?,
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
pub(crate) unsafe fn __jni_out_convert_ObjectBoundary8_to_wire_98f33cf06147b9ce<'a>(
    mut env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundary8,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __obj0: jni::sys::jlong = {
            let __enc0 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&v).left).left).left).value).clone(),
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
            __enc0
        };
        let __obj1: jni::sys::jlong = {
            let __enc1 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&v).left).left).right).value).clone(),
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
            __enc1
        };
        let __obj2: jni::sys::jlong = {
            let __enc2 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&v).left).right).left).value).clone(),
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
        let __obj3: jni::sys::jlong = {
            let __enc3 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&v).left).right).right).value).clone(),
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
            __enc3
        };
        let __obj4: jni::sys::jlong = {
            let __enc4 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&v).right).left).left).value).clone(),
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
            __enc4
        };
        let __obj5: jni::sys::jlong = {
            let __enc5 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&v).right).left).right).value).clone(),
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
            __enc5
        };
        let __obj6: jni::sys::jlong = {
            let __enc6 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&v).right).right).left).value).clone(),
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
            __enc6
        };
        let __obj7: jni::sys::jlong = {
            let __enc7 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&(&(&(&v).right).right).right).value).clone(),
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
            __enc7
        };
        let __obj = env
            .call_static_method(
                "io/prebindgen/perftest/ObjectBoundary8",
                "fromParts",
                "(JJJJJJJJ)Lio/prebindgen/perftest/ObjectBoundary8;",
                &[
                    jni::objects::JValue::from(__obj0),
                    jni::objects::JValue::from(__obj1),
                    jni::objects::JValue::from(__obj2),
                    jni::objects::JValue::from(__obj3),
                    jni::objects::JValue::from(__obj4),
                    jni::objects::JValue::from(__obj5),
                    jni::objects::JValue::from(__obj6),
                    jni::objects::JValue::from(__obj7),
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundaryLeaf_jni_product_intermediate_tuple_1a6620ad58b014cf<
    'env,
    'a,
>(
    env: &mut jni::JNIEnv<'env>,
    v: (jni::sys::jlong,),
) -> ::core::result::Result<perftest_flat::ObjectBoundaryLeaf, __JniErr> {
    ::core::result::Result::Ok(perftest_flat::ObjectBoundaryLeaf {
        value: __jni_in_convert_wire_to_i64_da07d745d9e26f71(env, &((v).0))?,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_ObjectBoundaryLeaf_3132a7517e2ab837<
    'env,
    'v,
>(
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
        perftest_flat::ObjectBoundaryLeaf {
            value: __jni_in_convert_wire_to_i64_da07d745d9e26f71(env, &__value_raw)?,
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
pub(crate) unsafe fn __jni_out_convert_ObjectBoundaryLeaf_to_wire_5fe53ac5e29e73ac<'a>(
    mut env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::ObjectBoundaryLeaf,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __obj0: jni::sys::jlong = {
            let __enc0 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&v).value).clone(),
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
            __enc0
        };
        let __obj = env
            .call_static_method(
                "io/prebindgen/perftest/ObjectBoundaryLeaf",
                "fromParts",
                "(J)Lio/prebindgen/perftest/ObjectBoundaryLeaf;",
                &[jni::objects::JValue::from(__obj0)],
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
pub(crate) unsafe fn __jni_in_convert_wire_to_Option_Box_String_jni_optional_intermediate_input_niche_87b03b4201168b29<
    'env,
    'v,
>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JString<'v>,
) -> ::core::result::Result<
    ::core::option::Option<::std::boxed::Box<::std::string::String>>,
    __JniErr,
> {
    ::core::result::Result::Ok({
        if v.is_null() {
            ::core::option::Option::None
        } else {
            let __present = v;
            ::core::option::Option::Some(
                __jni_in_convert_wire_to_Box_String_7bf3c88ef26eb8e6(env, __present)?,
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
pub(crate) unsafe fn __jni_out_convert_Option_Box_String_jni_optional_intermediate_output_niche_to_wire_57342b1f497b4507<
    'a,
>(
    env: &mut jni::JNIEnv<'a>,
    v: ::core::option::Option<::std::boxed::Box<::std::string::String>>,
) -> ::core::result::Result<jni::objects::JString<'a>, __JniErr> {
    ::core::result::Result::Ok({
        match v {
            ::core::option::Option::Some(__value) => {
                __jni_out_convert_Box_String_to_wire_445d29257759cad9(env, __value)?
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
pub(crate) unsafe fn __jni_out_convert_Option_Payload_jni_optional_intermediate_output_gated_to_wire_730d90b5c47c15bd<
    'a,
>(
    env: &mut jni::JNIEnv<'a>,
    v: ::core::option::Option<perftest_flat::Payload>,
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
                (
                    1u8,
                    __jni_out_convert_Payload_jni_product_intermediate_tuple_to_wire_eec5c9986df64b1f(
                        env,
                        __value,
                    )?,
                )
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
#[inline(always)]
pub(crate) unsafe fn __jni_out_convert_Payload_jni_product_intermediate_tuple_to_wire_eec5c9986df64b1f<
    'a,
>(
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
        __jni_out_convert_i64_to_wire_15d458bf28dc9c80(env, v.id)?,
        __jni_out_convert_i32_to_wire_67173b19ae5a9348(env, v.seq)?,
        __jni_out_convert_f64_to_wire_61461de12ea6bc04(env, v.value)?,
        __jni_out_convert_bool_to_wire_3ee62077915d5228(env, v.flag)?,
        __jni_out_convert_Option_Box_String_jni_optional_intermediate_output_niche_to_wire_57342b1f497b4507(
            env,
            v.label,
        )?,
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
pub(crate) unsafe fn __jni_out_convert_Payload_to_wire_69366211464f4172<'a>(
    mut env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Payload,
) -> ::core::result::Result<jni::objects::JObject<'a>, __JniErr> {
    Ok({
        let __obj0: jni::sys::jlong = {
            let __enc0 = match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(
                &mut env,
                (&(&v).id).clone(),
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
            __enc0
        };
        let __obj1: jni::sys::jint = {
            let __enc1 = match __jni_out_convert_i32_to_wire_67173b19ae5a9348(
                &mut env,
                (&(&v).seq).clone(),
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
            __enc1
        };
        let __obj2: jni::sys::jdouble = {
            let __enc2 = match __jni_out_convert_f64_to_wire_61461de12ea6bc04(
                &mut env,
                (&(&v).value).clone(),
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
        let __obj3: jni::sys::jboolean = {
            let __enc3 = match __jni_out_convert_bool_to_wire_3ee62077915d5228(
                &mut env,
                (&(&v).flag).clone(),
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
            __enc3
        };
        let __obj4: jni::objects::JObject = {
            let __enc4 = match __jni_out_convert_Option_Box_String_jni_optional_intermediate_output_niche_to_wire_57342b1f497b4507(
                &mut env,
                (&(&v).label).clone(),
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
        let __obj = env
            .call_static_method(
                "io/prebindgen/perftest/Payload",
                "fromParts",
                "(JIDZLjava/lang/String;)Lio/prebindgen/perftest/Payload;",
                &[
                    jni::objects::JValue::from(__obj0),
                    jni::objects::JValue::from(__obj1),
                    jni::objects::JValue::from(__obj2),
                    jni::objects::JValue::from(__obj3),
                    jni::objects::JValue::Object(&__obj4),
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
pub(crate) unsafe fn __jni_out_convert_PayloadHandler_jni_handle_codec_own_output_to_wire_43900b235bf7afc4<
    'a,
>(
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
pub(crate) unsafe fn __jni_out_convert_PayloadVecHandler_jni_handle_codec_own_output_to_wire_43a858e44952716c<
    'a,
>(
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
pub(crate) unsafe fn __jni_out_convert_Storage_jni_handle_codec_own_output_to_wire_056c9dbddefcfc91<
    'a,
>(
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
pub(crate) unsafe fn __jni_out_convert_Token_jni_handle_codec_own_output_to_wire_b192d32dc3b323dd<
    'a,
>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::Token,
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
pub(crate) unsafe fn __jni_out_convert_TokenGc_jni_handle_codec_own_output_to_wire_57de0a29947b0f2f<
    'a,
>(
    env: &mut jni::JNIEnv<'a>,
    v: perftest_flat::TokenGc,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_bool_1be2f6c32f925207<'env, 'v>(
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
pub(crate) unsafe fn __jni_out_convert_bool_to_wire_3ee62077915d5228<'a>(
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
pub(crate) unsafe fn __jni_in_convert_wire_to_f64_b312e1b95182cdfd<'env, 'v>(
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
pub(crate) unsafe fn __jni_out_convert_f64_to_wire_61461de12ea6bc04<'a>(
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
pub(crate) unsafe fn __jni_in_convert_wire_to_i32_83b133e23cc76fc5<'env, 'v>(
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
pub(crate) unsafe fn __jni_out_convert_i32_to_wire_67173b19ae5a9348<'a>(
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
pub(crate) unsafe fn __jni_in_convert_wire_to_i64_da07d745d9e26f71<'env, 'v>(
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
pub(crate) unsafe fn __jni_out_convert_i64_to_wire_15d458bf28dc9c80<'a>(
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
pub(crate) unsafe fn __jni_in_convert_wire_to_impl_Fn_Payload_Send_Sync_static_edc6f00a317f99b5<
    'env,
    'v,
>(
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
                    ) = match __jni_out_convert_Payload_jni_product_intermediate_tuple_to_wire_c5a1c01cf2cbf49b(
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
pub(crate) unsafe fn __jni_in_convert_wire_to_impl_Fn_Payload_Send_Sync_static_d376cf1bd7477f3d<
    'env,
    'v,
>(
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
                .find_class("io/prebindgen/perftest/__PayloadFolderRawHolder")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        "find folder holder {}: {}",
                        "io/prebindgen/perftest/__PayloadFolderRawHolder", e
                    ),
                ))?;
            let __field = env
                .get_static_field(
                    &__cls,
                    "instance",
                    "Lio/prebindgen/perftest/PayloadFolderRaw;",
                )
                .and_then(|__v| __v.l())
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        "fetch folder singleton {}.{}: {}",
                        "io/prebindgen/perftest/__PayloadFolderRawHolder", "instance", e
                    ),
                ))?;
            env.new_global_ref(&__field)
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(format!("global-ref folder singleton: {}", e)))?
        };
        let __fold0_id = {
            let __cls = env
                .find_class("io/prebindgen/perftest/PayloadFolderRaw")
                .map_err(|e| <__JniErr as ::core::convert::From<
                    String,
                >>::from(
                    format!(
                        "find folder iface {}: {}",
                        "io/prebindgen/perftest/PayloadFolderRaw", e
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
                        "io/prebindgen/perftest/PayloadFolderRaw", e
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
                            ) = match __jni_out_convert_Payload_jni_product_intermediate_tuple_to_wire_c5a1c01cf2cbf49b(
                                &mut env,
                                __cb_elem,
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
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_largeFlatInputSum<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value_left_left_left_left_left_left_value: jni::sys::jlong,
    value_left_left_left_left_left_right_value: jni::sys::jlong,
    value_left_left_left_left_right_left_value: jni::sys::jlong,
    value_left_left_left_left_right_right_value: jni::sys::jlong,
    value_left_left_left_right_left_left_value: jni::sys::jlong,
    value_left_left_left_right_left_right_value: jni::sys::jlong,
    value_left_left_left_right_right_left_value: jni::sys::jlong,
    value_left_left_left_right_right_right_value: jni::sys::jlong,
    value_left_left_right_left_left_left_value: jni::sys::jlong,
    value_left_left_right_left_left_right_value: jni::sys::jlong,
    value_left_left_right_left_right_left_value: jni::sys::jlong,
    value_left_left_right_left_right_right_value: jni::sys::jlong,
    value_left_left_right_right_left_left_value: jni::sys::jlong,
    value_left_left_right_right_left_right_value: jni::sys::jlong,
    value_left_left_right_right_right_left_value: jni::sys::jlong,
    value_left_left_right_right_right_right_value: jni::sys::jlong,
    value_left_right_left_left_left_left_value: jni::sys::jlong,
    value_left_right_left_left_left_right_value: jni::sys::jlong,
    value_left_right_left_left_right_left_value: jni::sys::jlong,
    value_left_right_left_left_right_right_value: jni::sys::jlong,
    value_left_right_left_right_left_left_value: jni::sys::jlong,
    value_left_right_left_right_left_right_value: jni::sys::jlong,
    value_left_right_left_right_right_left_value: jni::sys::jlong,
    value_left_right_left_right_right_right_value: jni::sys::jlong,
    value_left_right_right_left_left_left_value: jni::sys::jlong,
    value_left_right_right_left_left_right_value: jni::sys::jlong,
    value_left_right_right_left_right_left_value: jni::sys::jlong,
    value_left_right_right_left_right_right_value: jni::sys::jlong,
    value_left_right_right_right_left_left_value: jni::sys::jlong,
    value_left_right_right_right_left_right_value: jni::sys::jlong,
    value_left_right_right_right_right_left_value: jni::sys::jlong,
    value_left_right_right_right_right_right_value: jni::sys::jlong,
    value_right_left_left_left_left_left_value: jni::sys::jlong,
    value_right_left_left_left_left_right_value: jni::sys::jlong,
    value_right_left_left_left_right_left_value: jni::sys::jlong,
    value_right_left_left_left_right_right_value: jni::sys::jlong,
    value_right_left_left_right_left_left_value: jni::sys::jlong,
    value_right_left_left_right_left_right_value: jni::sys::jlong,
    value_right_left_left_right_right_left_value: jni::sys::jlong,
    value_right_left_left_right_right_right_value: jni::sys::jlong,
    value_right_left_right_left_left_left_value: jni::sys::jlong,
    value_right_left_right_left_left_right_value: jni::sys::jlong,
    value_right_left_right_left_right_left_value: jni::sys::jlong,
    value_right_left_right_left_right_right_value: jni::sys::jlong,
    value_right_left_right_right_left_left_value: jni::sys::jlong,
    value_right_left_right_right_left_right_value: jni::sys::jlong,
    value_right_left_right_right_right_left_value: jni::sys::jlong,
    value_right_left_right_right_right_right_value: jni::sys::jlong,
    value_right_right_left_left_left_left_value: jni::sys::jlong,
    value_right_right_left_left_left_right_value: jni::sys::jlong,
    value_right_right_left_left_right_left_value: jni::sys::jlong,
    value_right_right_left_left_right_right_value: jni::sys::jlong,
    value_right_right_left_right_left_left_value: jni::sys::jlong,
    value_right_right_left_right_left_right_value: jni::sys::jlong,
    value_right_right_left_right_right_left_value: jni::sys::jlong,
    value_right_right_left_right_right_right_value: jni::sys::jlong,
    value_right_right_right_left_left_left_value: jni::sys::jlong,
    value_right_right_right_left_left_right_value: jni::sys::jlong,
    value_right_right_right_left_right_left_value: jni::sys::jlong,
    value_right_right_right_left_right_right_value: jni::sys::jlong,
    value_right_right_right_right_left_left_value: jni::sys::jlong,
    value_right_right_right_right_left_right_value: jni::sys::jlong,
    value_right_right_right_right_right_left_value: jni::sys::jlong,
    value_right_right_right_right_right_right_value: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let value = match __jni_in_convert_wire_to_ObjectBoundary64_jni_product_intermediate_tuple_0a10a5cc1f8d19f9(
        &mut env,
        (
            (
                (
                    (
                        (
                            (
                                (value_left_left_left_left_left_left_value,),
                                (value_left_left_left_left_left_right_value,),
                            ),
                            (
                                (value_left_left_left_left_right_left_value,),
                                (value_left_left_left_left_right_right_value,),
                            ),
                        ),
                        (
                            (
                                (value_left_left_left_right_left_left_value,),
                                (value_left_left_left_right_left_right_value,),
                            ),
                            (
                                (value_left_left_left_right_right_left_value,),
                                (value_left_left_left_right_right_right_value,),
                            ),
                        ),
                    ),
                    (
                        (
                            (
                                (value_left_left_right_left_left_left_value,),
                                (value_left_left_right_left_left_right_value,),
                            ),
                            (
                                (value_left_left_right_left_right_left_value,),
                                (value_left_left_right_left_right_right_value,),
                            ),
                        ),
                        (
                            (
                                (value_left_left_right_right_left_left_value,),
                                (value_left_left_right_right_left_right_value,),
                            ),
                            (
                                (value_left_left_right_right_right_left_value,),
                                (value_left_left_right_right_right_right_value,),
                            ),
                        ),
                    ),
                ),
                (
                    (
                        (
                            (
                                (value_left_right_left_left_left_left_value,),
                                (value_left_right_left_left_left_right_value,),
                            ),
                            (
                                (value_left_right_left_left_right_left_value,),
                                (value_left_right_left_left_right_right_value,),
                            ),
                        ),
                        (
                            (
                                (value_left_right_left_right_left_left_value,),
                                (value_left_right_left_right_left_right_value,),
                            ),
                            (
                                (value_left_right_left_right_right_left_value,),
                                (value_left_right_left_right_right_right_value,),
                            ),
                        ),
                    ),
                    (
                        (
                            (
                                (value_left_right_right_left_left_left_value,),
                                (value_left_right_right_left_left_right_value,),
                            ),
                            (
                                (value_left_right_right_left_right_left_value,),
                                (value_left_right_right_left_right_right_value,),
                            ),
                        ),
                        (
                            (
                                (value_left_right_right_right_left_left_value,),
                                (value_left_right_right_right_left_right_value,),
                            ),
                            (
                                (value_left_right_right_right_right_left_value,),
                                (value_left_right_right_right_right_right_value,),
                            ),
                        ),
                    ),
                ),
            ),
            (
                (
                    (
                        (
                            (
                                (value_right_left_left_left_left_left_value,),
                                (value_right_left_left_left_left_right_value,),
                            ),
                            (
                                (value_right_left_left_left_right_left_value,),
                                (value_right_left_left_left_right_right_value,),
                            ),
                        ),
                        (
                            (
                                (value_right_left_left_right_left_left_value,),
                                (value_right_left_left_right_left_right_value,),
                            ),
                            (
                                (value_right_left_left_right_right_left_value,),
                                (value_right_left_left_right_right_right_value,),
                            ),
                        ),
                    ),
                    (
                        (
                            (
                                (value_right_left_right_left_left_left_value,),
                                (value_right_left_right_left_left_right_value,),
                            ),
                            (
                                (value_right_left_right_left_right_left_value,),
                                (value_right_left_right_left_right_right_value,),
                            ),
                        ),
                        (
                            (
                                (value_right_left_right_right_left_left_value,),
                                (value_right_left_right_right_left_right_value,),
                            ),
                            (
                                (value_right_left_right_right_right_left_value,),
                                (value_right_left_right_right_right_right_value,),
                            ),
                        ),
                    ),
                ),
                (
                    (
                        (
                            (
                                (value_right_right_left_left_left_left_value,),
                                (value_right_right_left_left_left_right_value,),
                            ),
                            (
                                (value_right_right_left_left_right_left_value,),
                                (value_right_right_left_left_right_right_value,),
                            ),
                        ),
                        (
                            (
                                (value_right_right_left_right_left_left_value,),
                                (value_right_right_left_right_left_right_value,),
                            ),
                            (
                                (value_right_right_left_right_right_left_value,),
                                (value_right_right_left_right_right_right_value,),
                            ),
                        ),
                    ),
                    (
                        (
                            (
                                (value_right_right_right_left_left_left_value,),
                                (value_right_right_right_left_left_right_value,),
                            ),
                            (
                                (value_right_right_right_left_right_left_value,),
                                (value_right_right_right_left_right_right_value,),
                            ),
                        ),
                        (
                            (
                                (value_right_right_right_right_left_left_value,),
                                (value_right_right_right_right_left_right_value,),
                            ),
                            (
                                (value_right_right_right_right_right_left_value,),
                                (value_right_right_right_right_right_right_value,),
                            ),
                        ),
                    ),
                ),
            ),
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
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::large_flat_input_sum(&value);
    match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(&mut env, __out) {
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_largeObjectInputSum<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let value = match __jni_in_convert_wire_to_ObjectBoundary64Object_3c7cf869ff4a3b45(
        &mut env,
        &value,
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
    let __out = perftest_flat::large_object_input_sum(&value);
    match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(&mut env, __out) {
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_payloadHandlerNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    f: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let f = match __jni_in_convert_wire_to_impl_Fn_Payload_Send_Sync_static_edc6f00a317f99b5(
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
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::payload_handler_new(f);
    match __jni_out_convert_PayloadHandler_jni_handle_codec_own_output_to_wire_43900b235bf7afc4(
        &mut env,
        __out,
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
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_payloadVecHandlerNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    f: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let f = match __jni_in_convert_wire_to_impl_Fn_Payload_Send_Sync_static_d376cf1bd7477f3d(
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
            return 0 as jni::sys::jlong;
        }
    };
    let __out = perftest_flat::payload_vec_handler_new(f);
    match __jni_out_convert_PayloadVecHandler_jni_handle_codec_own_output_to_wire_43a858e44952716c(
        &mut env,
        __out,
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
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storageCallback<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    handler: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match __jni_in_convert_wire_to_Storage_jni_handle_codec_borrow_input_697104a332693566(
        &mut env,
        &s,
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
    let handler = match __jni_in_convert_wire_to_PayloadHandler_jni_handle_codec_borrow_input_f89cfeecbb4e240b(
        &mut env,
        &handler,
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
    let __out = perftest_flat::storage_callback(&s, &handler);
    match __jni_out_convert_unit_to_wire_9e1510fd173c1fd6(&mut env, __out) {
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storageCallbackVec<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    handler: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match __jni_in_convert_wire_to_Storage_jni_handle_codec_borrow_input_697104a332693566(
        &mut env,
        &s,
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
    let handler = match __jni_in_convert_wire_to_PayloadVecHandler_jni_handle_codec_borrow_input_1b365539726eca03(
        &mut env,
        &handler,
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
    let __out = perftest_flat::storage_callback_vec(&s, &handler);
    match __jni_out_convert_unit_to_wire_9e1510fd173c1fd6(&mut env, __out) {
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storageGet<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __builder: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match __jni_in_convert_wire_to_Storage_jni_handle_codec_borrow_input_697104a332693566(
        &mut env,
        &s,
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
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/perftest/PayloadBuilder";
    const __CB_DESCR: &str = "(JIDZLjava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::storage_get(&s);
    let (
        __chain_present,
        (__chain_wire0, __chain_wire1, __chain_wire2, __chain_wire3, __chain_wire4),
    ) = match __jni_out_convert_Option_Payload_jni_optional_intermediate_output_gated_to_wire_730d90b5c47c15bd(
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storageGetVec<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    __acc: jni::objects::JObject<'a>,
    __fold: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::objects::JObject<'a> {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let s = match __jni_in_convert_wire_to_Storage_jni_handle_codec_borrow_input_697104a332693566(
        &mut env,
        &s,
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
    #[allow(non_upper_case_globals)]
    static __CB_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __CB_FQN: &str = "io/prebindgen/perftest/PayloadFolderRaw";
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
                ) = match __jni_out_convert_Payload_jni_product_intermediate_tuple_to_wire_eec5c9986df64b1f(
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storageNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let __out = perftest_flat::storage_new();
    match __jni_out_convert_Storage_jni_handle_codec_own_output_to_wire_056c9dbddefcfc91(
        &mut env,
        __out,
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
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storagePutByRead<'a>(
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
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match __jni_in_convert_wire_to_Storage_jni_handle_codec_borrow_input_697104a332693566(
        &mut env,
        &s,
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
    let payload = match __jni_in_convert_wire_to_Payload_jni_product_intermediate_tuple_f379fad611b26734(
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
    match __jni_out_convert_unit_to_wire_9e1510fd173c1fd6(&mut env, __out) {
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storagePutByTake<'a>(
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
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match __jni_in_convert_wire_to_Storage_jni_handle_codec_borrow_input_697104a332693566(
        &mut env,
        &s,
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
    let payload = match __jni_in_convert_wire_to_Payload_jni_product_intermediate_tuple_f379fad611b26734(
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
    match __jni_out_convert_unit_to_wire_9e1510fd173c1fd6(&mut env, __out) {
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storagePutSlice<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    s: jni::sys::jlong,
    payloads_handle: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let mut s = match __jni_in_convert_wire_to_Storage_jni_handle_codec_borrow_input_697104a332693566(
        &mut env,
        &s,
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
    let payloads = unsafe {
        OwnedObject::from_raw(payloads_handle as *const Vec<perftest_flat::Payload>)
    };
    let __out = perftest_flat::storage_put_slice(&mut s, &payloads);
    match __jni_out_convert_unit_to_wire_9e1510fd173c1fd6(&mut env, __out) {
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_tokenGcNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let value = match __jni_in_convert_wire_to_i64_da07d745d9e26f71(&mut env, &value) {
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
    let __out = perftest_flat::token_gc_new(value);
    match __jni_out_convert_TokenGc_jni_handle_codec_own_output_to_wire_57de0a29947b0f2f(
        &mut env,
        __out,
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
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_tokenGcValue<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    t: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let t = match __jni_in_convert_wire_to_TokenGc_jni_handle_codec_borrow_input_7a9be906e3827840(
        &mut env,
        &t,
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
    let __out = perftest_flat::token_gc_value(&t);
    match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(&mut env, __out) {
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_tokenNew<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    value: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let value = match __jni_in_convert_wire_to_i64_da07d745d9e26f71(&mut env, &value) {
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
    let __out = perftest_flat::token_new(value);
    match __jni_out_convert_Token_jni_handle_codec_own_output_to_wire_b192d32dc3b323dd(
        &mut env,
        __out,
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
            0 as jni::sys::jlong
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_tokenValue<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    t: jni::sys::jlong,
    __error_sink: jni::objects::JObject<'a>,
) -> jni::sys::jlong {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let t = match __jni_in_convert_wire_to_Token_jni_handle_codec_borrow_input_52a62575119c2292(
        &mut env,
        &t,
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
    let __out = perftest_flat::token_value(&t);
    match __jni_out_convert_i64_to_wire_15d458bf28dc9c80(&mut env, __out) {
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
const _: () = {
    konst::assertc_eq!(
        perftest_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
