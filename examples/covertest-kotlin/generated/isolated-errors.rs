#[allow(
    non_snake_case,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
fn __pb_jni_error_duration_result_local_c64b5b1058c5c0d0(
    mut env: &mut ::prebindgen_jni_runtime::jni::JNIEnv<'_>,
    source: i64,
    reported: &mut Option<::prebindgen_jni_runtime::jni::sys::jobjectArray>,
    return_sink: Option<&::prebindgen_jni_runtime::jni::objects::JObject<'_>>,
) -> ::prebindgen_runtime::Result<()> {
    if let Some(__pb_return) = return_sink {
        {
            let ret_w0 = source;
            unsafe {
                ::prebindgen_jni_runtime::invoke_error_callback(
                    &mut env,
                    &__pb_return,
                    "domainError0",
                    "(J)V",
                    &[
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: ret_w0,
                        },
                    ],
                )
            }?;
            let _: () = {
                *reported = Some(std::ptr::null_mut());
            };
            Ok(())
        }
    } else {
        {
            let ret_w0 = source;
            let result = ::prebindgen_jni_runtime::array(&mut env, 2usize)?;
            let object = env
                .call_static_method(
                    "java/lang/Long",
                    "valueOf",
                    "(J)Ljava/lang/Long;",
                    &[::prebindgen_jni_runtime::jni::objects::JValue::Long(ret_w0)],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            env.set_object_array_element(&result, 1i32, &object)
                .map_err(|e| e.to_string())?;
            env.delete_local_ref(object).map_err(|e| e.to_string())?;
            ::prebindgen_jni_runtime::mark_domain_packet(&mut env, &result)?;
            let _: () = {
                *reported = Some(result.as_raw());
            };
            Ok(())
        }
    }
}
#[allow(
    clippy::let_and_return,
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_io_prebindgen_covertest_isolatederror_JNINative_call_1duration_1result_11a1b3c1552bf21b6<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: i64 = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
            let source_value = crate::output_contracts::duration_result(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        __pb_jni_error_duration_result_local_c64b5b1058c5c0d0(
                            &mut jni_environment,
                            ret_w1,
                            &mut domain_output,
                            Some(&__pb_return),
                        )?;
                        String::new()
                    });
                }
            };
            let ret_w2 = ret_w0;
            let __pb_direct_result = unsafe {
                (::prebindgen_jni_runtime::jni::sys::jvalue {
                    j: ret_w2,
                })
                    .j
            };
            let _: () = {
                direct_output = __pb_direct_result;
                success_transferred = true;
            };
            Ok(__pb_direct_result)
        }
    });
    match outcome {
        Ok(result) => result,
        Err(e) => {
            if domain_output.is_none()
                && matches!(jni_environment.exception_check(), Ok(false))
            {
                let reported = ::prebindgen_runtime::boundary(|| {
                    let message = ::prebindgen_jni_runtime::text_object(
                        &mut jni_environment,
                        &e,
                    )?;
                    unsafe {
                        ::prebindgen_jni_runtime::invoke_error_callback(
                            &mut jni_environment,
                            &__pb_return,
                            "bindingError",
                            "(Ljava/lang/String;Z)V",
                            &[
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    l: message.as_raw(),
                                },
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    z: u8::from(success_transferred),
                                },
                            ],
                        )
                    }
                });
                if let Err(message) = reported {
                    ::prebindgen_jni_runtime::fail(&mut jni_environment, &message);
                }
            }
            direct_output
        }
    }
}
