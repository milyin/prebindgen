/// Report a binding error to the Kotlin error sink, unless a
/// Kotlin exception is already pending.
#[allow(dead_code)]
fn __jni_signal(
    env: &mut ::prebindgen_jni_runtime::jni::JNIEnv,
    sink: &::prebindgen_jni_runtime::jni::objects::JObject,
    msg: &str,
) {
    static __M: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
    if env.exception_check().unwrap_or(false) {
        return;
    }
    let __s = match env.new_string(msg) {
        ::core::result::Result::Ok(s) => s,
        ::core::result::Result::Err(_) => return,
    };
    let _ = __M
        .call_object(
            env,
            "io/prebindgen/emitcheck/JniErrorHandler",
            "run",
            "(Ljava/lang/String;)Ljava/lang/Object;",
            sink,
            &[
                ::prebindgen_jni_runtime::jni::sys::jvalue {
                    l: __s.as_raw(),
                },
            ],
        );
}
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_emitcheck_ZKeyExpr_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<myflat::ZKeyExpr>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < myflat::ZKeyExpr > () >= 2,
    "`ZKeyExpr`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
);
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_emitcheck_ZSample_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<myflat::ZSample>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < myflat::ZSample > () >= 2,
    "`ZSample`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
);
#[no_mangle]
#[allow(
    non_snake_case,
    unused_mut,
    unused_variables,
    unused_braces,
    unused_parens,
    unused_unsafe,
    dead_code,
    clippy::all
)]
pub unsafe extern "system" fn Java_io_prebindgen_emitcheck_JNINative_zSampleSub<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    cb: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let cb = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __up = ::prebindgen_jni_runtime::Upcall::new(
                env,
                &cb,
                "run",
                "(ZLjava/lang/String;ZLjava/lang/String;[B[BLjava/lang/String;Ljava/lang/String;Ljava/lang/String;)V",
                50i32,
            )?;
            move |__a0: myflat::ZSample| {
                let __r = __up
                    .call_void(|env| {
                        let (
                            a0_opt_plain__present,
                            a0_opt_plain_z_keyexpr_as_str,
                            a0_opt_boxed__present,
                            a0_opt_boxed_z_keyexpr_as_str,
                            a0_seq_plain,
                            a0_seq_cow,
                            a0_text_plain,
                            a0_text_boxed,
                            a0_text_cow,
                        ) = {
                            let __v1 = __a0;
                            let __s1 = myflat::z_sample_to_struct(&__v1);
                            let (
                                a0_opt_plain__present,
                                a0_opt_plain_z_keyexpr_as_str,
                                a0_opt_boxed__present,
                                a0_opt_boxed_z_keyexpr_as_str,
                                a0_seq_plain,
                                a0_seq_cow,
                                a0_text_plain,
                                a0_text_boxed,
                                a0_text_cow,
                            ) = {
                                let myflat::ZSampleStruct {
                                    opt_plain: __f2_0,
                                    opt_boxed: __f2_1,
                                    seq_plain: __f2_2,
                                    seq_cow: __f2_3,
                                    text_plain: __f2_4,
                                    text_boxed: __f2_5,
                                    text_cow: __f2_6,
                                } = __s1;
                                let (
                                    a0_opt_plain__present,
                                    a0_opt_plain_z_keyexpr_as_str,
                                ) = match __f2_0 {
                                    ::core::option::Option::Some(__x3) => {
                                        let a0_opt_plain_z_keyexpr_as_str = {
                                            let __v4 = __x3;
                                            let a0_opt_plain_z_keyexpr_as_str = ::prebindgen_jni_runtime::new_string(
                                                env,
                                                ::core::convert::AsRef::<
                                                    str,
                                                >::as_ref(&myflat::z_keyexpr_as_str(&__v4)),
                                            )?;
                                            a0_opt_plain_z_keyexpr_as_str
                                        };
                                        (1u8, a0_opt_plain_z_keyexpr_as_str)
                                    }
                                    ::core::option::Option::None => {
                                        (
                                            0u8,
                                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                        )
                                    }
                                };
                                let (
                                    a0_opt_boxed__present,
                                    a0_opt_boxed_z_keyexpr_as_str,
                                ) = match (*__f2_1) {
                                    ::core::option::Option::Some(__x3) => {
                                        let a0_opt_boxed_z_keyexpr_as_str = {
                                            let __v4 = __x3;
                                            let a0_opt_boxed_z_keyexpr_as_str = ::prebindgen_jni_runtime::new_string(
                                                env,
                                                ::core::convert::AsRef::<
                                                    str,
                                                >::as_ref(&myflat::z_keyexpr_as_str(&__v4)),
                                            )?;
                                            a0_opt_boxed_z_keyexpr_as_str
                                        };
                                        (1u8, a0_opt_boxed_z_keyexpr_as_str)
                                    }
                                    ::core::option::Option::None => {
                                        (
                                            0u8,
                                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                        )
                                    }
                                };
                                let a0_seq_plain = ::prebindgen_jni_runtime::write_u8s(
                                    env,
                                    ::core::convert::AsRef::<[u8]>::as_ref(&__f2_2),
                                )?;
                                let a0_seq_cow = ::prebindgen_jni_runtime::write_u8s(
                                    env,
                                    ::core::convert::AsRef::<[u8]>::as_ref(&__f2_3),
                                )?;
                                let a0_text_plain = ::prebindgen_jni_runtime::new_string(
                                    env,
                                    ::core::convert::AsRef::<str>::as_ref(&__f2_4),
                                )?;
                                let a0_text_boxed = ::prebindgen_jni_runtime::new_string(
                                    env,
                                    ::core::convert::AsRef::<str>::as_ref(&(*__f2_5)),
                                )?;
                                let a0_text_cow = ::prebindgen_jni_runtime::new_string(
                                    env,
                                    ::core::convert::AsRef::<str>::as_ref(&__f2_6),
                                )?;
                                (
                                    a0_opt_plain__present,
                                    a0_opt_plain_z_keyexpr_as_str,
                                    a0_opt_boxed__present,
                                    a0_opt_boxed_z_keyexpr_as_str,
                                    a0_seq_plain,
                                    a0_seq_cow,
                                    a0_text_plain,
                                    a0_text_boxed,
                                    a0_text_cow,
                                )
                            };
                            (
                                a0_opt_plain__present,
                                a0_opt_plain_z_keyexpr_as_str,
                                a0_opt_boxed__present,
                                a0_opt_boxed_z_keyexpr_as_str,
                                a0_seq_plain,
                                a0_seq_cow,
                                a0_text_plain,
                                a0_text_boxed,
                                a0_text_cow,
                            )
                        };
                        ::core::result::Result::Ok(
                            ::std::vec![
                                ::prebindgen_jni_runtime::jni::sys::jvalue { z :
                                a0_opt_plain__present },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l :
                                a0_opt_plain_z_keyexpr_as_str.as_raw() },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { z :
                                a0_opt_boxed__present },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l :
                                a0_opt_boxed_z_keyexpr_as_str.as_raw() },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l :
                                a0_seq_plain.as_raw() },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l : a0_seq_cow
                                .as_raw() }, ::prebindgen_jni_runtime::jni::sys::jvalue { l
                                : a0_text_plain.as_raw() },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l :
                                a0_text_boxed.as_raw() },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l : a0_text_cow
                                .as_raw() }
                            ],
                        )
                    });
                if let ::core::result::Result::Err(__e) = __r {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (ZSample) + Send + Sync + 'static",
                        &__e,
                    );
                }
            }
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let __result = myflat::z_sample_sub(cb);
    let __r: ::core::result::Result<(), ::std::string::String> = (|| {
        let _ = __result;
        ::core::result::Result::Ok(())
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            ()
        }
    }
}
