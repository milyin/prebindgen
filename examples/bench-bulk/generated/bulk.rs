/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_drop_1Handler_1bb814c4111561dda(
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
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_Range_1800d3dd3eceec6d {
    pub v0: ::prebindgen_jni_runtime::bulk::Range,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_i64_i64_789cd54eb117685f {
    pub v0: i64,
    pub v1: i64,
    pub v2: i64,
    pub v3: i64,
    pub v4: i64,
    pub v5: i64,
    pub v6: i64,
    pub v7: i64,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_i64_a60d3960993b9a77 {
    pub v0: i64,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_i64_i64_e5733c882bf9c317 {
    pub v0: i64,
    pub v1: i64,
    pub v2: i64,
    pub v3: i64,
    pub v4: i64,
    pub v5: i64,
    pub v6: i64,
    pub v7: i64,
    pub v8: ::prebindgen_jni_runtime::bulk::Range,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_Range_f4d5524a08b9867b {
    pub v0: ::prebindgen_jni_runtime::bulk::Range,
}
#[no_mangle]
pub extern "system" fn Java_bench_bulk_JNINative_bulkLayouts(
    mut env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
) -> ::prebindgen_jni_runtime::jni::sys::jlongArray {
    let result = (|| -> Result<_, String> {
        let mut data = vec![
            1i64, 5322912667254039845i64, if cfg!(target_endian = "big") { 1 } else { 0
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
                ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                ::core::mem::align_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v0),
                ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v1),
                ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v2),
                ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v3),
                ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v4),
                ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v5),
                ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v6),
                ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v7),
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
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_i64_i64_e5733c882bf9c317>(),
                ::core::mem::align_of::<__PbBulk_i64_i64_e5733c882bf9c317>(),
                ::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v0),
                ::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v1),
                ::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v2),
                ::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v3),
                ::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v4),
                ::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v5),
                ::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v6),
                ::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v7),
                ::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v8),
            ])
                .into_iter()
                .map(|v| v as i64),
        );
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_Range_f4d5524a08b9867b>(),
                ::core::mem::align_of::<__PbBulk_Range_f4d5524a08b9867b>(),
                ::core::mem::offset_of!(__PbBulk_Range_f4d5524a08b9867b, v0),
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1scalar_1a4a5da3f79ebbcaf<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JByteArray<'local>,
    p0_w1_length: i32,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                Vec::new()
            };
            let p0_w1_reader = ::prebindgen_jni_runtime::bulk::Reader::new(&p0_w1_bytes);
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_i64_a60d3960993b9a77>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_a60d3960993b9a77, v0))?
            } else {
                0
            };
            let p0_w2 = if true { Some(p0_w3) } else { None };
            let p0_w2: i64 = ::core::convert::Into::into(p0_w2.expect("active value"));
            let __prebindgen_argument_0 = p0_w2;
            let source_value = bench_flat::scalar(__prebindgen_argument_0);
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_i64_a60d3960993b9a77>(),
            );
            let ret_w1 = source_value;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_a60d3960993b9a77, v0),
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
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1row_1sum_1db65517187cece4c<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JByteArray<'local>,
    p0_w1_length: i32,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                Vec::new()
            };
            let p0_w1_reader = ::prebindgen_jni_runtime::bulk::Reader::new(&p0_w1_bytes);
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3_f0 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v0))?
            } else {
                0
            };
            let p0_w3_f1 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v1))?
            } else {
                0
            };
            let p0_w3_f2 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v2))?
            } else {
                0
            };
            let p0_w3_f3 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v3))?
            } else {
                0
            };
            let p0_w3_f4 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v4))?
            } else {
                0
            };
            let p0_w3_f5 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v5))?
            } else {
                0
            };
            let p0_w3_f6 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v6))?
            } else {
                0
            };
            let p0_w3_f7 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v7))?
            } else {
                0
            };
            let p0_w2_f0 = if true { Some(p0_w3_f0) } else { None };
            let p0_w2_f1 = if true { Some(p0_w3_f1) } else { None };
            let p0_w2_f2 = if true { Some(p0_w3_f2) } else { None };
            let p0_w2_f3 = if true { Some(p0_w3_f3) } else { None };
            let p0_w2_f4 = if true { Some(p0_w3_f4) } else { None };
            let p0_w2_f5 = if true { Some(p0_w3_f5) } else { None };
            let p0_w2_f6 = if true { Some(p0_w3_f6) } else { None };
            let p0_w2_f7 = if true { Some(p0_w3_f7) } else { None };
            let p0_w2_f0: i64 = ::core::convert::Into::into(
                p0_w2_f0.expect("active value"),
            );
            let p0_w2_f1: i64 = ::core::convert::Into::into(
                p0_w2_f1.expect("active value"),
            );
            let p0_w2_f2: i64 = ::core::convert::Into::into(
                p0_w2_f2.expect("active value"),
            );
            let p0_w2_f3: i64 = ::core::convert::Into::into(
                p0_w2_f3.expect("active value"),
            );
            let p0_w2_f4: i64 = ::core::convert::Into::into(
                p0_w2_f4.expect("active value"),
            );
            let p0_w2_f5: i64 = ::core::convert::Into::into(
                p0_w2_f5.expect("active value"),
            );
            let p0_w2_f6: i64 = ::core::convert::Into::into(
                p0_w2_f6.expect("active value"),
            );
            let p0_w2_f7: i64 = ::core::convert::Into::into(
                p0_w2_f7.expect("active value"),
            );
            let p0_w2: bench_flat::Row = bench_flat::Row {
                a: p0_w2_f0,
                b: p0_w2_f1,
                c: p0_w2_f2,
                d: p0_w2_f3,
                e: p0_w2_f4,
                f: p0_w2_f5,
                g: p0_w2_f6,
                h: p0_w2_f7,
            };
            let __prebindgen_argument_0 = p0_w2;
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
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1numbers_1sum_147543406a5cdf588<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JByteArray<'local>,
    p0_w1_length: i32,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                Vec::new()
            };
            let p0_w1_reader = ::prebindgen_jni_runtime::bulk::Reader::new(&p0_w1_bytes);
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_Range_1800d3dd3eceec6d>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3 = if true {
                p0_w1_reader
                    .range(::core::mem::offset_of!(__PbBulk_Range_1800d3dd3eceec6d, v0))?
            } else {
                None
            };
            let mut p0_w3_staged = Vec::new();
            if true {
                let (p0_w3_start, p0_w4) = p0_w1_reader
                    .rows(
                        p0_w3,
                        ::core::mem::size_of::<__PbBulk_i64_a60d3960993b9a77>(),
                    )?;
                for p0_w5 in 0..p0_w4 {
                    let p0_w7 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<__PbBulk_i64_a60d3960993b9a77>())
                                + ::core::mem::offset_of!(__PbBulk_i64_a60d3960993b9a77, v0),
                        )?;
                    let p0_w6 = if true { Some(p0_w7) } else { None };
                    p0_w3_staged
                        .push(move || {
                            move || -> ::core::result::Result<i64, String> {
                                let p0_w6: i64 = ::core::convert::Into::into(
                                    p0_w6.expect("active value"),
                                );
                                let p0_w2 = p0_w6;
                                Ok(p0_w2)
                            }
                        });
                }
            }
            let p0_w3_acquired: Vec<_> = p0_w3_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w2: Vec<i64> = p0_w3_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w2;
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
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1rows_1sum_1d4a5ecfa43bf26b9<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JByteArray<'local>,
    p0_w1_length: i32,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                Vec::new()
            };
            let p0_w1_reader = ::prebindgen_jni_runtime::bulk::Reader::new(&p0_w1_bytes);
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_Range_f4d5524a08b9867b>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3 = if true {
                p0_w1_reader
                    .range(::core::mem::offset_of!(__PbBulk_Range_f4d5524a08b9867b, v0))?
            } else {
                None
            };
            let mut p0_w3_staged = Vec::new();
            if true {
                let (p0_w3_start, p0_w4) = p0_w1_reader
                    .rows(
                        p0_w3,
                        ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                    )?;
                for p0_w5 in 0..p0_w4 {
                    let p0_w7_f0 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v0
                                ),
                        )?;
                    let p0_w7_f1 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v1
                                ),
                        )?;
                    let p0_w7_f2 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v2
                                ),
                        )?;
                    let p0_w7_f3 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v3
                                ),
                        )?;
                    let p0_w7_f4 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v4
                                ),
                        )?;
                    let p0_w7_f5 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v5
                                ),
                        )?;
                    let p0_w7_f6 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v6
                                ),
                        )?;
                    let p0_w7_f7 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v7
                                ),
                        )?;
                    let p0_w6_f0 = if true { Some(p0_w7_f0) } else { None };
                    let p0_w6_f1 = if true { Some(p0_w7_f1) } else { None };
                    let p0_w6_f2 = if true { Some(p0_w7_f2) } else { None };
                    let p0_w6_f3 = if true { Some(p0_w7_f3) } else { None };
                    let p0_w6_f4 = if true { Some(p0_w7_f4) } else { None };
                    let p0_w6_f5 = if true { Some(p0_w7_f5) } else { None };
                    let p0_w6_f6 = if true { Some(p0_w7_f6) } else { None };
                    let p0_w6_f7 = if true { Some(p0_w7_f7) } else { None };
                    p0_w3_staged
                        .push(move || {
                            move || -> ::core::result::Result<bench_flat::Row, String> {
                                let p0_w6_f0: i64 = ::core::convert::Into::into(
                                    p0_w6_f0.expect("active value"),
                                );
                                let p0_w6_f1: i64 = ::core::convert::Into::into(
                                    p0_w6_f1.expect("active value"),
                                );
                                let p0_w6_f2: i64 = ::core::convert::Into::into(
                                    p0_w6_f2.expect("active value"),
                                );
                                let p0_w6_f3: i64 = ::core::convert::Into::into(
                                    p0_w6_f3.expect("active value"),
                                );
                                let p0_w6_f4: i64 = ::core::convert::Into::into(
                                    p0_w6_f4.expect("active value"),
                                );
                                let p0_w6_f5: i64 = ::core::convert::Into::into(
                                    p0_w6_f5.expect("active value"),
                                );
                                let p0_w6_f6: i64 = ::core::convert::Into::into(
                                    p0_w6_f6.expect("active value"),
                                );
                                let p0_w6_f7: i64 = ::core::convert::Into::into(
                                    p0_w6_f7.expect("active value"),
                                );
                                let p0_w6: bench_flat::Row = bench_flat::Row {
                                    a: p0_w6_f0,
                                    b: p0_w6_f1,
                                    c: p0_w6_f2,
                                    d: p0_w6_f3,
                                    e: p0_w6_f4,
                                    f: p0_w6_f5,
                                    g: p0_w6_f6,
                                    h: p0_w6_f7,
                                };
                                let p0_w2 = p0_w6;
                                Ok(p0_w2)
                            }
                        });
                }
            }
            let p0_w3_acquired: Vec<_> = p0_w3_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w2: Vec<bench_flat::Row> = p0_w3_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w2.as_slice();
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
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1rows_1make_16f7eb2a3ee93e0ae<
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
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_f4d5524a08b9867b>(),
            );
            let ret_w1_source = source_value;
            let ret_w3 = ret_w1_source.len();
            let mut ret_w1_commits = Vec::with_capacity(ret_w3);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                    ::core::mem::align_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                )?;
            for (ret_w4, ret_w2) in ret_w1_source.into_iter().enumerate() {
                let ret_w5_record = ret_w2;
                let ret_w5_f0 = ret_w5_record.a;
                let ret_w5_f1 = ret_w5_record.b;
                let ret_w5_f2 = ret_w5_record.c;
                let ret_w5_f3 = ret_w5_record.d;
                let ret_w5_f4 = ret_w5_record.e;
                let ret_w5_f5 = ret_w5_record.f;
                let ret_w5_f6 = ret_w5_record.g;
                let ret_w5_f7 = ret_w5_record.h;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v0
                            ),
                        ret_w5_f0,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v1
                            ),
                        ret_w5_f1,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v2
                            ),
                        ret_w5_f2,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v3
                            ),
                        ret_w5_f3,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v4
                            ),
                        ret_w5_f4,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v5
                            ),
                        ret_w5_f5,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v6
                            ),
                        ret_w5_f6,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v7
                            ),
                        ret_w5_f7,
                    )?;
                ret_w1_commits.push(move || {});
            }
            let ret_w1 = Some(ret_w1);
            ret_w0_writer
                .range(
                    ::core::mem::offset_of!(__PbBulk_Range_f4d5524a08b9867b, v0),
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
                for commit in ret_w1_commits {
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
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1rows_1for_1reduce_13c2e82abbc965f1c<
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
            let source_value = bench_flat::rows_for_reduce(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_f4d5524a08b9867b>(),
            );
            let ret_w1_source = source_value;
            let ret_w3 = ret_w1_source.len();
            let mut ret_w1_commits = Vec::with_capacity(ret_w3);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                    ::core::mem::align_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                )?;
            for (ret_w4, ret_w2) in ret_w1_source.into_iter().enumerate() {
                let ret_w5_record = ret_w2;
                let ret_w5_f0 = ret_w5_record.a;
                let ret_w5_f1 = ret_w5_record.b;
                let ret_w5_f2 = ret_w5_record.c;
                let ret_w5_f3 = ret_w5_record.d;
                let ret_w5_f4 = ret_w5_record.e;
                let ret_w5_f5 = ret_w5_record.f;
                let ret_w5_f6 = ret_w5_record.g;
                let ret_w5_f7 = ret_w5_record.h;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v0
                            ),
                        ret_w5_f0,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v1
                            ),
                        ret_w5_f1,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v2
                            ),
                        ret_w5_f2,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v3
                            ),
                        ret_w5_f3,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v4
                            ),
                        ret_w5_f4,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v5
                            ),
                        ret_w5_f5,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v6
                            ),
                        ret_w5_f6,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v7
                            ),
                        ret_w5_f7,
                    )?;
                ret_w1_commits.push(move || {});
            }
            let ret_w1 = Some(ret_w1);
            ret_w0_writer
                .range(
                    ::core::mem::offset_of!(__PbBulk_Range_f4d5524a08b9867b, v0),
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
                for commit in ret_w1_commits {
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
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1rows_1echo_15c26c8cf4751c454<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JByteArray<'local>,
    p0_w1_length: i32,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                Vec::new()
            };
            let p0_w1_reader = ::prebindgen_jni_runtime::bulk::Reader::new(&p0_w1_bytes);
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_Range_f4d5524a08b9867b>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3 = if true {
                p0_w1_reader
                    .range(::core::mem::offset_of!(__PbBulk_Range_f4d5524a08b9867b, v0))?
            } else {
                None
            };
            let mut p0_w3_staged = Vec::new();
            if true {
                let (p0_w3_start, p0_w4) = p0_w1_reader
                    .rows(
                        p0_w3,
                        ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                    )?;
                for p0_w5 in 0..p0_w4 {
                    let p0_w7_f0 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v0
                                ),
                        )?;
                    let p0_w7_f1 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v1
                                ),
                        )?;
                    let p0_w7_f2 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v2
                                ),
                        )?;
                    let p0_w7_f3 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v3
                                ),
                        )?;
                    let p0_w7_f4 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v4
                                ),
                        )?;
                    let p0_w7_f5 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v5
                                ),
                        )?;
                    let p0_w7_f6 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v6
                                ),
                        )?;
                    let p0_w7_f7 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_789cd54eb117685f,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_789cd54eb117685f, v7
                                ),
                        )?;
                    let p0_w6_f0 = if true { Some(p0_w7_f0) } else { None };
                    let p0_w6_f1 = if true { Some(p0_w7_f1) } else { None };
                    let p0_w6_f2 = if true { Some(p0_w7_f2) } else { None };
                    let p0_w6_f3 = if true { Some(p0_w7_f3) } else { None };
                    let p0_w6_f4 = if true { Some(p0_w7_f4) } else { None };
                    let p0_w6_f5 = if true { Some(p0_w7_f5) } else { None };
                    let p0_w6_f6 = if true { Some(p0_w7_f6) } else { None };
                    let p0_w6_f7 = if true { Some(p0_w7_f7) } else { None };
                    p0_w3_staged
                        .push(move || {
                            move || -> ::core::result::Result<bench_flat::Row, String> {
                                let p0_w6_f0: i64 = ::core::convert::Into::into(
                                    p0_w6_f0.expect("active value"),
                                );
                                let p0_w6_f1: i64 = ::core::convert::Into::into(
                                    p0_w6_f1.expect("active value"),
                                );
                                let p0_w6_f2: i64 = ::core::convert::Into::into(
                                    p0_w6_f2.expect("active value"),
                                );
                                let p0_w6_f3: i64 = ::core::convert::Into::into(
                                    p0_w6_f3.expect("active value"),
                                );
                                let p0_w6_f4: i64 = ::core::convert::Into::into(
                                    p0_w6_f4.expect("active value"),
                                );
                                let p0_w6_f5: i64 = ::core::convert::Into::into(
                                    p0_w6_f5.expect("active value"),
                                );
                                let p0_w6_f6: i64 = ::core::convert::Into::into(
                                    p0_w6_f6.expect("active value"),
                                );
                                let p0_w6_f7: i64 = ::core::convert::Into::into(
                                    p0_w6_f7.expect("active value"),
                                );
                                let p0_w6: bench_flat::Row = bench_flat::Row {
                                    a: p0_w6_f0,
                                    b: p0_w6_f1,
                                    c: p0_w6_f2,
                                    d: p0_w6_f3,
                                    e: p0_w6_f4,
                                    f: p0_w6_f5,
                                    g: p0_w6_f6,
                                    h: p0_w6_f7,
                                };
                                let p0_w2 = p0_w6;
                                Ok(p0_w2)
                            }
                        });
                }
            }
            let p0_w3_acquired: Vec<_> = p0_w3_staged
                .into_iter()
                .map(|acquire| acquire())
                .collect();
            let p0_w2: Vec<bench_flat::Row> = p0_w3_acquired
                .into_iter()
                .map(|construct| construct())
                .collect::<::core::result::Result<_, String>>()?;
            let __prebindgen_argument_0 = p0_w2;
            let source_value = bench_flat::rows_echo(__prebindgen_argument_0);
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_f4d5524a08b9867b>(),
            );
            let ret_w1_source = source_value;
            let ret_w3 = ret_w1_source.len();
            let mut ret_w1_commits = Vec::with_capacity(ret_w3);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                    ::core::mem::align_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                )?;
            for (ret_w4, ret_w2) in ret_w1_source.into_iter().enumerate() {
                let ret_w5_record = ret_w2;
                let ret_w5_f0 = ret_w5_record.a;
                let ret_w5_f1 = ret_w5_record.b;
                let ret_w5_f2 = ret_w5_record.c;
                let ret_w5_f3 = ret_w5_record.d;
                let ret_w5_f4 = ret_w5_record.e;
                let ret_w5_f5 = ret_w5_record.f;
                let ret_w5_f6 = ret_w5_record.g;
                let ret_w5_f7 = ret_w5_record.h;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v0
                            ),
                        ret_w5_f0,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v1
                            ),
                        ret_w5_f1,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v2
                            ),
                        ret_w5_f2,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v3
                            ),
                        ret_w5_f3,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v4
                            ),
                        ret_w5_f4,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v5
                            ),
                        ret_w5_f5,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v6
                            ),
                        ret_w5_f6,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_789cd54eb117685f,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_789cd54eb117685f, v7
                            ),
                        ret_w5_f7,
                    )?;
                ret_w1_commits.push(move || {});
            }
            let ret_w1 = Some(ret_w1);
            ret_w0_writer
                .range(
                    ::core::mem::offset_of!(__PbBulk_Range_f4d5524a08b9867b, v0),
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
                for commit in ret_w1_commits {
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
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1packet_1sum_1c07ede9eb5de050f<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JByteArray<'local>,
    p0_w1_length: i32,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                Vec::new()
            };
            let p0_w1_reader = ::prebindgen_jni_runtime::bulk::Reader::new(&p0_w1_bytes);
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_i64_i64_e5733c882bf9c317>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3_f0_f0 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v0))?
            } else {
                0
            };
            let p0_w3_f0_f1 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v1))?
            } else {
                0
            };
            let p0_w3_f0_f2 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v2))?
            } else {
                0
            };
            let p0_w3_f0_f3 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v3))?
            } else {
                0
            };
            let p0_w3_f0_f4 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v4))?
            } else {
                0
            };
            let p0_w3_f0_f5 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v5))?
            } else {
                0
            };
            let p0_w3_f0_f6 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v6))?
            } else {
                0
            };
            let p0_w3_f0_f7 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v7))?
            } else {
                0
            };
            let p0_w3_f1 = if true {
                p0_w1_reader
                    .range(
                        ::core::mem::offset_of!(__PbBulk_i64_i64_e5733c882bf9c317, v8),
                    )?
            } else {
                None
            };
            let p0_w2_f0_f0 = if true { Some(p0_w3_f0_f0) } else { None };
            let p0_w2_f0_f1 = if true { Some(p0_w3_f0_f1) } else { None };
            let p0_w2_f0_f2 = if true { Some(p0_w3_f0_f2) } else { None };
            let p0_w2_f0_f3 = if true { Some(p0_w3_f0_f3) } else { None };
            let p0_w2_f0_f4 = if true { Some(p0_w3_f0_f4) } else { None };
            let p0_w2_f0_f5 = if true { Some(p0_w3_f0_f5) } else { None };
            let p0_w2_f0_f6 = if true { Some(p0_w3_f0_f6) } else { None };
            let p0_w2_f0_f7 = if true { Some(p0_w3_f0_f7) } else { None };
            let p0_w2_f1 = if true {
                Some(p0_w1_reader.buffer_range(p0_w3_f1)?.ok_or("null bulk bytes")?)
            } else {
                None
            };
            let p0_w2_f0_f0: i64 = ::core::convert::Into::into(
                p0_w2_f0_f0.expect("active value"),
            );
            let p0_w2_f0_f1: i64 = ::core::convert::Into::into(
                p0_w2_f0_f1.expect("active value"),
            );
            let p0_w2_f0_f2: i64 = ::core::convert::Into::into(
                p0_w2_f0_f2.expect("active value"),
            );
            let p0_w2_f0_f3: i64 = ::core::convert::Into::into(
                p0_w2_f0_f3.expect("active value"),
            );
            let p0_w2_f0_f4: i64 = ::core::convert::Into::into(
                p0_w2_f0_f4.expect("active value"),
            );
            let p0_w2_f0_f5: i64 = ::core::convert::Into::into(
                p0_w2_f0_f5.expect("active value"),
            );
            let p0_w2_f0_f6: i64 = ::core::convert::Into::into(
                p0_w2_f0_f6.expect("active value"),
            );
            let p0_w2_f0_f7: i64 = ::core::convert::Into::into(
                p0_w2_f0_f7.expect("active value"),
            );
            let p0_w2_f0: bench_flat::Row = bench_flat::Row {
                a: p0_w2_f0_f0,
                b: p0_w2_f0_f1,
                c: p0_w2_f0_f2,
                d: p0_w2_f0_f3,
                e: p0_w2_f0_f4,
                f: p0_w2_f0_f5,
                g: p0_w2_f0_f6,
                h: p0_w2_f0_f7,
            };
            let p0_w2_f1: Vec<u8> = ::core::convert::Into::into(
                p0_w2_f1.expect("active value"),
            );
            let p0_w2: bench_flat::Packet = bench_flat::Packet {
                header: p0_w2_f0,
                bytes: p0_w2_f1,
            };
            let __prebindgen_argument_0 = p0_w2;
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
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1handler_1new_1916a49ec8343b596<
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
                "([B)J",
                "bench.bulk.RowsCallback",
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
                                let mut callback_arg_0_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                                    ::core::mem::size_of::<__PbBulk_Range_f4d5524a08b9867b>(),
                                );
                                let callback_arg_0_w1_source = callback_value_0;
                                let callback_arg_0_w3 = callback_arg_0_w1_source.len();
                                let mut callback_arg_0_w1_commits = Vec::with_capacity(
                                    callback_arg_0_w3,
                                );
                                let callback_arg_0_w1 = callback_arg_0_w0_writer
                                    .rows(
                                        callback_arg_0_w3,
                                        ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                                        ::core::mem::align_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                                    )?;
                                for (callback_arg_0_w4, callback_arg_0_w2) in callback_arg_0_w1_source
                                    .into_iter()
                                    .enumerate()
                                {
                                    let callback_arg_0_w5_record = callback_arg_0_w2;
                                    let callback_arg_0_w5_f0 = callback_arg_0_w5_record.a;
                                    let callback_arg_0_w5_f1 = callback_arg_0_w5_record.b;
                                    let callback_arg_0_w5_f2 = callback_arg_0_w5_record.c;
                                    let callback_arg_0_w5_f3 = callback_arg_0_w5_record.d;
                                    let callback_arg_0_w5_f4 = callback_arg_0_w5_record.e;
                                    let callback_arg_0_w5_f5 = callback_arg_0_w5_record.f;
                                    let callback_arg_0_w5_f6 = callback_arg_0_w5_record.g;
                                    let callback_arg_0_w5_f7 = callback_arg_0_w5_record.h;
                                    callback_arg_0_w0_writer
                                        .write(
                                            (callback_arg_0_w1.offset as usize
                                                + callback_arg_0_w4
                                                    * ::core::mem::size_of::<
                                                        __PbBulk_i64_i64_789cd54eb117685f,
                                                    >())
                                                + ::core::mem::offset_of!(
                                                    __PbBulk_i64_i64_789cd54eb117685f, v0
                                                ),
                                            callback_arg_0_w5_f0,
                                        )?;
                                    callback_arg_0_w0_writer
                                        .write(
                                            (callback_arg_0_w1.offset as usize
                                                + callback_arg_0_w4
                                                    * ::core::mem::size_of::<
                                                        __PbBulk_i64_i64_789cd54eb117685f,
                                                    >())
                                                + ::core::mem::offset_of!(
                                                    __PbBulk_i64_i64_789cd54eb117685f, v1
                                                ),
                                            callback_arg_0_w5_f1,
                                        )?;
                                    callback_arg_0_w0_writer
                                        .write(
                                            (callback_arg_0_w1.offset as usize
                                                + callback_arg_0_w4
                                                    * ::core::mem::size_of::<
                                                        __PbBulk_i64_i64_789cd54eb117685f,
                                                    >())
                                                + ::core::mem::offset_of!(
                                                    __PbBulk_i64_i64_789cd54eb117685f, v2
                                                ),
                                            callback_arg_0_w5_f2,
                                        )?;
                                    callback_arg_0_w0_writer
                                        .write(
                                            (callback_arg_0_w1.offset as usize
                                                + callback_arg_0_w4
                                                    * ::core::mem::size_of::<
                                                        __PbBulk_i64_i64_789cd54eb117685f,
                                                    >())
                                                + ::core::mem::offset_of!(
                                                    __PbBulk_i64_i64_789cd54eb117685f, v3
                                                ),
                                            callback_arg_0_w5_f3,
                                        )?;
                                    callback_arg_0_w0_writer
                                        .write(
                                            (callback_arg_0_w1.offset as usize
                                                + callback_arg_0_w4
                                                    * ::core::mem::size_of::<
                                                        __PbBulk_i64_i64_789cd54eb117685f,
                                                    >())
                                                + ::core::mem::offset_of!(
                                                    __PbBulk_i64_i64_789cd54eb117685f, v4
                                                ),
                                            callback_arg_0_w5_f4,
                                        )?;
                                    callback_arg_0_w0_writer
                                        .write(
                                            (callback_arg_0_w1.offset as usize
                                                + callback_arg_0_w4
                                                    * ::core::mem::size_of::<
                                                        __PbBulk_i64_i64_789cd54eb117685f,
                                                    >())
                                                + ::core::mem::offset_of!(
                                                    __PbBulk_i64_i64_789cd54eb117685f, v5
                                                ),
                                            callback_arg_0_w5_f5,
                                        )?;
                                    callback_arg_0_w0_writer
                                        .write(
                                            (callback_arg_0_w1.offset as usize
                                                + callback_arg_0_w4
                                                    * ::core::mem::size_of::<
                                                        __PbBulk_i64_i64_789cd54eb117685f,
                                                    >())
                                                + ::core::mem::offset_of!(
                                                    __PbBulk_i64_i64_789cd54eb117685f, v6
                                                ),
                                            callback_arg_0_w5_f6,
                                        )?;
                                    callback_arg_0_w0_writer
                                        .write(
                                            (callback_arg_0_w1.offset as usize
                                                + callback_arg_0_w4
                                                    * ::core::mem::size_of::<
                                                        __PbBulk_i64_i64_789cd54eb117685f,
                                                    >())
                                                + ::core::mem::offset_of!(
                                                    __PbBulk_i64_i64_789cd54eb117685f, v7
                                                ),
                                            callback_arg_0_w5_f7,
                                        )?;
                                    callback_arg_0_w1_commits.push(move || {});
                                }
                                let callback_arg_0_w1 = Some(callback_arg_0_w1);
                                callback_arg_0_w0_writer
                                    .range(
                                        ::core::mem::offset_of!(
                                            __PbBulk_Range_f4d5524a08b9867b, v0
                                        ),
                                        callback_arg_0_w1,
                                    )?;
                                let callback_arg_0_w0 = ::prebindgen_jni_runtime::jni::objects::JObject::from(
                                    callback_environment
                                        .byte_array_from_slice(callback_arg_0_w0_writer.bytes())
                                        .map_err(|e| e.to_string())?,
                                );
                                let callback_java_0_0 = callback_arg_0_w0;
                                let arguments = [
                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                        l: callback_java_0_0.as_raw(),
                                    },
                                ];
                                {
                                    for commit in callback_arg_0_w1_commits {
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
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1fire_16fdab6ebdaeffb11<
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
