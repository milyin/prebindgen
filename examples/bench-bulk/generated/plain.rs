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
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_drop_1RowReturnHandler_10972ff7cb1556afa(
    mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,
    id: i64,
) {
    if let Err(e) = ::prebindgen_runtime::boundary(|| unsafe {
        ::prebindgen_jni_runtime::drop_pointer::<bench_flat::RowReturnHandler>(id)
    }) {
        ::prebindgen_jni_runtime::fail(&mut env, &e);
    }
}
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_drop_1WideRowReturnHandler_1ed750f04931eb9ca(
    mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,
    id: i64,
) {
    if let Err(e) = ::prebindgen_runtime::boundary(|| unsafe {
        ::prebindgen_jni_runtime::drop_pointer::<bench_flat::WideRowReturnHandler>(id)
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1row_1make_1bcd43e7b136151dd<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::row_make(__prebindgen_argument_0);
            let ret_w0_record = source_value;
            let ret_w0_f0 = ret_w0_record.a;
            let ret_w0_f1 = ret_w0_record.b;
            let ret_w0_f2 = ret_w0_record.c;
            let ret_w0_f3 = ret_w0_record.d;
            let ret_w0_f4 = ret_w0_record.e;
            let ret_w0_f5 = ret_w0_record.f;
            let ret_w0_f6 = ret_w0_record.g;
            let ret_w0_f7 = ret_w0_record.h;
            static __PB_RETURN_METHOD_0: ::prebindgen_jni_runtime::ReturnCallback = ::prebindgen_jni_runtime::ReturnCallback::new();
            unsafe {
                __PB_RETURN_METHOD_0
                    .invoke(
                        &mut jni_environment,
                        &__pb_return,
                        "success0",
                        "(JJJJJJJJ)V",
                        &[
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f0,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f1,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f2,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f3,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f4,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f5,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f6,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f7,
                            },
                        ],
                    )
            }?;
            let _: () = {
                success_transferred = true;
            };
            Ok(())
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1wide_1row_1make_11dbd8ca2fd4616ac<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::wide_row_make(__prebindgen_argument_0);
            let ret_w0_record = source_value;
            let ret_w0_f0 = ret_w0_record.f0;
            let ret_w0_f1 = ret_w0_record.f1;
            let ret_w0_f2 = ret_w0_record.f2;
            let ret_w0_f3 = ret_w0_record.f3;
            let ret_w0_f4 = ret_w0_record.f4;
            let ret_w0_f5 = ret_w0_record.f5;
            let ret_w0_f6 = ret_w0_record.f6;
            let ret_w0_f7 = ret_w0_record.f7;
            let ret_w0_f8 = ret_w0_record.f8;
            let ret_w0_f9 = ret_w0_record.f9;
            let ret_w0_f10 = ret_w0_record.f10;
            let ret_w0_f11 = ret_w0_record.f11;
            let ret_w0_f12 = ret_w0_record.f12;
            let ret_w0_f13 = ret_w0_record.f13;
            let ret_w0_f14 = ret_w0_record.f14;
            let ret_w0_f15 = ret_w0_record.f15;
            let ret_w0_f16 = ret_w0_record.f16;
            let ret_w0_f17 = ret_w0_record.f17;
            let ret_w0_f18 = ret_w0_record.f18;
            let ret_w0_f19 = ret_w0_record.f19;
            let ret_w0_f20 = ret_w0_record.f20;
            let ret_w0_f21 = ret_w0_record.f21;
            let ret_w0_f22 = ret_w0_record.f22;
            let ret_w0_f23 = ret_w0_record.f23;
            let ret_w0_f24 = ret_w0_record.f24;
            let ret_w0_f25 = ret_w0_record.f25;
            let ret_w0_f26 = ret_w0_record.f26;
            let ret_w0_f27 = ret_w0_record.f27;
            let ret_w0_f28 = ret_w0_record.f28;
            let ret_w0_f29 = ret_w0_record.f29;
            let ret_w0_f30 = ret_w0_record.f30;
            let ret_w0_f31 = ret_w0_record.f31;
            static __PB_RETURN_METHOD_0: ::prebindgen_jni_runtime::ReturnCallback = ::prebindgen_jni_runtime::ReturnCallback::new();
            unsafe {
                __PB_RETURN_METHOD_0
                    .invoke(
                        &mut jni_environment,
                        &__pb_return,
                        "success0",
                        "(JJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJJ)V",
                        &[
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f0,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f1,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f2,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f3,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f4,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f5,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f6,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f7,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f8,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f9,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f10,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f11,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f12,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f13,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f14,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f15,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f16,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f17,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f18,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f19,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f20,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f21,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f22,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f23,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f24,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f25,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f26,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f27,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f28,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f29,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f30,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f31,
                            },
                        ],
                    )
            }?;
            let _: () = {
                success_transferred = true;
            };
            Ok(())
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1wide_1row_1sum_1861bdfef124a25ad<
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
    p0_w1_f8: i64,
    p0_w1_f9: i64,
    p0_w1_f10: i64,
    p0_w1_f11: i64,
    p0_w1_f12: i64,
    p0_w1_f13: i64,
    p0_w1_f14: i64,
    p0_w1_f15: i64,
    p0_w1_f16: i64,
    p0_w1_f17: i64,
    p0_w1_f18: i64,
    p0_w1_f19: i64,
    p0_w1_f20: i64,
    p0_w1_f21: i64,
    p0_w1_f22: i64,
    p0_w1_f23: i64,
    p0_w1_f24: i64,
    p0_w1_f25: i64,
    p0_w1_f26: i64,
    p0_w1_f27: i64,
    p0_w1_f28: i64,
    p0_w1_f29: i64,
    p0_w1_f30: i64,
    p0_w1_f31: i64,
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
            let p0_w0_f8 = if true { Some(p0_w1_f8) } else { None };
            let p0_w0_f9 = if true { Some(p0_w1_f9) } else { None };
            let p0_w0_f10 = if true { Some(p0_w1_f10) } else { None };
            let p0_w0_f11 = if true { Some(p0_w1_f11) } else { None };
            let p0_w0_f12 = if true { Some(p0_w1_f12) } else { None };
            let p0_w0_f13 = if true { Some(p0_w1_f13) } else { None };
            let p0_w0_f14 = if true { Some(p0_w1_f14) } else { None };
            let p0_w0_f15 = if true { Some(p0_w1_f15) } else { None };
            let p0_w0_f16 = if true { Some(p0_w1_f16) } else { None };
            let p0_w0_f17 = if true { Some(p0_w1_f17) } else { None };
            let p0_w0_f18 = if true { Some(p0_w1_f18) } else { None };
            let p0_w0_f19 = if true { Some(p0_w1_f19) } else { None };
            let p0_w0_f20 = if true { Some(p0_w1_f20) } else { None };
            let p0_w0_f21 = if true { Some(p0_w1_f21) } else { None };
            let p0_w0_f22 = if true { Some(p0_w1_f22) } else { None };
            let p0_w0_f23 = if true { Some(p0_w1_f23) } else { None };
            let p0_w0_f24 = if true { Some(p0_w1_f24) } else { None };
            let p0_w0_f25 = if true { Some(p0_w1_f25) } else { None };
            let p0_w0_f26 = if true { Some(p0_w1_f26) } else { None };
            let p0_w0_f27 = if true { Some(p0_w1_f27) } else { None };
            let p0_w0_f28 = if true { Some(p0_w1_f28) } else { None };
            let p0_w0_f29 = if true { Some(p0_w1_f29) } else { None };
            let p0_w0_f30 = if true { Some(p0_w1_f30) } else { None };
            let p0_w0_f31 = if true { Some(p0_w1_f31) } else { None };
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
            let p0_w0_f8: i64 = ::core::convert::Into::into(
                p0_w0_f8.expect("active value"),
            );
            let p0_w0_f9: i64 = ::core::convert::Into::into(
                p0_w0_f9.expect("active value"),
            );
            let p0_w0_f10: i64 = ::core::convert::Into::into(
                p0_w0_f10.expect("active value"),
            );
            let p0_w0_f11: i64 = ::core::convert::Into::into(
                p0_w0_f11.expect("active value"),
            );
            let p0_w0_f12: i64 = ::core::convert::Into::into(
                p0_w0_f12.expect("active value"),
            );
            let p0_w0_f13: i64 = ::core::convert::Into::into(
                p0_w0_f13.expect("active value"),
            );
            let p0_w0_f14: i64 = ::core::convert::Into::into(
                p0_w0_f14.expect("active value"),
            );
            let p0_w0_f15: i64 = ::core::convert::Into::into(
                p0_w0_f15.expect("active value"),
            );
            let p0_w0_f16: i64 = ::core::convert::Into::into(
                p0_w0_f16.expect("active value"),
            );
            let p0_w0_f17: i64 = ::core::convert::Into::into(
                p0_w0_f17.expect("active value"),
            );
            let p0_w0_f18: i64 = ::core::convert::Into::into(
                p0_w0_f18.expect("active value"),
            );
            let p0_w0_f19: i64 = ::core::convert::Into::into(
                p0_w0_f19.expect("active value"),
            );
            let p0_w0_f20: i64 = ::core::convert::Into::into(
                p0_w0_f20.expect("active value"),
            );
            let p0_w0_f21: i64 = ::core::convert::Into::into(
                p0_w0_f21.expect("active value"),
            );
            let p0_w0_f22: i64 = ::core::convert::Into::into(
                p0_w0_f22.expect("active value"),
            );
            let p0_w0_f23: i64 = ::core::convert::Into::into(
                p0_w0_f23.expect("active value"),
            );
            let p0_w0_f24: i64 = ::core::convert::Into::into(
                p0_w0_f24.expect("active value"),
            );
            let p0_w0_f25: i64 = ::core::convert::Into::into(
                p0_w0_f25.expect("active value"),
            );
            let p0_w0_f26: i64 = ::core::convert::Into::into(
                p0_w0_f26.expect("active value"),
            );
            let p0_w0_f27: i64 = ::core::convert::Into::into(
                p0_w0_f27.expect("active value"),
            );
            let p0_w0_f28: i64 = ::core::convert::Into::into(
                p0_w0_f28.expect("active value"),
            );
            let p0_w0_f29: i64 = ::core::convert::Into::into(
                p0_w0_f29.expect("active value"),
            );
            let p0_w0_f30: i64 = ::core::convert::Into::into(
                p0_w0_f30.expect("active value"),
            );
            let p0_w0_f31: i64 = ::core::convert::Into::into(
                p0_w0_f31.expect("active value"),
            );
            let p0_w0: bench_flat::WideRow = bench_flat::WideRow {
                f0: p0_w0_f0,
                f1: p0_w0_f1,
                f2: p0_w0_f2,
                f3: p0_w0_f3,
                f4: p0_w0_f4,
                f5: p0_w0_f5,
                f6: p0_w0_f6,
                f7: p0_w0_f7,
                f8: p0_w0_f8,
                f9: p0_w0_f9,
                f10: p0_w0_f10,
                f11: p0_w0_f11,
                f12: p0_w0_f12,
                f13: p0_w0_f13,
                f14: p0_w0_f14,
                f15: p0_w0_f15,
                f16: p0_w0_f16,
                f17: p0_w0_f17,
                f18: p0_w0_f18,
                f19: p0_w0_f19,
                f20: p0_w0_f20,
                f21: p0_w0_f21,
                f22: p0_w0_f22,
                f23: p0_w0_f23,
                f24: p0_w0_f24,
                f25: p0_w0_f25,
                f26: p0_w0_f26,
                f27: p0_w0_f27,
                f28: p0_w0_f28,
                f29: p0_w0_f29,
                f30: p0_w0_f30,
                f31: p0_w0_f31,
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::wide_row_sum(__prebindgen_argument_0);
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1row_1return_1handler_1new_19a8fcadcab7b9640<
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
                "(J)[Ljava/lang/Object;",
                "bench.plain.RowReturnCallback",
            )?;
            #[allow(unused_mut, clippy::collapsible_if)]
            let p0_callback_value = move |callback_source_0: i64| {
                p0_callback_retained
                    .with_env_result(
                        26i32,
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
                                            ::prebindgen_jni_runtime::jni::signature::ReturnType::Object,
                                            &arguments,
                                        )
                                }?;
                                let callback_result_rows = ::prebindgen_jni_runtime::jni::objects::JObjectArray::from(
                                    callback_result.l().map_err(|e| e.to_string())?,
                                );
                                let callback_return_w1_f0 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    0usize,
                                )?;
                                let callback_return_w1_f1 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    1usize,
                                )?;
                                let callback_return_w1_f2 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    2usize,
                                )?;
                                let callback_return_w1_f3 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    3usize,
                                )?;
                                let callback_return_w1_f4 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    4usize,
                                )?;
                                let callback_return_w1_f5 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    5usize,
                                )?;
                                let callback_return_w1_f6 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    6usize,
                                )?;
                                let callback_return_w1_f7 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    7usize,
                                )?;
                                let callback_return_w0_f0 = if true {
                                    Some(callback_return_w1_f0)
                                } else {
                                    None
                                };
                                let callback_return_w0_f1 = if true {
                                    Some(callback_return_w1_f1)
                                } else {
                                    None
                                };
                                let callback_return_w0_f2 = if true {
                                    Some(callback_return_w1_f2)
                                } else {
                                    None
                                };
                                let callback_return_w0_f3 = if true {
                                    Some(callback_return_w1_f3)
                                } else {
                                    None
                                };
                                let callback_return_w0_f4 = if true {
                                    Some(callback_return_w1_f4)
                                } else {
                                    None
                                };
                                let callback_return_w0_f5 = if true {
                                    Some(callback_return_w1_f5)
                                } else {
                                    None
                                };
                                let callback_return_w0_f6 = if true {
                                    Some(callback_return_w1_f6)
                                } else {
                                    None
                                };
                                let callback_return_w0_f7 = if true {
                                    Some(callback_return_w1_f7)
                                } else {
                                    None
                                };
                                let callback_return_w0_f0: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f0.expect("active value"),
                                );
                                let callback_return_w0_f1: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f1.expect("active value"),
                                );
                                let callback_return_w0_f2: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f2.expect("active value"),
                                );
                                let callback_return_w0_f3: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f3.expect("active value"),
                                );
                                let callback_return_w0_f4: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f4.expect("active value"),
                                );
                                let callback_return_w0_f5: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f5.expect("active value"),
                                );
                                let callback_return_w0_f6: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f6.expect("active value"),
                                );
                                let callback_return_w0_f7: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f7.expect("active value"),
                                );
                                let callback_return_w0: bench_flat::Row = bench_flat::Row {
                                    a: callback_return_w0_f0,
                                    b: callback_return_w0_f1,
                                    c: callback_return_w0_f2,
                                    d: callback_return_w0_f3,
                                    e: callback_return_w0_f4,
                                    f: callback_return_w0_f5,
                                    g: callback_return_w0_f6,
                                    h: callback_return_w0_f7,
                                };
                                let callback_return_value = callback_return_w0;
                                Ok(callback_return_value)
                            }
                        },
                    )
                    .unwrap_or_else(|error| panic!("callback failed: {}", error))
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = bench_flat::row_return_handler_new(
                __prebindgen_argument_0,
            );
            let mut ret_w0 = Box::new(::prebindgen_jni_runtime::Allocation {
                value: source_value,
            });
            let __pb_direct_result = unsafe {
                (::prebindgen_jni_runtime::jni::sys::jvalue {
                    j: ((&mut *ret_w0)
                        as *mut ::prebindgen_jni_runtime::Allocation<
                            bench_flat::RowReturnHandler,
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1wide_1row_1return_1handler_1new_167f38d7168020371<
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
                "(J)[Ljava/lang/Object;",
                "bench.plain.WideRowReturnCallback",
            )?;
            #[allow(unused_mut, clippy::collapsible_if)]
            let p0_callback_value = move |callback_source_0: i64| {
                p0_callback_retained
                    .with_env_result(
                        50i32,
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
                                            ::prebindgen_jni_runtime::jni::signature::ReturnType::Object,
                                            &arguments,
                                        )
                                }?;
                                let callback_result_rows = ::prebindgen_jni_runtime::jni::objects::JObjectArray::from(
                                    callback_result.l().map_err(|e| e.to_string())?,
                                );
                                let callback_return_w1_f0 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    0usize,
                                )?;
                                let callback_return_w1_f1 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    1usize,
                                )?;
                                let callback_return_w1_f2 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    2usize,
                                )?;
                                let callback_return_w1_f3 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    3usize,
                                )?;
                                let callback_return_w1_f4 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    4usize,
                                )?;
                                let callback_return_w1_f5 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    5usize,
                                )?;
                                let callback_return_w1_f6 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    6usize,
                                )?;
                                let callback_return_w1_f7 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    7usize,
                                )?;
                                let callback_return_w1_f8 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    8usize,
                                )?;
                                let callback_return_w1_f9 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    9usize,
                                )?;
                                let callback_return_w1_f10 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    10usize,
                                )?;
                                let callback_return_w1_f11 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    11usize,
                                )?;
                                let callback_return_w1_f12 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    12usize,
                                )?;
                                let callback_return_w1_f13 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    13usize,
                                )?;
                                let callback_return_w1_f14 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    14usize,
                                )?;
                                let callback_return_w1_f15 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    15usize,
                                )?;
                                let callback_return_w1_f16 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    16usize,
                                )?;
                                let callback_return_w1_f17 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    17usize,
                                )?;
                                let callback_return_w1_f18 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    18usize,
                                )?;
                                let callback_return_w1_f19 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    19usize,
                                )?;
                                let callback_return_w1_f20 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    20usize,
                                )?;
                                let callback_return_w1_f21 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    21usize,
                                )?;
                                let callback_return_w1_f22 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    22usize,
                                )?;
                                let callback_return_w1_f23 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    23usize,
                                )?;
                                let callback_return_w1_f24 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    24usize,
                                )?;
                                let callback_return_w1_f25 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    25usize,
                                )?;
                                let callback_return_w1_f26 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    26usize,
                                )?;
                                let callback_return_w1_f27 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    27usize,
                                )?;
                                let callback_return_w1_f28 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    28usize,
                                )?;
                                let callback_return_w1_f29 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    29usize,
                                )?;
                                let callback_return_w1_f30 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    30usize,
                                )?;
                                let callback_return_w1_f31 = ::prebindgen_jni_runtime::row_i64(
                                    &mut callback_environment,
                                    &(callback_result_rows),
                                    31usize,
                                )?;
                                let callback_return_w0_f0 = if true {
                                    Some(callback_return_w1_f0)
                                } else {
                                    None
                                };
                                let callback_return_w0_f1 = if true {
                                    Some(callback_return_w1_f1)
                                } else {
                                    None
                                };
                                let callback_return_w0_f2 = if true {
                                    Some(callback_return_w1_f2)
                                } else {
                                    None
                                };
                                let callback_return_w0_f3 = if true {
                                    Some(callback_return_w1_f3)
                                } else {
                                    None
                                };
                                let callback_return_w0_f4 = if true {
                                    Some(callback_return_w1_f4)
                                } else {
                                    None
                                };
                                let callback_return_w0_f5 = if true {
                                    Some(callback_return_w1_f5)
                                } else {
                                    None
                                };
                                let callback_return_w0_f6 = if true {
                                    Some(callback_return_w1_f6)
                                } else {
                                    None
                                };
                                let callback_return_w0_f7 = if true {
                                    Some(callback_return_w1_f7)
                                } else {
                                    None
                                };
                                let callback_return_w0_f8 = if true {
                                    Some(callback_return_w1_f8)
                                } else {
                                    None
                                };
                                let callback_return_w0_f9 = if true {
                                    Some(callback_return_w1_f9)
                                } else {
                                    None
                                };
                                let callback_return_w0_f10 = if true {
                                    Some(callback_return_w1_f10)
                                } else {
                                    None
                                };
                                let callback_return_w0_f11 = if true {
                                    Some(callback_return_w1_f11)
                                } else {
                                    None
                                };
                                let callback_return_w0_f12 = if true {
                                    Some(callback_return_w1_f12)
                                } else {
                                    None
                                };
                                let callback_return_w0_f13 = if true {
                                    Some(callback_return_w1_f13)
                                } else {
                                    None
                                };
                                let callback_return_w0_f14 = if true {
                                    Some(callback_return_w1_f14)
                                } else {
                                    None
                                };
                                let callback_return_w0_f15 = if true {
                                    Some(callback_return_w1_f15)
                                } else {
                                    None
                                };
                                let callback_return_w0_f16 = if true {
                                    Some(callback_return_w1_f16)
                                } else {
                                    None
                                };
                                let callback_return_w0_f17 = if true {
                                    Some(callback_return_w1_f17)
                                } else {
                                    None
                                };
                                let callback_return_w0_f18 = if true {
                                    Some(callback_return_w1_f18)
                                } else {
                                    None
                                };
                                let callback_return_w0_f19 = if true {
                                    Some(callback_return_w1_f19)
                                } else {
                                    None
                                };
                                let callback_return_w0_f20 = if true {
                                    Some(callback_return_w1_f20)
                                } else {
                                    None
                                };
                                let callback_return_w0_f21 = if true {
                                    Some(callback_return_w1_f21)
                                } else {
                                    None
                                };
                                let callback_return_w0_f22 = if true {
                                    Some(callback_return_w1_f22)
                                } else {
                                    None
                                };
                                let callback_return_w0_f23 = if true {
                                    Some(callback_return_w1_f23)
                                } else {
                                    None
                                };
                                let callback_return_w0_f24 = if true {
                                    Some(callback_return_w1_f24)
                                } else {
                                    None
                                };
                                let callback_return_w0_f25 = if true {
                                    Some(callback_return_w1_f25)
                                } else {
                                    None
                                };
                                let callback_return_w0_f26 = if true {
                                    Some(callback_return_w1_f26)
                                } else {
                                    None
                                };
                                let callback_return_w0_f27 = if true {
                                    Some(callback_return_w1_f27)
                                } else {
                                    None
                                };
                                let callback_return_w0_f28 = if true {
                                    Some(callback_return_w1_f28)
                                } else {
                                    None
                                };
                                let callback_return_w0_f29 = if true {
                                    Some(callback_return_w1_f29)
                                } else {
                                    None
                                };
                                let callback_return_w0_f30 = if true {
                                    Some(callback_return_w1_f30)
                                } else {
                                    None
                                };
                                let callback_return_w0_f31 = if true {
                                    Some(callback_return_w1_f31)
                                } else {
                                    None
                                };
                                let callback_return_w0_f0: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f0.expect("active value"),
                                );
                                let callback_return_w0_f1: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f1.expect("active value"),
                                );
                                let callback_return_w0_f2: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f2.expect("active value"),
                                );
                                let callback_return_w0_f3: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f3.expect("active value"),
                                );
                                let callback_return_w0_f4: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f4.expect("active value"),
                                );
                                let callback_return_w0_f5: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f5.expect("active value"),
                                );
                                let callback_return_w0_f6: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f6.expect("active value"),
                                );
                                let callback_return_w0_f7: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f7.expect("active value"),
                                );
                                let callback_return_w0_f8: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f8.expect("active value"),
                                );
                                let callback_return_w0_f9: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f9.expect("active value"),
                                );
                                let callback_return_w0_f10: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f10.expect("active value"),
                                );
                                let callback_return_w0_f11: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f11.expect("active value"),
                                );
                                let callback_return_w0_f12: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f12.expect("active value"),
                                );
                                let callback_return_w0_f13: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f13.expect("active value"),
                                );
                                let callback_return_w0_f14: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f14.expect("active value"),
                                );
                                let callback_return_w0_f15: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f15.expect("active value"),
                                );
                                let callback_return_w0_f16: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f16.expect("active value"),
                                );
                                let callback_return_w0_f17: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f17.expect("active value"),
                                );
                                let callback_return_w0_f18: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f18.expect("active value"),
                                );
                                let callback_return_w0_f19: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f19.expect("active value"),
                                );
                                let callback_return_w0_f20: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f20.expect("active value"),
                                );
                                let callback_return_w0_f21: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f21.expect("active value"),
                                );
                                let callback_return_w0_f22: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f22.expect("active value"),
                                );
                                let callback_return_w0_f23: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f23.expect("active value"),
                                );
                                let callback_return_w0_f24: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f24.expect("active value"),
                                );
                                let callback_return_w0_f25: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f25.expect("active value"),
                                );
                                let callback_return_w0_f26: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f26.expect("active value"),
                                );
                                let callback_return_w0_f27: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f27.expect("active value"),
                                );
                                let callback_return_w0_f28: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f28.expect("active value"),
                                );
                                let callback_return_w0_f29: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f29.expect("active value"),
                                );
                                let callback_return_w0_f30: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f30.expect("active value"),
                                );
                                let callback_return_w0_f31: i64 = ::core::convert::Into::into(
                                    callback_return_w0_f31.expect("active value"),
                                );
                                let callback_return_w0: bench_flat::WideRow = bench_flat::WideRow {
                                    f0: callback_return_w0_f0,
                                    f1: callback_return_w0_f1,
                                    f2: callback_return_w0_f2,
                                    f3: callback_return_w0_f3,
                                    f4: callback_return_w0_f4,
                                    f5: callback_return_w0_f5,
                                    f6: callback_return_w0_f6,
                                    f7: callback_return_w0_f7,
                                    f8: callback_return_w0_f8,
                                    f9: callback_return_w0_f9,
                                    f10: callback_return_w0_f10,
                                    f11: callback_return_w0_f11,
                                    f12: callback_return_w0_f12,
                                    f13: callback_return_w0_f13,
                                    f14: callback_return_w0_f14,
                                    f15: callback_return_w0_f15,
                                    f16: callback_return_w0_f16,
                                    f17: callback_return_w0_f17,
                                    f18: callback_return_w0_f18,
                                    f19: callback_return_w0_f19,
                                    f20: callback_return_w0_f20,
                                    f21: callback_return_w0_f21,
                                    f22: callback_return_w0_f22,
                                    f23: callback_return_w0_f23,
                                    f24: callback_return_w0_f24,
                                    f25: callback_return_w0_f25,
                                    f26: callback_return_w0_f26,
                                    f27: callback_return_w0_f27,
                                    f28: callback_return_w0_f28,
                                    f29: callback_return_w0_f29,
                                    f30: callback_return_w0_f30,
                                    f31: callback_return_w0_f31,
                                };
                                let callback_return_value = callback_return_w0;
                                Ok(callback_return_value)
                            }
                        },
                    )
                    .unwrap_or_else(|error| panic!("callback failed: {}", error))
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = bench_flat::wide_row_return_handler_new(
                __prebindgen_argument_0,
            );
            let mut ret_w0 = Box::new(::prebindgen_jni_runtime::Allocation {
                value: source_value,
            });
            let __pb_direct_result = unsafe {
                (::prebindgen_jni_runtime::jni::sys::jvalue {
                    j: ((&mut *ret_w0)
                        as *mut ::prebindgen_jni_runtime::Allocation<
                            bench_flat::WideRowReturnHandler,
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1row_1return_1fire_17c8a1183dcdd277c<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
    p1_w1: i64,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1
                    as *mut ::prebindgen_jni_runtime::Allocation<
                        bench_flat::RowReturnHandler,
                    >
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p0_w0: &bench_flat::RowReturnHandler = unsafe {
                &(*p0_w1_pointer).value
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = bench_flat::row_return_fire(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1wide_1row_1return_1fire_188ad5bbf9c8c73e2<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
    p1_w1: i64,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1
                    as *mut ::prebindgen_jni_runtime::Allocation<
                        bench_flat::WideRowReturnHandler,
                    >
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p0_w0: &bench_flat::WideRowReturnHandler = unsafe {
                &(*p0_w1_pointer).value
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let source_value = bench_flat::wide_row_return_fire(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1control_1b0073ee32096caea<
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
            let source_value = bench_flat::control(__prebindgen_argument_0);
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
            let mut p0_w0: Vec<i64> = Vec::new();
            if true {
                if p0_w1.is_null() {
                    return Err("null collection".into());
                }
                let p0_w2 = jni_environment
                    .get_array_length(&p0_w1)
                    .map_err(|e| e.to_string())? as usize;
                p0_w0.reserve_exact(p0_w2);
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
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<i64, String> {
                        let p0_w4: i64 = ::core::convert::Into::into(
                            p0_w4.expect("active value"),
                        );
                        let p0_w0 = p0_w4;
                        Ok(p0_w0)
                    })();
                    drop(p0_w1_frame);
                    p0_w0.push(p0_w1_element_built?);
                }
            }
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
                p0_w1_staged.reserve_exact(p0_w2);
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
                p0_w1_staged.reserve_exact(p0_w2);
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
const _: () = assert!(
    ::core::mem::size_of:: < bench_flat::RowReturnHandler > () != 0,
    "flat object `RowReturnHandler` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < bench_flat::WideRow > () != 0,
    "flat object `WideRow` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < bench_flat::WideRowReturnHandler > () != 0,
    "flat object `WideRowReturnHandler` must have nonzero size"
);
