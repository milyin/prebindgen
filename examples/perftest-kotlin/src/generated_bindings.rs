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
            "io/prebindgen/perftest/JniErrorHandler",
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
const _: () = {
    konst::assertc_eq!(
        perftest_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_perftest_PayloadHandler_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::PayloadHandler>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::PayloadHandler > () >= 2,
    "`PayloadHandler`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
);
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_perftest_PayloadVecHandler_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::PayloadVecHandler>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::PayloadVecHandler > () >= 2,
    "`PayloadVecHandler`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
);
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_perftest_Storage_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::Storage>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::Storage > () >= 2,
    "`Storage`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_tokenValue<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    t: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let t = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Token>(t)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::token_value(t);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jlong);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0
        }
    }
}
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_tokenNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let value = value;
    let __result = perftest_flat::token_new(value);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| {
        let r = ::prebindgen_jni_runtime::new_handle(__result);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0
        }
    }
}
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_perftest_Token_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::Token>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::Token > () >= 2,
    "`Token`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_tokenGcValue<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    t: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let t = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::TokenGc>(t)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::token_gc_value(t);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jlong);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0
        }
    }
}
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_tokenGcNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let value = value;
    let __result = perftest_flat::token_gc_new(value);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| {
        let r = ::prebindgen_jni_runtime::new_handle(__result);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0
        }
    }
}
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_perftest_TokenGc_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::TokenGc>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::TokenGc > () >= 2,
    "`TokenGc`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_largeFlatInputSum<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value_left_left_left_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let value = perftest_flat::ObjectBoundary64 {
        left: perftest_flat::ObjectBoundary32 {
            left: perftest_flat::ObjectBoundary16 {
                left: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_right_right_right_value,
                            },
                        },
                    },
                },
                right: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_right_right_right_value,
                            },
                        },
                    },
                },
            },
            right: perftest_flat::ObjectBoundary16 {
                left: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_right_right_right_value,
                            },
                        },
                    },
                },
                right: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_right_right_right_value,
                            },
                        },
                    },
                },
            },
        },
        right: perftest_flat::ObjectBoundary32 {
            left: perftest_flat::ObjectBoundary16 {
                left: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_right_right_right_value,
                            },
                        },
                    },
                },
                right: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_right_right_right_value,
                            },
                        },
                    },
                },
            },
            right: perftest_flat::ObjectBoundary16 {
                left: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_right_right_right_value,
                            },
                        },
                    },
                },
                right: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_right_right_right_value,
                            },
                        },
                    },
                },
            },
        },
    };
    let __result = perftest_flat::large_flat_input_sum(&value);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jlong);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0
        }
    }
}
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_largeObjectInputSum<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value_left_left_left_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_left_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_left_right_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_left_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_left_right_right_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_left_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_left_right_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_left_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_left_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_left_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_left_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_left_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_right_left_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_right_left_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_right_right_left_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_right_right_right_right_right_right_value: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let value = perftest_flat::ObjectBoundary64Object {
        left: perftest_flat::ObjectBoundary32 {
            left: perftest_flat::ObjectBoundary16 {
                left: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_left_right_right_right_value,
                            },
                        },
                    },
                },
                right: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_left_right_right_right_right_value,
                            },
                        },
                    },
                },
            },
            right: perftest_flat::ObjectBoundary16 {
                left: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_left_right_right_right_value,
                            },
                        },
                    },
                },
                right: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_left_right_right_right_right_right_value,
                            },
                        },
                    },
                },
            },
        },
        right: perftest_flat::ObjectBoundary32 {
            left: perftest_flat::ObjectBoundary16 {
                left: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_left_right_right_right_value,
                            },
                        },
                    },
                },
                right: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_left_right_right_right_right_value,
                            },
                        },
                    },
                },
            },
            right: perftest_flat::ObjectBoundary16 {
                left: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_left_right_right_right_value,
                            },
                        },
                    },
                },
                right: perftest_flat::ObjectBoundary8 {
                    left: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_left_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_left_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_left_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_left_right_right_value,
                            },
                        },
                    },
                    right: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_right_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_right_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_right_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_right_right_right_right_right_value,
                            },
                        },
                    },
                },
            },
        },
    };
    let __result = perftest_flat::large_object_input_sum(&value);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jlong);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0
        }
    }
}
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_storageNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let __result = perftest_flat::storage_new();
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| {
        let r = ::prebindgen_jni_runtime::new_handle(__result);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0
        }
    }
}
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_storageGet<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Storage>(s)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::storage_get(s);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r__present, r_id, r_seq, r_value, r_flag, r_label) = match __result {
            ::core::option::Option::Some(__x1) => {
                let (r_id, r_seq, r_value, r_flag, r_label) = {
                    let perftest_flat::Payload {
                        id: __f0,
                        seq: __f1,
                        value: __f2,
                        flag: __f3,
                        label: __f4,
                    } = __x1;
                    let r_id = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                    let r_seq = (__f1 as ::prebindgen_jni_runtime::jni::sys::jint);
                    let r_value = (__f2 as ::prebindgen_jni_runtime::jni::sys::jdouble);
                    let r_flag = (__f3 as u8);
                    let r_label = match __f4 {
                        ::core::option::Option::Some(__x3) => {
                            ::prebindgen_jni_runtime::new_string(
                                env,
                                ::core::convert::AsRef::<str>::as_ref(&(*__x3)),
                            )?
                        }
                        ::core::option::Option::None => {
                            ::prebindgen_jni_runtime::jni::objects::JObject::null()
                        }
                    };
                    (r_id, r_seq, r_value, r_flag, r_label)
                };
                (1u8, r_id, r_seq, r_value, r_flag, r_label)
            }
            ::core::option::Option::None => {
                (
                    0u8,
                    0,
                    0,
                    0.0f64,
                    0,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                )
            }
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/perftest/__Sink_Option_Payload",
                "run",
                "(ZJIDZLjava/lang/String;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        z: r__present,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_id,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_seq,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        d: r_value,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        z: r_flag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_label.as_raw(),
                    },
                ],
            )?;
        ::core::result::Result::Ok(__o.into_raw())
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            ::core::ptr::null_mut()
        }
    }
}
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_storagePutByTake<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    payload_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    payload_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    payload_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    payload_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    payload_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle_mut::<perftest_flat::Storage>(s)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let payload = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::Payload {
            id: payload_id,
            seq: payload_seq,
            value: payload_value,
            flag: (payload_flag != 0),
            label: if !payload_label.is_null() {
                ::core::option::Option::Some(
                    ::std::boxed::Box::new(
                        ::prebindgen_jni_runtime::read_string(env, &payload_label)?,
                    ),
                )
            } else {
                ::core::option::Option::None
            },
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let __result = perftest_flat::storage_put_by_take(s, payload);
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_storagePutByRead<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    payload_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    payload_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    payload_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    payload_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    payload_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle_mut::<perftest_flat::Storage>(s)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let payload = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::Payload {
            id: payload_id,
            seq: payload_seq,
            value: payload_value,
            flag: (payload_flag != 0),
            label: if !payload_label.is_null() {
                ::core::option::Option::Some(
                    ::std::boxed::Box::new(
                        ::prebindgen_jni_runtime::read_string(env, &payload_label)?,
                    ),
                )
            } else {
                ::core::option::Option::None
            },
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let __result = perftest_flat::storage_put_by_read(s, &payload);
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_payloadHandlerNew<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    f: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let f = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __up = ::prebindgen_jni_runtime::Upcall::new(
                env,
                &f,
                "run",
                "(JIDZLjava/lang/String;)V",
                42i32,
            )?;
            move |__a0: &perftest_flat::Payload| {
                let __r = __up
                    .call_void(|env| {
                        let (a0_id, a0_seq, a0_value, a0_flag, a0_label) = {
                            let perftest_flat::Payload {
                                id: __f0,
                                seq: __f1,
                                value: __f2,
                                flag: __f3,
                                label: __f4,
                            } = ::core::clone::Clone::clone(__a0);
                            let a0_id = (__f0
                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                            let a0_seq = (__f1
                                as ::prebindgen_jni_runtime::jni::sys::jint);
                            let a0_value = (__f2
                                as ::prebindgen_jni_runtime::jni::sys::jdouble);
                            let a0_flag = (__f3 as u8);
                            let a0_label = match __f4 {
                                ::core::option::Option::Some(__x2) => {
                                    ::prebindgen_jni_runtime::new_string(
                                        env,
                                        ::core::convert::AsRef::<str>::as_ref(&(*__x2)),
                                    )?
                                }
                                ::core::option::Option::None => {
                                    ::prebindgen_jni_runtime::jni::objects::JObject::null()
                                }
                            };
                            (a0_id, a0_seq, a0_value, a0_flag, a0_label)
                        };
                        ::core::result::Result::Ok(
                            ::std::vec![
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j : a0_id },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { i : a0_seq },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { d : a0_value },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { z : a0_flag },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l : a0_label
                                .as_raw() }
                            ],
                        )
                    });
                if let ::core::result::Result::Err(__e) = __r {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (& Payload) + Send + Sync + 'static",
                        &__e,
                    );
                }
            }
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::payload_handler_new(f);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| {
        let r = ::prebindgen_jni_runtime::new_handle(__result);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0
        }
    }
}
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_storageCallback<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    handler: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Storage>(s)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let handler = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<
                perftest_flat::PayloadHandler,
            >(handler)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let __result = perftest_flat::storage_callback(s, handler);
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_storagePutSlice<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    payloads__n: ::prebindgen_jni_runtime::jni::sys::jint,
    payloads_id: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    payloads_seq: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    payloads_value: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    payloads_flag: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    payloads_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle_mut::<perftest_flat::Storage>(s)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let payloads = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __n = payloads__n as usize;
            let __c0_0 = ::prebindgen_jni_runtime::read_longs(env, &payloads_id)?;
            let __c0_1 = ::prebindgen_jni_runtime::read_ints(env, &payloads_seq)?;
            let __c0_2 = ::prebindgen_jni_runtime::read_doubles(env, &payloads_value)?;
            let __c0_3 = ::prebindgen_jni_runtime::read_booleans(env, &payloads_flag)?;
            let mut __v = ::std::vec::Vec::with_capacity(__n);
            for __i in 0..__n {
                let __e0_id = __c0_0[__i];
                let __e0_seq = __c0_1[__i];
                let __e0_value = __c0_2[__i];
                let __e0_flag = __c0_3[__i];
                let __e0_label = ::prebindgen_jni_runtime::object_array_get(
                    env,
                    &payloads_label,
                    __i,
                )?;
                let __x = (|| -> ::core::result::Result<_, ::std::string::String> {
                    ::core::result::Result::Ok(perftest_flat::Payload {
                        id: __e0_id,
                        seq: __e0_seq,
                        value: __e0_value,
                        flag: (__e0_flag != 0),
                        label: if !__e0_label.is_null() {
                            ::core::option::Option::Some(
                                ::std::boxed::Box::new(
                                    ::prebindgen_jni_runtime::read_string(env, &__e0_label)?,
                                ),
                            )
                        } else {
                            ::core::option::Option::None
                        },
                    })
                })();
                ::prebindgen_jni_runtime::drop_local(env, __e0_label);
                __v.push(__x?);
            }
            __v
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let __result = perftest_flat::storage_put_slice(s, &payloads);
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_storageGetVec<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Storage>(s)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::storage_get_vec(s);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r__present, r__n, r_id, r_seq, r_value, r_flag, r_label) = match __result {
            ::core::option::Option::Some(__x1) => {
                let (r__n, r_id, r_seq, r_value, r_flag, r_label) = {
                    let __it = ::core::iter::IntoIterator::into_iter(__x1);
                    let __items: ::std::vec::Vec<_> = __it.collect();
                    let __n = __items.len();
                    let mut __c2_0 = ::std::vec::Vec::with_capacity(__n);
                    let mut __c2_1 = ::std::vec::Vec::with_capacity(__n);
                    let mut __c2_2 = ::std::vec::Vec::with_capacity(__n);
                    let mut __c2_3 = ::std::vec::Vec::with_capacity(__n);
                    let __c2_4 = ::prebindgen_jni_runtime::new_object_array(env, __n)?;
                    for (__i, __x2) in __items.into_iter().enumerate() {
                        let (__e2_id, __e2_seq, __e2_value, __e2_flag, __e2_label) = {
                            let perftest_flat::Payload {
                                id: __f0,
                                seq: __f1,
                                value: __f2,
                                flag: __f3,
                                label: __f4,
                            } = __x2;
                            let __e2_id = (__f0
                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                            let __e2_seq = (__f1
                                as ::prebindgen_jni_runtime::jni::sys::jint);
                            let __e2_value = (__f2
                                as ::prebindgen_jni_runtime::jni::sys::jdouble);
                            let __e2_flag = (__f3 as u8);
                            let __e2_label = match __f4 {
                                ::core::option::Option::Some(__x4) => {
                                    ::prebindgen_jni_runtime::new_string(
                                        env,
                                        ::core::convert::AsRef::<str>::as_ref(&(*__x4)),
                                    )?
                                }
                                ::core::option::Option::None => {
                                    ::prebindgen_jni_runtime::jni::objects::JObject::null()
                                }
                            };
                            (__e2_id, __e2_seq, __e2_value, __e2_flag, __e2_label)
                        };
                        __c2_0.push(__e2_id);
                        __c2_1.push(__e2_seq);
                        __c2_2.push(__e2_value);
                        __c2_3.push(__e2_flag);
                        ::prebindgen_jni_runtime::object_array_set(
                            env,
                            &__c2_4,
                            __i,
                            __e2_label,
                        )?;
                    }
                    (
                        __n as i32,
                        ::prebindgen_jni_runtime::write_longs(env, &__c2_0)?,
                        ::prebindgen_jni_runtime::write_ints(env, &__c2_1)?,
                        ::prebindgen_jni_runtime::write_doubles(env, &__c2_2)?,
                        ::prebindgen_jni_runtime::write_booleans(env, &__c2_3)?,
                        __c2_4,
                    )
                };
                (1u8, r__n, r_id, r_seq, r_value, r_flag, r_label)
            }
            ::core::option::Option::None => {
                (
                    0u8,
                    0,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                )
            }
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/perftest/__Sink_Option_Vec_Payload",
                "run",
                "(ZI[J[I[D[Z[Ljava/lang/Object;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        z: r__present,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__n,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_id.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_seq.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_value.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_flag.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_label.as_raw(),
                    },
                ],
            )?;
        ::core::result::Result::Ok(__o.into_raw())
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            ::core::ptr::null_mut()
        }
    }
}
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_payloadVecHandlerNew<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    f: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let f = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __up = ::prebindgen_jni_runtime::Upcall::new(
                env,
                &f,
                "run",
                "(I[J[I[D[Z[Ljava/lang/Object;)V",
                44i32,
            )?;
            move |__a0: &[perftest_flat::Payload]| {
                let __r = __up
                    .call_void(|env| {
                        let (a0__n, a0_id, a0_seq, a0_value, a0_flag, a0_label) = {
                            let __it = __a0.iter().cloned();
                            let __items: ::std::vec::Vec<_> = __it.collect();
                            let __n = __items.len();
                            let mut __c1_0 = ::std::vec::Vec::with_capacity(__n);
                            let mut __c1_1 = ::std::vec::Vec::with_capacity(__n);
                            let mut __c1_2 = ::std::vec::Vec::with_capacity(__n);
                            let mut __c1_3 = ::std::vec::Vec::with_capacity(__n);
                            let __c1_4 = ::prebindgen_jni_runtime::new_object_array(
                                env,
                                __n,
                            )?;
                            for (__i, __x1) in __items.into_iter().enumerate() {
                                let (
                                    __e1_id,
                                    __e1_seq,
                                    __e1_value,
                                    __e1_flag,
                                    __e1_label,
                                ) = {
                                    let perftest_flat::Payload {
                                        id: __f0,
                                        seq: __f1,
                                        value: __f2,
                                        flag: __f3,
                                        label: __f4,
                                    } = __x1;
                                    let __e1_id = (__f0
                                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                                    let __e1_seq = (__f1
                                        as ::prebindgen_jni_runtime::jni::sys::jint);
                                    let __e1_value = (__f2
                                        as ::prebindgen_jni_runtime::jni::sys::jdouble);
                                    let __e1_flag = (__f3 as u8);
                                    let __e1_label = match __f4 {
                                        ::core::option::Option::Some(__x3) => {
                                            ::prebindgen_jni_runtime::new_string(
                                                env,
                                                ::core::convert::AsRef::<str>::as_ref(&(*__x3)),
                                            )?
                                        }
                                        ::core::option::Option::None => {
                                            ::prebindgen_jni_runtime::jni::objects::JObject::null()
                                        }
                                    };
                                    (__e1_id, __e1_seq, __e1_value, __e1_flag, __e1_label)
                                };
                                __c1_0.push(__e1_id);
                                __c1_1.push(__e1_seq);
                                __c1_2.push(__e1_value);
                                __c1_3.push(__e1_flag);
                                ::prebindgen_jni_runtime::object_array_set(
                                    env,
                                    &__c1_4,
                                    __i,
                                    __e1_label,
                                )?;
                            }
                            (
                                __n as i32,
                                ::prebindgen_jni_runtime::write_longs(env, &__c1_0)?,
                                ::prebindgen_jni_runtime::write_ints(env, &__c1_1)?,
                                ::prebindgen_jni_runtime::write_doubles(env, &__c1_2)?,
                                ::prebindgen_jni_runtime::write_booleans(env, &__c1_3)?,
                                __c1_4,
                            )
                        };
                        ::core::result::Result::Ok(
                            ::std::vec![
                                ::prebindgen_jni_runtime::jni::sys::jvalue { i : a0__n },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l : a0_id
                                .as_raw() }, ::prebindgen_jni_runtime::jni::sys::jvalue { l
                                : a0_seq.as_raw() },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l : a0_value
                                .as_raw() }, ::prebindgen_jni_runtime::jni::sys::jvalue { l
                                : a0_flag.as_raw() },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l : a0_label
                                .as_raw() }
                            ],
                        )
                    });
                if let ::core::result::Result::Err(__e) = __r {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (& [Payload]) + Send + Sync + 'static",
                        &__e,
                    );
                }
            }
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::payload_vec_handler_new(f);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| {
        let r = ::prebindgen_jni_runtime::new_handle(__result);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0
        }
    }
}
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
pub unsafe extern "system" fn Java_io_prebindgen_perftest_JNINative_storageCallbackVec<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    handler: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Storage>(s)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let handler = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<
                perftest_flat::PayloadVecHandler,
            >(handler)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let __result = perftest_flat::storage_callback_vec(s, handler);
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
