/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_drop_1Storage_19651261943550368(
    mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,
    id: i64,
) {
    if let Err(e) = ::prebindgen_runtime::boundary(|| unsafe {
        ::prebindgen_jni_runtime::drop_pointer::<perftest_flat::Storage>(id)
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_call_1storage_1new_1a15b46920214a065<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobjectArray {
    #[allow(unused_mut)]
    let mut domain_output = None;
    match ::prebindgen_runtime::boundary(|| {
        {
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let source_value = perftest_flat::storage_new();
            let mut ret_w0 = Box::new(::prebindgen_jni_runtime::Allocation {
                value: source_value,
            });
            let result = ::prebindgen_jni_runtime::array(&mut jni_environment, 2usize)?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Long",
                    "valueOf",
                    "(J)Ljava/lang/Long;",
                    &[
                        ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                            ((&mut *ret_w0)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    perftest_flat::Storage,
                                >) as i64,
                        ),
                    ],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            jni_environment
                .set_object_array_element(&result, 1i32, &object)
                .map_err(|e| e.to_string())?;
            jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
            let _: () = {
                let _ = Box::into_raw(ret_w0);
            };
            Ok(result.into_raw())
        }
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_call_1storage_1get_1b7f3d4a82ad753f5<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
) -> ::prebindgen_jni_runtime::jni::sys::jobjectArray {
    #[allow(unused_mut)]
    let mut domain_output = None;
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1
                    as *mut ::prebindgen_jni_runtime::Allocation<perftest_flat::Storage>
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {
                if true {
                    for identity in ::core::iter::once((p0_w1_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, false));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let p0_w0: &perftest_flat::Storage = unsafe { &(*p0_w1_pointer).value };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::storage_get(__prebindgen_argument_0);
            let (
                ret_w0,
                (ret_w0_value0, ret_w0_value1, ret_w0_value2, ret_w0_value3),
                ret_w0_commit,
            ) = if let Some(ret_w1) = source_value {
                let ret_w2 = perftest_flat::payload_to_struct(&ret_w1);
                let ret_w3 = &(ret_w2);
                let ret_w4: &i64 = &ret_w3.id;
                let ret_w4 = *ret_w4;
                let ret_w5: &i32 = &ret_w3.seq;
                let ret_w5 = *ret_w5;
                let ret_w6: &f64 = &ret_w3.value;
                let ret_w6 = *ret_w6;
                let ret_w7: &bool = &ret_w3.flag;
                let ret_w7 = u8::from(*ret_w7);
                let ret_w0_values = (ret_w4, ret_w5, ret_w6, ret_w7);
                (1i32, ret_w0_values, Some(move || {}))
            } else {
                (0i32, (0, 0, 0.0, 0), None)
            };
            let result = ::prebindgen_jni_runtime::array(&mut jni_environment, 6usize)?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Integer",
                    "valueOf",
                    "(I)Ljava/lang/Integer;",
                    &[::prebindgen_jni_runtime::jni::objects::JValue::Int(ret_w0)],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            jni_environment
                .set_object_array_element(&result, 1i32, &object)
                .map_err(|e| e.to_string())?;
            jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Long",
                    "valueOf",
                    "(J)Ljava/lang/Long;",
                    &[
                        ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                            ret_w0_value0,
                        ),
                    ],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            jni_environment
                .set_object_array_element(&result, 2i32, &object)
                .map_err(|e| e.to_string())?;
            jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Integer",
                    "valueOf",
                    "(I)Ljava/lang/Integer;",
                    &[::prebindgen_jni_runtime::jni::objects::JValue::Int(ret_w0_value1)],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            jni_environment
                .set_object_array_element(&result, 3i32, &object)
                .map_err(|e| e.to_string())?;
            jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Double",
                    "valueOf",
                    "(D)Ljava/lang/Double;",
                    &[
                        ::prebindgen_jni_runtime::jni::objects::JValue::Double(
                            ret_w0_value2,
                        ),
                    ],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            jni_environment
                .set_object_array_element(&result, 4i32, &object)
                .map_err(|e| e.to_string())?;
            jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Boolean",
                    "valueOf",
                    "(Z)Ljava/lang/Boolean;",
                    &[
                        ::prebindgen_jni_runtime::jni::objects::JValue::Bool(
                            ret_w0_value3,
                        ),
                    ],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            jni_environment
                .set_object_array_element(&result, 5i32, &object)
                .map_err(|e| e.to_string())?;
            jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(result.into_raw())
        }
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_call_1payload_1new_14db541865be00283<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
    p1_w1: i32,
    p2_w1: f64,
    p3_w1: u8,
) -> ::prebindgen_jni_runtime::jni::sys::jobjectArray {
    #[allow(unused_mut)]
    let mut domain_output = None;
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            let p2_w0 = if true { Some(p2_w1) } else { None };
            let p3_w0 = if true { Some(p3_w1 != 0) } else { None };
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {};
            let _: () = {};
            let _: () = {};
            let _: () = {};
            let p0_w0: i64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i32 = ::core::convert::Into::into(p1_w0.expect("active value"));
            let __prebindgen_argument_1 = p1_w0;
            let p2_w0: f64 = ::core::convert::Into::into(p2_w0.expect("active value"));
            let __prebindgen_argument_2 = p2_w0;
            let p3_w0: bool = ::core::convert::Into::into(p3_w0.expect("active value"));
            let __prebindgen_argument_3 = p3_w0;
            let source_value = perftest_flat::payload_new(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
                __prebindgen_argument_2,
                __prebindgen_argument_3,
            );
            let ret_w0 = perftest_flat::payload_to_struct(&source_value);
            let ret_w1 = &(ret_w0);
            let ret_w2: &i64 = &ret_w1.id;
            let ret_w2 = *ret_w2;
            let ret_w3: &i32 = &ret_w1.seq;
            let ret_w3 = *ret_w3;
            let ret_w4: &f64 = &ret_w1.value;
            let ret_w4 = *ret_w4;
            let ret_w5: &bool = &ret_w1.flag;
            let ret_w5 = u8::from(*ret_w5);
            let result = ::prebindgen_jni_runtime::array(&mut jni_environment, 5usize)?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Long",
                    "valueOf",
                    "(J)Ljava/lang/Long;",
                    &[::prebindgen_jni_runtime::jni::objects::JValue::Long(ret_w2)],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            jni_environment
                .set_object_array_element(&result, 1i32, &object)
                .map_err(|e| e.to_string())?;
            jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Integer",
                    "valueOf",
                    "(I)Ljava/lang/Integer;",
                    &[::prebindgen_jni_runtime::jni::objects::JValue::Int(ret_w3)],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            jni_environment
                .set_object_array_element(&result, 2i32, &object)
                .map_err(|e| e.to_string())?;
            jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Double",
                    "valueOf",
                    "(D)Ljava/lang/Double;",
                    &[::prebindgen_jni_runtime::jni::objects::JValue::Double(ret_w4)],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            jni_environment
                .set_object_array_element(&result, 3i32, &object)
                .map_err(|e| e.to_string())?;
            jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Boolean",
                    "valueOf",
                    "(Z)Ljava/lang/Boolean;",
                    &[::prebindgen_jni_runtime::jni::objects::JValue::Bool(ret_w5)],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            jni_environment
                .set_object_array_element(&result, 4i32, &object)
                .map_err(|e| e.to_string())?;
            jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
            let _: () = {};
            Ok(result.into_raw())
        }
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_call_1storage_1with_1payload_1b44f150bf309e573<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w2: i64,
    p0_w4: i32,
    p0_w6: f64,
    p0_w8: u8,
) -> ::prebindgen_jni_runtime::jni::sys::jobjectArray {
    #[allow(unused_mut)]
    let mut domain_output = None;
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1 = if true { Some(p0_w2) } else { None };
            let p0_w3 = if true { Some(p0_w4) } else { None };
            let p0_w5 = if true { Some(p0_w6) } else { None };
            let p0_w7 = if true { Some(p0_w8 != 0) } else { None };
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {};
            let p0_w1: i64 = ::core::convert::Into::into(p0_w1.expect("active value"));
            let p0_w3: i32 = ::core::convert::Into::into(p0_w3.expect("active value"));
            let p0_w5: f64 = ::core::convert::Into::into(p0_w5.expect("active value"));
            let p0_w7: bool = ::core::convert::Into::into(p0_w7.expect("active value"));
            let p0_w0: perftest_flat::Payload = perftest_flat::payload_new(
                p0_w1,
                p0_w3,
                p0_w5,
                p0_w7,
            );
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::storage_with_payload(
                __prebindgen_argument_0,
            );
            let mut ret_w0 = Box::new(::prebindgen_jni_runtime::Allocation {
                value: source_value,
            });
            let result = ::prebindgen_jni_runtime::array(&mut jni_environment, 2usize)?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Long",
                    "valueOf",
                    "(J)Ljava/lang/Long;",
                    &[
                        ::prebindgen_jni_runtime::jni::objects::JValue::Long(
                            ((&mut *ret_w0)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    perftest_flat::Storage,
                                >) as i64,
                        ),
                    ],
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            jni_environment
                .set_object_array_element(&result, 1i32, &object)
                .map_err(|e| e.to_string())?;
            jni_environment.delete_local_ref(object).map_err(|e| e.to_string())?;
            let _: () = {
                let _ = Box::into_raw(ret_w0);
            };
            Ok(result.into_raw())
        }
    }) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
Only generated Kotlin wrappers may call this native entry point, with their handle locks held and live pointers of the declared types.*/
#[no_mangle]
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_call_1storage_1put_1opt_138084ee844ee97a7<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
    p1_w1: i32,
    p1_w4: i64,
    p1_w6: i32,
    p1_w8: f64,
    p1_w10: u8,
) -> ::prebindgen_jni_runtime::jni::sys::jobjectArray {
    #[allow(unused_mut)]
    let mut domain_output = None;
    match ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1
                    as *mut ::prebindgen_jni_runtime::Allocation<perftest_flat::Storage>
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            if p1_w1 != 0 && p1_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let p1_w3 = if p1_w1 == 1i32 { Some(p1_w4) } else { None };
            let p1_w5 = if p1_w1 == 1i32 { Some(p1_w6) } else { None };
            let p1_w7 = if p1_w1 == 1i32 { Some(p1_w8) } else { None };
            let p1_w9 = if p1_w1 == 1i32 { Some(p1_w10 != 0) } else { None };
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {
                if true {
                    for identity in ::core::iter::once((p0_w1_pointer.addr()) as usize) {
                        __prebindgen_resources.push((identity, true));
                    }
                }
                __prebindgen_resources.sort_unstable_by_key(|entry| entry.0);
                if __prebindgen_resources
                    .windows(2)
                    .any(|pair| pair[0].0 == pair[1].0 && (pair[0].1 || pair[1].1))
                {
                    return Err("conflicting input aliases".into());
                }
            };
            let _: () = {};
            let p0_w0: &mut perftest_flat::Storage = unsafe {
                &mut (*p0_w1_pointer).value
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0 = if p1_w1 == 1i32 {
                let p1_w3: i64 = ::core::convert::Into::into(
                    p1_w3.expect("active value"),
                );
                let p1_w5: i32 = ::core::convert::Into::into(
                    p1_w5.expect("active value"),
                );
                let p1_w7: f64 = ::core::convert::Into::into(
                    p1_w7.expect("active value"),
                );
                let p1_w9: bool = ::core::convert::Into::into(
                    p1_w9.expect("active value"),
                );
                let p1_w2: perftest_flat::Payload = perftest_flat::payload_new(
                    p1_w3,
                    p1_w5,
                    p1_w7,
                    p1_w9,
                );
                Some(p1_w2)
            } else {
                None
            };
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::storage_put_opt(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = u8::from(source_value);
            let result = ::prebindgen_jni_runtime::array(&mut jni_environment, 2usize)?;
            let object = jni_environment
                .call_static_method(
                    "java/lang/Boolean",
                    "valueOf",
                    "(Z)Ljava/lang/Boolean;",
                    &[::prebindgen_jni_runtime::jni::objects::JValue::Bool(ret_w0)],
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
    }) {
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
const _: () = {
    konst::assertc_eq!(
        perftest_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::Payload > () != 0,
    "flat object `Payload` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::Storage > () != 0,
    "flat object `Storage` must have nonzero size"
);
