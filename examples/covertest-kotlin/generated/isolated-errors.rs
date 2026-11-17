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
            let ret_w2 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w3) => {
                    let _: () = ((|| -> ::prebindgen_runtime::Result<()> {
                        {
                            let ret_w0 = ret_w3;
                            unsafe {
                                ::prebindgen_jni_runtime::invoke_error_callback(
                                    &mut jni_environment,
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
                                domain_output = Some(std::ptr::null_mut());
                            };
                            Ok(())
                        }
                    })())?;
                    return Err(String::new());
                }
            };
            let ret_w0 = ret_w2;
            let __pb_direct_result = unsafe {
                (::prebindgen_jni_runtime::jni::sys::jvalue {
                    j: ret_w0,
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_isolatederror_JNINative_call_1mapped_1result_130adb72426881056<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
) -> ::prebindgen_jni_runtime::jni::sys::jobjectArray {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: i64 = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
            let source_value = crate::output_contracts::mapped_result(
                __prebindgen_argument_0,
            );
            let ret_w4 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w5) => {
                    let _: () = ((|| -> ::prebindgen_runtime::Result<()> {
                        let ret_w1 = ret_w5;
                        let ret_w2 = crate::output_contracts::mapped_message(&ret_w1);
                        {
                            let ret_w0: ::prebindgen_jni_runtime::jni::objects::JObject<
                                '_,
                            > = {
                                let ret_w0 = ret_w2;
                                ::prebindgen_jni_runtime::text_object(
                                    &mut jni_environment,
                                    ::core::convert::AsRef::<str>::as_ref(&ret_w0),
                                )?
                            };
                            let result = ::prebindgen_jni_runtime::array(
                                &mut jni_environment,
                                2usize,
                            )?;
                            let object = ret_w0;
                            jni_environment
                                .set_object_array_element(&result, 1i32, &object)
                                .map_err(|e| e.to_string())?;
                            jni_environment
                                .delete_local_ref(object)
                                .map_err(|e| e.to_string())?;
                            ::prebindgen_jni_runtime::mark_domain_packet(
                                &mut jni_environment,
                                &result,
                            )?;
                            let _: () = {
                                domain_output = Some(result.as_raw());
                            };
                            Ok(())
                        }
                    })())?;
                    return Err(String::new());
                }
            };
            let ret_w0 = ret_w4;
            let result = ::prebindgen_jni_runtime::array(&mut jni_environment, 2usize)?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Long",
                    "valueOf",
                    "(J)Ljava/lang/Long;",
                    &[::prebindgen_jni_runtime::jni::objects::JValue::Long(ret_w0)],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            jni_environment
                .set_object_array_element(&result, 1i32, &object)
                .map_err(|e| e.to_string())?;
            jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
            let _: () = {};
            Ok(result.into_raw())
        }
    });
    match outcome {
        Ok(result) => result,
        Err(e) => {
            if let Some(result) = domain_output {
                result
            } else {
                match ::prebindgen_jni_runtime::binding_packet(
                    &mut jni_environment,
                    &e,
                ) {
                    Ok(result) => result.into_raw(),
                    Err(message) => {
                        ::prebindgen_jni_runtime::fail(&mut jni_environment, &message);
                        std::ptr::null_mut()
                    }
                }
            }
        }
    }
}
