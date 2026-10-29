/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_drop_1Handler_1bb814c4111561dda(
    mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,
    id: i64,
) {
    if let Err(e) = ::prebindgen_runtime::boundary(|| unsafe {
        ::prebindgen_jni_runtime::drop_pointer::<bench_flat::Handler>(id)
    }) {
        ::prebindgen_jni_runtime::fail(&mut env, &e);
    }
}
#[allow(
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1scalar_1a4a5da3f79ebbcaf<
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
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::scalar(__prebindgen_argument_0);
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
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1row_1sum_1db65517187cece4c<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1_f0: i64,
    p0_w1_f1: i64,
    p0_w1_f2: i64,
    p0_w1_f3: i64,
    p0_w1_f4: i64,
    p0_w1_f5: i64,
    p0_w1_f6: i64,
    p0_w1_f7: i64,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0_f0 = if true { Some(p0_w1_f0) } else { None };
            let p0_w0_f1 = if true { Some(p0_w1_f1) } else { None };
            let p0_w0_f2 = if true { Some(p0_w1_f2) } else { None };
            let p0_w0_f3 = if true { Some(p0_w1_f3) } else { None };
            let p0_w0_f4 = if true { Some(p0_w1_f4) } else { None };
            let p0_w0_f5 = if true { Some(p0_w1_f5) } else { None };
            let p0_w0_f6 = if true { Some(p0_w1_f6) } else { None };
            let p0_w0_f7 = if true { Some(p0_w1_f7) } else { None };
            let p0_w0_f0: i64 = ::core::convert::Into::into(
                p0_w0_f0.expect("active value"),
            );
            let p0_w0_f1: i64 = ::core::convert::Into::into(
                p0_w0_f1.expect("active value"),
            );
            let p0_w0_f2: i64 = ::core::convert::Into::into(
                p0_w0_f2.expect("active value"),
            );
            let p0_w0_f3: i64 = ::core::convert::Into::into(
                p0_w0_f3.expect("active value"),
            );
            let p0_w0_f4: i64 = ::core::convert::Into::into(
                p0_w0_f4.expect("active value"),
            );
            let p0_w0_f5: i64 = ::core::convert::Into::into(
                p0_w0_f5.expect("active value"),
            );
            let p0_w0_f6: i64 = ::core::convert::Into::into(
                p0_w0_f6.expect("active value"),
            );
            let p0_w0_f7: i64 = ::core::convert::Into::into(
                p0_w0_f7.expect("active value"),
            );
            let p0_w0: bench_flat::Row = bench_flat::Row {
                a: p0_w0_f0,
                b: p0_w0_f1,
                c: p0_w0_f2,
                d: p0_w0_f3,
                e: p0_w0_f4,
                f: p0_w0_f5,
                g: p0_w0_f6,
                h: p0_w0_f7,
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::row_sum(__prebindgen_argument_0);
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
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1numbers_1sum_147543406a5cdf588<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JObjectArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    match ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                if p0_w1.is_null() {
                    return Err("null collection".into());
                }
                let p0_w2 = jni_environment
                    .get_array_length(&p0_w1)
                    .map_err(|e| e.to_string())? as usize;
                for p0_w3 in 0..p0_w2 {
                    let p0_w1_frame = unsafe {
                        ::prebindgen_jni_runtime::LocalFrame::new(&jni_environment, 9i32)
                    }?;
                    let p0_w1_row = ::prebindgen_jni_runtime::row(
                        &mut jni_environment,
                        &p0_w1,
                        p0_w3,
                        1usize,
                    )?;
                    let p0_w5 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        0usize,
                    );
                    if p0_w5.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w4 = if true {
                        Some(*p0_w5.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    drop(p0_w1_frame);
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<i64, String> {
                                let p0_w4: i64 = ::core::convert::Into::into(
                                    p0_w4.expect("active value"),
                                );
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<i64> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::numbers_sum(__prebindgen_argument_0);
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
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1rows_1sum_1d4a5ecfa43bf26b9<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JObjectArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    match ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                if p0_w1.is_null() {
                    return Err("null collection".into());
                }
                let p0_w2 = jni_environment
                    .get_array_length(&p0_w1)
                    .map_err(|e| e.to_string())? as usize;
                for p0_w3 in 0..p0_w2 {
                    let p0_w1_frame = unsafe {
                        ::prebindgen_jni_runtime::LocalFrame::new(
                            &jni_environment,
                            16i32,
                        )
                    }?;
                    let p0_w1_row = ::prebindgen_jni_runtime::row(
                        &mut jni_environment,
                        &p0_w1,
                        p0_w3,
                        8usize,
                    )?;
                    let p0_w5_f0 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        0usize,
                    );
                    if p0_w5_f0.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f1 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        1usize,
                    );
                    if p0_w5_f1.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f2 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        2usize,
                    );
                    if p0_w5_f2.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f3 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        3usize,
                    );
                    if p0_w5_f3.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f4 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        4usize,
                    );
                    if p0_w5_f4.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f5 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        5usize,
                    );
                    if p0_w5_f5.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f6 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        6usize,
                    );
                    if p0_w5_f6.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f7 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        7usize,
                    );
                    if p0_w5_f7.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w4_f0 = if true {
                        Some(*p0_w5_f0.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f1 = if true {
                        Some(*p0_w5_f1.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f2 = if true {
                        Some(*p0_w5_f2.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f3 = if true {
                        Some(*p0_w5_f3.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f4 = if true {
                        Some(*p0_w5_f4.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f5 = if true {
                        Some(*p0_w5_f5.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f6 = if true {
                        Some(*p0_w5_f6.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f7 = if true {
                        Some(*p0_w5_f7.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    drop(p0_w1_frame);
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<bench_flat::Row, String> {
                                let p0_w4_f0: i64 = ::core::convert::Into::into(
                                    p0_w4_f0.expect("active value"),
                                );
                                let p0_w4_f1: i64 = ::core::convert::Into::into(
                                    p0_w4_f1.expect("active value"),
                                );
                                let p0_w4_f2: i64 = ::core::convert::Into::into(
                                    p0_w4_f2.expect("active value"),
                                );
                                let p0_w4_f3: i64 = ::core::convert::Into::into(
                                    p0_w4_f3.expect("active value"),
                                );
                                let p0_w4_f4: i64 = ::core::convert::Into::into(
                                    p0_w4_f4.expect("active value"),
                                );
                                let p0_w4_f5: i64 = ::core::convert::Into::into(
                                    p0_w4_f5.expect("active value"),
                                );
                                let p0_w4_f6: i64 = ::core::convert::Into::into(
                                    p0_w4_f6.expect("active value"),
                                );
                                let p0_w4_f7: i64 = ::core::convert::Into::into(
                                    p0_w4_f7.expect("active value"),
                                );
                                let p0_w4: bench_flat::Row = bench_flat::Row {
                                    a: p0_w4_f0,
                                    b: p0_w4_f1,
                                    c: p0_w4_f2,
                                    d: p0_w4_f3,
                                    e: p0_w4_f4,
                                    f: p0_w4_f5,
                                    g: p0_w4_f6,
                                    h: p0_w4_f7,
                                };
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<bench_flat::Row> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0.as_slice();
            let source_value = bench_flat::rows_sum(__prebindgen_argument_0);
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
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1rows_1make_16f7eb2a3ee93e0ae<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
    p1_w1: i64,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = bench_flat::rows_make(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let ret_w0 = ::prebindgen_jni_runtime::array(&mut jni_environment, ret_w2)?;
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w0_frame = unsafe {
                    ::prebindgen_jni_runtime::LocalFrame::new(&jni_environment, 16i32)
                }?;
                let ret_w4_record = ret_w1;
                let ret_w4_f0 = ret_w4_record.a;
                let ret_w4_f1 = ret_w4_record.b;
                let ret_w4_f2 = ret_w4_record.c;
                let ret_w4_f3 = ret_w4_record.d;
                let ret_w4_f4 = ret_w4_record.e;
                let ret_w4_f5 = ret_w4_record.f;
                let ret_w4_f6 = ret_w4_record.g;
                let ret_w4_f7 = ret_w4_record.h;
                let ret_w0_row = ::prebindgen_jni_runtime::array(
                    &mut jni_environment,
                    8usize,
                )?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f0,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 0i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f1,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 1i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f2,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 2i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f3,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 3i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f4,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 4i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f5,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 5i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f6,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 6i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f7,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 7i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0, ret_w3 as i32, &ret_w0_row)
                    .map_err(|e| e.to_string())?;
                drop(ret_w0_frame);
                ret_w0_commits.push(move || {});
            }
            let ret_w0 = Some(ret_w0);
            let __pb_direct_object = match (ret_w0).as_ref() {
                Some(value) => {
                    jni_environment.new_local_ref(value).map_err(|e| e.to_string())?
                }
                None => ::prebindgen_jni_runtime::jni::objects::JObject::null(),
            };
            let __pb_direct_result = unsafe {
                (::prebindgen_jni_runtime::jni::sys::jvalue {
                    l: __pb_direct_object.as_raw(),
                })
                    .l
            };
            let _: () = {
                for commit in ret_w0_commits {
                    commit();
                }
                direct_output = __pb_direct_result;
                success_transferred = true;
            };
            Ok(__pb_direct_result)
        }
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1rows_1echo_15c26c8cf4751c454<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JObjectArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    match ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w1_staged = Vec::new();
            if true {
                if p0_w1.is_null() {
                    return Err("null collection".into());
                }
                let p0_w2 = jni_environment
                    .get_array_length(&p0_w1)
                    .map_err(|e| e.to_string())? as usize;
                for p0_w3 in 0..p0_w2 {
                    let p0_w1_frame = unsafe {
                        ::prebindgen_jni_runtime::LocalFrame::new(
                            &jni_environment,
                            16i32,
                        )
                    }?;
                    let p0_w1_row = ::prebindgen_jni_runtime::row(
                        &mut jni_environment,
                        &p0_w1,
                        p0_w3,
                        8usize,
                    )?;
                    let p0_w5_f0 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        0usize,
                    );
                    if p0_w5_f0.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f1 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        1usize,
                    );
                    if p0_w5_f1.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f2 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        2usize,
                    );
                    if p0_w5_f2.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f3 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        3usize,
                    );
                    if p0_w5_f3.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f4 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        4usize,
                    );
                    if p0_w5_f4.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f5 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        5usize,
                    );
                    if p0_w5_f5.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f6 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        6usize,
                    );
                    if p0_w5_f6.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f7 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        7usize,
                    );
                    if p0_w5_f7.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w4_f0 = if true {
                        Some(*p0_w5_f0.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f1 = if true {
                        Some(*p0_w5_f1.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f2 = if true {
                        Some(*p0_w5_f2.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f3 = if true {
                        Some(*p0_w5_f3.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f4 = if true {
                        Some(*p0_w5_f4.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f5 = if true {
                        Some(*p0_w5_f5.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f6 = if true {
                        Some(*p0_w5_f6.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    let p0_w4_f7 = if true {
                        Some(*p0_w5_f7.as_ref().map_err(|e| e.clone())?)
                    } else {
                        None
                    };
                    drop(p0_w1_frame);
                    p0_w1_staged
                        .push(move || {
                            move || -> ::core::result::Result<bench_flat::Row, String> {
                                let p0_w4_f0: i64 = ::core::convert::Into::into(
                                    p0_w4_f0.expect("active value"),
                                );
                                let p0_w4_f1: i64 = ::core::convert::Into::into(
                                    p0_w4_f1.expect("active value"),
                                );
                                let p0_w4_f2: i64 = ::core::convert::Into::into(
                                    p0_w4_f2.expect("active value"),
                                );
                                let p0_w4_f3: i64 = ::core::convert::Into::into(
                                    p0_w4_f3.expect("active value"),
                                );
                                let p0_w4_f4: i64 = ::core::convert::Into::into(
                                    p0_w4_f4.expect("active value"),
                                );
                                let p0_w4_f5: i64 = ::core::convert::Into::into(
                                    p0_w4_f5.expect("active value"),
                                );
                                let p0_w4_f6: i64 = ::core::convert::Into::into(
                                    p0_w4_f6.expect("active value"),
                                );
                                let p0_w4_f7: i64 = ::core::convert::Into::into(
                                    p0_w4_f7.expect("active value"),
                                );
                                let p0_w4: bench_flat::Row = bench_flat::Row {
                                    a: p0_w4_f0,
                                    b: p0_w4_f1,
                                    c: p0_w4_f2,
                                    d: p0_w4_f3,
                                    e: p0_w4_f4,
                                    f: p0_w4_f5,
                                    g: p0_w4_f6,
                                    h: p0_w4_f7,
                                };
                                let p0_w0 = p0_w4;
                                Ok(p0_w0)
                            }
                        });
                }
            }
            let p0_w1_acquired: Vec<_> = p0_w1_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w0: Vec<bench_flat::Row> = p0_w1_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::rows_echo(__prebindgen_argument_0);
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let ret_w0 = ::prebindgen_jni_runtime::array(&mut jni_environment, ret_w2)?;
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w0_frame = unsafe {
                    ::prebindgen_jni_runtime::LocalFrame::new(&jni_environment, 16i32)
                }?;
                let ret_w4_record = ret_w1;
                let ret_w4_f0 = ret_w4_record.a;
                let ret_w4_f1 = ret_w4_record.b;
                let ret_w4_f2 = ret_w4_record.c;
                let ret_w4_f3 = ret_w4_record.d;
                let ret_w4_f4 = ret_w4_record.e;
                let ret_w4_f5 = ret_w4_record.f;
                let ret_w4_f6 = ret_w4_record.g;
                let ret_w4_f7 = ret_w4_record.h;
                let ret_w0_row = ::prebindgen_jni_runtime::array(
                    &mut jni_environment,
                    8usize,
                )?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f0,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 0i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f1,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 1i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f2,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 2i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f3,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 3i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f4,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 4i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f5,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 5i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f6,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 6i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                let object = jni_environment
                    .call_static_method(
                        "java/lang/Long",
                        "valueOf",
                        "(J)Ljava/lang/Long;",
                        &[
                            ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                ret_w4_f7,
                            ),
                        ],
                    )
                    .and_then(|v| v.l())
                    .map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0_row, 7i32, &object)
                    .map_err(|e| e.to_string())?;
                jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
                jni_environment
                    .set_object_array_element(&ret_w0, ret_w3 as i32, &ret_w0_row)
                    .map_err(|e| e.to_string())?;
                drop(ret_w0_frame);
                ret_w0_commits.push(move || {});
            }
            let ret_w0 = Some(ret_w0);
            let __pb_direct_object = match (ret_w0).as_ref() {
                Some(value) => {
                    jni_environment.new_local_ref(value).map_err(|e| e.to_string())?
                }
                None => ::prebindgen_jni_runtime::jni::objects::JObject::null(),
            };
            let __pb_direct_result = unsafe {
                (::prebindgen_jni_runtime::jni::sys::jvalue {
                    l: __pb_direct_object.as_raw(),
                })
                    .l
            };
            let _: () = {
                for commit in ret_w0_commits {
                    commit();
                }
                direct_output = __pb_direct_result;
                success_transferred = true;
            };
            Ok(__pb_direct_result)
        }
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1packet_1sum_1c07ede9eb5de050f<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1_f0_f0: i64,
    p0_w1_f0_f1: i64,
    p0_w1_f0_f2: i64,
    p0_w1_f0_f3: i64,
    p0_w1_f0_f4: i64,
    p0_w1_f0_f5: i64,
    p0_w1_f0_f6: i64,
    p0_w1_f0_f7: i64,
    p0_w1_f1: ::prebindgen_jni_runtime::jni::objects::JByteArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0_f0_f0 = if true { Some(p0_w1_f0_f0) } else { None };
            let p0_w0_f0_f1 = if true { Some(p0_w1_f0_f1) } else { None };
            let p0_w0_f0_f2 = if true { Some(p0_w1_f0_f2) } else { None };
            let p0_w0_f0_f3 = if true { Some(p0_w1_f0_f3) } else { None };
            let p0_w0_f0_f4 = if true { Some(p0_w1_f0_f4) } else { None };
            let p0_w0_f0_f5 = if true { Some(p0_w1_f0_f5) } else { None };
            let p0_w0_f0_f6 = if true { Some(p0_w1_f0_f6) } else { None };
            let p0_w0_f0_f7 = if true { Some(p0_w1_f0_f7) } else { None };
            let p0_w0_f1 = if true {
                Some(::prebindgen_jni_runtime::bytes(&mut jni_environment, &p0_w1_f1)?)
            } else {
                None
            };
            let p0_w0_f0_f0: i64 = ::core::convert::Into::into(
                p0_w0_f0_f0.expect("active value"),
            );
            let p0_w0_f0_f1: i64 = ::core::convert::Into::into(
                p0_w0_f0_f1.expect("active value"),
            );
            let p0_w0_f0_f2: i64 = ::core::convert::Into::into(
                p0_w0_f0_f2.expect("active value"),
            );
            let p0_w0_f0_f3: i64 = ::core::convert::Into::into(
                p0_w0_f0_f3.expect("active value"),
            );
            let p0_w0_f0_f4: i64 = ::core::convert::Into::into(
                p0_w0_f0_f4.expect("active value"),
            );
            let p0_w0_f0_f5: i64 = ::core::convert::Into::into(
                p0_w0_f0_f5.expect("active value"),
            );
            let p0_w0_f0_f6: i64 = ::core::convert::Into::into(
                p0_w0_f0_f6.expect("active value"),
            );
            let p0_w0_f0_f7: i64 = ::core::convert::Into::into(
                p0_w0_f0_f7.expect("active value"),
            );
            let p0_w0_f0: bench_flat::Row = bench_flat::Row {
                a: p0_w0_f0_f0,
                b: p0_w0_f0_f1,
                c: p0_w0_f0_f2,
                d: p0_w0_f0_f3,
                e: p0_w0_f0_f4,
                f: p0_w0_f0_f5,
                g: p0_w0_f0_f6,
                h: p0_w0_f0_f7,
            };
            let p0_w0_f1: Vec<u8> = ::core::convert::Into::into(
                p0_w0_f1.expect("active value"),
            );
            let p0_w0: bench_flat::Packet = bench_flat::Packet {
                header: p0_w0_f0,
                bytes: p0_w0_f1,
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::packet_sum(__prebindgen_argument_0);
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
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1handler_1new_1916a49ec8343b596<
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
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "([Ljava/lang/Object;)J",
                "bench.plain.RowsCallback",
            )?;
            #[allow(unused_mut, clippy::collapsible_if)]
            let p0_callback_value = move |callback_source_0: Vec<bench_flat::Row>| {
                p0_callback_retained
                    .with_env_result(
                        17i32,
                        |callback_environment| {
                            let mut callback_environment = callback_environment;
                            let callback_value_0 = callback_source_0;
                            {
                                let callback_arg_0_w0_source = callback_value_0;
                                let callback_arg_0_w2 = callback_arg_0_w0_source.len();
                                let mut callback_arg_0_w0_commits = Vec::with_capacity(
                                    callback_arg_0_w2,
                                );
                                let callback_arg_0_w0 = ::prebindgen_jni_runtime::array(
                                    &mut callback_environment,
                                    callback_arg_0_w2,
                                )?;
                                for (callback_arg_0_w3, callback_arg_0_w1) in callback_arg_0_w0_source
                                    .into_iter()
                                    .enumerate()
                                {
                                    let callback_arg_0_w0_frame = unsafe {
                                        ::prebindgen_jni_runtime::LocalFrame::new(
                                            &callback_environment,
                                            16i32,
                                        )
                                    }?;
                                    let callback_arg_0_w4_record = callback_arg_0_w1;
                                    let callback_arg_0_w4_f0 = callback_arg_0_w4_record.a;
                                    let callback_arg_0_w4_f1 = callback_arg_0_w4_record.b;
                                    let callback_arg_0_w4_f2 = callback_arg_0_w4_record.c;
                                    let callback_arg_0_w4_f3 = callback_arg_0_w4_record.d;
                                    let callback_arg_0_w4_f4 = callback_arg_0_w4_record.e;
                                    let callback_arg_0_w4_f5 = callback_arg_0_w4_record.f;
                                    let callback_arg_0_w4_f6 = callback_arg_0_w4_record.g;
                                    let callback_arg_0_w4_f7 = callback_arg_0_w4_record.h;
                                    let callback_arg_0_w0_row = ::prebindgen_jni_runtime::array(
                                        &mut callback_environment,
                                        8usize,
                                    )?;
                                    let object = callback_environment
                                        .call_static_method(
                                            "java/lang/Long",
                                            "valueOf",
                                            "(J)Ljava/lang/Long;",
                                            &[
                                                ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                                    callback_arg_0_w4_f0,
                                                ),
                                            ],
                                        )
                                        .and_then(|v| v.l())
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .set_object_array_element(
                                            &callback_arg_0_w0_row,
                                            0i32,
                                            &object,
                                        )
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .delete_local_ref(object)
                                        .map_err(|e| e.to_string())?;
                                    let object = callback_environment
                                        .call_static_method(
                                            "java/lang/Long",
                                            "valueOf",
                                            "(J)Ljava/lang/Long;",
                                            &[
                                                ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                                    callback_arg_0_w4_f1,
                                                ),
                                            ],
                                        )
                                        .and_then(|v| v.l())
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .set_object_array_element(
                                            &callback_arg_0_w0_row,
                                            1i32,
                                            &object,
                                        )
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .delete_local_ref(object)
                                        .map_err(|e| e.to_string())?;
                                    let object = callback_environment
                                        .call_static_method(
                                            "java/lang/Long",
                                            "valueOf",
                                            "(J)Ljava/lang/Long;",
                                            &[
                                                ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                                    callback_arg_0_w4_f2,
                                                ),
                                            ],
                                        )
                                        .and_then(|v| v.l())
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .set_object_array_element(
                                            &callback_arg_0_w0_row,
                                            2i32,
                                            &object,
                                        )
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .delete_local_ref(object)
                                        .map_err(|e| e.to_string())?;
                                    let object = callback_environment
                                        .call_static_method(
                                            "java/lang/Long",
                                            "valueOf",
                                            "(J)Ljava/lang/Long;",
                                            &[
                                                ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                                    callback_arg_0_w4_f3,
                                                ),
                                            ],
                                        )
                                        .and_then(|v| v.l())
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .set_object_array_element(
                                            &callback_arg_0_w0_row,
                                            3i32,
                                            &object,
                                        )
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .delete_local_ref(object)
                                        .map_err(|e| e.to_string())?;
                                    let object = callback_environment
                                        .call_static_method(
                                            "java/lang/Long",
                                            "valueOf",
                                            "(J)Ljava/lang/Long;",
                                            &[
                                                ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                                    callback_arg_0_w4_f4,
                                                ),
                                            ],
                                        )
                                        .and_then(|v| v.l())
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .set_object_array_element(
                                            &callback_arg_0_w0_row,
                                            4i32,
                                            &object,
                                        )
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .delete_local_ref(object)
                                        .map_err(|e| e.to_string())?;
                                    let object = callback_environment
                                        .call_static_method(
                                            "java/lang/Long",
                                            "valueOf",
                                            "(J)Ljava/lang/Long;",
                                            &[
                                                ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                                    callback_arg_0_w4_f5,
                                                ),
                                            ],
                                        )
                                        .and_then(|v| v.l())
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .set_object_array_element(
                                            &callback_arg_0_w0_row,
                                            5i32,
                                            &object,
                                        )
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .delete_local_ref(object)
                                        .map_err(|e| e.to_string())?;
                                    let object = callback_environment
                                        .call_static_method(
                                            "java/lang/Long",
                                            "valueOf",
                                            "(J)Ljava/lang/Long;",
                                            &[
                                                ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                                    callback_arg_0_w4_f6,
                                                ),
                                            ],
                                        )
                                        .and_then(|v| v.l())
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .set_object_array_element(
                                            &callback_arg_0_w0_row,
                                            6i32,
                                            &object,
                                        )
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .delete_local_ref(object)
                                        .map_err(|e| e.to_string())?;
                                    let object = callback_environment
                                        .call_static_method(
                                            "java/lang/Long",
                                            "valueOf",
                                            "(J)Ljava/lang/Long;",
                                            &[
                                                ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                                                    callback_arg_0_w4_f7,
                                                ),
                                            ],
                                        )
                                        .and_then(|v| v.l())
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .set_object_array_element(
                                            &callback_arg_0_w0_row,
                                            7i32,
                                            &object,
                                        )
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .delete_local_ref(object)
                                        .map_err(|e| e.to_string())?;
                                    callback_environment
                                        .set_object_array_element(
                                            &callback_arg_0_w0,
                                            callback_arg_0_w3 as i32,
                                            &callback_arg_0_w0_row,
                                        )
                                        .map_err(|e| e.to_string())?;
                                    drop(callback_arg_0_w0_frame);
                                    callback_arg_0_w0_commits.push(move || {});
                                }
                                let callback_arg_0_w0 = Some(callback_arg_0_w0);
                                let callback_java_0_0 = match (callback_arg_0_w0).as_ref() {
                                    Some(value) => {
                                        callback_environment
                                            .new_local_ref(value)
                                            .map_err(|e| e.to_string())?
                                    }
                                    None => {
                                        ::prebindgen_jni_runtime::jni::objects::JObject::null()
                                    }
                                };
                                let arguments = [
                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                        l: callback_java_0_0.as_raw(),
                                    },
                                ];
                                {
                                    for commit in callback_arg_0_w0_commits {
                                        commit();
                                    }
                                }
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
                                let callback_return_w0 = if true {
                                    Some(callback_return_w1)
                                } else {
                                    None
                                };
                                let callback_return_w0: i64 = ::core::convert::Into::into(
                                    callback_return_w0.expect("active value"),
                                );
                                let callback_return_value = callback_return_w0;
                                Ok(callback_return_value)
                            }
                        },
                    )
                    .unwrap_or_else(|error| panic!("callback failed: {}", error))
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = bench_flat::handler_new(__prebindgen_argument_0);
            let mut ret_w0 = Box::new(::prebindgen_jni_runtime::Allocation {
                value: source_value,
            });
            let __pb_direct_result = unsafe {
                (::prebindgen_jni_runtime::jni::sys::jvalue {
                    j: ((&mut *ret_w0)
                        as *mut ::prebindgen_jni_runtime::Allocation<
                            bench_flat::Handler,
                        >) as i64,
                })
                    .j
            };
            let _: () = {
                let _ = Box::into_raw(ret_w0);
                direct_output = __pb_direct_result;
                success_transferred = true;
            };
            Ok(__pb_direct_result)
        }
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1fire_16fdab6ebdaeffb11<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
    p1_w1: i64,
    p2_w1: i64,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut ::prebindgen_jni_runtime::Allocation<bench_flat::Handler>
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p2_w0 = if true { Some(p2_w1) } else { None };
            let p0_w0: &bench_flat::Handler = unsafe { &(*p0_w1_pointer).value };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let p2_w0: i64 = ::core::convert::Into::into(p2_w0.expect("active value"));
            let __prebindgen_argument_2 = p2_w0;
            let source_value = bench_flat::fire(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
                __prebindgen_argument_2,
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
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1rows_1for_1reduce_1fold_1587a8aef1d9a0485<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
    p1_w1: i64,
    __pb_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
    __pb_output_accepted: ::prebindgen_jni_runtime::jni::objects::JBooleanArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    match ::prebindgen_runtime::boundary(|| {
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
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = bench_flat::rows_for_reduce(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0_source = source_value;
            let ret_w2 = ret_w0_source.len();
            let mut ret_w0_commits = Vec::with_capacity(ret_w2);
            let ret_w0_callback_class = jni_environment
                .get_object_class(&__pb_callback)
                .map_err(|e| e.to_string())?;
            let ret_w0_callback_method = jni_environment
                .get_method_id(&ret_w0_callback_class, "run", "(JJJJJJJJ)V")
                .map_err(|e| e.to_string())?;
            jni_environment
                .delete_local_ref(ret_w0_callback_class)
                .map_err(|e| e.to_string())?;
            for (ret_w3, ret_w1) in ret_w0_source.into_iter().enumerate() {
                let ret_w0_callback_frame = unsafe {
                    ::prebindgen_jni_runtime::LocalFrame::new(&jni_environment, 16i32)
                }?;
                let ret_w4_record = ret_w1;
                let ret_w4_f0 = ret_w4_record.a;
                let ret_w4_f1 = ret_w4_record.b;
                let ret_w4_f2 = ret_w4_record.c;
                let ret_w4_f3 = ret_w4_record.d;
                let ret_w4_f4 = ret_w4_record.e;
                let ret_w4_f5 = ret_w4_record.f;
                let ret_w4_f6 = ret_w4_record.g;
                let ret_w4_f7 = ret_w4_record.h;
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
                                    j: ret_w4_f0,
                                },
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ret_w4_f1,
                                },
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ret_w4_f2,
                                },
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ret_w4_f3,
                                },
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ret_w4_f4,
                                },
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ret_w4_f5,
                                },
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ret_w4_f6,
                                },
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ret_w4_f7,
                                },
                            ],
                        )
                }
                    .map_err(|e| e.to_string())?;
                drop(ret_w0_callback_frame);
                ret_w0_commits.push(move || {});
            }
            let ret_w0 = Some(::prebindgen_jni_runtime::array(&mut jni_environment, 0)?);
            let __pb_direct_object = match (ret_w0).as_ref() {
                Some(value) => {
                    jni_environment.new_local_ref(value).map_err(|e| e.to_string())?
                }
                None => ::prebindgen_jni_runtime::jni::objects::JObject::null(),
            };
            let __pb_direct_result = unsafe {
                (::prebindgen_jni_runtime::jni::sys::jvalue {
                    l: __pb_direct_object.as_raw(),
                })
                    .l
            };
            jni_environment
                .set_boolean_array_region(&__pb_output_accepted, 0, &[1])
                .map_err(|e| e.to_string())?;
            let _: () = {
                for commit in ret_w0_commits {
                    commit();
                }
                direct_output = __pb_direct_result;
                success_transferred = true;
            };
            Ok(__pb_direct_result)
        }
    }) {
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
const _: () = {
    konst::assertc_eq!(
        bench_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
const _: () = assert!(
    ::core::mem::size_of:: < bench_flat::Handler > () != 0,
    "flat object `Handler` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < bench_flat::Packet > () != 0,
    "flat object `Packet` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < bench_flat::Row > () != 0,
    "flat object `Row` must have nonzero size"
);
