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
pub(crate) unsafe extern "C" fn Java_io_prebindgen_perftest_PayloadHandler_freePtr(
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
pub(crate) unsafe extern "C" fn Java_io_prebindgen_perftest_PayloadVecHandler_freePtr(
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
pub(crate) unsafe extern "C" fn Java_io_prebindgen_perftest_Storage_freePtr(
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
                "io/prebindgen/perftest/Payload",
                "fromParts",
                "(JIDZLjava/lang/String;)Lio/prebindgen/perftest/Payload;",
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
pub(crate) unsafe fn i32_to_jint_a3e3b6ef<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: i32,
) -> ::core::result::Result<jni::sys::jint, __JniErr> {
    Ok(v as jni::sys::jint)
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
pub(crate) unsafe fn jint_to_i32_a3e3b6ef<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jint,
) -> ::core::result::Result<i32, __JniErr> {
    Ok(*v)
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
pub(crate) unsafe fn jlong_to_Storage_1b233abd<'env, 'v>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::sys::jlong,
) -> ::core::result::Result<OwnedObject<perftest_flat::Storage>, __JniErr> {
    Ok(unsafe { OwnedObject::from_raw(*v as *const perftest_flat::Storage) })
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
pub(crate) unsafe fn unit_to_unit_9ecccf8e<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: (),
) -> ::core::result::Result<(), __JniErr> {
    Ok(v)
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_payloadHandlerNew<'a>(
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
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_payloadVecHandlerNew<'a>(
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
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storageCallback<'a>(
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
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storageCallbackVec<'a>(
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
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storageGet<'a>(
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
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
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
    const __CB_FQN: &str = "io/prebindgen/perftest/PayloadBuilder";
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storageGetVec<'a>(
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
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
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
    const __CB_FQN: &str = "io/prebindgen/perftest/PayloadFolderRaw";
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storageNew<'a>(
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
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
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
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
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
    #[allow(unused_variables)]
    let __ze_defaults = |env: &mut jni::JNIEnv| -> ::std::vec::Vec<jni::sys::jvalue> {
        ::std::vec![]
    };
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen::lang::CachedIfaceMethod = ::prebindgen::lang::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
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
pub unsafe extern "C" fn Java_io_prebindgen_perftest_JNINative_storagePutSlice<'a>(
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
    const __SINK_FQN: &str = "io/prebindgen/perftest/JniErrorHandler";
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
const _: () = {
    konst::assertc_eq!(
        perftest_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
