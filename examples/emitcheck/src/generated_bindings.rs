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
pub(crate) unsafe extern "C" fn Java_io_prebindgen_emitcheck_ZKeyExpr_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut myflat::ZKeyExpr));
    }
}
const _: () = {
    if ::core::mem::align_of::<myflat::ZKeyExpr>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub(crate) unsafe extern "C" fn Java_io_prebindgen_emitcheck_ZSample_freePtr(
    _env: jni::JNIEnv,
    _class: jni::objects::JClass,
    ptr: jni::sys::jlong,
) {
    if ptr != 0 && (ptr & 1) == 0 {
        drop(Box::from_raw(ptr as *mut myflat::ZSample));
    }
}
const _: () = {
    if ::core::mem::align_of::<myflat::ZSample>() < 2 {
        panic!("opaque handle types must have alignment >= 2 (bit 0 is the closed tag)");
    }
};
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    dead_code,
    clippy::useless_conversion,
    clippy::needless_question_mark,
    clippy::let_and_return,
    clippy::nonminimal_bool,
    clippy::eq_op
)]
pub(crate) unsafe fn __jni_out_convert_jni_text_codec_borrowed_to_wire_bc4fe45698de9c2e<
    'a,
>(
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
pub(crate) unsafe fn __jni_out_convert_Cow_static_u8_to_wire_d9a36c77f96791fc<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: ::std::borrow::Cow<'static, [u8]>,
) -> ::core::result::Result<jni::objects::JByteArray<'a>, __JniErr> {
    Ok({
        env.byte_array_from_slice(&v)
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
pub(crate) unsafe fn __jni_out_convert_Cow_static_str_to_wire_1c9aa86df48df08e<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: ::std::borrow::Cow<'static, str>,
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
pub(crate) unsafe fn __jni_out_convert_jni_text_codec_owned_to_wire_1b6cdff0ec9adbcb<'a>(
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
pub(crate) unsafe fn __jni_out_convert_Vec_u8_to_wire_e9499bf0a706b1a2<'a>(
    env: &mut jni::JNIEnv<'a>,
    v: ::std::vec::Vec<u8>,
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
pub(crate) unsafe fn __jni_in_convert_wire_to_impl_Fn_ZSample_Send_Sync_static_79b89d8516de6259<
    'env,
    'v,
>(
    env: &mut jni::JNIEnv<'env>,
    v: &jni::objects::JObject<'v>,
) -> ::core::result::Result<impl Fn(myflat::ZSample) + Send + Sync + 'static, __JniErr> {
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
                format!("Unable to get callback class for {}: {}", "Fn(ZSample)", e),
            ))?;
        let __invoke_id = env
            .get_method_id(
                &__invoke_class,
                "run",
                "(Ljava/lang/String;Ljava/lang/String;[B[BLjava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
            )
            .map_err(|e| <__JniErr as ::core::convert::From<
                String,
            >>::from(format!("Unable to resolve run for {}: {}", "Fn(ZSample)", e)))?;
        Box::new(move |__cb_arg0: myflat::ZSample| {
            let _ = (|| -> ::core::result::Result<(), __JniErr> {
                let mut env = java_vm
                    .attach_current_thread_as_daemon()
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("Attach thread for {}: {}", "Fn(ZSample)", e)))?;
                env.push_local_frame(20)
                    .map_err(|e| <__JniErr as ::core::convert::From<
                        String,
                    >>::from(format!("push local frame for {}: {}", "Fn(ZSample)", e)))?;
                let __frame_res = (|| -> ::core::result::Result<(), __JniErr> {
                    let __vf0 = myflat::z_sample_to_struct(&__cb_arg0);
                    let __cb0_obj0: jni::objects::JObject = {
                        let __o0: &::core::option::Option<_> = &(&__vf0).opt_plain;
                        match __o0 {
                            ::core::option::Option::Some(__n0) => {
                                let __enc0 = match __jni_out_convert_jni_text_codec_borrowed_to_wire_bc4fe45698de9c2e(
                                    &mut env,
                                    myflat::z_keyexpr_as_str(__n0),
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
                                __enc0.into()
                            }
                            ::core::option::Option::None => jni::objects::JObject::null(),
                        }
                    };
                    let __cb0_obj1: jni::objects::JObject = {
                        let __o0: &::core::option::Option<_> = &(&__vf0).opt_boxed;
                        match __o0 {
                            ::core::option::Option::Some(__n0) => {
                                let __enc1 = match __jni_out_convert_jni_text_codec_borrowed_to_wire_bc4fe45698de9c2e(
                                    &mut env,
                                    myflat::z_keyexpr_as_str(__n0),
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
                                __enc1.into()
                            }
                            ::core::option::Option::None => jni::objects::JObject::null(),
                        }
                    };
                    let __cb0_obj2: jni::objects::JObject = {
                        let __enc2 = match __jni_out_convert_Vec_u8_to_wire_e9499bf0a706b1a2(
                            &mut env,
                            (&__vf0.seq_plain).clone(),
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
                        __enc2.into()
                    };
                    let __cb0_obj3: jni::objects::JObject = {
                        let __enc3 = match __jni_out_convert_Cow_static_u8_to_wire_d9a36c77f96791fc(
                            &mut env,
                            (&__vf0.seq_cow).clone(),
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
                        __enc3.into()
                    };
                    let __cb0_obj4: jni::objects::JObject = {
                        let __enc4 = match __jni_out_convert_jni_text_codec_owned_to_wire_1b6cdff0ec9adbcb(
                            &mut env,
                            (&__vf0.text_plain).clone(),
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
                    let __cb0_obj5: jni::objects::JObject = {
                        let __enc5 = match __jni_out_convert_Box_String_to_wire_445d29257759cad9(
                            &mut env,
                            (&__vf0.text_boxed).clone(),
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
                        __enc5.into()
                    };
                    let __cb0_obj6: jni::objects::JObject = {
                        let __enc6 = match __jni_out_convert_Cow_static_str_to_wire_1c9aa86df48df08e(
                            &mut env,
                            (&__vf0.text_cow).clone(),
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
                        __enc6.into()
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
                .map_err(|e| tracing::error!("{} callback error: {e}", "Fn(ZSample)"));
        })
    })
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, dead_code)]
pub unsafe extern "C" fn Java_io_prebindgen_emitcheck_JNINative_zSampleSub<'a>(
    mut env: jni::JNIEnv<'a>,
    _class: jni::objects::JClass<'a>,
    cb: jni::objects::JObject<'a>,
    __error_sink: jni::objects::JObject<'a>,
) -> () {
    #[allow(non_upper_case_globals)]
    static __SINK_MID: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    const __SINK_FQN: &str = "io/prebindgen/emitcheck/JniErrorHandler";
    const __SINK_DESCR: &str = "(Ljava/lang/String;)Ljava/lang/Object;";
    let cb = match __jni_in_convert_wire_to_impl_Fn_ZSample_Send_Sync_static_79b89d8516de6259(
        &mut env,
        &cb,
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
    let __out = myflat::z_sample_sub(cb);
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
