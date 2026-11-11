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
    non_snake_case,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
fn __pb_jni_error_i64_typed_7b82463b61f2de72(
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
/**# Safety
Only generated Kotlin ownership release paths may call this entry point. The pointer must be a live allocation of the declared type with no outstanding native access; an opted-in Cleaner may release an unreachable wrapper without its monitor.*/
#[no_mangle]
pub unsafe extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_drop_1DurationPair_1a80e8ee6ea4081d2(
    mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,
    id: i64,
) {
    if let Err(e) = ::prebindgen_runtime::boundary(|| unsafe {
        ::prebindgen_jni_runtime::drop_pointer::<
            crate::output_contracts::DurationPair,
        >(id)
    }) {
        ::prebindgen_jni_runtime::fail(&mut env, &e);
    }
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_Range_1800d3dd3eceec6d {
    pub v0: ::prebindgen_jni_runtime::bulk::Range,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_i64_a60d3960993b9a77 {
    pub v0: i64,
}
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_bulkLayouts(
    mut env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
) -> ::prebindgen_jni_runtime::jni::sys::jlongArray {
    let result = (|| -> Result<_, String> {
        let mut data = vec![
            1i64, 7413971952934655829i64, if cfg!(target_endian = "big") { 1 } else { 0
            }, ::core::mem::size_of:: < usize > () as i64
        ];
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_Range_1800d3dd3eceec6d>(),
                ::core::mem::align_of::<__PbBulk_Range_1800d3dd3eceec6d>(),
                ::core::mem::offset_of!(__PbBulk_Range_1800d3dd3eceec6d, v0),
            ])
                .into_iter()
                .map(|v| v as i64),
        );
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_i64_a60d3960993b9a77>(),
                ::core::mem::align_of::<__PbBulk_i64_a60d3960993b9a77>(),
                ::core::mem::offset_of!(__PbBulk_i64_a60d3960993b9a77, v0),
            ])
                .into_iter()
                .map(|v| v as i64),
        );
        let array = env.new_long_array(data.len() as i32).map_err(|e| e.to_string())?;
        env.set_long_array_region(&array, 0, &data).map_err(|e| e.to_string())?;
        Ok(array.into_raw())
    })();
    match result {
        Ok(array) => array,
        Err(message) => {
            let _ = env.throw_new("java/lang/IllegalStateException", message);
            std::ptr::null_mut()
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_call_1duration_1echo_19eff5d3e7eedf8fc<
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
            let source_value = crate::output_contracts::duration_echo(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_call_1duration_1packet_1f309e8fdef6a1a5c<
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
            let source_value = crate::output_contracts::duration_packet(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_call_1duration_1pair_17f1ff9c108dc7c4e<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = crate::output_contracts::duration_pair();
            let ret_w0 = crate::output_contracts::pair_a(&source_value);
            let ret_w1 = ret_w0;
            let ret_w2 = crate::output_contracts::pair_b(&source_value);
            let ret_w3 = ret_w2;
            static __PB_RETURN_METHOD_0: ::prebindgen_jni_runtime::ReturnCallback = ::prebindgen_jni_runtime::ReturnCallback::new();
            unsafe {
                __PB_RETURN_METHOD_0
                    .invoke(
                        &mut jni_environment,
                        &__pb_return,
                        "success0",
                        "(JJ)V",
                        &[
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w1,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w3,
                            },
                        ],
                    )
            }?;
            let _: () = {
                success_transferred = true;
            };
            Ok(())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_call_1duration_1series_10147306c7d8df97a<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = crate::output_contracts::duration_series();
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_1800d3dd3eceec6d>(),
            );
            let ret_w1_source: ::std::vec::Vec<i64> = source_value;
            let ret_w3 = ::std::vec::Vec::len(&ret_w1_source);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_a60d3960993b9a77>(),
                    ::core::mem::align_of::<__PbBulk_i64_a60d3960993b9a77>(),
                )?;
            for (ret_w4, ret_w2) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w1_source),
            ) {
                let ret_w5 = ret_w2;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<__PbBulk_i64_a60d3960993b9a77>())
                            + ::core::mem::offset_of!(__PbBulk_i64_a60d3960993b9a77, v0),
                        ret_w5,
                    )?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {})();
            }
            let ret_w1 = Some(ret_w1);
            ret_w0_writer
                .range(
                    ::core::mem::offset_of!(__PbBulk_Range_1800d3dd3eceec6d, v0),
                    ret_w1,
                )?;
            let ret_w0 = ::prebindgen_jni_runtime::jni::objects::JObject::from(
                jni_environment
                    .byte_array_from_slice(ret_w0_writer.bytes())
                    .map_err(|e| e.to_string())?,
            );
            let __pb_direct_object = ret_w0;
            let __pb_direct_result = unsafe {
                (::prebindgen_jni_runtime::jni::sys::jvalue {
                    l: __pb_direct_object.as_raw(),
                })
                    .l
            };
            let _: () = {
                ret_w0_writer.commit();
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_call_1duration_1optional_1cfe965c40fd3f602<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: u8,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: bool = ::core::convert::Into::into(p0_w1 != 0);
            let __prebindgen_argument_0 = p0_w0;
            let source_value = crate::output_contracts::duration_optional(
                __prebindgen_argument_0,
            );
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = ret_w1;
                let ret_w0_values = (ret_w2,);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0,), None)
            };
            static __PB_RETURN_METHOD_0: ::prebindgen_jni_runtime::ReturnCallback = ::prebindgen_jni_runtime::ReturnCallback::new();
            unsafe {
                __PB_RETURN_METHOD_0
                    .invoke(
                        &mut jni_environment,
                        &__pb_return,
                        "success0",
                        "(IJ)V",
                        &[
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                i: ret_w0,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_value0,
                            },
                        ],
                    )
            }?;
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
                success_transferred = true;
            };
            Ok(())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_call_1duration_1result_11a1b3c1552bf21b6<
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
                        __pb_jni_error_i64_typed_7b82463b61f2de72(
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_call_1plain_1result_13f31a43f28b72fff<
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
            let source_value = crate::output_contracts::plain_result(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        __pb_jni_error_i64_typed_7b82463b61f2de72(
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_call_1duration_1callback_190cbff1e8af6c9e8<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    custom_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let custom_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &custom_callback,
                "(J)J",
                "io.prebindgen.covertest.outputcontracts.DurationCallback",
            )?;
            #[allow(unused_mut, clippy::collapsible_if)]
            let callback_value = move |callback_source_0: i64| {
                custom_callback_retained
                    .with_env_result(
                        17i32,
                        |callback_environment| {
                            let mut callback_environment = callback_environment;
                            let callback_value_0 = callback_source_0;
                            {
                                let ret_w0 = callback_value_0;
                                let arguments = [
                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                        j: ret_w0,
                                    },
                                ];
                                {}
                                let callback_result = unsafe {
                                    custom_callback_retained
                                        .invoke_return(
                                            callback_environment,
                                            ::prebindgen_jni_runtime::jni::signature::ReturnType::Primitive(
                                                ::prebindgen_jni_runtime::jni::signature::Primitive::Long,
                                            ),
                                            &arguments,
                                        )
                                }?;
                                let callback_duration_wire = callback_result
                                    .j()
                                    .map_err(|e| e.to_string())?;
                                let callback_duration_value: i64 = ::core::convert::Into::into(
                                    callback_duration_wire,
                                );
                                let callback_return_value = callback_duration_value;
                                Ok::<_, String>(callback_return_value)
                            }
                        },
                    )
                    .unwrap_or_else(|error| panic!("callback failed: {}", error))
            };
            let __prebindgen_argument_0 = callback_value;
            let source_value = crate::output_contracts::duration_callback(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_call_1plain_1callback_11c7d8c18a52f8e01<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "(J)J",
                "io.prebindgen.covertest.outputcontracts.i64Toi64Callback",
            )?;
            #[allow(unused_mut, clippy::collapsible_if)]
            let p0_callback_value = move |callback_source_0: i64| {
                p0_callback_retained
                    .with_env_result(
                        17i32,
                        |callback_environment| {
                            let mut callback_environment = callback_environment;
                            let callback_value_0 = callback_source_0;
                            {
                                let callback_arg_0_w0 = callback_value_0;
                                let arguments = [
                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                        j: callback_arg_0_w0,
                                    },
                                ];
                                {}
                                let callback_result = unsafe {
                                    p0_callback_retained
                                        .invoke_return(
                                            callback_environment,
                                            ::prebindgen_jni_runtime::jni::signature::ReturnType::Primitive(
                                                ::prebindgen_jni_runtime::jni::signature::Primitive::Long,
                                            ),
                                            &arguments,
                                        )
                                }?;
                                let callback_return_w1 = callback_result
                                    .j()
                                    .map_err(|e| e.to_string())?;
                                let callback_return_w0: i64 = ::core::convert::Into::into(
                                    callback_return_w1,
                                );
                                let callback_return_value = callback_return_w0;
                                Ok::<_, String>(callback_return_value)
                            }
                        },
                    )
                    .unwrap_or_else(|error| panic!("callback failed: {}", error))
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = crate::output_contracts::plain_callback(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
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
static __PB_SEQUENCE_CAPACITY: ::std::sync::OnceLock<
    (
        ::prebindgen_jni_runtime::jni::objects::GlobalRef,
        ::prebindgen_jni_runtime::jni::objects::JMethodID,
    ),
> = ::std::sync::OnceLock::new();
#[no_mangle]
pub extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_init_1SequenceCapacity_1e067e94ae7b50e2a(
    mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,
    capacity: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if __PB_SEQUENCE_CAPACITY.get().is_none() {
            let reserve = env
                .get_method_id(&capacity, "reserve", "(I)V")
                .map_err(|e| e.to_string())?;
            let class = env.new_global_ref(&capacity).map_err(|e| e.to_string())?;
            let _ = __PB_SEQUENCE_CAPACITY.set((class, reserve));
        }
        let (class, _) = __PB_SEQUENCE_CAPACITY
            .get()
            .expect("capacity method initialized");
        if !env.is_same_object(class.as_obj(), &capacity).map_err(|e| e.to_string())? {
            return Err(
                "collection capacity interface registered with a different class".into(),
            );
        }
        Ok(())
    }) {
        ::prebindgen_jni_runtime::fail(&mut env, &error);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_outputcontracts_JNINative_call_1duration_1series_1fold_1c232f7a7643f8f2b<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    __pb_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
    __pb_output_accepted: ::prebindgen_jni_runtime::jni::objects::JBooleanArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i32 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i32 = Default::default();
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        if jni_environment
            .get_array_length(&__pb_output_accepted)
            .map_err(|e| e.to_string())? != 1
        {
            return Err("invalid sequence output acknowledgement".into());
        }
        if __pb_callback.is_null() {
            return Err("null sequence callback".into());
        }
        {
            let source_value = crate::output_contracts::duration_series();
            let ret_w0_source: ::std::vec::Vec<i64> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            jni_environment
                .set_boolean_array_region(&__pb_output_accepted, 0, &[1])
                .map_err(|e| e.to_string())?;
            let ret_w0_callback_class = jni_environment
                .get_object_class(&__pb_callback)
                .map_err(|e| e.to_string())?;
            let ret_w0_callback_method = jni_environment
                .get_method_id(&ret_w0_callback_class, "run", "(J)V")
                .map_err(|e| e.to_string())?;
            if ret_w2 >= 64 {
                let capacity = <i32 as ::core::convert::TryFrom<_>>::try_from(ret_w2)
                    .map_err(|_| "collection exceeds JVM capacity".to_string())?;
                let (_, reserve) = __PB_SEQUENCE_CAPACITY
                    .get()
                    .ok_or_else(|| {
                        "collection capacity hint is not initialized".to_string()
                    })?;
                unsafe {
                    jni_environment
                        .call_method_unchecked(
                            &__pb_callback,
                            *reserve,
                            ::prebindgen_jni_runtime::jni::signature::ReturnType::Primitive(
                                ::prebindgen_jni_runtime::jni::signature::Primitive::Void,
                            ),
                            &[
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    i: capacity,
                                },
                            ],
                        )
                }
                    .map_err(|e| e.to_string())?;
            }
            jni_environment
                .delete_local_ref(ret_w0_callback_class)
                .map_err(|e| e.to_string())?;
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w4 = ret_w1;
                unsafe {
                    jni_environment
                        .call_method_unchecked(
                            &__pb_callback,
                            ret_w0_callback_method,
                            ::prebindgen_jni_runtime::jni::signature::ReturnType::Primitive(
                                ::prebindgen_jni_runtime::jni::signature::Primitive::Void,
                            ),
                            &[
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ret_w4,
                                },
                            ],
                        )
                }
                    .map_err(|e| e.to_string())?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {})();
            }
            let ret_w0 = 0i32;
            let __pb_direct_result = unsafe {
                (::prebindgen_jni_runtime::jni::sys::jvalue {
                    i: ret_w0,
                })
                    .i
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
const _: () = assert!(
    ::core::mem::size_of:: < crate ::output_contracts::DurationPair > () != 0,
    "flat object `DurationPair` must have nonzero size"
);
