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
            "io/prebindgen/covertest/JniErrorHandler",
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_payloadLabelLen<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    p_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    p_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    p_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    p_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    p_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let p = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::Payload {
            id: p_id,
            seq: p_seq,
            value: p_value,
            flag: (p_flag != 0),
            label: if !p_label.is_null() {
                ::core::option::Option::Some(
                    ::std::boxed::Box::new(
                        ::prebindgen_jni_runtime::read_string(env, &p_label)?,
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
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::payload_label_len(&p);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match __result {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::box_jlong(
                    env,
                    (__x1 as ::prebindgen_jni_runtime::jni::sys::jlong),
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_covertest_model_Report_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::Report>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::Report > () >= 2,
    "`Report`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
);
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_covertest_model_Probe_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::Probe>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::Probe > () >= 2,
    "`Probe`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
);
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_covertest_model_Span_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::Span>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::Span > () >= 2,
    "`Span`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
);
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_covertest_model_SpanHolder_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::SpanHolder>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::SpanHolder > () >= 2,
    "`SpanHolder`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_stampSecs<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s_secs: ::prebindgen_jni_runtime::jni::sys::jlong,
    s_nanos: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let s = perftest_flat::Stamp {
        secs: s_secs,
        nanos: s_nanos,
    };
    let __result = perftest_flat::stamp_secs(&s);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_stampNanos<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s_secs: ::prebindgen_jni_runtime::jni::sys::jlong,
    s_nanos: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let s = perftest_flat::Stamp {
        secs: s_secs,
        nanos: s_nanos,
    };
    let __result = perftest_flat::stamp_nanos(&s);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageErrorMessage<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    e: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let e = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::StorageError>(e)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::storage_error_message(e);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = ::prebindgen_jni_runtime::new_string(
            env,
            ::core::convert::AsRef::<str>::as_ref(&__result),
        )?;
        ::core::result::Result::Ok(r.into_raw())
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
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_covertest_errors_StorageError_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::StorageError>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::StorageError > () >= 2,
    "`StorageError`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summaryCount<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Summary>(s)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::summary_count(s);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summaryTotal<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jdouble {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Summary>(s)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0.0f64;
        }
    };
    let __result = perftest_flat::summary_total(s);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jdouble,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jdouble);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0.0f64
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summaryScaled<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    factor: ::prebindgen_jni_runtime::jni::sys::jdouble,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jdouble {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Summary>(s)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0.0f64;
        }
    };
    let factor = factor;
    let __result = perftest_flat::summary_scaled(s, factor);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jdouble,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jdouble);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0.0f64
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summaryMean<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jdouble {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Summary>(s)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0.0f64;
        }
    };
    let __result = crate::summary_mean(s);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jdouble,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jdouble);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0.0f64
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summaryNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    count: ::prebindgen_jni_runtime::jni::sys::jlong,
    total: ::prebindgen_jni_runtime::jni::sys::jdouble,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let count = count;
    let total = total;
    let __result = perftest_flat::summary_new(count, total);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summaryFromMean<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    count: ::prebindgen_jni_runtime::jni::sys::jlong,
    mean: ::prebindgen_jni_runtime::jni::sys::jdouble,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let count = count;
    let mean = mean;
    let __result = crate::summary_from_mean(count, mean);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| match __result {
        ::core::result::Result::Ok(__ok) => {
            let r = ::prebindgen_jni_runtime::new_handle(__ok);
            ::core::result::Result::Ok(r)
        }
        ::core::result::Result::Err(__e) => {
            ::core::result::Result::Err(::std::string::ToString::to_string(&__e))
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_analytics_Summary_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::Summary>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::Summary > () >= 2,
    "`Summary`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
);
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_covertest_analytics_SummaryVault_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::Archive>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::Archive > () >= 2,
    "`Archive`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageLen<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
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
            return 0;
        }
    };
    let __result = perftest_flat::storage_len(s);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageContains<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    id: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jboolean {
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
            return 0;
        }
    };
    let id = id;
    let __result = perftest_flat::storage_contains(s, id);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jboolean,
        ::std::string::String,
    > = (|| {
        let r = (__result as u8);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageWithPayload<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    payload_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    payload_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    payload_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    payload_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    payload_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
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
            return 0;
        }
    };
    let __result = perftest_flat::storage_with_payload(payload);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_Storage_freePtr(
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
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_covertest_PayloadHandler_freePtr(
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_StorageHandler_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::StorageHandler>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::StorageHandler > () >= 2,
    "`StorageHandler`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
);
#[no_mangle]
#[allow(non_snake_case)]
pub unsafe extern "system" fn Java_io_prebindgen_covertest_PayloadVecHandler_freePtr(
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_constGetCoverMagic<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let __result = perftest_flat::COVER_MAGIC;
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_constGetCoverTag<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let __result = perftest_flat::COVER_TAG;
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = ::prebindgen_jni_runtime::new_string(
            env,
            ::core::convert::AsRef::<str>::as_ref(&__result),
        )?;
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_coverTagRuntime<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let __result = perftest_flat::cover_tag_runtime();
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = ::prebindgen_jni_runtime::new_string(
            env,
            ::core::convert::AsRef::<str>::as_ref(&__result),
        )?;
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_constGetCoverVersion<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let __result = crate::cover_version();
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = ::prebindgen_jni_runtime::new_string(
            env,
            ::core::convert::AsRef::<str>::as_ref(&__result),
        )?;
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_constGetCoverBanner<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let __result = {
        #[allow(unused_imports)]
        use cov_helpers::*;
        #[allow(unused_imports)]
        use perftest_flat::*;
        format!("{COVER_TAG}:{COVER_MAGIC:#x}")
    };
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = ::prebindgen_jni_runtime::new_string(
            env,
            ::core::convert::AsRef::<str>::as_ref(&__result),
        )?;
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_escape_1probe_1value<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    p: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let p = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::EscapeProbe>(p)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::escape_probe_value(p);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_escapeProbeNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let value = value;
    let __result = perftest_flat::escape_probe_new(value);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_esc_1pkg_Esc_1Probe_freePtr(
    _env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
    ptr: ::prebindgen_jni_runtime::jni::sys::jlong,
) {
    ::prebindgen_jni_runtime::free_handle::<perftest_flat::EscapeProbe>(ptr)
}
const _: () = assert!(
    ::core::mem::align_of:: < perftest_flat::EscapeProbe > () >= 2,
    "`EscapeProbe`: a handle type must have alignment >= 2 (bit 0 is the closed tag)"
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_payloadPriority<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    p_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    p_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    p_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    p_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    p_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jint {
    let mut __env = __env;
    let env = &mut __env;
    let p = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::Payload {
            id: p_id,
            seq: p_seq,
            value: p_value,
            flag: (p_flag != 0),
            label: if !p_label.is_null() {
                ::core::option::Option::Some(
                    ::std::boxed::Box::new(
                        ::prebindgen_jni_runtime::read_string(env, &p_label)?,
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
            return 0;
        }
    };
    let __result = perftest_flat::payload_priority(&p);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jint,
        ::std::string::String,
    > = (|| {
        let r = (match __result {
            perftest_flat::Priority::Low => 0,
            perftest_flat::Priority::Normal => 1,
            perftest_flat::Priority::High => 2,
        } as i32);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_priorityWeight<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    p: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jint {
    let mut __env = __env;
    let env = &mut __env;
    let p = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            (match p {
                0 => ::core::result::Result::Ok(perftest_flat::Priority::Low),
                1 => ::core::result::Result::Ok(perftest_flat::Priority::Normal),
                2 => ::core::result::Result::Ok(perftest_flat::Priority::High),
                __v => {
                    ::core::result::Result::Err(
                        ::std::format!("invalid value {} for enum `Priority`", __v),
                    )
                }
            })?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::priority_weight(p);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jint,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jint);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_priorityOr<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    p__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    p: ::prebindgen_jni_runtime::jni::sys::jint,
    fallback: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jint {
    let mut __env = __env;
    let env = &mut __env;
    let p = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            if (p__present != 0) {
                ::core::option::Option::Some(
                    (match p {
                        0 => ::core::result::Result::Ok(perftest_flat::Priority::Low),
                        1 => ::core::result::Result::Ok(perftest_flat::Priority::Normal),
                        2 => ::core::result::Result::Ok(perftest_flat::Priority::High),
                        __v => {
                            ::core::result::Result::Err(
                                ::std::format!("invalid value {} for enum `Priority`", __v),
                            )
                        }
                    })?,
                )
            } else {
                ::core::option::Option::None
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let fallback = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            (match fallback {
                0 => ::core::result::Result::Ok(perftest_flat::Priority::Low),
                1 => ::core::result::Result::Ok(perftest_flat::Priority::Normal),
                2 => ::core::result::Result::Ok(perftest_flat::Priority::High),
                __v => {
                    ::core::result::Result::Err(
                        ::std::format!("invalid value {} for enum `Priority`", __v),
                    )
                }
            })?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::priority_or(p, fallback);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jint,
        ::std::string::String,
    > = (|| {
        let r = (match __result {
            perftest_flat::Priority::Low => 0,
            perftest_flat::Priority::Normal => 1,
            perftest_flat::Priority::High => 2,
        } as i32);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_stampNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    secs: ::prebindgen_jni_runtime::jni::sys::jlong,
    nanos: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let secs = secs;
    let nanos = nanos;
    let __result = perftest_flat::stamp_new(secs, nanos);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r_secs, r_nanos) = {
            let perftest_flat::Stamp { secs: __f0, nanos: __f1 } = __result;
            let r_secs = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
            let r_nanos = (__f1 as ::prebindgen_jni_runtime::jni::sys::jlong);
            (r_secs, r_nanos)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Stamp",
                "run",
                "(JJ)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_secs,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_nanos,
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_stampSeries<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    count: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let count = count;
    let __result = perftest_flat::stamp_series(count);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r__n, r_secs, r_nanos) = {
            let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(
                    __result,
                )
                .collect();
            let __n = __items.len();
            let mut __c1_0 = ::std::vec::Vec::with_capacity(__n);
            let mut __c1_1 = ::std::vec::Vec::with_capacity(__n);
            for (__i, __x1) in __items.into_iter().enumerate() {
                let (__e1_secs, __e1_nanos) = {
                    let perftest_flat::Stamp { secs: __f0, nanos: __f1 } = __x1;
                    let __e1_secs = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                    let __e1_nanos = (__f1 as ::prebindgen_jni_runtime::jni::sys::jlong);
                    (__e1_secs, __e1_nanos)
                };
                __c1_0.push(__e1_secs);
                __c1_1.push(__e1_nanos);
            }
            (
                __n as i32,
                ::prebindgen_jni_runtime::write_longs(env, &__c1_0)?,
                ::prebindgen_jni_runtime::write_longs(env, &__c1_1)?,
            )
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Vec_Stamp",
                "run",
                "(I[J[J)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__n,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_secs.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_nanos.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_celsiusDouble<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    c: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jint {
    let mut __env = __env;
    let env = &mut __env;
    let c = {
        let __r0 = c;
        <perftest_flat::Celsius as ::core::convert::From<i32>>::from(__r0)
    };
    let __result = perftest_flat::celsius_double(c);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jint,
        ::std::string::String,
    > = (|| {
        let r = {
            let __r1 = <perftest_flat::Celsius as ::core::convert::Into<
                i32,
            >>::into(__result);
            (__r1 as ::prebindgen_jni_runtime::jni::sys::jint)
        };
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_percentScale<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    p: ::prebindgen_jni_runtime::jni::sys::jint,
    factor: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jint {
    let mut __env = __env;
    let env = &mut __env;
    let p = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __r0 = p;
            <perftest_flat::Percent as ::core::convert::TryFrom<i32>>::try_from(__r0)
                .map_err(|__e| ::std::string::ToString::to_string(&__e))?
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let factor = factor;
    let __result = perftest_flat::percent_scale(p, factor);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jint,
        ::std::string::String,
    > = (|| {
        let r = {
            let __r1 = crate::percent_out(__result)
                .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
            (__r1 as ::prebindgen_jni_runtime::jni::sys::jint)
        };
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_percentOptional<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    p__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    p: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let p = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            if (p__present != 0) {
                ::core::option::Option::Some({
                    let __r0 = p;
                    <perftest_flat::Percent as ::core::convert::TryFrom<
                        i32,
                    >>::try_from(__r0)
                        .map_err(|__e| ::std::string::ToString::to_string(&__e))?
                })
            } else {
                ::core::option::Option::None
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::percent_optional(p);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match __result {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::box_jint(
                    env,
                    {
                        let __r2 = crate::percent_out(__x1)
                            .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
                        (__r2 as ::prebindgen_jni_runtime::jni::sys::jint)
                    },
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_percentInvalidOutput<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let __result = perftest_flat::percent_invalid_output();
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match __result {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::box_jint(
                    env,
                    {
                        let __r2 = crate::percent_out(__x1)
                            .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
                        (__r2 as ::prebindgen_jni_runtime::jni::sys::jint)
                    },
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_labelReverse<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    l: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let l = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __r0 = ::prebindgen_jni_runtime::read_string(env, &l)?;
            crate::label_in(__r0)
                .map_err(|__e| ::std::string::ToString::to_string(&__e))?
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::label_reverse(l);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = {
            let __r1 = crate::label_out(__result);
            ::prebindgen_jni_runtime::new_string(
                env,
                ::core::convert::AsRef::<str>::as_ref(&__r1),
            )?
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_labelSeriesEcho<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    labels__n: ::prebindgen_jni_runtime::jni::sys::jint,
    labels: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let labels = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __n = labels__n as usize;
            let mut __v = ::std::vec::Vec::with_capacity(__n);
            for __i in 0..__n {
                let __e0 = ::prebindgen_jni_runtime::object_array_get(
                    env,
                    &labels,
                    __i,
                )?;
                let __x = (|| -> ::core::result::Result<_, ::std::string::String> {
                    ::core::result::Result::Ok({
                        let __r1 = ::prebindgen_jni_runtime::read_string(env, &__e0)?;
                        crate::label_in(__r1)
                            .map_err(|__e| ::std::string::ToString::to_string(&__e))?
                    })
                })();
                ::prebindgen_jni_runtime::drop_local(env, __e0);
                __v.push(__x?);
            }
            __v
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::label_series_echo(labels);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r__n, r) = {
            let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(
                    __result,
                )
                .collect();
            let __n = __items.len();
            let __c1_0 = ::prebindgen_jni_runtime::new_object_array(env, __n)?;
            for (__i, __x1) in __items.into_iter().enumerate() {
                let __e1 = {
                    let __r2 = crate::label_out(__x1);
                    ::prebindgen_jni_runtime::new_string(
                        env,
                        ::core::convert::AsRef::<str>::as_ref(&__r2),
                    )?
                };
                ::prebindgen_jni_runtime::object_array_set(env, &__c1_0, __i, __e1)?;
            }
            (__n as i32, __c1_0)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Vec_Label",
                "run",
                "(I[Ljava/lang/Object;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__n,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_annotatedNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    payload_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    payload_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    payload_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    payload_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    payload_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ttl__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    ttl: ::prebindgen_jni_runtime::jni::sys::jlong,
    priority__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    priority: ::prebindgen_jni_runtime::jni::sys::jint,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
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
            return ::core::ptr::null_mut();
        }
    };
    let ttl = if (ttl__present != 0) {
        ::core::option::Option::Some(ttl)
    } else {
        ::core::option::Option::None
    };
    let priority = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            if (priority__present != 0) {
                ::core::option::Option::Some(
                    (match priority {
                        0 => ::core::result::Result::Ok(perftest_flat::Priority::Low),
                        1 => ::core::result::Result::Ok(perftest_flat::Priority::Normal),
                        2 => ::core::result::Result::Ok(perftest_flat::Priority::High),
                        __v => {
                            ::core::result::Result::Err(
                                ::std::format!("invalid value {} for enum `Priority`", __v),
                            )
                        }
                    })?,
                )
            } else {
                ::core::option::Option::None
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::annotated_new(payload, ttl, priority);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (
            r_payload_id,
            r_payload_seq,
            r_payload_value,
            r_payload_flag,
            r_payload_label,
            r_alternate__present,
            r_alternate_id,
            r_alternate_seq,
            r_alternate_value,
            r_alternate_flag,
            r_alternate_label,
            r_ttl,
            r_priority,
        ) = {
            let perftest_flat::Annotated {
                payload: __f0,
                alternate: __f1,
                ttl: __f2,
                priority: __f3,
            } = __result;
            let (
                r_payload_id,
                r_payload_seq,
                r_payload_value,
                r_payload_flag,
                r_payload_label,
            ) = {
                let perftest_flat::Payload {
                    id: __f0,
                    seq: __f1,
                    value: __f2,
                    flag: __f3,
                    label: __f4,
                } = __f0;
                let r_payload_id = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                let r_payload_seq = (__f1 as ::prebindgen_jni_runtime::jni::sys::jint);
                let r_payload_value = (__f2
                    as ::prebindgen_jni_runtime::jni::sys::jdouble);
                let r_payload_flag = (__f3 as u8);
                let r_payload_label = match __f4 {
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
                (
                    r_payload_id,
                    r_payload_seq,
                    r_payload_value,
                    r_payload_flag,
                    r_payload_label,
                )
            };
            let (
                r_alternate__present,
                r_alternate_id,
                r_alternate_seq,
                r_alternate_value,
                r_alternate_flag,
                r_alternate_label,
            ) = match __f1 {
                ::core::option::Option::Some(__x2) => {
                    let (
                        r_alternate_id,
                        r_alternate_seq,
                        r_alternate_value,
                        r_alternate_flag,
                        r_alternate_label,
                    ) = {
                        let perftest_flat::Payload {
                            id: __f0,
                            seq: __f1,
                            value: __f2,
                            flag: __f3,
                            label: __f4,
                        } = __x2;
                        let r_alternate_id = (__f0
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        let r_alternate_seq = (__f1
                            as ::prebindgen_jni_runtime::jni::sys::jint);
                        let r_alternate_value = (__f2
                            as ::prebindgen_jni_runtime::jni::sys::jdouble);
                        let r_alternate_flag = (__f3 as u8);
                        let r_alternate_label = match __f4 {
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
                        (
                            r_alternate_id,
                            r_alternate_seq,
                            r_alternate_value,
                            r_alternate_flag,
                            r_alternate_label,
                        )
                    };
                    (
                        1u8,
                        r_alternate_id,
                        r_alternate_seq,
                        r_alternate_value,
                        r_alternate_flag,
                        r_alternate_label,
                    )
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
            let r_ttl = match __f2 {
                ::core::option::Option::Some(__x2) => {
                    ::prebindgen_jni_runtime::box_jlong(
                        env,
                        (__x2 as ::prebindgen_jni_runtime::jni::sys::jlong),
                    )?
                }
                ::core::option::Option::None => {
                    ::prebindgen_jni_runtime::jni::objects::JObject::null()
                }
            };
            let r_priority = match __f3 {
                ::core::option::Option::Some(__x2) => {
                    ::prebindgen_jni_runtime::box_jint(
                        env,
                        (match __x2 {
                            perftest_flat::Priority::Low => 0,
                            perftest_flat::Priority::Normal => 1,
                            perftest_flat::Priority::High => 2,
                        } as i32),
                    )?
                }
                ::core::option::Option::None => {
                    ::prebindgen_jni_runtime::jni::objects::JObject::null()
                }
            };
            (
                r_payload_id,
                r_payload_seq,
                r_payload_value,
                r_payload_flag,
                r_payload_label,
                r_alternate__present,
                r_alternate_id,
                r_alternate_seq,
                r_alternate_value,
                r_alternate_flag,
                r_alternate_label,
                r_ttl,
                r_priority,
            )
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Annotated",
                "run",
                "(JIDZLjava/lang/String;ZJIDZLjava/lang/String;Ljava/lang/Long;Ljava/lang/Integer;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_payload_id,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_payload_seq,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        d: r_payload_value,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        z: r_payload_flag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_payload_label.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        z: r_alternate__present,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_alternate_id,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_alternate_seq,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        d: r_alternate_value,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        z: r_alternate_flag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_alternate_label.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_ttl.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_priority.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_annotatedAlternateValue<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a_payload_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    a_payload_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    a_payload_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    a_payload_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_payload_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_alternate__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_alternate_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    a_alternate_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    a_alternate_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    a_alternate_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_alternate_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_ttl__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_ttl: ::prebindgen_jni_runtime::jni::sys::jlong,
    a_priority__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_priority: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::Annotated {
            payload: perftest_flat::Payload {
                id: a_payload_id,
                seq: a_payload_seq,
                value: a_payload_value,
                flag: (a_payload_flag != 0),
                label: if !a_payload_label.is_null() {
                    ::core::option::Option::Some(
                        ::std::boxed::Box::new(
                            ::prebindgen_jni_runtime::read_string(env, &a_payload_label)?,
                        ),
                    )
                } else {
                    ::core::option::Option::None
                },
            },
            alternate: if (a_alternate__present != 0) {
                ::core::option::Option::Some(perftest_flat::Payload {
                    id: a_alternate_id,
                    seq: a_alternate_seq,
                    value: a_alternate_value,
                    flag: (a_alternate_flag != 0),
                    label: if !a_alternate_label.is_null() {
                        ::core::option::Option::Some(
                            ::std::boxed::Box::new(
                                ::prebindgen_jni_runtime::read_string(
                                    env,
                                    &a_alternate_label,
                                )?,
                            ),
                        )
                    } else {
                        ::core::option::Option::None
                    },
                })
            } else {
                ::core::option::Option::None
            },
            ttl: if (a_ttl__present != 0) {
                ::core::option::Option::Some(a_ttl)
            } else {
                ::core::option::Option::None
            },
            priority: if (a_priority__present != 0) {
                ::core::option::Option::Some(
                    (match a_priority {
                        0 => ::core::result::Result::Ok(perftest_flat::Priority::Low),
                        1 => ::core::result::Result::Ok(perftest_flat::Priority::Normal),
                        2 => ::core::result::Result::Ok(perftest_flat::Priority::High),
                        __v => {
                            ::core::result::Result::Err(
                                ::std::format!("invalid value {} for enum `Priority`", __v),
                            )
                        }
                    })?,
                )
            } else {
                ::core::option::Option::None
            },
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::annotated_alternate_value(&a);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match __result {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::box_jdouble(
                    env,
                    (__x1 as ::prebindgen_jni_runtime::jni::sys::jdouble),
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_annotatedTtl<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a_payload_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    a_payload_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    a_payload_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    a_payload_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_payload_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_alternate__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_alternate_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    a_alternate_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    a_alternate_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    a_alternate_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_alternate_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_ttl__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_ttl: ::prebindgen_jni_runtime::jni::sys::jlong,
    a_priority__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_priority: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::Annotated {
            payload: perftest_flat::Payload {
                id: a_payload_id,
                seq: a_payload_seq,
                value: a_payload_value,
                flag: (a_payload_flag != 0),
                label: if !a_payload_label.is_null() {
                    ::core::option::Option::Some(
                        ::std::boxed::Box::new(
                            ::prebindgen_jni_runtime::read_string(env, &a_payload_label)?,
                        ),
                    )
                } else {
                    ::core::option::Option::None
                },
            },
            alternate: if (a_alternate__present != 0) {
                ::core::option::Option::Some(perftest_flat::Payload {
                    id: a_alternate_id,
                    seq: a_alternate_seq,
                    value: a_alternate_value,
                    flag: (a_alternate_flag != 0),
                    label: if !a_alternate_label.is_null() {
                        ::core::option::Option::Some(
                            ::std::boxed::Box::new(
                                ::prebindgen_jni_runtime::read_string(
                                    env,
                                    &a_alternate_label,
                                )?,
                            ),
                        )
                    } else {
                        ::core::option::Option::None
                    },
                })
            } else {
                ::core::option::Option::None
            },
            ttl: if (a_ttl__present != 0) {
                ::core::option::Option::Some(a_ttl)
            } else {
                ::core::option::Option::None
            },
            priority: if (a_priority__present != 0) {
                ::core::option::Option::Some(
                    (match a_priority {
                        0 => ::core::result::Result::Ok(perftest_flat::Priority::Low),
                        1 => ::core::result::Result::Ok(perftest_flat::Priority::Normal),
                        2 => ::core::result::Result::Ok(perftest_flat::Priority::High),
                        __v => {
                            ::core::result::Result::Err(
                                ::std::format!("invalid value {} for enum `Priority`", __v),
                            )
                        }
                    })?,
                )
            } else {
                ::core::option::Option::None
            },
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::annotated_ttl(&a);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match __result {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::box_jlong(
                    env,
                    (__x1 as ::prebindgen_jni_runtime::jni::sys::jlong),
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_annotatedPriority<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a_payload_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    a_payload_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    a_payload_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    a_payload_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_payload_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_alternate__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_alternate_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    a_alternate_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    a_alternate_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    a_alternate_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_alternate_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_ttl__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_ttl: ::prebindgen_jni_runtime::jni::sys::jlong,
    a_priority__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_priority: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::Annotated {
            payload: perftest_flat::Payload {
                id: a_payload_id,
                seq: a_payload_seq,
                value: a_payload_value,
                flag: (a_payload_flag != 0),
                label: if !a_payload_label.is_null() {
                    ::core::option::Option::Some(
                        ::std::boxed::Box::new(
                            ::prebindgen_jni_runtime::read_string(env, &a_payload_label)?,
                        ),
                    )
                } else {
                    ::core::option::Option::None
                },
            },
            alternate: if (a_alternate__present != 0) {
                ::core::option::Option::Some(perftest_flat::Payload {
                    id: a_alternate_id,
                    seq: a_alternate_seq,
                    value: a_alternate_value,
                    flag: (a_alternate_flag != 0),
                    label: if !a_alternate_label.is_null() {
                        ::core::option::Option::Some(
                            ::std::boxed::Box::new(
                                ::prebindgen_jni_runtime::read_string(
                                    env,
                                    &a_alternate_label,
                                )?,
                            ),
                        )
                    } else {
                        ::core::option::Option::None
                    },
                })
            } else {
                ::core::option::Option::None
            },
            ttl: if (a_ttl__present != 0) {
                ::core::option::Option::Some(a_ttl)
            } else {
                ::core::option::Option::None
            },
            priority: if (a_priority__present != 0) {
                ::core::option::Option::Some(
                    (match a_priority {
                        0 => ::core::result::Result::Ok(perftest_flat::Priority::Low),
                        1 => ::core::result::Result::Ok(perftest_flat::Priority::Normal),
                        2 => ::core::result::Result::Ok(perftest_flat::Priority::High),
                        __v => {
                            ::core::result::Result::Err(
                                ::std::format!("invalid value {} for enum `Priority`", __v),
                            )
                        }
                    })?,
                )
            } else {
                ::core::option::Option::None
            },
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::annotated_priority(&a);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match __result {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::box_jint(
                    env,
                    (match __x1 {
                        perftest_flat::Priority::Low => 0,
                        perftest_flat::Priority::Normal => 1,
                        perftest_flat::Priority::High => 2,
                    } as i32),
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_annotatedPayloadValue<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a_payload_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    a_payload_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    a_payload_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    a_payload_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_payload_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_alternate__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_alternate_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    a_alternate_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    a_alternate_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    a_alternate_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_alternate_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_ttl__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_ttl: ::prebindgen_jni_runtime::jni::sys::jlong,
    a_priority__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    a_priority: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jdouble {
    let mut __env = __env;
    let env = &mut __env;
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::Annotated {
            payload: perftest_flat::Payload {
                id: a_payload_id,
                seq: a_payload_seq,
                value: a_payload_value,
                flag: (a_payload_flag != 0),
                label: if !a_payload_label.is_null() {
                    ::core::option::Option::Some(
                        ::std::boxed::Box::new(
                            ::prebindgen_jni_runtime::read_string(env, &a_payload_label)?,
                        ),
                    )
                } else {
                    ::core::option::Option::None
                },
            },
            alternate: if (a_alternate__present != 0) {
                ::core::option::Option::Some(perftest_flat::Payload {
                    id: a_alternate_id,
                    seq: a_alternate_seq,
                    value: a_alternate_value,
                    flag: (a_alternate_flag != 0),
                    label: if !a_alternate_label.is_null() {
                        ::core::option::Option::Some(
                            ::std::boxed::Box::new(
                                ::prebindgen_jni_runtime::read_string(
                                    env,
                                    &a_alternate_label,
                                )?,
                            ),
                        )
                    } else {
                        ::core::option::Option::None
                    },
                })
            } else {
                ::core::option::Option::None
            },
            ttl: if (a_ttl__present != 0) {
                ::core::option::Option::Some(a_ttl)
            } else {
                ::core::option::Option::None
            },
            priority: if (a_priority__present != 0) {
                ::core::option::Option::Some(
                    (match a_priority {
                        0 => ::core::result::Result::Ok(perftest_flat::Priority::Low),
                        1 => ::core::result::Result::Ok(perftest_flat::Priority::Normal),
                        2 => ::core::result::Result::Ok(perftest_flat::Priority::High),
                        __v => {
                            ::core::result::Result::Err(
                                ::std::format!("invalid value {} for enum `Priority`", __v),
                            )
                        }
                    })?,
                )
            } else {
                ::core::option::Option::None
            },
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0.0f64;
        }
    };
    let __result = perftest_flat::annotated_payload_value(&a);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jdouble,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jdouble);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0.0f64
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_observationNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    which: ::prebindgen_jni_runtime::jni::sys::jint,
    with_fallback: ::prebindgen_jni_runtime::jni::sys::jboolean,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let which = which;
    let with_fallback = (with_fallback != 0);
    let __result = perftest_flat::observation_new(which, with_fallback);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (
            r_id,
            r_reading__tag,
            r_reading_exact_v0,
            r_reading_range_low,
            r_reading_range_high,
            r_reading_labeled_v0,
            r_reading_labeled_v1,
            r_reading_companion_v0,
            r_fallback__present,
            r_fallback__tag,
            r_fallback_exact_v0,
            r_fallback_range_low,
            r_fallback_range_high,
            r_fallback_labeled_v0,
            r_fallback_labeled_v1,
            r_fallback_companion_v0,
            r_note,
        ) = {
            let perftest_flat::Observation {
                id: __f0,
                reading: __f1,
                fallback: __f2,
                note: __f3,
            } = __result;
            let r_id = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
            let (
                r_reading__tag,
                r_reading_exact_v0,
                r_reading_range_low,
                r_reading_range_high,
                r_reading_labeled_v0,
                r_reading_labeled_v1,
                r_reading_companion_v0,
            ) = match __f1 {
                perftest_flat::Reading::Missing => {
                    (
                        0i32,
                        0,
                        0,
                        0,
                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                        0,
                        0,
                    )
                }
                perftest_flat::Reading::Exact(__f0) => {
                    let r_reading_exact_v0 = (__f0
                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                    (
                        1i32,
                        r_reading_exact_v0,
                        0,
                        0,
                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                        0,
                        0,
                    )
                }
                perftest_flat::Reading::Range { low: __f0, high: __f1 } => {
                    let r_reading_range_low = (__f0
                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                    let r_reading_range_high = (__f1
                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                    (
                        2i32,
                        0,
                        r_reading_range_low,
                        r_reading_range_high,
                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                        0,
                        0,
                    )
                }
                perftest_flat::Reading::Labeled(__f0, __f1) => {
                    let r_reading_labeled_v0 = ::prebindgen_jni_runtime::new_string(
                        env,
                        ::core::convert::AsRef::<str>::as_ref(&__f0),
                    )?;
                    let r_reading_labeled_v1 = (match __f1 {
                        perftest_flat::Priority::Low => 0,
                        perftest_flat::Priority::Normal => 1,
                        perftest_flat::Priority::High => 2,
                    } as i32);
                    (3i32, 0, 0, 0, r_reading_labeled_v0, r_reading_labeled_v1, 0)
                }
                perftest_flat::Reading::Companion(__f0) => {
                    let r_reading_companion_v0 = (__f0
                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                    (
                        4i32,
                        0,
                        0,
                        0,
                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                        0,
                        r_reading_companion_v0,
                    )
                }
            };
            let (
                r_fallback__present,
                r_fallback__tag,
                r_fallback_exact_v0,
                r_fallback_range_low,
                r_fallback_range_high,
                r_fallback_labeled_v0,
                r_fallback_labeled_v1,
                r_fallback_companion_v0,
            ) = match __f2 {
                ::core::option::Option::Some(__x2) => {
                    let (
                        r_fallback__tag,
                        r_fallback_exact_v0,
                        r_fallback_range_low,
                        r_fallback_range_high,
                        r_fallback_labeled_v0,
                        r_fallback_labeled_v1,
                        r_fallback_companion_v0,
                    ) = match __x2 {
                        perftest_flat::Reading::Missing => {
                            (
                                0i32,
                                0,
                                0,
                                0,
                                ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                0,
                                0,
                            )
                        }
                        perftest_flat::Reading::Exact(__f0) => {
                            let r_fallback_exact_v0 = (__f0
                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                            (
                                1i32,
                                r_fallback_exact_v0,
                                0,
                                0,
                                ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                0,
                                0,
                            )
                        }
                        perftest_flat::Reading::Range { low: __f0, high: __f1 } => {
                            let r_fallback_range_low = (__f0
                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                            let r_fallback_range_high = (__f1
                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                            (
                                2i32,
                                0,
                                r_fallback_range_low,
                                r_fallback_range_high,
                                ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                0,
                                0,
                            )
                        }
                        perftest_flat::Reading::Labeled(__f0, __f1) => {
                            let r_fallback_labeled_v0 = ::prebindgen_jni_runtime::new_string(
                                env,
                                ::core::convert::AsRef::<str>::as_ref(&__f0),
                            )?;
                            let r_fallback_labeled_v1 = (match __f1 {
                                perftest_flat::Priority::Low => 0,
                                perftest_flat::Priority::Normal => 1,
                                perftest_flat::Priority::High => 2,
                            } as i32);
                            (
                                3i32,
                                0,
                                0,
                                0,
                                r_fallback_labeled_v0,
                                r_fallback_labeled_v1,
                                0,
                            )
                        }
                        perftest_flat::Reading::Companion(__f0) => {
                            let r_fallback_companion_v0 = (__f0
                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                            (
                                4i32,
                                0,
                                0,
                                0,
                                ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                0,
                                r_fallback_companion_v0,
                            )
                        }
                    };
                    (
                        1u8,
                        r_fallback__tag,
                        r_fallback_exact_v0,
                        r_fallback_range_low,
                        r_fallback_range_high,
                        r_fallback_labeled_v0,
                        r_fallback_labeled_v1,
                        r_fallback_companion_v0,
                    )
                }
                ::core::option::Option::None => {
                    (
                        0u8,
                        0,
                        0,
                        0,
                        0,
                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                        0,
                        0,
                    )
                }
            };
            let r_note = ::prebindgen_jni_runtime::new_string(
                env,
                ::core::convert::AsRef::<str>::as_ref(&__f3),
            )?;
            (
                r_id,
                r_reading__tag,
                r_reading_exact_v0,
                r_reading_range_low,
                r_reading_range_high,
                r_reading_labeled_v0,
                r_reading_labeled_v1,
                r_reading_companion_v0,
                r_fallback__present,
                r_fallback__tag,
                r_fallback_exact_v0,
                r_fallback_range_low,
                r_fallback_range_high,
                r_fallback_labeled_v0,
                r_fallback_labeled_v1,
                r_fallback_companion_v0,
                r_note,
            )
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Observation",
                "run",
                "(JIJJJLjava/lang/String;IJZIJJJLjava/lang/String;IJLjava/lang/String;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_id,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_reading__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_reading_exact_v0,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_reading_range_low,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_reading_range_high,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_reading_labeled_v0.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_reading_labeled_v1,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_reading_companion_v0,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        z: r_fallback__present,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_fallback__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_fallback_exact_v0,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_fallback_range_low,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_fallback_range_high,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_fallback_labeled_v0.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_fallback_labeled_v1,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_fallback_companion_v0,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_note.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_observationWhich<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    o_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    o_reading__tag: ::prebindgen_jni_runtime::jni::sys::jint,
    o_reading_exact_v0: ::prebindgen_jni_runtime::jni::sys::jlong,
    o_reading_range_low: ::prebindgen_jni_runtime::jni::sys::jlong,
    o_reading_range_high: ::prebindgen_jni_runtime::jni::sys::jlong,
    o_reading_labeled_v0: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    o_reading_labeled_v1: ::prebindgen_jni_runtime::jni::sys::jint,
    o_reading_companion_v0: ::prebindgen_jni_runtime::jni::sys::jlong,
    o_fallback__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    o_fallback__tag: ::prebindgen_jni_runtime::jni::sys::jint,
    o_fallback_exact_v0: ::prebindgen_jni_runtime::jni::sys::jlong,
    o_fallback_range_low: ::prebindgen_jni_runtime::jni::sys::jlong,
    o_fallback_range_high: ::prebindgen_jni_runtime::jni::sys::jlong,
    o_fallback_labeled_v0: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    o_fallback_labeled_v1: ::prebindgen_jni_runtime::jni::sys::jint,
    o_fallback_companion_v0: ::prebindgen_jni_runtime::jni::sys::jlong,
    o_note: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jint {
    let mut __env = __env;
    let env = &mut __env;
    let o = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::Observation {
            id: o_id,
            reading: (match o_reading__tag {
                0i32 => ::core::result::Result::Ok(perftest_flat::Reading::Missing),
                1i32 => {
                    ::core::result::Result::Ok(
                        perftest_flat::Reading::Exact(o_reading_exact_v0),
                    )
                }
                2i32 => {
                    ::core::result::Result::Ok(perftest_flat::Reading::Range {
                        low: o_reading_range_low,
                        high: o_reading_range_high,
                    })
                }
                3i32 => {
                    ::core::result::Result::Ok(
                        perftest_flat::Reading::Labeled(
                            ::prebindgen_jni_runtime::read_string(
                                env,
                                &o_reading_labeled_v0,
                            )?,
                            (match o_reading_labeled_v1 {
                                0 => {
                                    ::core::result::Result::Ok(perftest_flat::Priority::Low)
                                }
                                1 => {
                                    ::core::result::Result::Ok(perftest_flat::Priority::Normal)
                                }
                                2 => {
                                    ::core::result::Result::Ok(perftest_flat::Priority::High)
                                }
                                __v => {
                                    ::core::result::Result::Err(
                                        ::std::format!("invalid value {} for enum `Priority`", __v),
                                    )
                                }
                            })?,
                        ),
                    )
                }
                4i32 => {
                    ::core::result::Result::Ok(
                        perftest_flat::Reading::Companion(o_reading_companion_v0),
                    )
                }
                __t => {
                    ::core::result::Result::Err(
                        ::std::format!("Reading: invalid tag {}", __t),
                    )
                }
            })?,
            fallback: if (o_fallback__present != 0) {
                ::core::option::Option::Some(
                    (match o_fallback__tag {
                        0i32 => {
                            ::core::result::Result::Ok(perftest_flat::Reading::Missing)
                        }
                        1i32 => {
                            ::core::result::Result::Ok(
                                perftest_flat::Reading::Exact(o_fallback_exact_v0),
                            )
                        }
                        2i32 => {
                            ::core::result::Result::Ok(perftest_flat::Reading::Range {
                                low: o_fallback_range_low,
                                high: o_fallback_range_high,
                            })
                        }
                        3i32 => {
                            ::core::result::Result::Ok(
                                perftest_flat::Reading::Labeled(
                                    ::prebindgen_jni_runtime::read_string(
                                        env,
                                        &o_fallback_labeled_v0,
                                    )?,
                                    (match o_fallback_labeled_v1 {
                                        0 => {
                                            ::core::result::Result::Ok(perftest_flat::Priority::Low)
                                        }
                                        1 => {
                                            ::core::result::Result::Ok(perftest_flat::Priority::Normal)
                                        }
                                        2 => {
                                            ::core::result::Result::Ok(perftest_flat::Priority::High)
                                        }
                                        __v => {
                                            ::core::result::Result::Err(
                                                ::std::format!("invalid value {} for enum `Priority`", __v),
                                            )
                                        }
                                    })?,
                                ),
                            )
                        }
                        4i32 => {
                            ::core::result::Result::Ok(
                                perftest_flat::Reading::Companion(o_fallback_companion_v0),
                            )
                        }
                        __t => {
                            ::core::result::Result::Err(
                                ::std::format!("Reading: invalid tag {}", __t),
                            )
                        }
                    })?,
                )
            } else {
                ::core::option::Option::None
            },
            note: ::prebindgen_jni_runtime::read_string(env, &o_note)?,
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::observation_which(o);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jint,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jint);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_taggedNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    which: ::prebindgen_jni_runtime::jni::sys::jint,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let which = which;
    let __result = perftest_flat::tagged_new(which);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r_id, r_marker__tag, r_marker_ranked_v0) = {
            let perftest_flat::Tagged { id: __f0, marker: __f1 } = __result;
            let r_id = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
            let (r_marker__tag, r_marker_ranked_v0) = match __f1 {
                perftest_flat::Marker::None_ => {
                    (0i32, ::prebindgen_jni_runtime::jni::objects::JObject::null())
                }
                perftest_flat::Marker::Ranked(__f0) => {
                    let r_marker_ranked_v0 = match __f0 {
                        ::core::option::Option::Some(__x3) => {
                            ::prebindgen_jni_runtime::box_jint(
                                env,
                                (match __x3 {
                                    perftest_flat::Priority::Low => 0,
                                    perftest_flat::Priority::Normal => 1,
                                    perftest_flat::Priority::High => 2,
                                } as i32),
                            )?
                        }
                        ::core::option::Option::None => {
                            ::prebindgen_jni_runtime::jni::objects::JObject::null()
                        }
                    };
                    (1i32, r_marker_ranked_v0)
                }
            };
            (r_id, r_marker__tag, r_marker_ranked_v0)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Tagged",
                "run",
                "(JILjava/lang/Integer;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_id,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_marker__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_marker_ranked_v0.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_taggedRank<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    t_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    t_marker__tag: ::prebindgen_jni_runtime::jni::sys::jint,
    t_marker_ranked_v0__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    t_marker_ranked_v0: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jint {
    let mut __env = __env;
    let env = &mut __env;
    let t = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::Tagged {
            id: t_id,
            marker: (match t_marker__tag {
                0i32 => ::core::result::Result::Ok(perftest_flat::Marker::None_),
                1i32 => {
                    ::core::result::Result::Ok(
                        perftest_flat::Marker::Ranked(
                            if (t_marker_ranked_v0__present != 0) {
                                ::core::option::Option::Some(
                                    (match t_marker_ranked_v0 {
                                        0 => {
                                            ::core::result::Result::Ok(perftest_flat::Priority::Low)
                                        }
                                        1 => {
                                            ::core::result::Result::Ok(perftest_flat::Priority::Normal)
                                        }
                                        2 => {
                                            ::core::result::Result::Ok(perftest_flat::Priority::High)
                                        }
                                        __v => {
                                            ::core::result::Result::Err(
                                                ::std::format!("invalid value {} for enum `Priority`", __v),
                                            )
                                        }
                                    })?,
                                )
                            } else {
                                ::core::option::Option::None
                            },
                        ),
                    )
                }
                __t => {
                    ::core::result::Result::Err(
                        ::std::format!("Marker: invalid tag {}", __t),
                    )
                }
            })?,
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::tagged_rank(t);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jint,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jint);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_markerOf<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    which: ::prebindgen_jni_runtime::jni::sys::jint,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let which = which;
    let __result = perftest_flat::marker_of(which);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r__tag, r_ranked_v0) = match __result {
            perftest_flat::Marker::None_ => {
                (0i32, ::prebindgen_jni_runtime::jni::objects::JObject::null())
            }
            perftest_flat::Marker::Ranked(__f0) => {
                let r_ranked_v0 = match __f0 {
                    ::core::option::Option::Some(__x2) => {
                        ::prebindgen_jni_runtime::box_jint(
                            env,
                            (match __x2 {
                                perftest_flat::Priority::Low => 0,
                                perftest_flat::Priority::Normal => 1,
                                perftest_flat::Priority::High => 2,
                            } as i32),
                        )?
                    }
                    ::core::option::Option::None => {
                        ::prebindgen_jni_runtime::jni::objects::JObject::null()
                    }
                };
                (1i32, r_ranked_v0)
            }
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Marker",
                "run",
                "(ILjava/lang/Integer;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_ranked_v0.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_readingOf<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    which: ::prebindgen_jni_runtime::jni::sys::jint,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let which = which;
    let __result = perftest_flat::reading_of(which);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (
            r__tag,
            r_exact_v0,
            r_range_low,
            r_range_high,
            r_labeled_v0,
            r_labeled_v1,
            r_companion_v0,
        ) = match __result {
            perftest_flat::Reading::Missing => {
                (
                    0i32,
                    0,
                    0,
                    0,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    0,
                    0,
                )
            }
            perftest_flat::Reading::Exact(__f0) => {
                let r_exact_v0 = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                (
                    1i32,
                    r_exact_v0,
                    0,
                    0,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    0,
                    0,
                )
            }
            perftest_flat::Reading::Range { low: __f0, high: __f1 } => {
                let r_range_low = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                let r_range_high = (__f1 as ::prebindgen_jni_runtime::jni::sys::jlong);
                (
                    2i32,
                    0,
                    r_range_low,
                    r_range_high,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    0,
                    0,
                )
            }
            perftest_flat::Reading::Labeled(__f0, __f1) => {
                let r_labeled_v0 = ::prebindgen_jni_runtime::new_string(
                    env,
                    ::core::convert::AsRef::<str>::as_ref(&__f0),
                )?;
                let r_labeled_v1 = (match __f1 {
                    perftest_flat::Priority::Low => 0,
                    perftest_flat::Priority::Normal => 1,
                    perftest_flat::Priority::High => 2,
                } as i32);
                (3i32, 0, 0, 0, r_labeled_v0, r_labeled_v1, 0)
            }
            perftest_flat::Reading::Companion(__f0) => {
                let r_companion_v0 = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                (
                    4i32,
                    0,
                    0,
                    0,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    0,
                    r_companion_v0,
                )
            }
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Reading",
                "run",
                "(IJJJLjava/lang/String;IJ)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_exact_v0,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_range_low,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_range_high,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_labeled_v0.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_labeled_v1,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_companion_v0,
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_readingMaybe<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    which: ::prebindgen_jni_runtime::jni::sys::jint,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let which = which;
    let __result = perftest_flat::reading_maybe(which);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (
            r__present,
            r__tag,
            r_exact_v0,
            r_range_low,
            r_range_high,
            r_labeled_v0,
            r_labeled_v1,
            r_companion_v0,
        ) = match __result {
            ::core::option::Option::Some(__x1) => {
                let (
                    r__tag,
                    r_exact_v0,
                    r_range_low,
                    r_range_high,
                    r_labeled_v0,
                    r_labeled_v1,
                    r_companion_v0,
                ) = match __x1 {
                    perftest_flat::Reading::Missing => {
                        (
                            0i32,
                            0,
                            0,
                            0,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            0,
                            0,
                        )
                    }
                    perftest_flat::Reading::Exact(__f0) => {
                        let r_exact_v0 = (__f0
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        (
                            1i32,
                            r_exact_v0,
                            0,
                            0,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            0,
                            0,
                        )
                    }
                    perftest_flat::Reading::Range { low: __f0, high: __f1 } => {
                        let r_range_low = (__f0
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        let r_range_high = (__f1
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        (
                            2i32,
                            0,
                            r_range_low,
                            r_range_high,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            0,
                            0,
                        )
                    }
                    perftest_flat::Reading::Labeled(__f0, __f1) => {
                        let r_labeled_v0 = ::prebindgen_jni_runtime::new_string(
                            env,
                            ::core::convert::AsRef::<str>::as_ref(&__f0),
                        )?;
                        let r_labeled_v1 = (match __f1 {
                            perftest_flat::Priority::Low => 0,
                            perftest_flat::Priority::Normal => 1,
                            perftest_flat::Priority::High => 2,
                        } as i32);
                        (3i32, 0, 0, 0, r_labeled_v0, r_labeled_v1, 0)
                    }
                    perftest_flat::Reading::Companion(__f0) => {
                        let r_companion_v0 = (__f0
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        (
                            4i32,
                            0,
                            0,
                            0,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            0,
                            r_companion_v0,
                        )
                    }
                };
                (
                    1u8,
                    r__tag,
                    r_exact_v0,
                    r_range_low,
                    r_range_high,
                    r_labeled_v0,
                    r_labeled_v1,
                    r_companion_v0,
                )
            }
            ::core::option::Option::None => {
                (
                    0u8,
                    0,
                    0,
                    0,
                    0,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    0,
                    0,
                )
            }
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Option_Reading",
                "run",
                "(ZIJJJLjava/lang/String;IJ)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        z: r__present,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_exact_v0,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_range_low,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_range_high,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_labeled_v0.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_labeled_v1,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_companion_v0,
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_readingSeries<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    n: ::prebindgen_jni_runtime::jni::sys::jint,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let n = n;
    let __result = perftest_flat::reading_series(n);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (
            r__n,
            r__tag,
            r_exact_v0,
            r_range_low,
            r_range_high,
            r_labeled_v0,
            r_labeled_v1,
            r_companion_v0,
        ) = {
            let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(
                    __result,
                )
                .collect();
            let __n = __items.len();
            let mut __c1_0 = ::std::vec::Vec::with_capacity(__n);
            let mut __c1_1 = ::std::vec::Vec::with_capacity(__n);
            let mut __c1_2 = ::std::vec::Vec::with_capacity(__n);
            let mut __c1_3 = ::std::vec::Vec::with_capacity(__n);
            let __c1_4 = ::prebindgen_jni_runtime::new_object_array(env, __n)?;
            let mut __c1_5 = ::std::vec::Vec::with_capacity(__n);
            let mut __c1_6 = ::std::vec::Vec::with_capacity(__n);
            for (__i, __x1) in __items.into_iter().enumerate() {
                let (
                    __e1__tag,
                    __e1_exact_v0,
                    __e1_range_low,
                    __e1_range_high,
                    __e1_labeled_v0,
                    __e1_labeled_v1,
                    __e1_companion_v0,
                ) = match __x1 {
                    perftest_flat::Reading::Missing => {
                        (
                            0i32,
                            0,
                            0,
                            0,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            0,
                            0,
                        )
                    }
                    perftest_flat::Reading::Exact(__f0) => {
                        let __e1_exact_v0 = (__f0
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        (
                            1i32,
                            __e1_exact_v0,
                            0,
                            0,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            0,
                            0,
                        )
                    }
                    perftest_flat::Reading::Range { low: __f0, high: __f1 } => {
                        let __e1_range_low = (__f0
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        let __e1_range_high = (__f1
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        (
                            2i32,
                            0,
                            __e1_range_low,
                            __e1_range_high,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            0,
                            0,
                        )
                    }
                    perftest_flat::Reading::Labeled(__f0, __f1) => {
                        let __e1_labeled_v0 = ::prebindgen_jni_runtime::new_string(
                            env,
                            ::core::convert::AsRef::<str>::as_ref(&__f0),
                        )?;
                        let __e1_labeled_v1 = (match __f1 {
                            perftest_flat::Priority::Low => 0,
                            perftest_flat::Priority::Normal => 1,
                            perftest_flat::Priority::High => 2,
                        } as i32);
                        (3i32, 0, 0, 0, __e1_labeled_v0, __e1_labeled_v1, 0)
                    }
                    perftest_flat::Reading::Companion(__f0) => {
                        let __e1_companion_v0 = (__f0
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        (
                            4i32,
                            0,
                            0,
                            0,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            0,
                            __e1_companion_v0,
                        )
                    }
                };
                __c1_0.push(__e1__tag);
                __c1_1.push(__e1_exact_v0);
                __c1_2.push(__e1_range_low);
                __c1_3.push(__e1_range_high);
                ::prebindgen_jni_runtime::object_array_set(
                    env,
                    &__c1_4,
                    __i,
                    __e1_labeled_v0,
                )?;
                __c1_5.push(__e1_labeled_v1);
                __c1_6.push(__e1_companion_v0);
            }
            (
                __n as i32,
                ::prebindgen_jni_runtime::write_ints(env, &__c1_0)?,
                ::prebindgen_jni_runtime::write_longs(env, &__c1_1)?,
                ::prebindgen_jni_runtime::write_longs(env, &__c1_2)?,
                ::prebindgen_jni_runtime::write_longs(env, &__c1_3)?,
                __c1_4,
                ::prebindgen_jni_runtime::write_ints(env, &__c1_5)?,
                ::prebindgen_jni_runtime::write_longs(env, &__c1_6)?,
            )
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Vec_Reading",
                "run",
                "(I[I[J[J[J[Ljava/lang/Object;[I[J)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__n,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r__tag.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_exact_v0.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_range_low.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_range_high.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_labeled_v0.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_labeled_v1.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_companion_v0.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_readingEach<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    n: ::prebindgen_jni_runtime::jni::sys::jint,
    sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let n = n;
    let sink = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __up = ::prebindgen_jni_runtime::Upcall::new(
                env,
                &sink,
                "run",
                "(IJJJLjava/lang/String;IJ)V",
                46i32,
            )?;
            move |__a0: perftest_flat::Reading| {
                let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                    __up.call_void(|env| {
                        let (
                            a0__tag,
                            a0_exact_v0,
                            a0_range_low,
                            a0_range_high,
                            a0_labeled_v0,
                            a0_labeled_v1,
                            a0_companion_v0,
                        ) = match __a0 {
                            perftest_flat::Reading::Missing => {
                                (
                                    0i32,
                                    0,
                                    0,
                                    0,
                                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                    0,
                                    0,
                                )
                            }
                            perftest_flat::Reading::Exact(__f0) => {
                                let a0_exact_v0 = (__f0
                                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                                (
                                    1i32,
                                    a0_exact_v0,
                                    0,
                                    0,
                                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                    0,
                                    0,
                                )
                            }
                            perftest_flat::Reading::Range { low: __f0, high: __f1 } => {
                                let a0_range_low = (__f0
                                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                                let a0_range_high = (__f1
                                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                                (
                                    2i32,
                                    0,
                                    a0_range_low,
                                    a0_range_high,
                                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                    0,
                                    0,
                                )
                            }
                            perftest_flat::Reading::Labeled(__f0, __f1) => {
                                let a0_labeled_v0 = ::prebindgen_jni_runtime::new_string(
                                    env,
                                    ::core::convert::AsRef::<str>::as_ref(&__f0),
                                )?;
                                let a0_labeled_v1 = (match __f1 {
                                    perftest_flat::Priority::Low => 0,
                                    perftest_flat::Priority::Normal => 1,
                                    perftest_flat::Priority::High => 2,
                                } as i32);
                                (3i32, 0, 0, 0, a0_labeled_v0, a0_labeled_v1, 0)
                            }
                            perftest_flat::Reading::Companion(__f0) => {
                                let a0_companion_v0 = (__f0
                                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                                (
                                    4i32,
                                    0,
                                    0,
                                    0,
                                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                    0,
                                    a0_companion_v0,
                                )
                            }
                        };
                        ::core::result::Result::Ok(
                            ::std::vec![
                                ::prebindgen_jni_runtime::jni::sys::jvalue { i : a0__tag },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j : a0_exact_v0
                                }, ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_range_low }, ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j : a0_range_high },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l :
                                a0_labeled_v0.as_raw() },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { i :
                                a0_labeled_v1 }, ::prebindgen_jni_runtime::jni::sys::jvalue
                                { j : a0_companion_v0 }
                            ],
                        )
                    })?;
                    ::core::result::Result::Ok(())
                })();
                if let ::core::result::Result::Err(__err) = __res {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (Reading) + Send + Sync + 'static",
                        &__err,
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
    let __result = perftest_flat::reading_each(n, sink);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_lookupOf<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    count: ::prebindgen_jni_runtime::jni::sys::jlong,
    total: ::prebindgen_jni_runtime::jni::sys::jdouble,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let count = count;
    let total = total;
    let __result = perftest_flat::lookup_of(count, total);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r__tag, r_found_v0, r_failed_v0) = match __result {
            perftest_flat::Lookup::Absent => {
                (0i32, 0, ::prebindgen_jni_runtime::jni::objects::JObject::null())
            }
            perftest_flat::Lookup::Found(__f0) => {
                let r_found_v0 = ::prebindgen_jni_runtime::new_handle(__f0);
                (
                    1i32,
                    r_found_v0,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                )
            }
            perftest_flat::Lookup::Failed(__f0) => {
                let r_failed_v0 = ::prebindgen_jni_runtime::new_string(
                    env,
                    ::core::convert::AsRef::<str>::as_ref(&__f0),
                )?;
                (2i32, 0, r_failed_v0)
            }
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Lookup",
                "run",
                "(IJLjava/lang/String;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_found_v0,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_failed_v0.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_lookupEach<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    n: ::prebindgen_jni_runtime::jni::sys::jlong,
    total: ::prebindgen_jni_runtime::jni::sys::jdouble,
    sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let n = n;
    let total = total;
    let sink = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __up = ::prebindgen_jni_runtime::Upcall::new(
                env,
                &sink,
                "run",
                "(IJLjava/lang/String;)V",
                38i32,
            )?;
            move |__a0: perftest_flat::Lookup| {
                let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                    __up.call_void(|env| {
                        let (a0__tag, a0_found_v0, a0_failed_v0) = match __a0 {
                            perftest_flat::Lookup::Absent => {
                                (
                                    0i32,
                                    0,
                                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                )
                            }
                            perftest_flat::Lookup::Found(__f0) => {
                                let a0_found_v0 = ::prebindgen_jni_runtime::new_handle(
                                    __f0,
                                );
                                (
                                    1i32,
                                    a0_found_v0,
                                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                )
                            }
                            perftest_flat::Lookup::Failed(__f0) => {
                                let a0_failed_v0 = ::prebindgen_jni_runtime::new_string(
                                    env,
                                    ::core::convert::AsRef::<str>::as_ref(&__f0),
                                )?;
                                (2i32, 0, a0_failed_v0)
                            }
                        };
                        ::core::result::Result::Ok(
                            ::std::vec![
                                ::prebindgen_jni_runtime::jni::sys::jvalue { i : a0__tag },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j : a0_found_v0
                                }, ::prebindgen_jni_runtime::jni::sys::jvalue { l :
                                a0_failed_v0.as_raw() }
                            ],
                        )
                    })?;
                    ::core::result::Result::Ok(())
                })();
                if let ::core::result::Result::Err(__err) = __res {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (Lookup) + Send + Sync + 'static",
                        &__err,
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
    let __result = perftest_flat::lookup_each(n, total, sink);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_verdictNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    id: ::prebindgen_jni_runtime::jni::sys::jlong,
    count: ::prebindgen_jni_runtime::jni::sys::jlong,
    total: ::prebindgen_jni_runtime::jni::sys::jdouble,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let id = id;
    let count = count;
    let total = total;
    let __result = perftest_flat::verdict_new(id, count, total);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r_id, r_outcome__tag, r_outcome_found_v0, r_outcome_failed_v0) = {
            let perftest_flat::Verdict { id: __f0, outcome: __f1 } = __result;
            let r_id = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
            let (r_outcome__tag, r_outcome_found_v0, r_outcome_failed_v0) = match __f1 {
                perftest_flat::Lookup::Absent => {
                    (0i32, 0, ::prebindgen_jni_runtime::jni::objects::JObject::null())
                }
                perftest_flat::Lookup::Found(__f0) => {
                    let r_outcome_found_v0 = ::prebindgen_jni_runtime::new_handle(__f0);
                    (
                        1i32,
                        r_outcome_found_v0,
                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    )
                }
                perftest_flat::Lookup::Failed(__f0) => {
                    let r_outcome_failed_v0 = ::prebindgen_jni_runtime::new_string(
                        env,
                        ::core::convert::AsRef::<str>::as_ref(&__f0),
                    )?;
                    (2i32, 0, r_outcome_failed_v0)
                }
            };
            (r_id, r_outcome__tag, r_outcome_found_v0, r_outcome_failed_v0)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Verdict",
                "run",
                "(JIJLjava/lang/String;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_id,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_outcome__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_outcome_found_v0,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_outcome_failed_v0.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_dossierNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    note: ::prebindgen_jni_runtime::jni::sys::jlong,
    tag: ::prebindgen_jni_runtime::jni::sys::jlong,
    count: ::prebindgen_jni_runtime::jni::sys::jlong,
    total: ::prebindgen_jni_runtime::jni::sys::jdouble,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let note = note;
    let tag = tag;
    let count = count;
    let total = total;
    let __result = perftest_flat::dossier_new(note, tag, count, total);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r_note, r_holder_tag, r_holder_summary) = {
            let perftest_flat::Dossier { note: __f0, holder: __f1 } = __result;
            let r_note = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
            let (r_holder_tag, r_holder_summary) = {
                let perftest_flat::Holder { tag: __f0, summary: __f1 } = __f1;
                let r_holder_tag = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                let r_holder_summary = ::prebindgen_jni_runtime::new_handle(__f1);
                (r_holder_tag, r_holder_summary)
            };
            (r_note, r_holder_tag, r_holder_summary)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Dossier",
                "run",
                "(JJJ)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_note,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_holder_tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_holder_summary,
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_reportEach<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    n: ::prebindgen_jni_runtime::jni::sys::jlong,
    sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let n = n;
    let sink = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __up = ::prebindgen_jni_runtime::Upcall::new(
                env,
                &sink,
                "run",
                "(JDZJJJJIJLjava/lang/String;Ljava/lang/String;)V",
                54i32,
            )?;
            move |__a0: perftest_flat::Report| {
                let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                    __up.call_void(|env| {
                        let (
                            a0_summary_count,
                            a0_summary_total,
                            a0_taken__present,
                            a0_taken_secs,
                            a0_taken_nanos,
                            a0_origin_secs,
                            a0_origin_nanos,
                            a0_outcome__tag,
                            a0_outcome_found_v0,
                            a0_outcome_failed_v0,
                            a0_label,
                        ) = {
                            let __v1 = __a0;
                            let __s1 = perftest_flat::report_into_struct(__v1);
                            let (
                                a0_summary_count,
                                a0_summary_total,
                                a0_taken__present,
                                a0_taken_secs,
                                a0_taken_nanos,
                                a0_origin_secs,
                                a0_origin_nanos,
                                a0_outcome__tag,
                                a0_outcome_found_v0,
                                a0_outcome_failed_v0,
                                a0_label,
                            ) = {
                                let perftest_flat::ReportStruct {
                                    summary: __f2_0,
                                    taken: __f2_1,
                                    origin: __f2_2,
                                    outcome: __f2_3,
                                    label: __f2_4,
                                } = __s1;
                                let (a0_summary_count, a0_summary_total) = {
                                    let __v3 = __f2_0;
                                    let a0_summary_count = (perftest_flat::summary_count(&__v3)
                                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                                    let a0_summary_total = (perftest_flat::summary_total(&__v3)
                                        as ::prebindgen_jni_runtime::jni::sys::jdouble);
                                    (a0_summary_count, a0_summary_total)
                                };
                                let (a0_taken__present, a0_taken_secs, a0_taken_nanos) = match __f2_1 {
                                    ::core::option::Option::Some(__x3) => {
                                        let (a0_taken_secs, a0_taken_nanos) = {
                                            let perftest_flat::Stamp { secs: __f0, nanos: __f1 } = __x3;
                                            let a0_taken_secs = (__f0
                                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                                            let a0_taken_nanos = (__f1
                                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                                            (a0_taken_secs, a0_taken_nanos)
                                        };
                                        (1u8, a0_taken_secs, a0_taken_nanos)
                                    }
                                    ::core::option::Option::None => (0u8, 0, 0),
                                };
                                let (a0_origin_secs, a0_origin_nanos) = {
                                    let perftest_flat::Stamp { secs: __f3_0, nanos: __f3_1 } = __f2_2;
                                    let a0_origin_secs = (__f3_0
                                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                                    let a0_origin_nanos = (__f3_1
                                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                                    (a0_origin_secs, a0_origin_nanos)
                                };
                                let (
                                    a0_outcome__tag,
                                    a0_outcome_found_v0,
                                    a0_outcome_failed_v0,
                                ) = match __f2_3 {
                                    perftest_flat::Lookup::Absent => {
                                        (
                                            0i32,
                                            0,
                                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                        )
                                    }
                                    perftest_flat::Lookup::Found(__f0) => {
                                        let a0_outcome_found_v0 = ::prebindgen_jni_runtime::new_handle(
                                            __f0,
                                        );
                                        (
                                            1i32,
                                            a0_outcome_found_v0,
                                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                        )
                                    }
                                    perftest_flat::Lookup::Failed(__f0) => {
                                        let a0_outcome_failed_v0 = ::prebindgen_jni_runtime::new_string(
                                            env,
                                            ::core::convert::AsRef::<str>::as_ref(&__f0),
                                        )?;
                                        (2i32, 0, a0_outcome_failed_v0)
                                    }
                                };
                                let a0_label = ::prebindgen_jni_runtime::new_string(
                                    env,
                                    ::core::convert::AsRef::<str>::as_ref(&__f2_4),
                                )?;
                                (
                                    a0_summary_count,
                                    a0_summary_total,
                                    a0_taken__present,
                                    a0_taken_secs,
                                    a0_taken_nanos,
                                    a0_origin_secs,
                                    a0_origin_nanos,
                                    a0_outcome__tag,
                                    a0_outcome_found_v0,
                                    a0_outcome_failed_v0,
                                    a0_label,
                                )
                            };
                            (
                                a0_summary_count,
                                a0_summary_total,
                                a0_taken__present,
                                a0_taken_secs,
                                a0_taken_nanos,
                                a0_origin_secs,
                                a0_origin_nanos,
                                a0_outcome__tag,
                                a0_outcome_found_v0,
                                a0_outcome_failed_v0,
                                a0_label,
                            )
                        };
                        ::core::result::Result::Ok(
                            ::std::vec![
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_summary_count },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { d :
                                a0_summary_total },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { z :
                                a0_taken__present },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_taken_secs }, ::prebindgen_jni_runtime::jni::sys::jvalue
                                { j : a0_taken_nanos },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_origin_secs }, ::prebindgen_jni_runtime::jni::sys::jvalue
                                { j : a0_origin_nanos },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { i :
                                a0_outcome__tag },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_outcome_found_v0 },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l :
                                a0_outcome_failed_v0.as_raw() },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l : a0_label
                                .as_raw() }
                            ],
                        )
                    })?;
                    ::core::result::Result::Ok(())
                })();
                if let ::core::result::Result::Err(__err) = __res {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (Report) + Send + Sync + 'static",
                        &__err,
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
    let __result = perftest_flat::report_each(n, sink);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_probeNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    seq: ::prebindgen_jni_runtime::jni::sys::jlong,
    count: ::prebindgen_jni_runtime::jni::sys::jlong,
    total: ::prebindgen_jni_runtime::jni::sys::jdouble,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let seq = seq;
    let count = count;
    let total = total;
    let __result = perftest_flat::probe_new(seq, count, total);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let __x = __result;
        {
            let (
                r_seq,
                r_outcome__present,
                r_outcome__tag,
                r_outcome_found_v0,
                r_outcome_failed_v0,
            ) = {
                let __v1 = __x;
                let __s1 = perftest_flat::probe_to_struct(&__v1);
                let (
                    r_seq,
                    r_outcome__present,
                    r_outcome__tag,
                    r_outcome_found_v0,
                    r_outcome_failed_v0,
                ) = {
                    let perftest_flat::ProbeStruct { seq: __f2_0, outcome: __f2_1 } = __s1;
                    let r_seq = (__f2_0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                    let (
                        r_outcome__present,
                        r_outcome__tag,
                        r_outcome_found_v0,
                        r_outcome_failed_v0,
                    ) = match __f2_1 {
                        ::core::option::Option::Some(__x3) => {
                            let (
                                r_outcome__tag,
                                r_outcome_found_v0,
                                r_outcome_failed_v0,
                            ) = match __x3 {
                                perftest_flat::Lookup::Absent => {
                                    (
                                        0i32,
                                        0,
                                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                    )
                                }
                                perftest_flat::Lookup::Found(__f0) => {
                                    let r_outcome_found_v0 = ::prebindgen_jni_runtime::new_handle(
                                        __f0,
                                    );
                                    (
                                        1i32,
                                        r_outcome_found_v0,
                                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                    )
                                }
                                perftest_flat::Lookup::Failed(__f0) => {
                                    let r_outcome_failed_v0 = ::prebindgen_jni_runtime::new_string(
                                        env,
                                        ::core::convert::AsRef::<str>::as_ref(&__f0),
                                    )?;
                                    (2i32, 0, r_outcome_failed_v0)
                                }
                            };
                            (
                                1u8,
                                r_outcome__tag,
                                r_outcome_found_v0,
                                r_outcome_failed_v0,
                            )
                        }
                        ::core::option::Option::None => {
                            (
                                0u8,
                                0,
                                0,
                                ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            )
                        }
                    };
                    (
                        r_seq,
                        r_outcome__present,
                        r_outcome__tag,
                        r_outcome_found_v0,
                        r_outcome_failed_v0,
                    )
                };
                (
                    r_seq,
                    r_outcome__present,
                    r_outcome__tag,
                    r_outcome_found_v0,
                    r_outcome_failed_v0,
                )
            };
            static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
            let __o = __S
                .call_object(
                    env,
                    "io/prebindgen/covertest/model/ProbeBuilderRaw",
                    "run",
                    "(JZIJLjava/lang/String;)Ljava/lang/Object;",
                    &__sink,
                    &[
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_seq,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            z: r_outcome__present,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            i: r_outcome__tag,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_outcome_found_v0,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            l: r_outcome_failed_v0.as_raw(),
                        },
                    ],
                )?;
            ::core::result::Result::Ok(__o.into_raw())
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_probeEach<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    n: ::prebindgen_jni_runtime::jni::sys::jlong,
    total: ::prebindgen_jni_runtime::jni::sys::jdouble,
    sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let n = n;
    let total = total;
    let sink = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __up = ::prebindgen_jni_runtime::Upcall::new(
                env,
                &sink,
                "run",
                "(JZIJLjava/lang/String;)V",
                42i32,
            )?;
            move |__a0: perftest_flat::Probe| {
                let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                    __up.call_void(|env| {
                        let (
                            a0_seq,
                            a0_outcome__present,
                            a0_outcome__tag,
                            a0_outcome_found_v0,
                            a0_outcome_failed_v0,
                        ) = {
                            let __v1 = __a0;
                            let __s1 = perftest_flat::probe_to_struct(&__v1);
                            let (
                                a0_seq,
                                a0_outcome__present,
                                a0_outcome__tag,
                                a0_outcome_found_v0,
                                a0_outcome_failed_v0,
                            ) = {
                                let perftest_flat::ProbeStruct {
                                    seq: __f2_0,
                                    outcome: __f2_1,
                                } = __s1;
                                let a0_seq = (__f2_0
                                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                                let (
                                    a0_outcome__present,
                                    a0_outcome__tag,
                                    a0_outcome_found_v0,
                                    a0_outcome_failed_v0,
                                ) = match __f2_1 {
                                    ::core::option::Option::Some(__x3) => {
                                        let (
                                            a0_outcome__tag,
                                            a0_outcome_found_v0,
                                            a0_outcome_failed_v0,
                                        ) = match __x3 {
                                            perftest_flat::Lookup::Absent => {
                                                (
                                                    0i32,
                                                    0,
                                                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                                )
                                            }
                                            perftest_flat::Lookup::Found(__f0) => {
                                                let a0_outcome_found_v0 = ::prebindgen_jni_runtime::new_handle(
                                                    __f0,
                                                );
                                                (
                                                    1i32,
                                                    a0_outcome_found_v0,
                                                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                                )
                                            }
                                            perftest_flat::Lookup::Failed(__f0) => {
                                                let a0_outcome_failed_v0 = ::prebindgen_jni_runtime::new_string(
                                                    env,
                                                    ::core::convert::AsRef::<str>::as_ref(&__f0),
                                                )?;
                                                (2i32, 0, a0_outcome_failed_v0)
                                            }
                                        };
                                        (
                                            1u8,
                                            a0_outcome__tag,
                                            a0_outcome_found_v0,
                                            a0_outcome_failed_v0,
                                        )
                                    }
                                    ::core::option::Option::None => {
                                        (
                                            0u8,
                                            0,
                                            0,
                                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                        )
                                    }
                                };
                                (
                                    a0_seq,
                                    a0_outcome__present,
                                    a0_outcome__tag,
                                    a0_outcome_found_v0,
                                    a0_outcome_failed_v0,
                                )
                            };
                            (
                                a0_seq,
                                a0_outcome__present,
                                a0_outcome__tag,
                                a0_outcome_found_v0,
                                a0_outcome_failed_v0,
                            )
                        };
                        ::core::result::Result::Ok(
                            ::std::vec![
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j : a0_seq },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { z :
                                a0_outcome__present },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { i :
                                a0_outcome__tag },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_outcome_found_v0 },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l :
                                a0_outcome_failed_v0.as_raw() }
                            ],
                        )
                    })?;
                    ::core::result::Result::Ok(())
                })();
                if let ::core::result::Result::Err(__err) = __res {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (Probe) + Send + Sync + 'static",
                        &__err,
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
    let __result = perftest_flat::probe_each(n, total, sink);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_ledgerEach<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    n: ::prebindgen_jni_runtime::jni::sys::jlong,
    sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let n = n;
    let sink = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __up = ::prebindgen_jni_runtime::Upcall::new(
                env,
                &sink,
                "run",
                "(ZJDZJJJJIJLjava/lang/String;Ljava/lang/String;ZJDZJJJJIJLjava/lang/String;Ljava/lang/String;)V",
                80i32,
            )?;
            move |__a0: perftest_flat::Ledger| {
                let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                    __up.call_void(|env| {
                        let (
                            a0_filed__present,
                            a0_filed_summary_count,
                            a0_filed_summary_total,
                            a0_filed_taken__present,
                            a0_filed_taken_secs,
                            a0_filed_taken_nanos,
                            a0_filed_origin_secs,
                            a0_filed_origin_nanos,
                            a0_filed_outcome__tag,
                            a0_filed_outcome_found_v0,
                            a0_filed_outcome_failed_v0,
                            a0_filed_label,
                            a0_archived__present,
                            a0_archived_summary_count,
                            a0_archived_summary_total,
                            a0_archived_taken__present,
                            a0_archived_taken_secs,
                            a0_archived_taken_nanos,
                            a0_archived_origin_secs,
                            a0_archived_origin_nanos,
                            a0_archived_outcome__tag,
                            a0_archived_outcome_found_v0,
                            a0_archived_outcome_failed_v0,
                            a0_archived_label,
                        ) = {
                            let __v1 = __a0;
                            let (
                                a0_filed__present,
                                a0_filed_summary_count,
                                a0_filed_summary_total,
                                a0_filed_taken__present,
                                a0_filed_taken_secs,
                                a0_filed_taken_nanos,
                                a0_filed_origin_secs,
                                a0_filed_origin_nanos,
                                a0_filed_outcome__tag,
                                a0_filed_outcome_found_v0,
                                a0_filed_outcome_failed_v0,
                                a0_filed_label,
                            ) = match perftest_flat::ledger_filed(&__v1) {
                                ::core::option::Option::Some(__x2) => {
                                    let (
                                        a0_filed_summary_count,
                                        a0_filed_summary_total,
                                        a0_filed_taken__present,
                                        a0_filed_taken_secs,
                                        a0_filed_taken_nanos,
                                        a0_filed_origin_secs,
                                        a0_filed_origin_nanos,
                                        a0_filed_outcome__tag,
                                        a0_filed_outcome_found_v0,
                                        a0_filed_outcome_failed_v0,
                                        a0_filed_label,
                                    ) = {
                                        let __v3 = __x2;
                                        let __s3 = perftest_flat::report_into_struct(
                                            ::core::clone::Clone::clone(__v3),
                                        );
                                        let (
                                            a0_filed_summary_count,
                                            a0_filed_summary_total,
                                            a0_filed_taken__present,
                                            a0_filed_taken_secs,
                                            a0_filed_taken_nanos,
                                            a0_filed_origin_secs,
                                            a0_filed_origin_nanos,
                                            a0_filed_outcome__tag,
                                            a0_filed_outcome_found_v0,
                                            a0_filed_outcome_failed_v0,
                                            a0_filed_label,
                                        ) = {
                                            let perftest_flat::ReportStruct {
                                                summary: __f4_0,
                                                taken: __f4_1,
                                                origin: __f4_2,
                                                outcome: __f4_3,
                                                label: __f4_4,
                                            } = __s3;
                                            let (a0_filed_summary_count, a0_filed_summary_total) = {
                                                let __v5 = __f4_0;
                                                let a0_filed_summary_count = (perftest_flat::summary_count(
                                                    &__v5,
                                                ) as ::prebindgen_jni_runtime::jni::sys::jlong);
                                                let a0_filed_summary_total = (perftest_flat::summary_total(
                                                    &__v5,
                                                ) as ::prebindgen_jni_runtime::jni::sys::jdouble);
                                                (a0_filed_summary_count, a0_filed_summary_total)
                                            };
                                            let (
                                                a0_filed_taken__present,
                                                a0_filed_taken_secs,
                                                a0_filed_taken_nanos,
                                            ) = match __f4_1 {
                                                ::core::option::Option::Some(__x5) => {
                                                    let (a0_filed_taken_secs, a0_filed_taken_nanos) = {
                                                        let perftest_flat::Stamp { secs: __f0, nanos: __f1 } = __x5;
                                                        let a0_filed_taken_secs = (__f0
                                                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                                                        let a0_filed_taken_nanos = (__f1
                                                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                                                        (a0_filed_taken_secs, a0_filed_taken_nanos)
                                                    };
                                                    (1u8, a0_filed_taken_secs, a0_filed_taken_nanos)
                                                }
                                                ::core::option::Option::None => (0u8, 0, 0),
                                            };
                                            let (a0_filed_origin_secs, a0_filed_origin_nanos) = {
                                                let perftest_flat::Stamp { secs: __f5_0, nanos: __f5_1 } = __f4_2;
                                                let a0_filed_origin_secs = (__f5_0
                                                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                                                let a0_filed_origin_nanos = (__f5_1
                                                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                                                (a0_filed_origin_secs, a0_filed_origin_nanos)
                                            };
                                            let (
                                                a0_filed_outcome__tag,
                                                a0_filed_outcome_found_v0,
                                                a0_filed_outcome_failed_v0,
                                            ) = match __f4_3 {
                                                perftest_flat::Lookup::Absent => {
                                                    (
                                                        0i32,
                                                        0,
                                                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                                    )
                                                }
                                                perftest_flat::Lookup::Found(__f0) => {
                                                    let a0_filed_outcome_found_v0 = ::prebindgen_jni_runtime::new_handle(
                                                        __f0,
                                                    );
                                                    (
                                                        1i32,
                                                        a0_filed_outcome_found_v0,
                                                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                                    )
                                                }
                                                perftest_flat::Lookup::Failed(__f0) => {
                                                    let a0_filed_outcome_failed_v0 = ::prebindgen_jni_runtime::new_string(
                                                        env,
                                                        ::core::convert::AsRef::<str>::as_ref(&__f0),
                                                    )?;
                                                    (2i32, 0, a0_filed_outcome_failed_v0)
                                                }
                                            };
                                            let a0_filed_label = ::prebindgen_jni_runtime::new_string(
                                                env,
                                                ::core::convert::AsRef::<str>::as_ref(&__f4_4),
                                            )?;
                                            (
                                                a0_filed_summary_count,
                                                a0_filed_summary_total,
                                                a0_filed_taken__present,
                                                a0_filed_taken_secs,
                                                a0_filed_taken_nanos,
                                                a0_filed_origin_secs,
                                                a0_filed_origin_nanos,
                                                a0_filed_outcome__tag,
                                                a0_filed_outcome_found_v0,
                                                a0_filed_outcome_failed_v0,
                                                a0_filed_label,
                                            )
                                        };
                                        (
                                            a0_filed_summary_count,
                                            a0_filed_summary_total,
                                            a0_filed_taken__present,
                                            a0_filed_taken_secs,
                                            a0_filed_taken_nanos,
                                            a0_filed_origin_secs,
                                            a0_filed_origin_nanos,
                                            a0_filed_outcome__tag,
                                            a0_filed_outcome_found_v0,
                                            a0_filed_outcome_failed_v0,
                                            a0_filed_label,
                                        )
                                    };
                                    (
                                        1u8,
                                        a0_filed_summary_count,
                                        a0_filed_summary_total,
                                        a0_filed_taken__present,
                                        a0_filed_taken_secs,
                                        a0_filed_taken_nanos,
                                        a0_filed_origin_secs,
                                        a0_filed_origin_nanos,
                                        a0_filed_outcome__tag,
                                        a0_filed_outcome_found_v0,
                                        a0_filed_outcome_failed_v0,
                                        a0_filed_label,
                                    )
                                }
                                ::core::option::Option::None => {
                                    (
                                        0u8,
                                        0,
                                        0.0f64,
                                        0,
                                        0,
                                        0,
                                        0,
                                        0,
                                        0,
                                        0,
                                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                    )
                                }
                            };
                            let (
                                a0_archived__present,
                                a0_archived_summary_count,
                                a0_archived_summary_total,
                                a0_archived_taken__present,
                                a0_archived_taken_secs,
                                a0_archived_taken_nanos,
                                a0_archived_origin_secs,
                                a0_archived_origin_nanos,
                                a0_archived_outcome__tag,
                                a0_archived_outcome_found_v0,
                                a0_archived_outcome_failed_v0,
                                a0_archived_label,
                            ) = match perftest_flat::ledger_archived(&__v1) {
                                ::core::option::Option::Some(__x2) => {
                                    let (
                                        a0_archived_summary_count,
                                        a0_archived_summary_total,
                                        a0_archived_taken__present,
                                        a0_archived_taken_secs,
                                        a0_archived_taken_nanos,
                                        a0_archived_origin_secs,
                                        a0_archived_origin_nanos,
                                        a0_archived_outcome__tag,
                                        a0_archived_outcome_found_v0,
                                        a0_archived_outcome_failed_v0,
                                        a0_archived_label,
                                    ) = {
                                        let __v3 = __x2;
                                        let __s3 = perftest_flat::report_into_struct(__v3);
                                        let (
                                            a0_archived_summary_count,
                                            a0_archived_summary_total,
                                            a0_archived_taken__present,
                                            a0_archived_taken_secs,
                                            a0_archived_taken_nanos,
                                            a0_archived_origin_secs,
                                            a0_archived_origin_nanos,
                                            a0_archived_outcome__tag,
                                            a0_archived_outcome_found_v0,
                                            a0_archived_outcome_failed_v0,
                                            a0_archived_label,
                                        ) = {
                                            let perftest_flat::ReportStruct {
                                                summary: __f4_0,
                                                taken: __f4_1,
                                                origin: __f4_2,
                                                outcome: __f4_3,
                                                label: __f4_4,
                                            } = __s3;
                                            let (
                                                a0_archived_summary_count,
                                                a0_archived_summary_total,
                                            ) = {
                                                let __v5 = __f4_0;
                                                let a0_archived_summary_count = (perftest_flat::summary_count(
                                                    &__v5,
                                                ) as ::prebindgen_jni_runtime::jni::sys::jlong);
                                                let a0_archived_summary_total = (perftest_flat::summary_total(
                                                    &__v5,
                                                ) as ::prebindgen_jni_runtime::jni::sys::jdouble);
                                                (a0_archived_summary_count, a0_archived_summary_total)
                                            };
                                            let (
                                                a0_archived_taken__present,
                                                a0_archived_taken_secs,
                                                a0_archived_taken_nanos,
                                            ) = match __f4_1 {
                                                ::core::option::Option::Some(__x5) => {
                                                    let (a0_archived_taken_secs, a0_archived_taken_nanos) = {
                                                        let perftest_flat::Stamp { secs: __f0, nanos: __f1 } = __x5;
                                                        let a0_archived_taken_secs = (__f0
                                                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                                                        let a0_archived_taken_nanos = (__f1
                                                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                                                        (a0_archived_taken_secs, a0_archived_taken_nanos)
                                                    };
                                                    (1u8, a0_archived_taken_secs, a0_archived_taken_nanos)
                                                }
                                                ::core::option::Option::None => (0u8, 0, 0),
                                            };
                                            let (a0_archived_origin_secs, a0_archived_origin_nanos) = {
                                                let perftest_flat::Stamp { secs: __f5_0, nanos: __f5_1 } = __f4_2;
                                                let a0_archived_origin_secs = (__f5_0
                                                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                                                let a0_archived_origin_nanos = (__f5_1
                                                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                                                (a0_archived_origin_secs, a0_archived_origin_nanos)
                                            };
                                            let (
                                                a0_archived_outcome__tag,
                                                a0_archived_outcome_found_v0,
                                                a0_archived_outcome_failed_v0,
                                            ) = match __f4_3 {
                                                perftest_flat::Lookup::Absent => {
                                                    (
                                                        0i32,
                                                        0,
                                                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                                    )
                                                }
                                                perftest_flat::Lookup::Found(__f0) => {
                                                    let a0_archived_outcome_found_v0 = ::prebindgen_jni_runtime::new_handle(
                                                        __f0,
                                                    );
                                                    (
                                                        1i32,
                                                        a0_archived_outcome_found_v0,
                                                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                                    )
                                                }
                                                perftest_flat::Lookup::Failed(__f0) => {
                                                    let a0_archived_outcome_failed_v0 = ::prebindgen_jni_runtime::new_string(
                                                        env,
                                                        ::core::convert::AsRef::<str>::as_ref(&__f0),
                                                    )?;
                                                    (2i32, 0, a0_archived_outcome_failed_v0)
                                                }
                                            };
                                            let a0_archived_label = ::prebindgen_jni_runtime::new_string(
                                                env,
                                                ::core::convert::AsRef::<str>::as_ref(&__f4_4),
                                            )?;
                                            (
                                                a0_archived_summary_count,
                                                a0_archived_summary_total,
                                                a0_archived_taken__present,
                                                a0_archived_taken_secs,
                                                a0_archived_taken_nanos,
                                                a0_archived_origin_secs,
                                                a0_archived_origin_nanos,
                                                a0_archived_outcome__tag,
                                                a0_archived_outcome_found_v0,
                                                a0_archived_outcome_failed_v0,
                                                a0_archived_label,
                                            )
                                        };
                                        (
                                            a0_archived_summary_count,
                                            a0_archived_summary_total,
                                            a0_archived_taken__present,
                                            a0_archived_taken_secs,
                                            a0_archived_taken_nanos,
                                            a0_archived_origin_secs,
                                            a0_archived_origin_nanos,
                                            a0_archived_outcome__tag,
                                            a0_archived_outcome_found_v0,
                                            a0_archived_outcome_failed_v0,
                                            a0_archived_label,
                                        )
                                    };
                                    (
                                        1u8,
                                        a0_archived_summary_count,
                                        a0_archived_summary_total,
                                        a0_archived_taken__present,
                                        a0_archived_taken_secs,
                                        a0_archived_taken_nanos,
                                        a0_archived_origin_secs,
                                        a0_archived_origin_nanos,
                                        a0_archived_outcome__tag,
                                        a0_archived_outcome_found_v0,
                                        a0_archived_outcome_failed_v0,
                                        a0_archived_label,
                                    )
                                }
                                ::core::option::Option::None => {
                                    (
                                        0u8,
                                        0,
                                        0.0f64,
                                        0,
                                        0,
                                        0,
                                        0,
                                        0,
                                        0,
                                        0,
                                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                        ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                    )
                                }
                            };
                            (
                                a0_filed__present,
                                a0_filed_summary_count,
                                a0_filed_summary_total,
                                a0_filed_taken__present,
                                a0_filed_taken_secs,
                                a0_filed_taken_nanos,
                                a0_filed_origin_secs,
                                a0_filed_origin_nanos,
                                a0_filed_outcome__tag,
                                a0_filed_outcome_found_v0,
                                a0_filed_outcome_failed_v0,
                                a0_filed_label,
                                a0_archived__present,
                                a0_archived_summary_count,
                                a0_archived_summary_total,
                                a0_archived_taken__present,
                                a0_archived_taken_secs,
                                a0_archived_taken_nanos,
                                a0_archived_origin_secs,
                                a0_archived_origin_nanos,
                                a0_archived_outcome__tag,
                                a0_archived_outcome_found_v0,
                                a0_archived_outcome_failed_v0,
                                a0_archived_label,
                            )
                        };
                        ::core::result::Result::Ok(
                            ::std::vec![
                                ::prebindgen_jni_runtime::jni::sys::jvalue { z :
                                a0_filed__present },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_filed_summary_count },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { d :
                                a0_filed_summary_total },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { z :
                                a0_filed_taken__present },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_filed_taken_secs },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_filed_taken_nanos },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_filed_origin_secs },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_filed_origin_nanos },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { i :
                                a0_filed_outcome__tag },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_filed_outcome_found_v0 },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l :
                                a0_filed_outcome_failed_v0.as_raw() },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l :
                                a0_filed_label.as_raw() },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { z :
                                a0_archived__present },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_archived_summary_count },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { d :
                                a0_archived_summary_total },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { z :
                                a0_archived_taken__present },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_archived_taken_secs },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_archived_taken_nanos },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_archived_origin_secs },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_archived_origin_nanos },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { i :
                                a0_archived_outcome__tag },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j :
                                a0_archived_outcome_found_v0 },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l :
                                a0_archived_outcome_failed_v0.as_raw() },
                                ::prebindgen_jni_runtime::jni::sys::jvalue { l :
                                a0_archived_label.as_raw() }
                            ],
                        )
                    })?;
                    ::core::result::Result::Ok(())
                })();
                if let ::core::result::Result::Err(__err) = __res {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (Ledger) + Send + Sync + 'static",
                        &__err,
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
    let __result = perftest_flat::ledger_each(n, sink);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_spanHolderNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    seq: ::prebindgen_jni_runtime::jni::sys::jlong,
    required_ms: ::prebindgen_jni_runtime::jni::sys::jlong,
    delay_ms: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let seq = seq;
    let required_ms = (required_ms as u64);
    let delay_ms = delay_ms;
    let __result = perftest_flat::span_holder_new(seq, required_ms, delay_ms);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let __x = __result;
        {
            let (r_span__present, r_span_required, r_span_delay) = {
                let __v1 = __x;
                let (r_span__present, r_span_required, r_span_delay) = match perftest_flat::span_holder_span(
                    &__v1,
                ) {
                    ::core::option::Option::Some(__x2) => {
                        let (r_span_required, r_span_delay) = {
                            let __v3 = __x2;
                            let __s3 = perftest_flat::span_to_struct(__v3);
                            let (r_span_required, r_span_delay) = {
                                let perftest_flat::SpanStruct {
                                    required: __f4_0,
                                    delay: __f4_1,
                                } = __s3;
                                let r_span_required = {
                                    let __r5 = crate::duration_to_millis(__f4_0)
                                        .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
                                    let __r5 = ::prebindgen_jni_runtime::check_domain(
                                        __r5,
                                        0u128,
                                        86400000u128,
                                        "Duration",
                                    )?;
                                    (__r5 as ::prebindgen_jni_runtime::jni::sys::jlong)
                                };
                                let r_span_delay = match __f4_1 {
                                    ::core::option::Option::Some(__x5) => {
                                        ::prebindgen_jni_runtime::box_jlong(
                                            env,
                                            {
                                                let __r6 = crate::duration_to_millis(__x5)
                                                    .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
                                                let __r6 = ::prebindgen_jni_runtime::check_domain(
                                                    __r6,
                                                    0u128,
                                                    86400000u128,
                                                    "Duration",
                                                )?;
                                                (__r6 as ::prebindgen_jni_runtime::jni::sys::jlong)
                                            },
                                        )?
                                    }
                                    ::core::option::Option::None => {
                                        ::prebindgen_jni_runtime::jni::objects::JObject::null()
                                    }
                                };
                                (r_span_required, r_span_delay)
                            };
                            (r_span_required, r_span_delay)
                        };
                        (1u8, r_span_required, r_span_delay)
                    }
                    ::core::option::Option::None => {
                        (0u8, 0, ::prebindgen_jni_runtime::jni::objects::JObject::null())
                    }
                };
                (r_span__present, r_span_required, r_span_delay)
            };
            static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
            let __o = __S
                .call_object(
                    env,
                    "io/prebindgen/covertest/model/SpanHolderBuilderRaw",
                    "run",
                    "(ZJLjava/lang/Long;)Ljava/lang/Object;",
                    &__sink,
                    &[
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            z: r_span__present,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_span_required,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            l: r_span_delay.as_raw(),
                        },
                    ],
                )?;
            ::core::result::Result::Ok(__o.into_raw())
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_boxedNoteEcho<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    note: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let note = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::std::boxed::Box::new(
                if !note.is_null() {
                    ::core::option::Option::Some(
                        ::prebindgen_jni_runtime::read_string(env, &note)?,
                    )
                } else {
                    ::core::option::Option::None
                },
            ),
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::boxed_note_echo(note);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match (*(*__result)) {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::new_string(
                    env,
                    ::core::convert::AsRef::<str>::as_ref(&__x1),
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_plainNoteEcho<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    note: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let note = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            if !note.is_null() {
                ::core::option::Option::Some(
                    ::prebindgen_jni_runtime::read_string(env, &note)?,
                )
            } else {
                ::core::option::Option::None
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::plain_note_echo(note);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match __result {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::new_string(
                    env,
                    ::core::convert::AsRef::<str>::as_ref(&__x1),
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_wrappedFieldsSum<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    w_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    w_boxed__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    w_boxed: ::prebindgen_jni_runtime::jni::sys::jlong,
    w_plain__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    w_plain: ::prebindgen_jni_runtime::jni::sys::jlong,
    w_boxed_enum: ::prebindgen_jni_runtime::jni::sys::jint,
    w_plain_enum: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let w = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::WrappedFields {
            id: w_id,
            boxed: ::std::boxed::Box::new(
                if (w_boxed__present != 0) {
                    ::core::option::Option::Some(w_boxed)
                } else {
                    ::core::option::Option::None
                },
            ),
            plain: if (w_plain__present != 0) {
                ::core::option::Option::Some(w_plain)
            } else {
                ::core::option::Option::None
            },
            boxed_enum: ::std::boxed::Box::new(
                (match w_boxed_enum {
                    0 => ::core::result::Result::Ok(perftest_flat::Priority::Low),
                    1 => ::core::result::Result::Ok(perftest_flat::Priority::Normal),
                    2 => ::core::result::Result::Ok(perftest_flat::Priority::High),
                    __v => {
                        ::core::result::Result::Err(
                            ::std::format!("invalid value {} for enum `Priority`", __v),
                        )
                    }
                })?,
            ),
            plain_enum: (match w_plain_enum {
                0 => ::core::result::Result::Ok(perftest_flat::Priority::Low),
                1 => ::core::result::Result::Ok(perftest_flat::Priority::Normal),
                2 => ::core::result::Result::Ok(perftest_flat::Priority::High),
                __v => {
                    ::core::result::Result::Err(
                        ::std::format!("invalid value {} for enum `Priority`", __v),
                    )
                }
            })?,
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::wrapped_fields_sum(w);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_holderTagOr<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    h__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    h_tag: ::prebindgen_jni_runtime::jni::sys::jlong,
    h_summary: ::prebindgen_jni_runtime::jni::sys::jlong,
    fallback: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let h = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            if (h__present != 0) {
                ::core::option::Option::Some(perftest_flat::Holder {
                    tag: h_tag,
                    summary: ::prebindgen_jni_runtime::take_handle::<
                        perftest_flat::Summary,
                    >(h_summary)?,
                })
            } else {
                ::core::option::Option::None
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let fallback = fallback;
    let __result = perftest_flat::holder_tag_or(h, fallback);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_boxedPayloadId<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    p_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    p_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    p_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    p_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    p_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let p = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::std::boxed::Box::new(perftest_flat::Payload {
                id: p_id,
                seq: p_seq,
                value: p_value,
                flag: (p_flag != 0),
                label: if !p_label.is_null() {
                    ::core::option::Option::Some(
                        ::std::boxed::Box::new(
                            ::prebindgen_jni_runtime::read_string(env, &p_label)?,
                        ),
                    )
                } else {
                    ::core::option::Option::None
                },
            }),
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::boxed_payload_id(p);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_boxedOptPayloadId<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    p__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    p_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    p_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    p_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    p_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    p_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let p = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::std::boxed::Box::new(
                if (p__present != 0) {
                    ::core::option::Option::Some(perftest_flat::Payload {
                        id: p_id,
                        seq: p_seq,
                        value: p_value,
                        flag: (p_flag != 0),
                        label: if !p_label.is_null() {
                            ::core::option::Option::Some(
                                ::std::boxed::Box::new(
                                    ::prebindgen_jni_runtime::read_string(env, &p_label)?,
                                ),
                            )
                        } else {
                            ::core::option::Option::None
                        },
                    })
                } else {
                    ::core::option::Option::None
                },
            ),
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::boxed_opt_payload_id(p);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_boxedOptPriorityWeight<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    p__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    p: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let p = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::std::boxed::Box::new(
                if (p__present != 0) {
                    ::core::option::Option::Some(
                        (match p {
                            0 => ::core::result::Result::Ok(perftest_flat::Priority::Low),
                            1 => {
                                ::core::result::Result::Ok(perftest_flat::Priority::Normal)
                            }
                            2 => {
                                ::core::result::Result::Ok(perftest_flat::Priority::High)
                            }
                            __v => {
                                ::core::result::Result::Err(
                                    ::std::format!("invalid value {} for enum `Priority`", __v),
                                )
                            }
                        })?,
                    )
                } else {
                    ::core::option::Option::None
                },
            ),
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::boxed_opt_priority_weight(p);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_boxedElemIdSum<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    ps__n: ::prebindgen_jni_runtime::jni::sys::jint,
    ps_id: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_seq: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_value: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_flag: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let ps = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __n = ps__n as usize;
            let __c0_0 = ::prebindgen_jni_runtime::read_longs(env, &ps_id)?;
            let __c0_1 = ::prebindgen_jni_runtime::read_ints(env, &ps_seq)?;
            let __c0_2 = ::prebindgen_jni_runtime::read_doubles(env, &ps_value)?;
            let __c0_3 = ::prebindgen_jni_runtime::read_booleans(env, &ps_flag)?;
            let mut __v = ::std::vec::Vec::with_capacity(__n);
            for __i in 0..__n {
                let __e0_id = __c0_0[__i];
                let __e0_seq = __c0_1[__i];
                let __e0_value = __c0_2[__i];
                let __e0_flag = __c0_3[__i];
                let __e0_label = ::prebindgen_jni_runtime::object_array_get(
                    env,
                    &ps_label,
                    __i,
                )?;
                let __x = (|| -> ::core::result::Result<_, ::std::string::String> {
                    ::core::result::Result::Ok(
                        ::std::boxed::Box::new(perftest_flat::Payload {
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
                        }),
                    )
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
            return 0;
        }
    };
    let __result = perftest_flat::boxed_elem_id_sum(ps);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_boxedRunIdSum<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    ps__n: ::prebindgen_jni_runtime::jni::sys::jint,
    ps_id: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_seq: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_value: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_flag: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let ps = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::std::boxed::Box::new({
                let __n = ps__n as usize;
                let __c0_0 = ::prebindgen_jni_runtime::read_longs(env, &ps_id)?;
                let __c0_1 = ::prebindgen_jni_runtime::read_ints(env, &ps_seq)?;
                let __c0_2 = ::prebindgen_jni_runtime::read_doubles(env, &ps_value)?;
                let __c0_3 = ::prebindgen_jni_runtime::read_booleans(env, &ps_flag)?;
                let mut __v = ::std::vec::Vec::with_capacity(__n);
                for __i in 0..__n {
                    let __e0_id = __c0_0[__i];
                    let __e0_seq = __c0_1[__i];
                    let __e0_value = __c0_2[__i];
                    let __e0_flag = __c0_3[__i];
                    let __e0_label = ::prebindgen_jni_runtime::object_array_get(
                        env,
                        &ps_label,
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
            }),
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::boxed_run_id_sum(ps);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_sliceIdSum<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    ps__n: ::prebindgen_jni_runtime::jni::sys::jint,
    ps_id: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_seq: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_value: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_flag: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let ps = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __n = ps__n as usize;
            let __c0_0 = ::prebindgen_jni_runtime::read_longs(env, &ps_id)?;
            let __c0_1 = ::prebindgen_jni_runtime::read_ints(env, &ps_seq)?;
            let __c0_2 = ::prebindgen_jni_runtime::read_doubles(env, &ps_value)?;
            let __c0_3 = ::prebindgen_jni_runtime::read_booleans(env, &ps_flag)?;
            let mut __v = ::std::vec::Vec::with_capacity(__n);
            for __i in 0..__n {
                let __e0_id = __c0_0[__i];
                let __e0_seq = __c0_1[__i];
                let __e0_value = __c0_2[__i];
                let __e0_flag = __c0_3[__i];
                let __e0_label = ::prebindgen_jni_runtime::object_array_get(
                    env,
                    &ps_label,
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
            return 0;
        }
    };
    let __result = perftest_flat::slice_id_sum(&ps);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_refVecIdSum<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    ps__n: ::prebindgen_jni_runtime::jni::sys::jint,
    ps_id: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_seq: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_value: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_flag: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    ps_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let ps = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __n = ps__n as usize;
            let __c0_0 = ::prebindgen_jni_runtime::read_longs(env, &ps_id)?;
            let __c0_1 = ::prebindgen_jni_runtime::read_ints(env, &ps_seq)?;
            let __c0_2 = ::prebindgen_jni_runtime::read_doubles(env, &ps_value)?;
            let __c0_3 = ::prebindgen_jni_runtime::read_booleans(env, &ps_flag)?;
            let mut __v = ::std::vec::Vec::with_capacity(__n);
            for __i in 0..__n {
                let __e0_id = __c0_0[__i];
                let __e0_seq = __c0_1[__i];
                let __e0_value = __c0_2[__i];
                let __e0_flag = __c0_3[__i];
                let __e0_label = ::prebindgen_jni_runtime::object_array_get(
                    env,
                    &ps_label,
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
            return 0;
        }
    };
    let __result = perftest_flat::ref_vec_id_sum(&ps);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_boxedLatest<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Archive>(a)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::boxed_latest(a);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| match (*__result) {
        ::core::option::Option::Some(__some) => {
            let __x = __some;
            {
                let (r_count, r_total) = {
                    let __v1 = __x;
                    let r_count = (perftest_flat::summary_count(&__v1)
                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                    let r_total = (perftest_flat::summary_total(&__v1)
                        as ::prebindgen_jni_runtime::jni::sys::jdouble);
                    (r_count, r_total)
                };
                static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
                let __o = __S
                    .call_object(
                        env,
                        "io/prebindgen/covertest/analytics/SummaryBuilderRaw",
                        "run",
                        "(JD)Ljava/lang/Object;",
                        &__sink,
                        &[
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: r_count,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                d: r_total,
                            },
                        ],
                    )?;
                ::core::result::Result::Ok(__o.into_raw())
            }
        }
        ::core::option::Option::None => {
            ::core::result::Result::Ok(::core::ptr::null_mut())
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_ledgerNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    n: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let n = n;
    let __result = perftest_flat::ledger_new(n);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let __x = __result;
        {
            let (
                r_filed__present,
                r_filed_summary_count,
                r_filed_summary_total,
                r_filed_taken__present,
                r_filed_taken_secs,
                r_filed_taken_nanos,
                r_filed_origin_secs,
                r_filed_origin_nanos,
                r_filed_outcome__tag,
                r_filed_outcome_found_v0,
                r_filed_outcome_failed_v0,
                r_filed_label,
                r_archived__present,
                r_archived_summary_count,
                r_archived_summary_total,
                r_archived_taken__present,
                r_archived_taken_secs,
                r_archived_taken_nanos,
                r_archived_origin_secs,
                r_archived_origin_nanos,
                r_archived_outcome__tag,
                r_archived_outcome_found_v0,
                r_archived_outcome_failed_v0,
                r_archived_label,
            ) = {
                let __v1 = __x;
                let (
                    r_filed__present,
                    r_filed_summary_count,
                    r_filed_summary_total,
                    r_filed_taken__present,
                    r_filed_taken_secs,
                    r_filed_taken_nanos,
                    r_filed_origin_secs,
                    r_filed_origin_nanos,
                    r_filed_outcome__tag,
                    r_filed_outcome_found_v0,
                    r_filed_outcome_failed_v0,
                    r_filed_label,
                ) = match perftest_flat::ledger_filed(&__v1) {
                    ::core::option::Option::Some(__x2) => {
                        let (
                            r_filed_summary_count,
                            r_filed_summary_total,
                            r_filed_taken__present,
                            r_filed_taken_secs,
                            r_filed_taken_nanos,
                            r_filed_origin_secs,
                            r_filed_origin_nanos,
                            r_filed_outcome__tag,
                            r_filed_outcome_found_v0,
                            r_filed_outcome_failed_v0,
                            r_filed_label,
                        ) = {
                            let __v3 = __x2;
                            let __s3 = perftest_flat::report_into_struct(
                                ::core::clone::Clone::clone(__v3),
                            );
                            let (
                                r_filed_summary_count,
                                r_filed_summary_total,
                                r_filed_taken__present,
                                r_filed_taken_secs,
                                r_filed_taken_nanos,
                                r_filed_origin_secs,
                                r_filed_origin_nanos,
                                r_filed_outcome__tag,
                                r_filed_outcome_found_v0,
                                r_filed_outcome_failed_v0,
                                r_filed_label,
                            ) = {
                                let perftest_flat::ReportStruct {
                                    summary: __f4_0,
                                    taken: __f4_1,
                                    origin: __f4_2,
                                    outcome: __f4_3,
                                    label: __f4_4,
                                } = __s3;
                                let (r_filed_summary_count, r_filed_summary_total) = {
                                    let __v5 = __f4_0;
                                    let r_filed_summary_count = (perftest_flat::summary_count(
                                        &__v5,
                                    ) as ::prebindgen_jni_runtime::jni::sys::jlong);
                                    let r_filed_summary_total = (perftest_flat::summary_total(
                                        &__v5,
                                    ) as ::prebindgen_jni_runtime::jni::sys::jdouble);
                                    (r_filed_summary_count, r_filed_summary_total)
                                };
                                let (
                                    r_filed_taken__present,
                                    r_filed_taken_secs,
                                    r_filed_taken_nanos,
                                ) = match __f4_1 {
                                    ::core::option::Option::Some(__x5) => {
                                        let (r_filed_taken_secs, r_filed_taken_nanos) = {
                                            let perftest_flat::Stamp { secs: __f0, nanos: __f1 } = __x5;
                                            let r_filed_taken_secs = (__f0
                                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                                            let r_filed_taken_nanos = (__f1
                                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                                            (r_filed_taken_secs, r_filed_taken_nanos)
                                        };
                                        (1u8, r_filed_taken_secs, r_filed_taken_nanos)
                                    }
                                    ::core::option::Option::None => (0u8, 0, 0),
                                };
                                let (r_filed_origin_secs, r_filed_origin_nanos) = {
                                    let perftest_flat::Stamp { secs: __f5_0, nanos: __f5_1 } = __f4_2;
                                    let r_filed_origin_secs = (__f5_0
                                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                                    let r_filed_origin_nanos = (__f5_1
                                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                                    (r_filed_origin_secs, r_filed_origin_nanos)
                                };
                                let (
                                    r_filed_outcome__tag,
                                    r_filed_outcome_found_v0,
                                    r_filed_outcome_failed_v0,
                                ) = match __f4_3 {
                                    perftest_flat::Lookup::Absent => {
                                        (
                                            0i32,
                                            0,
                                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                        )
                                    }
                                    perftest_flat::Lookup::Found(__f0) => {
                                        let r_filed_outcome_found_v0 = ::prebindgen_jni_runtime::new_handle(
                                            __f0,
                                        );
                                        (
                                            1i32,
                                            r_filed_outcome_found_v0,
                                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                        )
                                    }
                                    perftest_flat::Lookup::Failed(__f0) => {
                                        let r_filed_outcome_failed_v0 = ::prebindgen_jni_runtime::new_string(
                                            env,
                                            ::core::convert::AsRef::<str>::as_ref(&__f0),
                                        )?;
                                        (2i32, 0, r_filed_outcome_failed_v0)
                                    }
                                };
                                let r_filed_label = ::prebindgen_jni_runtime::new_string(
                                    env,
                                    ::core::convert::AsRef::<str>::as_ref(&__f4_4),
                                )?;
                                (
                                    r_filed_summary_count,
                                    r_filed_summary_total,
                                    r_filed_taken__present,
                                    r_filed_taken_secs,
                                    r_filed_taken_nanos,
                                    r_filed_origin_secs,
                                    r_filed_origin_nanos,
                                    r_filed_outcome__tag,
                                    r_filed_outcome_found_v0,
                                    r_filed_outcome_failed_v0,
                                    r_filed_label,
                                )
                            };
                            (
                                r_filed_summary_count,
                                r_filed_summary_total,
                                r_filed_taken__present,
                                r_filed_taken_secs,
                                r_filed_taken_nanos,
                                r_filed_origin_secs,
                                r_filed_origin_nanos,
                                r_filed_outcome__tag,
                                r_filed_outcome_found_v0,
                                r_filed_outcome_failed_v0,
                                r_filed_label,
                            )
                        };
                        (
                            1u8,
                            r_filed_summary_count,
                            r_filed_summary_total,
                            r_filed_taken__present,
                            r_filed_taken_secs,
                            r_filed_taken_nanos,
                            r_filed_origin_secs,
                            r_filed_origin_nanos,
                            r_filed_outcome__tag,
                            r_filed_outcome_found_v0,
                            r_filed_outcome_failed_v0,
                            r_filed_label,
                        )
                    }
                    ::core::option::Option::None => {
                        (
                            0u8,
                            0,
                            0.0f64,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                        )
                    }
                };
                let (
                    r_archived__present,
                    r_archived_summary_count,
                    r_archived_summary_total,
                    r_archived_taken__present,
                    r_archived_taken_secs,
                    r_archived_taken_nanos,
                    r_archived_origin_secs,
                    r_archived_origin_nanos,
                    r_archived_outcome__tag,
                    r_archived_outcome_found_v0,
                    r_archived_outcome_failed_v0,
                    r_archived_label,
                ) = match perftest_flat::ledger_archived(&__v1) {
                    ::core::option::Option::Some(__x2) => {
                        let (
                            r_archived_summary_count,
                            r_archived_summary_total,
                            r_archived_taken__present,
                            r_archived_taken_secs,
                            r_archived_taken_nanos,
                            r_archived_origin_secs,
                            r_archived_origin_nanos,
                            r_archived_outcome__tag,
                            r_archived_outcome_found_v0,
                            r_archived_outcome_failed_v0,
                            r_archived_label,
                        ) = {
                            let __v3 = __x2;
                            let __s3 = perftest_flat::report_into_struct(__v3);
                            let (
                                r_archived_summary_count,
                                r_archived_summary_total,
                                r_archived_taken__present,
                                r_archived_taken_secs,
                                r_archived_taken_nanos,
                                r_archived_origin_secs,
                                r_archived_origin_nanos,
                                r_archived_outcome__tag,
                                r_archived_outcome_found_v0,
                                r_archived_outcome_failed_v0,
                                r_archived_label,
                            ) = {
                                let perftest_flat::ReportStruct {
                                    summary: __f4_0,
                                    taken: __f4_1,
                                    origin: __f4_2,
                                    outcome: __f4_3,
                                    label: __f4_4,
                                } = __s3;
                                let (r_archived_summary_count, r_archived_summary_total) = {
                                    let __v5 = __f4_0;
                                    let r_archived_summary_count = (perftest_flat::summary_count(
                                        &__v5,
                                    ) as ::prebindgen_jni_runtime::jni::sys::jlong);
                                    let r_archived_summary_total = (perftest_flat::summary_total(
                                        &__v5,
                                    ) as ::prebindgen_jni_runtime::jni::sys::jdouble);
                                    (r_archived_summary_count, r_archived_summary_total)
                                };
                                let (
                                    r_archived_taken__present,
                                    r_archived_taken_secs,
                                    r_archived_taken_nanos,
                                ) = match __f4_1 {
                                    ::core::option::Option::Some(__x5) => {
                                        let (r_archived_taken_secs, r_archived_taken_nanos) = {
                                            let perftest_flat::Stamp { secs: __f0, nanos: __f1 } = __x5;
                                            let r_archived_taken_secs = (__f0
                                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                                            let r_archived_taken_nanos = (__f1
                                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                                            (r_archived_taken_secs, r_archived_taken_nanos)
                                        };
                                        (1u8, r_archived_taken_secs, r_archived_taken_nanos)
                                    }
                                    ::core::option::Option::None => (0u8, 0, 0),
                                };
                                let (r_archived_origin_secs, r_archived_origin_nanos) = {
                                    let perftest_flat::Stamp { secs: __f5_0, nanos: __f5_1 } = __f4_2;
                                    let r_archived_origin_secs = (__f5_0
                                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                                    let r_archived_origin_nanos = (__f5_1
                                        as ::prebindgen_jni_runtime::jni::sys::jlong);
                                    (r_archived_origin_secs, r_archived_origin_nanos)
                                };
                                let (
                                    r_archived_outcome__tag,
                                    r_archived_outcome_found_v0,
                                    r_archived_outcome_failed_v0,
                                ) = match __f4_3 {
                                    perftest_flat::Lookup::Absent => {
                                        (
                                            0i32,
                                            0,
                                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                        )
                                    }
                                    perftest_flat::Lookup::Found(__f0) => {
                                        let r_archived_outcome_found_v0 = ::prebindgen_jni_runtime::new_handle(
                                            __f0,
                                        );
                                        (
                                            1i32,
                                            r_archived_outcome_found_v0,
                                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                                        )
                                    }
                                    perftest_flat::Lookup::Failed(__f0) => {
                                        let r_archived_outcome_failed_v0 = ::prebindgen_jni_runtime::new_string(
                                            env,
                                            ::core::convert::AsRef::<str>::as_ref(&__f0),
                                        )?;
                                        (2i32, 0, r_archived_outcome_failed_v0)
                                    }
                                };
                                let r_archived_label = ::prebindgen_jni_runtime::new_string(
                                    env,
                                    ::core::convert::AsRef::<str>::as_ref(&__f4_4),
                                )?;
                                (
                                    r_archived_summary_count,
                                    r_archived_summary_total,
                                    r_archived_taken__present,
                                    r_archived_taken_secs,
                                    r_archived_taken_nanos,
                                    r_archived_origin_secs,
                                    r_archived_origin_nanos,
                                    r_archived_outcome__tag,
                                    r_archived_outcome_found_v0,
                                    r_archived_outcome_failed_v0,
                                    r_archived_label,
                                )
                            };
                            (
                                r_archived_summary_count,
                                r_archived_summary_total,
                                r_archived_taken__present,
                                r_archived_taken_secs,
                                r_archived_taken_nanos,
                                r_archived_origin_secs,
                                r_archived_origin_nanos,
                                r_archived_outcome__tag,
                                r_archived_outcome_found_v0,
                                r_archived_outcome_failed_v0,
                                r_archived_label,
                            )
                        };
                        (
                            1u8,
                            r_archived_summary_count,
                            r_archived_summary_total,
                            r_archived_taken__present,
                            r_archived_taken_secs,
                            r_archived_taken_nanos,
                            r_archived_origin_secs,
                            r_archived_origin_nanos,
                            r_archived_outcome__tag,
                            r_archived_outcome_found_v0,
                            r_archived_outcome_failed_v0,
                            r_archived_label,
                        )
                    }
                    ::core::option::Option::None => {
                        (
                            0u8,
                            0,
                            0.0f64,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            0,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                        )
                    }
                };
                (
                    r_filed__present,
                    r_filed_summary_count,
                    r_filed_summary_total,
                    r_filed_taken__present,
                    r_filed_taken_secs,
                    r_filed_taken_nanos,
                    r_filed_origin_secs,
                    r_filed_origin_nanos,
                    r_filed_outcome__tag,
                    r_filed_outcome_found_v0,
                    r_filed_outcome_failed_v0,
                    r_filed_label,
                    r_archived__present,
                    r_archived_summary_count,
                    r_archived_summary_total,
                    r_archived_taken__present,
                    r_archived_taken_secs,
                    r_archived_taken_nanos,
                    r_archived_origin_secs,
                    r_archived_origin_nanos,
                    r_archived_outcome__tag,
                    r_archived_outcome_found_v0,
                    r_archived_outcome_failed_v0,
                    r_archived_label,
                )
            };
            static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
            let __o = __S
                .call_object(
                    env,
                    "io/prebindgen/covertest/LedgerBuilderRaw",
                    "run",
                    "(ZJDZJJJJIJLjava/lang/String;Ljava/lang/String;ZJDZJJJJIJLjava/lang/String;Ljava/lang/String;)Ljava/lang/Object;",
                    &__sink,
                    &[
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            z: r_filed__present,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_filed_summary_count,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            d: r_filed_summary_total,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            z: r_filed_taken__present,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_filed_taken_secs,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_filed_taken_nanos,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_filed_origin_secs,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_filed_origin_nanos,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            i: r_filed_outcome__tag,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_filed_outcome_found_v0,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            l: r_filed_outcome_failed_v0.as_raw(),
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            l: r_filed_label.as_raw(),
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            z: r_archived__present,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_archived_summary_count,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            d: r_archived_summary_total,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            z: r_archived_taken__present,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_archived_taken_secs,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_archived_taken_nanos,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_archived_origin_secs,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_archived_origin_nanos,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            i: r_archived_outcome__tag,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_archived_outcome_found_v0,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            l: r_archived_outcome_failed_v0.as_raw(),
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            l: r_archived_label.as_raw(),
                        },
                    ],
                )?;
            ::core::result::Result::Ok(__o.into_raw())
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_archiveSetReading<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a: ::prebindgen_jni_runtime::jni::sys::jlong,
    which: ::prebindgen_jni_runtime::jni::sys::jint,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle_mut::<perftest_flat::Archive>(a)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let which = which;
    let __result = perftest_flat::archive_set_reading(a, which);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_archiveReading<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Archive>(a)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::archive_reading(a);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (
            r__tag,
            r_exact_v0,
            r_range_low,
            r_range_high,
            r_labeled_v0,
            r_labeled_v1,
            r_companion_v0,
        ) = match ::core::clone::Clone::clone(__result) {
            perftest_flat::Reading::Missing => {
                (
                    0i32,
                    0,
                    0,
                    0,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    0,
                    0,
                )
            }
            perftest_flat::Reading::Exact(__f0) => {
                let r_exact_v0 = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                (
                    1i32,
                    r_exact_v0,
                    0,
                    0,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    0,
                    0,
                )
            }
            perftest_flat::Reading::Range { low: __f0, high: __f1 } => {
                let r_range_low = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                let r_range_high = (__f1 as ::prebindgen_jni_runtime::jni::sys::jlong);
                (
                    2i32,
                    0,
                    r_range_low,
                    r_range_high,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    0,
                    0,
                )
            }
            perftest_flat::Reading::Labeled(__f0, __f1) => {
                let r_labeled_v0 = ::prebindgen_jni_runtime::new_string(
                    env,
                    ::core::convert::AsRef::<str>::as_ref(&__f0),
                )?;
                let r_labeled_v1 = (match __f1 {
                    perftest_flat::Priority::Low => 0,
                    perftest_flat::Priority::Normal => 1,
                    perftest_flat::Priority::High => 2,
                } as i32);
                (3i32, 0, 0, 0, r_labeled_v0, r_labeled_v1, 0)
            }
            perftest_flat::Reading::Companion(__f0) => {
                let r_companion_v0 = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                (
                    4i32,
                    0,
                    0,
                    0,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    0,
                    r_companion_v0,
                )
            }
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_ref_Reading",
                "run",
                "(IJJJLjava/lang/String;IJ)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_exact_v0,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_range_low,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_range_high,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_labeled_v0.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_labeled_v1,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_companion_v0,
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_archiveReadingMaybe<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Archive>(a)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::archive_reading_maybe(a);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (
            r__present,
            r__tag,
            r_exact_v0,
            r_range_low,
            r_range_high,
            r_labeled_v0,
            r_labeled_v1,
            r_companion_v0,
        ) = match __result {
            ::core::option::Option::Some(__x1) => {
                let (
                    r__tag,
                    r_exact_v0,
                    r_range_low,
                    r_range_high,
                    r_labeled_v0,
                    r_labeled_v1,
                    r_companion_v0,
                ) = match ::core::clone::Clone::clone(__x1) {
                    perftest_flat::Reading::Missing => {
                        (
                            0i32,
                            0,
                            0,
                            0,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            0,
                            0,
                        )
                    }
                    perftest_flat::Reading::Exact(__f0) => {
                        let r_exact_v0 = (__f0
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        (
                            1i32,
                            r_exact_v0,
                            0,
                            0,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            0,
                            0,
                        )
                    }
                    perftest_flat::Reading::Range { low: __f0, high: __f1 } => {
                        let r_range_low = (__f0
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        let r_range_high = (__f1
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        (
                            2i32,
                            0,
                            r_range_low,
                            r_range_high,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            0,
                            0,
                        )
                    }
                    perftest_flat::Reading::Labeled(__f0, __f1) => {
                        let r_labeled_v0 = ::prebindgen_jni_runtime::new_string(
                            env,
                            ::core::convert::AsRef::<str>::as_ref(&__f0),
                        )?;
                        let r_labeled_v1 = (match __f1 {
                            perftest_flat::Priority::Low => 0,
                            perftest_flat::Priority::Normal => 1,
                            perftest_flat::Priority::High => 2,
                        } as i32);
                        (3i32, 0, 0, 0, r_labeled_v0, r_labeled_v1, 0)
                    }
                    perftest_flat::Reading::Companion(__f0) => {
                        let r_companion_v0 = (__f0
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        (
                            4i32,
                            0,
                            0,
                            0,
                            ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                            0,
                            r_companion_v0,
                        )
                    }
                };
                (
                    1u8,
                    r__tag,
                    r_exact_v0,
                    r_range_low,
                    r_range_high,
                    r_labeled_v0,
                    r_labeled_v1,
                    r_companion_v0,
                )
            }
            ::core::option::Option::None => {
                (
                    0u8,
                    0,
                    0,
                    0,
                    0,
                    ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                    0,
                    0,
                )
            }
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Option_ref_Reading",
                "run",
                "(ZIJJJLjava/lang/String;IJ)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        z: r__present,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_exact_v0,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_range_low,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_range_high,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_labeled_v0.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_labeled_v1,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_companion_v0,
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_holdEcho<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    h__tag: ::prebindgen_jni_runtime::jni::sys::jint,
    h_for_v0: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let h = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            (match h__tag {
                0i32 => ::core::result::Result::Ok(perftest_flat::Hold::Indefinite),
                1i32 => {
                    ::core::result::Result::Ok(
                        perftest_flat::Hold::For({
                            let __r0 = (h_for_v0 as u64);
                            let __r0 = ::prebindgen_jni_runtime::check_domain(
                                __r0,
                                0u128,
                                86400000u128,
                                "Duration",
                            )?;
                            crate::duration_from_millis(__r0)
                        }),
                    )
                }
                __t => {
                    ::core::result::Result::Err(
                        ::std::format!("Hold: invalid tag {}", __t),
                    )
                }
            })?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::hold_echo(h);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r__tag, r_for_v0) = match __result {
            perftest_flat::Hold::Indefinite => (0i32, 0),
            perftest_flat::Hold::For(__f0) => {
                let r_for_v0 = {
                    let __r2 = crate::duration_to_millis(__f0)
                        .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
                    let __r2 = ::prebindgen_jni_runtime::check_domain(
                        __r2,
                        0u128,
                        86400000u128,
                        "Duration",
                    )?;
                    (__r2 as ::prebindgen_jni_runtime::jni::sys::jlong)
                };
                (1i32, r_for_v0)
            }
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Hold",
                "run",
                "(IJ)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_for_v0,
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_holdPolicyEcho<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    p_hold__tag: ::prebindgen_jni_runtime::jni::sys::jint,
    p_hold_for_v0: ::prebindgen_jni_runtime::jni::sys::jlong,
    p_grace__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    p_grace__tag: ::prebindgen_jni_runtime::jni::sys::jint,
    p_grace_for_v0: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let p = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::HoldPolicy {
            hold: (match p_hold__tag {
                0i32 => ::core::result::Result::Ok(perftest_flat::Hold::Indefinite),
                1i32 => {
                    ::core::result::Result::Ok(
                        perftest_flat::Hold::For({
                            let __r0 = (p_hold_for_v0 as u64);
                            let __r0 = ::prebindgen_jni_runtime::check_domain(
                                __r0,
                                0u128,
                                86400000u128,
                                "Duration",
                            )?;
                            crate::duration_from_millis(__r0)
                        }),
                    )
                }
                __t => {
                    ::core::result::Result::Err(
                        ::std::format!("Hold: invalid tag {}", __t),
                    )
                }
            })?,
            grace: if (p_grace__present != 0) {
                ::core::option::Option::Some(
                    (match p_grace__tag {
                        0i32 => {
                            ::core::result::Result::Ok(perftest_flat::Hold::Indefinite)
                        }
                        1i32 => {
                            ::core::result::Result::Ok(
                                perftest_flat::Hold::For({
                                    let __r0 = (p_grace_for_v0 as u64);
                                    let __r0 = ::prebindgen_jni_runtime::check_domain(
                                        __r0,
                                        0u128,
                                        86400000u128,
                                        "Duration",
                                    )?;
                                    crate::duration_from_millis(__r0)
                                }),
                            )
                        }
                        __t => {
                            ::core::result::Result::Err(
                                ::std::format!("Hold: invalid tag {}", __t),
                            )
                        }
                    })?,
                )
            } else {
                ::core::option::Option::None
            },
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::hold_policy_echo(p);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (
            r_hold__tag,
            r_hold_for_v0,
            r_grace__present,
            r_grace__tag,
            r_grace_for_v0,
        ) = {
            let perftest_flat::HoldPolicy { hold: __f0, grace: __f1 } = __result;
            let (r_hold__tag, r_hold_for_v0) = match __f0 {
                perftest_flat::Hold::Indefinite => (0i32, 0),
                perftest_flat::Hold::For(__f0) => {
                    let r_hold_for_v0 = {
                        let __r3 = crate::duration_to_millis(__f0)
                            .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
                        let __r3 = ::prebindgen_jni_runtime::check_domain(
                            __r3,
                            0u128,
                            86400000u128,
                            "Duration",
                        )?;
                        (__r3 as ::prebindgen_jni_runtime::jni::sys::jlong)
                    };
                    (1i32, r_hold_for_v0)
                }
            };
            let (r_grace__present, r_grace__tag, r_grace_for_v0) = match __f1 {
                ::core::option::Option::Some(__x2) => {
                    let (r_grace__tag, r_grace_for_v0) = match __x2 {
                        perftest_flat::Hold::Indefinite => (0i32, 0),
                        perftest_flat::Hold::For(__f0) => {
                            let r_grace_for_v0 = {
                                let __r4 = crate::duration_to_millis(__f0)
                                    .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
                                let __r4 = ::prebindgen_jni_runtime::check_domain(
                                    __r4,
                                    0u128,
                                    86400000u128,
                                    "Duration",
                                )?;
                                (__r4 as ::prebindgen_jni_runtime::jni::sys::jlong)
                            };
                            (1i32, r_grace_for_v0)
                        }
                    };
                    (1u8, r_grace__tag, r_grace_for_v0)
                }
                ::core::option::Option::None => (0u8, 0, 0),
            };
            (r_hold__tag, r_hold_for_v0, r_grace__present, r_grace__tag, r_grace_for_v0)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_HoldPolicy",
                "run",
                "(IJZIJ)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_hold__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_hold_for_v0,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        z: r_grace__present,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_grace__tag,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_grace_for_v0,
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_cacheConfigWeight<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    cache__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    cache_replies_priority: ::prebindgen_jni_runtime::jni::sys::jint,
    cache_replies_max_samples: ::prebindgen_jni_runtime::jni::sys::jlong,
    cache_ttl: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jint {
    let mut __env = __env;
    let env = &mut __env;
    let cache = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            if (cache__present != 0) {
                ::core::option::Option::Some(perftest_flat::CacheConfig {
                    replies: perftest_flat::RepliesConfig {
                        priority: (match cache_replies_priority {
                            0 => ::core::result::Result::Ok(perftest_flat::Priority::Low),
                            1 => {
                                ::core::result::Result::Ok(perftest_flat::Priority::Normal)
                            }
                            2 => {
                                ::core::result::Result::Ok(perftest_flat::Priority::High)
                            }
                            __v => {
                                ::core::result::Result::Err(
                                    ::std::format!("invalid value {} for enum `Priority`", __v),
                                )
                            }
                        })?,
                        max_samples: cache_replies_max_samples,
                    },
                    ttl: cache_ttl,
                })
            } else {
                ::core::option::Option::None
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::cache_config_weight(cache);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jint,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jint);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_objectBoundaryValue<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value__packJ: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let value = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __value__packJ = ::prebindgen_jni_runtime::read_longs(
                env,
                &value__packJ,
            )?;
            ::prebindgen_jni_runtime::check_packed(
                __value__packJ.len(),
                127usize,
                "value__packJ",
            )?;
            let value_left_left_left_left_left_left_left_value = __value__packJ[0usize];
            let value_left_left_left_left_left_left_right_value = __value__packJ[1usize];
            let value_left_left_left_left_left_right_left_value = __value__packJ[2usize];
            let value_left_left_left_left_left_right_right_value = __value__packJ[3usize];
            let value_left_left_left_left_right_left_left_value = __value__packJ[4usize];
            let value_left_left_left_left_right_left_right_value = __value__packJ[5usize];
            let value_left_left_left_left_right_right_left_value = __value__packJ[6usize];
            let value_left_left_left_left_right_right_right_value = __value__packJ[7usize];
            let value_left_left_left_right_left_left_left_value = __value__packJ[8usize];
            let value_left_left_left_right_left_left_right_value = __value__packJ[9usize];
            let value_left_left_left_right_left_right_left_value = __value__packJ[10usize];
            let value_left_left_left_right_left_right_right_value = __value__packJ[11usize];
            let value_left_left_left_right_right_left_left_value = __value__packJ[12usize];
            let value_left_left_left_right_right_left_right_value = __value__packJ[13usize];
            let value_left_left_left_right_right_right_left_value = __value__packJ[14usize];
            let value_left_left_left_right_right_right_right_value = __value__packJ[15usize];
            let value_left_left_right_left_left_left_left_value = __value__packJ[16usize];
            let value_left_left_right_left_left_left_right_value = __value__packJ[17usize];
            let value_left_left_right_left_left_right_left_value = __value__packJ[18usize];
            let value_left_left_right_left_left_right_right_value = __value__packJ[19usize];
            let value_left_left_right_left_right_left_left_value = __value__packJ[20usize];
            let value_left_left_right_left_right_left_right_value = __value__packJ[21usize];
            let value_left_left_right_left_right_right_left_value = __value__packJ[22usize];
            let value_left_left_right_left_right_right_right_value = __value__packJ[23usize];
            let value_left_left_right_right_left_left_left_value = __value__packJ[24usize];
            let value_left_left_right_right_left_left_right_value = __value__packJ[25usize];
            let value_left_left_right_right_left_right_left_value = __value__packJ[26usize];
            let value_left_left_right_right_left_right_right_value = __value__packJ[27usize];
            let value_left_left_right_right_right_left_left_value = __value__packJ[28usize];
            let value_left_left_right_right_right_left_right_value = __value__packJ[29usize];
            let value_left_left_right_right_right_right_left_value = __value__packJ[30usize];
            let value_left_left_right_right_right_right_right_value = __value__packJ[31usize];
            let value_left_right_left_left_left_left_left_value = __value__packJ[32usize];
            let value_left_right_left_left_left_left_right_value = __value__packJ[33usize];
            let value_left_right_left_left_left_right_left_value = __value__packJ[34usize];
            let value_left_right_left_left_left_right_right_value = __value__packJ[35usize];
            let value_left_right_left_left_right_left_left_value = __value__packJ[36usize];
            let value_left_right_left_left_right_left_right_value = __value__packJ[37usize];
            let value_left_right_left_left_right_right_left_value = __value__packJ[38usize];
            let value_left_right_left_left_right_right_right_value = __value__packJ[39usize];
            let value_left_right_left_right_left_left_left_value = __value__packJ[40usize];
            let value_left_right_left_right_left_left_right_value = __value__packJ[41usize];
            let value_left_right_left_right_left_right_left_value = __value__packJ[42usize];
            let value_left_right_left_right_left_right_right_value = __value__packJ[43usize];
            let value_left_right_left_right_right_left_left_value = __value__packJ[44usize];
            let value_left_right_left_right_right_left_right_value = __value__packJ[45usize];
            let value_left_right_left_right_right_right_left_value = __value__packJ[46usize];
            let value_left_right_left_right_right_right_right_value = __value__packJ[47usize];
            let value_left_right_right_left_left_left_left_value = __value__packJ[48usize];
            let value_left_right_right_left_left_left_right_value = __value__packJ[49usize];
            let value_left_right_right_left_left_right_left_value = __value__packJ[50usize];
            let value_left_right_right_left_left_right_right_value = __value__packJ[51usize];
            let value_left_right_right_left_right_left_left_value = __value__packJ[52usize];
            let value_left_right_right_left_right_left_right_value = __value__packJ[53usize];
            let value_left_right_right_left_right_right_left_value = __value__packJ[54usize];
            let value_left_right_right_left_right_right_right_value = __value__packJ[55usize];
            let value_left_right_right_right_left_left_left_value = __value__packJ[56usize];
            let value_left_right_right_right_left_left_right_value = __value__packJ[57usize];
            let value_left_right_right_right_left_right_left_value = __value__packJ[58usize];
            let value_left_right_right_right_left_right_right_value = __value__packJ[59usize];
            let value_left_right_right_right_right_left_left_value = __value__packJ[60usize];
            let value_left_right_right_right_right_left_right_value = __value__packJ[61usize];
            let value_left_right_right_right_right_right_left_value = __value__packJ[62usize];
            let value_left_right_right_right_right_right_right_value = __value__packJ[63usize];
            let value_right_leaves32_left_left_left_left_left_value = __value__packJ[64usize];
            let value_right_leaves32_left_left_left_left_right_value = __value__packJ[65usize];
            let value_right_leaves32_left_left_left_right_left_value = __value__packJ[66usize];
            let value_right_leaves32_left_left_left_right_right_value = __value__packJ[67usize];
            let value_right_leaves32_left_left_right_left_left_value = __value__packJ[68usize];
            let value_right_leaves32_left_left_right_left_right_value = __value__packJ[69usize];
            let value_right_leaves32_left_left_right_right_left_value = __value__packJ[70usize];
            let value_right_leaves32_left_left_right_right_right_value = __value__packJ[71usize];
            let value_right_leaves32_left_right_left_left_left_value = __value__packJ[72usize];
            let value_right_leaves32_left_right_left_left_right_value = __value__packJ[73usize];
            let value_right_leaves32_left_right_left_right_left_value = __value__packJ[74usize];
            let value_right_leaves32_left_right_left_right_right_value = __value__packJ[75usize];
            let value_right_leaves32_left_right_right_left_left_value = __value__packJ[76usize];
            let value_right_leaves32_left_right_right_left_right_value = __value__packJ[77usize];
            let value_right_leaves32_left_right_right_right_left_value = __value__packJ[78usize];
            let value_right_leaves32_left_right_right_right_right_value = __value__packJ[79usize];
            let value_right_leaves32_right_left_left_left_left_value = __value__packJ[80usize];
            let value_right_leaves32_right_left_left_left_right_value = __value__packJ[81usize];
            let value_right_leaves32_right_left_left_right_left_value = __value__packJ[82usize];
            let value_right_leaves32_right_left_left_right_right_value = __value__packJ[83usize];
            let value_right_leaves32_right_left_right_left_left_value = __value__packJ[84usize];
            let value_right_leaves32_right_left_right_left_right_value = __value__packJ[85usize];
            let value_right_leaves32_right_left_right_right_left_value = __value__packJ[86usize];
            let value_right_leaves32_right_left_right_right_right_value = __value__packJ[87usize];
            let value_right_leaves32_right_right_left_left_left_value = __value__packJ[88usize];
            let value_right_leaves32_right_right_left_left_right_value = __value__packJ[89usize];
            let value_right_leaves32_right_right_left_right_left_value = __value__packJ[90usize];
            let value_right_leaves32_right_right_left_right_right_value = __value__packJ[91usize];
            let value_right_leaves32_right_right_right_left_left_value = __value__packJ[92usize];
            let value_right_leaves32_right_right_right_left_right_value = __value__packJ[93usize];
            let value_right_leaves32_right_right_right_right_left_value = __value__packJ[94usize];
            let value_right_leaves32_right_right_right_right_right_value = __value__packJ[95usize];
            let value_right_leaves16_left_left_left_left_value = __value__packJ[96usize];
            let value_right_leaves16_left_left_left_right_value = __value__packJ[97usize];
            let value_right_leaves16_left_left_right_left_value = __value__packJ[98usize];
            let value_right_leaves16_left_left_right_right_value = __value__packJ[99usize];
            let value_right_leaves16_left_right_left_left_value = __value__packJ[100usize];
            let value_right_leaves16_left_right_left_right_value = __value__packJ[101usize];
            let value_right_leaves16_left_right_right_left_value = __value__packJ[102usize];
            let value_right_leaves16_left_right_right_right_value = __value__packJ[103usize];
            let value_right_leaves16_right_left_left_left_value = __value__packJ[104usize];
            let value_right_leaves16_right_left_left_right_value = __value__packJ[105usize];
            let value_right_leaves16_right_left_right_left_value = __value__packJ[106usize];
            let value_right_leaves16_right_left_right_right_value = __value__packJ[107usize];
            let value_right_leaves16_right_right_left_left_value = __value__packJ[108usize];
            let value_right_leaves16_right_right_left_right_value = __value__packJ[109usize];
            let value_right_leaves16_right_right_right_left_value = __value__packJ[110usize];
            let value_right_leaves16_right_right_right_right_value = __value__packJ[111usize];
            let value_right_leaves8_left_left_left_value = __value__packJ[112usize];
            let value_right_leaves8_left_left_right_value = __value__packJ[113usize];
            let value_right_leaves8_left_right_left_value = __value__packJ[114usize];
            let value_right_leaves8_left_right_right_value = __value__packJ[115usize];
            let value_right_leaves8_right_left_left_value = __value__packJ[116usize];
            let value_right_leaves8_right_left_right_value = __value__packJ[117usize];
            let value_right_leaves8_right_right_left_value = __value__packJ[118usize];
            let value_right_leaves8_right_right_right_value = __value__packJ[119usize];
            let value_right_leaves4_left_left_value = __value__packJ[120usize];
            let value_right_leaves4_left_right_value = __value__packJ[121usize];
            let value_right_leaves4_right_left_value = __value__packJ[122usize];
            let value_right_leaves4_right_right_value = __value__packJ[123usize];
            let value_right_leaves2_left_value = __value__packJ[124usize];
            let value_right_leaves2_right_value = __value__packJ[125usize];
            let value_right_leaf_value = __value__packJ[126usize];
            perftest_flat::ObjectBoundary {
                left: perftest_flat::ObjectBoundary64 {
                    left: perftest_flat::ObjectBoundary32 {
                        left: perftest_flat::ObjectBoundary16 {
                            left: perftest_flat::ObjectBoundary8 {
                                left: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_left_left_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_left_left_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_left_left_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_left_left_right_right_value,
                                        },
                                    },
                                },
                                right: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_left_right_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_left_right_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_left_right_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_left_right_right_right_value,
                                        },
                                    },
                                },
                            },
                            right: perftest_flat::ObjectBoundary8 {
                                left: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_right_left_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_right_left_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_right_left_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_right_left_right_right_value,
                                        },
                                    },
                                },
                                right: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_right_right_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_right_right_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_right_right_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_left_right_right_right_right_value,
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
                                            value: value_left_left_right_left_left_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_left_left_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_left_left_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_left_left_right_right_value,
                                        },
                                    },
                                },
                                right: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_left_right_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_left_right_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_left_right_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_left_right_right_right_value,
                                        },
                                    },
                                },
                            },
                            right: perftest_flat::ObjectBoundary8 {
                                left: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_right_left_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_right_left_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_right_left_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_right_left_right_right_value,
                                        },
                                    },
                                },
                                right: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_right_right_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_right_right_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_right_right_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_left_right_right_right_right_right_value,
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
                                            value: value_left_right_left_left_left_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_left_left_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_left_left_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_left_left_right_right_value,
                                        },
                                    },
                                },
                                right: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_left_right_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_left_right_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_left_right_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_left_right_right_right_value,
                                        },
                                    },
                                },
                            },
                            right: perftest_flat::ObjectBoundary8 {
                                left: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_right_left_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_right_left_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_right_left_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_right_left_right_right_value,
                                        },
                                    },
                                },
                                right: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_right_right_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_right_right_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_right_right_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_left_right_right_right_right_value,
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
                                            value: value_left_right_right_left_left_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_left_left_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_left_left_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_left_left_right_right_value,
                                        },
                                    },
                                },
                                right: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_left_right_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_left_right_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_left_right_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_left_right_right_right_value,
                                        },
                                    },
                                },
                            },
                            right: perftest_flat::ObjectBoundary8 {
                                left: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_right_left_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_right_left_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_right_left_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_right_left_right_right_value,
                                        },
                                    },
                                },
                                right: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_right_right_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_right_right_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_right_right_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_left_right_right_right_right_right_right_value,
                                        },
                                    },
                                },
                            },
                        },
                    },
                },
                right: perftest_flat::ObjectBoundary63 {
                    leaves32: perftest_flat::ObjectBoundary32 {
                        left: perftest_flat::ObjectBoundary16 {
                            left: perftest_flat::ObjectBoundary8 {
                                left: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_left_left_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_left_left_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_left_left_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_left_left_right_right_value,
                                        },
                                    },
                                },
                                right: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_left_right_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_left_right_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_left_right_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_left_right_right_right_value,
                                        },
                                    },
                                },
                            },
                            right: perftest_flat::ObjectBoundary8 {
                                left: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_right_left_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_right_left_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_right_left_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_right_left_right_right_value,
                                        },
                                    },
                                },
                                right: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_right_right_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_right_right_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_right_right_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_left_right_right_right_right_value,
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
                                            value: value_right_leaves32_right_left_left_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_left_left_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_left_left_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_left_left_right_right_value,
                                        },
                                    },
                                },
                                right: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_left_right_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_left_right_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_left_right_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_left_right_right_right_value,
                                        },
                                    },
                                },
                            },
                            right: perftest_flat::ObjectBoundary8 {
                                left: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_right_left_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_right_left_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_right_left_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_right_left_right_right_value,
                                        },
                                    },
                                },
                                right: perftest_flat::ObjectBoundary4 {
                                    left: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_right_right_left_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_right_right_left_right_value,
                                        },
                                    },
                                    right: perftest_flat::ObjectBoundary2 {
                                        left: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_right_right_right_left_value,
                                        },
                                        right: perftest_flat::ObjectBoundaryLeaf {
                                            value: value_right_leaves32_right_right_right_right_right_value,
                                        },
                                    },
                                },
                            },
                        },
                    },
                    leaves16: perftest_flat::ObjectBoundary16 {
                        left: perftest_flat::ObjectBoundary8 {
                            left: perftest_flat::ObjectBoundary4 {
                                left: perftest_flat::ObjectBoundary2 {
                                    left: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_left_left_left_left_value,
                                    },
                                    right: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_left_left_left_right_value,
                                    },
                                },
                                right: perftest_flat::ObjectBoundary2 {
                                    left: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_left_left_right_left_value,
                                    },
                                    right: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_left_left_right_right_value,
                                    },
                                },
                            },
                            right: perftest_flat::ObjectBoundary4 {
                                left: perftest_flat::ObjectBoundary2 {
                                    left: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_left_right_left_left_value,
                                    },
                                    right: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_left_right_left_right_value,
                                    },
                                },
                                right: perftest_flat::ObjectBoundary2 {
                                    left: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_left_right_right_left_value,
                                    },
                                    right: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_left_right_right_right_value,
                                    },
                                },
                            },
                        },
                        right: perftest_flat::ObjectBoundary8 {
                            left: perftest_flat::ObjectBoundary4 {
                                left: perftest_flat::ObjectBoundary2 {
                                    left: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_right_left_left_left_value,
                                    },
                                    right: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_right_left_left_right_value,
                                    },
                                },
                                right: perftest_flat::ObjectBoundary2 {
                                    left: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_right_left_right_left_value,
                                    },
                                    right: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_right_left_right_right_value,
                                    },
                                },
                            },
                            right: perftest_flat::ObjectBoundary4 {
                                left: perftest_flat::ObjectBoundary2 {
                                    left: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_right_right_left_left_value,
                                    },
                                    right: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_right_right_left_right_value,
                                    },
                                },
                                right: perftest_flat::ObjectBoundary2 {
                                    left: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_right_right_right_left_value,
                                    },
                                    right: perftest_flat::ObjectBoundaryLeaf {
                                        value: value_right_leaves16_right_right_right_right_value,
                                    },
                                },
                            },
                        },
                    },
                    leaves8: perftest_flat::ObjectBoundary8 {
                        left: perftest_flat::ObjectBoundary4 {
                            left: perftest_flat::ObjectBoundary2 {
                                left: perftest_flat::ObjectBoundaryLeaf {
                                    value: value_right_leaves8_left_left_left_value,
                                },
                                right: perftest_flat::ObjectBoundaryLeaf {
                                    value: value_right_leaves8_left_left_right_value,
                                },
                            },
                            right: perftest_flat::ObjectBoundary2 {
                                left: perftest_flat::ObjectBoundaryLeaf {
                                    value: value_right_leaves8_left_right_left_value,
                                },
                                right: perftest_flat::ObjectBoundaryLeaf {
                                    value: value_right_leaves8_left_right_right_value,
                                },
                            },
                        },
                        right: perftest_flat::ObjectBoundary4 {
                            left: perftest_flat::ObjectBoundary2 {
                                left: perftest_flat::ObjectBoundaryLeaf {
                                    value: value_right_leaves8_right_left_left_value,
                                },
                                right: perftest_flat::ObjectBoundaryLeaf {
                                    value: value_right_leaves8_right_left_right_value,
                                },
                            },
                            right: perftest_flat::ObjectBoundary2 {
                                left: perftest_flat::ObjectBoundaryLeaf {
                                    value: value_right_leaves8_right_right_left_value,
                                },
                                right: perftest_flat::ObjectBoundaryLeaf {
                                    value: value_right_leaves8_right_right_right_value,
                                },
                            },
                        },
                    },
                    leaves4: perftest_flat::ObjectBoundary4 {
                        left: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_leaves4_left_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_leaves4_left_right_value,
                            },
                        },
                        right: perftest_flat::ObjectBoundary2 {
                            left: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_leaves4_right_left_value,
                            },
                            right: perftest_flat::ObjectBoundaryLeaf {
                                value: value_right_leaves4_right_right_value,
                            },
                        },
                    },
                    leaves2: perftest_flat::ObjectBoundary2 {
                        left: perftest_flat::ObjectBoundaryLeaf {
                            value: value_right_leaves2_left_value,
                        },
                        right: perftest_flat::ObjectBoundaryLeaf {
                            value: value_right_leaves2_right_value,
                        },
                    },
                    leaf: perftest_flat::ObjectBoundaryLeaf {
                        value: value_right_leaf_value,
                    },
                },
            }
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::object_boundary_value(&value);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_unsignedRoundTrip<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    byte: ::prebindgen_jni_runtime::jni::sys::jint,
    short: ::prebindgen_jni_runtime::jni::sys::jint,
    int: ::prebindgen_jni_runtime::jni::sys::jlong,
    long: ::prebindgen_jni_runtime::jni::sys::jlong,
    maybe_long__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    maybe_long: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let byte = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            <u8 as ::core::convert::TryFrom<_>>::try_from(byte)
                .map_err(|_| ::std::format!("u8 input out of range: {}", byte))?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let short = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            <u16 as ::core::convert::TryFrom<_>>::try_from(short)
                .map_err(|_| ::std::format!("u16 input out of range: {}", short))?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let int = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            <u32 as ::core::convert::TryFrom<_>>::try_from(int)
                .map_err(|_| ::std::format!("u32 input out of range: {}", int))?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let long = (long as u64);
    let maybe_long = if (maybe_long__present != 0) {
        ::core::option::Option::Some((maybe_long as u64))
    } else {
        ::core::option::Option::None
    };
    let __result = perftest_flat::unsigned_round_trip(
        byte,
        short,
        int,
        long,
        maybe_long,
    );
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r_byte, r_short, r_int, r_long, r_maybe_long) = {
            let perftest_flat::Unsigned {
                byte: __f0,
                short: __f1,
                int: __f2,
                long: __f3,
                maybe_long: __f4,
            } = __result;
            let r_byte = (__f0 as ::prebindgen_jni_runtime::jni::sys::jint);
            let r_short = (__f1 as ::prebindgen_jni_runtime::jni::sys::jint);
            let r_int = (__f2 as ::prebindgen_jni_runtime::jni::sys::jlong);
            let r_long = (__f3 as ::prebindgen_jni_runtime::jni::sys::jlong);
            let r_maybe_long = match __f4 {
                ::core::option::Option::Some(__x2) => {
                    ::prebindgen_jni_runtime::box_jlong(
                        env,
                        (__x2 as ::prebindgen_jni_runtime::jni::sys::jlong),
                    )?
                }
                ::core::option::Option::None => {
                    ::prebindgen_jni_runtime::jni::objects::JObject::null()
                }
            };
            (r_byte, r_short, r_int, r_long, r_maybe_long)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Unsigned",
                "run",
                "(IIJJLjava/lang/Long;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_byte,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_short,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_int,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_long,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_maybe_long.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_unsignedOptional<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    value: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let value = if (value__present != 0) {
        ::core::option::Option::Some((value as u64))
    } else {
        ::core::option::Option::None
    };
    let __result = perftest_flat::unsigned_optional(value);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match __result {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::box_jlong(
                    env,
                    (__x1 as ::prebindgen_jni_runtime::jni::sys::jlong),
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_unsignedDataMaybe<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value_byte: ::prebindgen_jni_runtime::jni::sys::jint,
    value_short: ::prebindgen_jni_runtime::jni::sys::jint,
    value_int: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_long: ::prebindgen_jni_runtime::jni::sys::jlong,
    value_maybe_long__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    value_maybe_long: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let value = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::Unsigned {
            byte: <u8 as ::core::convert::TryFrom<_>>::try_from(value_byte)
                .map_err(|_| ::std::format!("u8 input out of range: {}", value_byte))?,
            short: <u16 as ::core::convert::TryFrom<_>>::try_from(value_short)
                .map_err(|_| ::std::format!("u16 input out of range: {}", value_short))?,
            int: <u32 as ::core::convert::TryFrom<_>>::try_from(value_int)
                .map_err(|_| ::std::format!("u32 input out of range: {}", value_int))?,
            long: (value_long as u64),
            maybe_long: if (value_maybe_long__present != 0) {
                ::core::option::Option::Some((value_maybe_long as u64))
            } else {
                ::core::option::Option::None
            },
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::unsigned_data_maybe(&value);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match __result {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::box_jlong(
                    env,
                    (__x1 as ::prebindgen_jni_runtime::jni::sys::jlong),
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_unsignedEmit<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value: ::prebindgen_jni_runtime::jni::sys::jlong,
    f: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let value = (value as u64);
    let f = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __up = ::prebindgen_jni_runtime::Upcall::new(
                env,
                &f,
                "run",
                "(J)V",
                34i32,
            )?;
            move |__a0: u64| {
                let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                    __up.call_void(|env| {
                        let a0 = (__a0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                        ::core::result::Result::Ok(
                            ::std::vec![
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j : a0 }
                            ],
                        )
                    })?;
                    ::core::result::Result::Ok(())
                })();
                if let ::core::result::Result::Err(__err) = __res {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (u64) + Send + Sync + 'static",
                        &__err,
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
    let __result = perftest_flat::unsigned_emit(value, f);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_unsignedSeries<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let __result = perftest_flat::unsigned_series();
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r__n, r) = {
            let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(
                    __result,
                )
                .collect();
            let __n = __items.len();
            let mut __c1_0 = ::std::vec::Vec::with_capacity(__n);
            for (__i, __x1) in __items.into_iter().enumerate() {
                let __e1 = (__x1 as ::prebindgen_jni_runtime::jni::sys::jlong);
                __c1_0.push(__e1);
            }
            (__n as i32, ::prebindgen_jni_runtime::write_longs(env, &__c1_0)?)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Vec_u64",
                "run",
                "(I[J)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__n,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_blobValueNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    secs: ::prebindgen_jni_runtime::jni::sys::jlong,
    id: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    chunks__n: ::prebindgen_jni_runtime::jni::sys::jint,
    chunks: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let secs = secs;
    let id = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(::prebindgen_jni_runtime::read_u8s(env, &id)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let chunks = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __n = chunks__n as usize;
            let mut __v = ::std::vec::Vec::with_capacity(__n);
            for __i in 0..__n {
                let __e0 = ::prebindgen_jni_runtime::object_array_get(
                    env,
                    &chunks,
                    __i,
                )?;
                let __x = (|| -> ::core::result::Result<_, ::std::string::String> {
                    ::core::result::Result::Ok(
                        ::prebindgen_jni_runtime::read_u8s(env, &__e0)?,
                    )
                })();
                ::prebindgen_jni_runtime::drop_local(env, __e0);
                __v.push(__x?);
            }
            __v
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::blob_value_new(secs, id, chunks);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r_stamp_secs, r_stamp_nanos, r_id, r_chunks__n, r_chunks) = {
            let perftest_flat::BlobValue { stamp: __f0, id: __f1, chunks: __f2 } = __result;
            let (r_stamp_secs, r_stamp_nanos) = {
                let perftest_flat::Stamp { secs: __f0, nanos: __f1 } = __f0;
                let r_stamp_secs = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                let r_stamp_nanos = (__f1 as ::prebindgen_jni_runtime::jni::sys::jlong);
                (r_stamp_secs, r_stamp_nanos)
            };
            let r_id = ::prebindgen_jni_runtime::write_u8s(
                env,
                ::core::convert::AsRef::<[u8]>::as_ref(&__f1),
            )?;
            let (r_chunks__n, r_chunks) = {
                let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(
                        __f2,
                    )
                    .collect();
                let __n = __items.len();
                let __c2_0 = ::prebindgen_jni_runtime::new_object_array(env, __n)?;
                for (__i, __x2) in __items.into_iter().enumerate() {
                    let __e2 = ::prebindgen_jni_runtime::write_u8s(
                        env,
                        ::core::convert::AsRef::<[u8]>::as_ref(&__x2),
                    )?;
                    ::prebindgen_jni_runtime::object_array_set(env, &__c2_0, __i, __e2)?;
                }
                (__n as i32, __c2_0)
            };
            (r_stamp_secs, r_stamp_nanos, r_id, r_chunks__n, r_chunks)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_BlobValue",
                "run",
                "(JJ[BI[Ljava/lang/Object;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_stamp_secs,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_stamp_nanos,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_id.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_chunks__n,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_chunks.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_blobValueEcho<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value__packI: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    value__packJ: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    value__packL: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let value = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __value__packI = ::prebindgen_jni_runtime::read_ints(
                env,
                &value__packI,
            )?;
            ::prebindgen_jni_runtime::check_packed(
                __value__packI.len(),
                1usize,
                "value__packI",
            )?;
            let value_chunks__n = __value__packI[0usize];
            let __value__packJ = ::prebindgen_jni_runtime::read_longs(
                env,
                &value__packJ,
            )?;
            ::prebindgen_jni_runtime::check_packed(
                __value__packJ.len(),
                2usize,
                "value__packJ",
            )?;
            let value_stamp_secs = __value__packJ[0usize];
            let value_stamp_nanos = __value__packJ[1usize];
            ::prebindgen_jni_runtime::check_packed(
                ::prebindgen_jni_runtime::object_array_len(env, &value__packL)?,
                2usize,
                "value__packL",
            )?;
            let value_id = ::prebindgen_jni_runtime::object_array_get(
                env,
                &value__packL,
                0usize,
            )?;
            let value_chunks = ::prebindgen_jni_runtime::object_array_get(
                env,
                &value__packL,
                1usize,
            )?;
            perftest_flat::BlobValue {
                stamp: perftest_flat::Stamp {
                    secs: value_stamp_secs,
                    nanos: value_stamp_nanos,
                },
                id: ::prebindgen_jni_runtime::read_u8s(env, &value_id)?,
                chunks: {
                    let __n = value_chunks__n as usize;
                    let mut __v = ::std::vec::Vec::with_capacity(__n);
                    for __i in 0..__n {
                        let __e0 = ::prebindgen_jni_runtime::object_array_get(
                            env,
                            &value_chunks,
                            __i,
                        )?;
                        let __x = (|| -> ::core::result::Result<
                            _,
                            ::std::string::String,
                        > {
                            ::core::result::Result::Ok(
                                ::prebindgen_jni_runtime::read_u8s(env, &__e0)?,
                            )
                        })();
                        ::prebindgen_jni_runtime::drop_local(env, __e0);
                        __v.push(__x?);
                    }
                    __v
                },
            }
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::blob_value_echo(value);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r_stamp_secs, r_stamp_nanos, r_id, r_chunks__n, r_chunks) = {
            let perftest_flat::BlobValue { stamp: __f0, id: __f1, chunks: __f2 } = __result;
            let (r_stamp_secs, r_stamp_nanos) = {
                let perftest_flat::Stamp { secs: __f0, nanos: __f1 } = __f0;
                let r_stamp_secs = (__f0 as ::prebindgen_jni_runtime::jni::sys::jlong);
                let r_stamp_nanos = (__f1 as ::prebindgen_jni_runtime::jni::sys::jlong);
                (r_stamp_secs, r_stamp_nanos)
            };
            let r_id = ::prebindgen_jni_runtime::write_u8s(
                env,
                ::core::convert::AsRef::<[u8]>::as_ref(&__f1),
            )?;
            let (r_chunks__n, r_chunks) = {
                let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(
                        __f2,
                    )
                    .collect();
                let __n = __items.len();
                let __c2_0 = ::prebindgen_jni_runtime::new_object_array(env, __n)?;
                for (__i, __x2) in __items.into_iter().enumerate() {
                    let __e2 = ::prebindgen_jni_runtime::write_u8s(
                        env,
                        ::core::convert::AsRef::<[u8]>::as_ref(&__x2),
                    )?;
                    ::prebindgen_jni_runtime::object_array_set(env, &__c2_0, __i, __e2)?;
                }
                (__n as i32, __c2_0)
            };
            (r_stamp_secs, r_stamp_nanos, r_id, r_chunks__n, r_chunks)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_BlobValue",
                "run",
                "(JJ[BI[Ljava/lang/Object;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_stamp_secs,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_stamp_nanos,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_id.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r_chunks__n,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_chunks.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_arraysEcho<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a_bytes: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_shorts: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_ints: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_longs: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_doubles: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_flags: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    a_raw: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(perftest_flat::Arrays {
            bytes: ::prebindgen_jni_runtime::fixed::<
                u8,
                4usize,
            >(
                ::prebindgen_jni_runtime::read_bytes(env, &a_bytes)?
                    .into_iter()
                    .map(|__x| __x as u8)
                    .collect(),
            )?,
            shorts: ::prebindgen_jni_runtime::fixed::<
                i16,
                2usize,
            >(
                ::prebindgen_jni_runtime::read_shorts(env, &a_shorts)?
                    .into_iter()
                    .map(|__x| __x as i16)
                    .collect(),
            )?,
            ints: ::prebindgen_jni_runtime::fixed::<
                i32,
                3usize,
            >(
                ::prebindgen_jni_runtime::read_ints(env, &a_ints)?
                    .into_iter()
                    .map(|__x| __x as i32)
                    .collect(),
            )?,
            longs: ::prebindgen_jni_runtime::fixed::<
                i64,
                2usize,
            >(
                ::prebindgen_jni_runtime::read_longs(env, &a_longs)?
                    .into_iter()
                    .map(|__x| __x as i64)
                    .collect(),
            )?,
            doubles: ::prebindgen_jni_runtime::fixed::<
                f64,
                2usize,
            >(
                ::prebindgen_jni_runtime::read_doubles(env, &a_doubles)?
                    .into_iter()
                    .map(|__x| __x as f64)
                    .collect(),
            )?,
            flags: ::prebindgen_jni_runtime::fixed::<
                bool,
                3usize,
            >(
                ::prebindgen_jni_runtime::read_booleans(env, &a_flags)?
                    .into_iter()
                    .map(|__x| __x != 0)
                    .collect(),
            )?,
            raw: ::prebindgen_jni_runtime::fixed::<
                u64,
                2usize,
            >(
                ::prebindgen_jni_runtime::read_longs(env, &a_raw)?
                    .into_iter()
                    .map(|__x| __x as u64)
                    .collect(),
            )?,
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::arrays_echo(a);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r_bytes, r_shorts, r_ints, r_longs, r_doubles, r_flags, r_raw) = {
            let perftest_flat::Arrays {
                bytes: __f0,
                shorts: __f1,
                ints: __f2,
                longs: __f3,
                doubles: __f4,
                flags: __f5,
                raw: __f6,
            } = __result;
            let r_bytes = ::prebindgen_jni_runtime::write_bytes(
                env,
                &__f0
                    .iter()
                    .map(|__x| *__x as ::prebindgen_jni_runtime::jni::sys::jbyte)
                    .collect::<::std::vec::Vec<_>>(),
            )?;
            let r_shorts = ::prebindgen_jni_runtime::write_shorts(
                env,
                &__f1
                    .iter()
                    .map(|__x| *__x as ::prebindgen_jni_runtime::jni::sys::jshort)
                    .collect::<::std::vec::Vec<_>>(),
            )?;
            let r_ints = ::prebindgen_jni_runtime::write_ints(
                env,
                &__f2
                    .iter()
                    .map(|__x| *__x as ::prebindgen_jni_runtime::jni::sys::jint)
                    .collect::<::std::vec::Vec<_>>(),
            )?;
            let r_longs = ::prebindgen_jni_runtime::write_longs(
                env,
                &__f3
                    .iter()
                    .map(|__x| *__x as ::prebindgen_jni_runtime::jni::sys::jlong)
                    .collect::<::std::vec::Vec<_>>(),
            )?;
            let r_doubles = ::prebindgen_jni_runtime::write_doubles(
                env,
                &__f4
                    .iter()
                    .map(|__x| *__x as ::prebindgen_jni_runtime::jni::sys::jdouble)
                    .collect::<::std::vec::Vec<_>>(),
            )?;
            let r_flags = ::prebindgen_jni_runtime::write_booleans(
                env,
                &__f5.iter().map(|__x| u8::from(*__x)).collect::<::std::vec::Vec<_>>(),
            )?;
            let r_raw = ::prebindgen_jni_runtime::write_longs(
                env,
                &__f6
                    .iter()
                    .map(|__x| *__x as ::prebindgen_jni_runtime::jni::sys::jlong)
                    .collect::<::std::vec::Vec<_>>(),
            )?;
            (r_bytes, r_shorts, r_ints, r_longs, r_doubles, r_flags, r_raw)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Arrays",
                "run",
                "([B[S[I[J[D[Z[J)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_bytes.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_shorts.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_ints.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_longs.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_doubles.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_flags.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_raw.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_durationOptional<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    value: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let value = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            if (value__present != 0) {
                ::core::option::Option::Some({
                    let __r0 = (value as u64);
                    let __r0 = ::prebindgen_jni_runtime::check_domain(
                        __r0,
                        0u128,
                        86400000u128,
                        "Duration",
                    )?;
                    crate::duration_from_millis(__r0)
                })
            } else {
                ::core::option::Option::None
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::duration_optional(value);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match __result {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::box_jlong(
                    env,
                    {
                        let __r2 = crate::duration_to_millis(__x1)
                            .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
                        let __r2 = ::prebindgen_jni_runtime::check_domain(
                            __r2,
                            0u128,
                            86400000u128,
                            "Duration",
                        )?;
                        (__r2 as ::prebindgen_jni_runtime::jni::sys::jlong)
                    },
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_boxedDurationEcho<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let value = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::std::boxed::Box::new({
                let __r0 = (value as u64);
                let __r0 = ::prebindgen_jni_runtime::check_domain(
                    __r0,
                    0u128,
                    86400000u128,
                    "Duration",
                )?;
                crate::duration_from_millis(__r0)
            }),
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::boxed_duration_echo(value);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| {
        let r = {
            let __r1 = crate::duration_to_millis((*__result))
                .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
            let __r1 = ::prebindgen_jni_runtime::check_domain(
                __r1,
                0u128,
                86400000u128,
                "Duration",
            )?;
            (__r1 as ::prebindgen_jni_runtime::jni::sys::jlong)
        };
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_durationBoundaryEcho<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value__packZ: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    value__packJ: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let value = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __value__packZ = ::prebindgen_jni_runtime::read_booleans(
                env,
                &value__packZ,
            )?;
            ::prebindgen_jni_runtime::check_packed(
                __value__packZ.len(),
                1usize,
                "value__packZ",
            )?;
            let value_delay__present = __value__packZ[0usize];
            let __value__packJ = ::prebindgen_jni_runtime::read_longs(
                env,
                &value__packJ,
            )?;
            ::prebindgen_jni_runtime::check_packed(
                __value__packJ.len(),
                2usize,
                "value__packJ",
            )?;
            let value_required = __value__packJ[0usize];
            let value_delay = __value__packJ[1usize];
            perftest_flat::DurationBoundary {
                required: {
                    let __r0 = (value_required as u64);
                    let __r0 = ::prebindgen_jni_runtime::check_domain(
                        __r0,
                        0u128,
                        86400000u128,
                        "Duration",
                    )?;
                    crate::duration_from_millis(__r0)
                },
                delay: if (value_delay__present != 0) {
                    ::core::option::Option::Some({
                        let __r0 = (value_delay as u64);
                        let __r0 = ::prebindgen_jni_runtime::check_domain(
                            __r0,
                            0u128,
                            86400000u128,
                            "Duration",
                        )?;
                        crate::duration_from_millis(__r0)
                    })
                } else {
                    ::core::option::Option::None
                },
            }
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::duration_boundary_echo(&value);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r_required, r_delay) = {
            let perftest_flat::DurationBoundary { required: __f0, delay: __f1 } = __result;
            let r_required = {
                let __r2 = crate::duration_to_millis(__f0)
                    .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
                let __r2 = ::prebindgen_jni_runtime::check_domain(
                    __r2,
                    0u128,
                    86400000u128,
                    "Duration",
                )?;
                (__r2 as ::prebindgen_jni_runtime::jni::sys::jlong)
            };
            let r_delay = match __f1 {
                ::core::option::Option::Some(__x2) => {
                    ::prebindgen_jni_runtime::box_jlong(
                        env,
                        {
                            let __r3 = crate::duration_to_millis(__x2)
                                .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
                            let __r3 = ::prebindgen_jni_runtime::check_domain(
                                __r3,
                                0u128,
                                86400000u128,
                                "Duration",
                            )?;
                            (__r3 as ::prebindgen_jni_runtime::jni::sys::jlong)
                        },
                    )?
                }
                ::core::option::Option::None => {
                    ::prebindgen_jni_runtime::jni::objects::JObject::null()
                }
            };
            (r_required, r_delay)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_DurationBoundary",
                "run",
                "(JLjava/lang/Long;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: r_required,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r_delay.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_durationEmit<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    value: ::prebindgen_jni_runtime::jni::sys::jlong,
    f: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let value = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __r0 = (value as u64);
            let __r0 = ::prebindgen_jni_runtime::check_domain(
                __r0,
                0u128,
                86400000u128,
                "Duration",
            )?;
            crate::duration_from_millis(__r0)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let f = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __up = ::prebindgen_jni_runtime::Upcall::new(
                env,
                &f,
                "run",
                "(J)V",
                34i32,
            )?;
            move |__a0: perftest_flat::Duration| {
                let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                    __up.call_void(|env| {
                        let a0 = {
                            let __r1 = crate::duration_to_millis(__a0)
                                .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
                            let __r1 = ::prebindgen_jni_runtime::check_domain(
                                __r1,
                                0u128,
                                86400000u128,
                                "Duration",
                            )?;
                            (__r1 as ::prebindgen_jni_runtime::jni::sys::jlong)
                        };
                        ::core::result::Result::Ok(
                            ::std::vec![
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j : a0 }
                            ],
                        )
                    })?;
                    ::core::result::Result::Ok(())
                })();
                if let ::core::result::Result::Err(__err) = __res {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (Duration) + Send + Sync + 'static",
                        &__err,
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
    let __result = perftest_flat::duration_emit(value, f);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_durationOutOfRange<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let __result = perftest_flat::duration_out_of_range();
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match __result {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::box_jlong(
                    env,
                    {
                        let __r2 = crate::duration_to_millis(__x1)
                            .map_err(|__e| ::std::string::ToString::to_string(&__e))?;
                        let __r2 = ::prebindgen_jni_runtime::check_domain(
                            __r2,
                            0u128,
                            86400000u128,
                            "Duration",
                        )?;
                        (__r2 as ::prebindgen_jni_runtime::jni::sys::jlong)
                    },
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageSummary<'a>(
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
    let __result = perftest_flat::storage_summary(s);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let __x = __result;
        {
            let (r_count, r_total) = {
                let __v1 = __x;
                let r_count = (perftest_flat::summary_count(&__v1)
                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                let r_total = (perftest_flat::summary_total(&__v1)
                    as ::prebindgen_jni_runtime::jni::sys::jdouble);
                (r_count, r_total)
            };
            static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
            let __o = __S
                .call_object(
                    env,
                    "io/prebindgen/covertest/analytics/SummaryBuilderRaw",
                    "run",
                    "(JD)Ljava/lang/Object;",
                    &__sink,
                    &[
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_count,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            d: r_total,
                        },
                    ],
                )?;
            ::core::result::Result::Ok(__o.into_raw())
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summaryDescribe<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s_sel: ::prebindgen_jni_runtime::jni::sys::jint,
    s_00__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    s_00: ::prebindgen_jni_runtime::jni::sys::jlong,
    s_01__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    s_01: ::prebindgen_jni_runtime::jni::sys::jdouble,
    s_1: ::prebindgen_jni_runtime::jni::sys::jlong,
    verbose: ::prebindgen_jni_runtime::jni::sys::jboolean,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            match s_sel {
                0i32 => {
                    ::prebindgen_jni_runtime::MaybeOwned::Owned(
                        perftest_flat::summary_new(
                            ::core::result::Result::<
                                _,
                                ::std::string::String,
                            >::Ok(
                                    if (s_00__present != 0) {
                                        ::core::option::Option::Some(s_00)
                                    } else {
                                        ::core::option::Option::None
                                    },
                                )?
                                .ok_or_else(|| ::std::string::String::from(
                                    "missing argument `count` for `s` variant 0",
                                ))?,
                            ::core::result::Result::<
                                _,
                                ::std::string::String,
                            >::Ok(
                                    if (s_01__present != 0) {
                                        ::core::option::Option::Some(s_01)
                                    } else {
                                        ::core::option::Option::None
                                    },
                                )?
                                .ok_or_else(|| ::std::string::String::from(
                                    "missing argument `total` for `s` variant 0",
                                ))?,
                        ),
                    )
                }
                1i32 => {
                    ::prebindgen_jni_runtime::MaybeOwned::Borrowed(
                        ::prebindgen_jni_runtime::borrow_handle::<
                            perftest_flat::Summary,
                        >(s_1)?,
                    )
                }
                __s => {
                    return ::core::result::Result::Err(
                        ::std::format!("invalid selector {} for `s`", __s),
                    );
                }
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let verbose = (verbose != 0);
    let __result = crate::summary_describe(&*s, verbose);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = ::prebindgen_jni_runtime::new_string(
            env,
            ::core::convert::AsRef::<str>::as_ref(&__result),
        )?;
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageMatchesSummary<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    expected_sel: ::prebindgen_jni_runtime::jni::sys::jint,
    expected_00__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    expected_00: ::prebindgen_jni_runtime::jni::sys::jlong,
    expected_01__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    expected_01: ::prebindgen_jni_runtime::jni::sys::jdouble,
    expected_1: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jboolean {
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
            return 0;
        }
    };
    let expected = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            match expected_sel {
                0i32 => {
                    perftest_flat::summary_new(
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (expected_00__present != 0) {
                                    ::core::option::Option::Some(expected_00)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `count` for `expected` variant 0",
                            ))?,
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (expected_01__present != 0) {
                                    ::core::option::Option::Some(expected_01)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `total` for `expected` variant 0",
                            ))?,
                    )
                }
                1i32 => {
                    ::prebindgen_jni_runtime::take_handle::<
                        perftest_flat::Summary,
                    >(expected_1)?
                }
                __s => {
                    return ::core::result::Result::Err(
                        ::std::format!("invalid selector {} for `expected`", __s),
                    );
                }
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::storage_matches_summary(s, expected);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jboolean,
        ::std::string::String,
    > = (|| {
        let r = (__result as u8);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageSummaryHandle<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
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
            return 0;
        }
    };
    let __result = perftest_flat::storage_summary_handle(s);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summaryTotalRaw<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jdouble {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::take_handle::<perftest_flat::Summary>(s)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0.0f64;
        }
    };
    let __result = perftest_flat::summary_total_raw(s);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jdouble,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jdouble);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0.0f64
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageSummaryFull<
    'a,
>(
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
    let __result = perftest_flat::storage_summary_full(s);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let __x = __result;
        {
            let (r_count, r_total, r_handle) = {
                let __v1 = __x;
                let r_count = (perftest_flat::summary_count(&__v1)
                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                let r_total = (perftest_flat::summary_total(&__v1)
                    as ::prebindgen_jni_runtime::jni::sys::jdouble);
                let r_handle = ::prebindgen_jni_runtime::new_handle(__v1);
                (r_count, r_total, r_handle)
            };
            static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
            let __o = __S
                .call_object(
                    env,
                    "io/prebindgen/covertest/analytics/SummaryStorageSummaryFullBuilderRaw",
                    "run",
                    "(JDJ)Ljava/lang/Object;",
                    &__sink,
                    &[
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_count,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            d: r_total,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_handle,
                        },
                    ],
                )?;
            ::core::result::Result::Ok(__o.into_raw())
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageSummaryProbe<
    'a,
>(
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
    let __result = perftest_flat::storage_summary_probe(s);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let __x = __result;
        {
            let (r_count, r_total, r_handle) = {
                let __v1 = __x;
                let r_count = (perftest_flat::summary_count(&__v1)
                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                let r_total = (perftest_flat::summary_total(&__v1)
                    as ::prebindgen_jni_runtime::jni::sys::jdouble);
                let r_handle = match crate::summary_if_nonempty(&__v1) {
                    ::core::option::Option::Some(__x2) => {
                        ::prebindgen_jni_runtime::box_jlong(
                            env,
                            ::prebindgen_jni_runtime::new_handle(
                                ::core::clone::Clone::clone(__x2),
                            ),
                        )?
                    }
                    ::core::option::Option::None => {
                        ::prebindgen_jni_runtime::jni::objects::JObject::null()
                    }
                };
                (r_count, r_total, r_handle)
            };
            static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
            let __o = __S
                .call_object(
                    env,
                    "io/prebindgen/covertest/analytics/SummaryStorageSummaryProbeBuilderRaw",
                    "run",
                    "(JDLjava/lang/Long;)Ljava/lang/Object;",
                    &__sink,
                    &[
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_count,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            d: r_total,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            l: r_handle.as_raw(),
                        },
                    ],
                )?;
            ::core::result::Result::Ok(__o.into_raw())
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageExpectSummary<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    expected_sel: ::prebindgen_jni_runtime::jni::sys::jint,
    expected_00__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    expected_00: ::prebindgen_jni_runtime::jni::sys::jlong,
    expected_01__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    expected_01: ::prebindgen_jni_runtime::jni::sys::jdouble,
    expected_1: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jboolean {
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
            return 0;
        }
    };
    let expected = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            match expected_sel {
                0i32 => {
                    perftest_flat::summary_new(
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (expected_00__present != 0) {
                                    ::core::option::Option::Some(expected_00)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `count` for `expected` variant 0",
                            ))?,
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (expected_01__present != 0) {
                                    ::core::option::Option::Some(expected_01)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `total` for `expected` variant 0",
                            ))?,
                    )
                }
                1i32 => {
                    ::prebindgen_jni_runtime::take_handle::<
                        perftest_flat::Summary,
                    >(expected_1)?
                }
                __s => {
                    return ::core::result::Result::Err(
                        ::std::format!("invalid selector {} for `expected`", __s),
                    );
                }
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::storage_expect_summary(s, expected);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jboolean,
        ::std::string::String,
    > = (|| {
        let r = (__result as u8);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summaryPrefer<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    primary_sel: ::prebindgen_jni_runtime::jni::sys::jint,
    primary_00__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    primary_00: ::prebindgen_jni_runtime::jni::sys::jlong,
    primary_01__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    primary_01: ::prebindgen_jni_runtime::jni::sys::jdouble,
    primary_1: ::prebindgen_jni_runtime::jni::sys::jlong,
    fallback_sel: ::prebindgen_jni_runtime::jni::sys::jint,
    fallback_00__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    fallback_00: ::prebindgen_jni_runtime::jni::sys::jlong,
    fallback_01__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    fallback_01: ::prebindgen_jni_runtime::jni::sys::jdouble,
    fallback_1: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let primary = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            match primary_sel {
                0i32 => {
                    perftest_flat::summary_new(
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (primary_00__present != 0) {
                                    ::core::option::Option::Some(primary_00)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `count` for `primary` variant 0",
                            ))?,
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (primary_01__present != 0) {
                                    ::core::option::Option::Some(primary_01)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `total` for `primary` variant 0",
                            ))?,
                    )
                }
                1i32 => {
                    ::prebindgen_jni_runtime::take_handle::<
                        perftest_flat::Summary,
                    >(primary_1)?
                }
                __s => {
                    return ::core::result::Result::Err(
                        ::std::format!("invalid selector {} for `primary`", __s),
                    );
                }
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let fallback = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            match fallback_sel {
                0i32 => {
                    perftest_flat::summary_new(
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (fallback_00__present != 0) {
                                    ::core::option::Option::Some(fallback_00)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `count` for `fallback` variant 0",
                            ))?,
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (fallback_01__present != 0) {
                                    ::core::option::Option::Some(fallback_01)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `total` for `fallback` variant 0",
                            ))?,
                    )
                }
                1i32 => {
                    ::prebindgen_jni_runtime::take_handle::<
                        perftest_flat::Summary,
                    >(fallback_1)?
                }
                __s => {
                    return ::core::result::Result::Err(
                        ::std::format!("invalid selector {} for `fallback`", __s),
                    );
                }
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = cov_helpers::summary_prefer(primary, fallback);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summaryMerge<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    primary_sel: ::prebindgen_jni_runtime::jni::sys::jint,
    primary_00__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    primary_00: ::prebindgen_jni_runtime::jni::sys::jlong,
    primary_01__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    primary_01: ::prebindgen_jni_runtime::jni::sys::jdouble,
    primary_1: ::prebindgen_jni_runtime::jni::sys::jlong,
    fallback_sel: ::prebindgen_jni_runtime::jni::sys::jint,
    fallback_00__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    fallback_00: ::prebindgen_jni_runtime::jni::sys::jlong,
    fallback_01__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    fallback_01: ::prebindgen_jni_runtime::jni::sys::jdouble,
    fallback_1: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let primary = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            match primary_sel {
                0i32 => {
                    perftest_flat::summary_new(
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (primary_00__present != 0) {
                                    ::core::option::Option::Some(primary_00)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `count` for `primary` variant 0",
                            ))?,
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (primary_01__present != 0) {
                                    ::core::option::Option::Some(primary_01)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `total` for `primary` variant 0",
                            ))?,
                    )
                }
                1i32 => {
                    ::prebindgen_jni_runtime::take_handle::<
                        perftest_flat::Summary,
                    >(primary_1)?
                }
                __s => {
                    return ::core::result::Result::Err(
                        ::std::format!("invalid selector {} for `primary`", __s),
                    );
                }
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let fallback = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            match fallback_sel {
                0i32 => {
                    perftest_flat::summary_new(
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (fallback_00__present != 0) {
                                    ::core::option::Option::Some(fallback_00)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `count` for `fallback` variant 0",
                            ))?,
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (fallback_01__present != 0) {
                                    ::core::option::Option::Some(fallback_01)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `total` for `fallback` variant 0",
                            ))?,
                    )
                }
                1i32 => {
                    ::prebindgen_jni_runtime::take_handle::<
                        perftest_flat::Summary,
                    >(fallback_1)?
                }
                __s => {
                    return ::core::result::Result::Err(
                        ::std::format!("invalid selector {} for `fallback`", __s),
                    );
                }
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::summary_merge(primary, fallback);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let __x = __result;
        {
            let (r_count, r_total) = {
                let __v1 = __x;
                let r_count = (perftest_flat::summary_count(&__v1)
                    as ::prebindgen_jni_runtime::jni::sys::jlong);
                let r_total = (perftest_flat::summary_total(&__v1)
                    as ::prebindgen_jni_runtime::jni::sys::jdouble);
                (r_count, r_total)
            };
            static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
            let __o = __S
                .call_object(
                    env,
                    "io/prebindgen/covertest/analytics/SummaryBuilderRaw",
                    "run",
                    "(JD)Ljava/lang/Object;",
                    &__sink,
                    &[
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            j: r_count,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            d: r_total,
                        },
                    ],
                )?;
            ::core::result::Result::Ok(__o.into_raw())
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summaryTotalOpt<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s_sel: ::prebindgen_jni_runtime::jni::sys::jint,
    s_00__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    s_00: ::prebindgen_jni_runtime::jni::sys::jlong,
    s_01__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    s_01: ::prebindgen_jni_runtime::jni::sys::jdouble,
    s_1: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jdouble {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            match s_sel {
                0i32 => {
                    ::core::option::Option::Some(
                        ::prebindgen_jni_runtime::MaybeOwned::Owned(
                            perftest_flat::summary_new(
                                ::core::result::Result::<
                                    _,
                                    ::std::string::String,
                                >::Ok(
                                        if (s_00__present != 0) {
                                            ::core::option::Option::Some(s_00)
                                        } else {
                                            ::core::option::Option::None
                                        },
                                    )?
                                    .ok_or_else(|| ::std::string::String::from(
                                        "missing argument `count` for `s` variant 0",
                                    ))?,
                                ::core::result::Result::<
                                    _,
                                    ::std::string::String,
                                >::Ok(
                                        if (s_01__present != 0) {
                                            ::core::option::Option::Some(s_01)
                                        } else {
                                            ::core::option::Option::None
                                        },
                                    )?
                                    .ok_or_else(|| ::std::string::String::from(
                                        "missing argument `total` for `s` variant 0",
                                    ))?,
                            ),
                        ),
                    )
                }
                1i32 => {
                    ::core::option::Option::Some(
                        ::prebindgen_jni_runtime::MaybeOwned::Borrowed(
                            ::prebindgen_jni_runtime::borrow_handle::<
                                perftest_flat::Summary,
                            >(s_1)?,
                        ),
                    )
                }
                -1 => ::core::option::Option::None,
                __s => {
                    return ::core::result::Result::Err(
                        ::std::format!("invalid selector {} for `s`", __s),
                    );
                }
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0.0f64;
        }
    };
    let __result = cov_helpers::summary_total_opt(s.as_deref());
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jdouble,
        ::std::string::String,
    > = (|| {
        let r = (__result as ::prebindgen_jni_runtime::jni::sys::jdouble);
        ::core::result::Result::Ok(r)
    })();
    match __r {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !__err.is_empty() {
                __jni_signal(env, &__error_sink, &__err);
            }
            0.0f64
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summarySeries<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    count: ::prebindgen_jni_runtime::jni::sys::jlong,
    start: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let count = count;
    let start = start;
    let __result = perftest_flat::summary_series(count, start);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(__result)
            .collect();
        {
            let (__cn, __colw0, __colw1) = {
                let __n = __items.len();
                let mut __col0 = ::std::vec::Vec::with_capacity(__n);
                let mut __col1 = ::std::vec::Vec::with_capacity(__n);
                for (__i, __x) in __items.into_iter().enumerate() {
                    let (r_count, r_total) = {
                        let __v1 = __x;
                        let r_count = (perftest_flat::summary_count(&__v1)
                            as ::prebindgen_jni_runtime::jni::sys::jlong);
                        let r_total = (perftest_flat::summary_total(&__v1)
                            as ::prebindgen_jni_runtime::jni::sys::jdouble);
                        (r_count, r_total)
                    };
                    __col0.push(r_count);
                    __col1.push(r_total);
                }
                (
                    __n as i32,
                    ::prebindgen_jni_runtime::write_longs(env, &__col0)?,
                    ::prebindgen_jni_runtime::write_doubles(env, &__col1)?,
                )
            };
            static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
            let __o = __S
                .call_object(
                    env,
                    "io/prebindgen/covertest/analytics/SummaryFolderColumns",
                    "run",
                    "(I[J[D)Ljava/lang/Object;",
                    &__sink,
                    &[
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            i: __cn,
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            l: __colw0.as_raw(),
                        },
                        ::prebindgen_jni_runtime::jni::sys::jvalue {
                            l: __colw1.as_raw(),
                        },
                    ],
                )?;
            ::core::result::Result::Ok(__o.into_raw())
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_summarySeriesOpt<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    count: ::prebindgen_jni_runtime::jni::sys::jlong,
    start: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let count = count;
    let start = start;
    let __result = perftest_flat::summary_series_opt(count, start);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| match __result {
        ::core::option::Option::Some(__some) => {
            let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(
                    __some,
                )
                .collect();
            {
                let (__cn, __colw0, __colw1) = {
                    let __n = __items.len();
                    let mut __col0 = ::std::vec::Vec::with_capacity(__n);
                    let mut __col1 = ::std::vec::Vec::with_capacity(__n);
                    for (__i, __x) in __items.into_iter().enumerate() {
                        let (r_count, r_total) = {
                            let __v1 = __x;
                            let r_count = (perftest_flat::summary_count(&__v1)
                                as ::prebindgen_jni_runtime::jni::sys::jlong);
                            let r_total = (perftest_flat::summary_total(&__v1)
                                as ::prebindgen_jni_runtime::jni::sys::jdouble);
                            (r_count, r_total)
                        };
                        __col0.push(r_count);
                        __col1.push(r_total);
                    }
                    (
                        __n as i32,
                        ::prebindgen_jni_runtime::write_longs(env, &__col0)?,
                        ::prebindgen_jni_runtime::write_doubles(env, &__col1)?,
                    )
                };
                static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
                let __o = __S
                    .call_object(
                        env,
                        "io/prebindgen/covertest/analytics/SummaryFolderColumns",
                        "run",
                        "(I[J[D)Ljava/lang/Object;",
                        &__sink,
                        &[
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                i: __cn,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                l: __colw0.as_raw(),
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                l: __colw1.as_raw(),
                            },
                        ],
                    )?;
                ::core::result::Result::Ok(__o.into_raw())
            }
        }
        ::core::option::Option::None => {
            ::core::result::Result::Ok(::core::ptr::null_mut())
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_archiveNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let __result = perftest_flat::archive_new();
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_archiveStore<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a: ::prebindgen_jni_runtime::jni::sys::jlong,
    s_sel: ::prebindgen_jni_runtime::jni::sys::jint,
    s_00__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    s_00: ::prebindgen_jni_runtime::jni::sys::jlong,
    s_01__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    s_01: ::prebindgen_jni_runtime::jni::sys::jdouble,
    s_1: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle_mut::<perftest_flat::Archive>(a)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            match s_sel {
                0i32 => {
                    perftest_flat::summary_new(
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (s_00__present != 0) {
                                    ::core::option::Option::Some(s_00)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `count` for `s` variant 0",
                            ))?,
                        ::core::result::Result::<
                            _,
                            ::std::string::String,
                        >::Ok(
                                if (s_01__present != 0) {
                                    ::core::option::Option::Some(s_01)
                                } else {
                                    ::core::option::Option::None
                                },
                            )?
                            .ok_or_else(|| ::std::string::String::from(
                                "missing argument `total` for `s` variant 0",
                            ))?,
                    )
                }
                1i32 => {
                    ::prebindgen_jni_runtime::take_handle::<perftest_flat::Summary>(s_1)?
                }
                __s => {
                    return ::core::result::Result::Err(
                        ::std::format!("invalid selector {} for `s`", __s),
                    );
                }
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let __result = perftest_flat::archive_store(a, s);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_archiveLatest<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Archive>(a)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::archive_latest(a);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = match __result {
            ::core::option::Option::Some(__x1) => {
                ::prebindgen_jni_runtime::box_jlong(
                    env,
                    ::prebindgen_jni_runtime::new_handle(
                        ::core::clone::Clone::clone(__x1),
                    ),
                )?
            }
            ::core::option::Option::None => {
                ::prebindgen_jni_runtime::jni::objects::JObject::null()
            }
        };
        ::core::result::Result::Ok(r.into_raw())
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageNew<'a>(
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageGet<'a>(
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
                "io/prebindgen/covertest/__Sink_Option_Payload",
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storagePutByTake<
    'a,
>(
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storagePutByRead<
    'a,
>(
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storagePutSlice<'a>(
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageGetVec<'a>(
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
                    let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(
                            __x1,
                        )
                        .collect();
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
                "io/prebindgen/covertest/__Sink_Option_Vec_Payload",
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_payloadHandlerNew<
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
                let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                    __up.call_void(|env| {
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
                    })?;
                    ::core::result::Result::Ok(())
                })();
                if let ::core::result::Result::Err(__err) = __res {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (& Payload) + Send + Sync + 'static",
                        &__err,
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageCallback<'a>(
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_payloadVecHandlerNew<
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
                let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                    __up.call_void(|env| {
                        let (a0__n, a0_id, a0_seq, a0_value, a0_flag, a0_label) = {
                            let __items: ::std::vec::Vec<_> = __a0
                                .iter()
                                .cloned()
                                .collect();
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
                    })?;
                    ::core::result::Result::Ok(())
                })();
                if let ::core::result::Result::Err(__err) = __res {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (& [Payload]) + Send + Sync + 'static",
                        &__err,
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageCallbackVec<
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageTryWithLabel<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __domain_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let label = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(::prebindgen_jni_runtime::read_string(env, &label)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::storage_try_with_label(&label);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| match __result {
        ::core::result::Result::Ok(__ok) => {
            let r = ::prebindgen_jni_runtime::new_handle(__ok);
            ::core::result::Result::Ok(r)
        }
        ::core::result::Result::Err(__e) => {
            let (e_message, e_handle) = {
                let __v1 = __e;
                let e_message = ::prebindgen_jni_runtime::new_string(
                    env,
                    ::core::convert::AsRef::<
                        str,
                    >::as_ref(&perftest_flat::storage_error_message(&__v1)),
                )?;
                let e_handle = ::prebindgen_jni_runtime::new_handle(__v1);
                (e_message, e_handle)
            };
            static __D: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
            __D.call_void(
                env,
                "io/prebindgen/covertest/errors/StorageErrorHandlerRaw",
                "run",
                "(Ljava/lang/String;J)V",
                &__domain_sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: e_message.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: e_handle,
                    },
                ],
            )?;
            ::core::result::Result::Err(::std::string::String::new())
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageTryFromStamp<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s_secs: ::prebindgen_jni_runtime::jni::sys::jlong,
    s_nanos: ::prebindgen_jni_runtime::jni::sys::jlong,
    tag: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __domain_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let s = perftest_flat::Stamp {
        secs: s_secs,
        nanos: s_nanos,
    };
    let tag = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::fixed::<
                u8,
                2usize,
            >(
                ::prebindgen_jni_runtime::read_bytes(env, &tag)?
                    .into_iter()
                    .map(|__x| __x as u8)
                    .collect(),
            )?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::storage_try_from_stamp(s, tag);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| match __result {
        ::core::result::Result::Ok(__ok) => {
            let r = ::prebindgen_jni_runtime::new_handle(__ok);
            ::core::result::Result::Ok(r)
        }
        ::core::result::Result::Err(__e) => {
            let (e_message, e_handle) = {
                let __v1 = __e;
                let e_message = ::prebindgen_jni_runtime::new_string(
                    env,
                    ::core::convert::AsRef::<
                        str,
                    >::as_ref(&perftest_flat::storage_error_message(&__v1)),
                )?;
                let e_handle = ::prebindgen_jni_runtime::new_handle(__v1);
                (e_message, e_handle)
            };
            static __D: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
            __D.call_void(
                env,
                "io/prebindgen/covertest/errors/StorageErrorHandlerRaw",
                "run",
                "(Ljava/lang/String;J)V",
                &__domain_sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: e_message.as_raw(),
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        j: e_handle,
                    },
                ],
            )?;
            ::core::result::Result::Err(::std::string::String::new())
        }
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageShards<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    count: ::prebindgen_jni_runtime::jni::sys::jlong,
    each: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let count = count;
    let each = each;
    let __result = perftest_flat::storage_shards(count, each);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r__n, r) = {
            let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(
                    __result,
                )
                .collect();
            let __n = __items.len();
            let mut __c1_0 = ::std::vec::Vec::with_capacity(__n);
            for (__i, __x1) in __items.into_iter().enumerate() {
                let __e1 = ::prebindgen_jni_runtime::new_handle(__x1);
                __c1_0.push(__e1);
            }
            (__n as i32, ::prebindgen_jni_runtime::write_longs(env, &__c1_0)?)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Vec_Storage",
                "run",
                "(I[J)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__n,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageShardsOpt<
    'a,
>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    count: ::prebindgen_jni_runtime::jni::sys::jlong,
    each: ::prebindgen_jni_runtime::jni::sys::jlong,
    __sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let count = count;
    let each = each;
    let __result = perftest_flat::storage_shards_opt(count, each);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r__present, r__n, r) = match __result {
            ::core::option::Option::Some(__x1) => {
                let (r__n, r) = {
                    let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(
                            __x1,
                        )
                        .collect();
                    let __n = __items.len();
                    let mut __c2_0 = ::std::vec::Vec::with_capacity(__n);
                    for (__i, __x2) in __items.into_iter().enumerate() {
                        let __e2 = ::prebindgen_jni_runtime::new_handle(__x2);
                        __c2_0.push(__e2);
                    }
                    (__n as i32, ::prebindgen_jni_runtime::write_longs(env, &__c2_0)?)
                };
                (1u8, r__n, r)
            }
            ::core::option::Option::None => {
                (0u8, 0, ::prebindgen_jni_runtime::jni::objects::JObject::null())
            }
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Option_Vec_Storage",
                "run",
                "(ZI[J)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        z: r__present,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__n,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageHandlerNew<
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
                "(J)V",
                34i32,
            )?;
            move |__a0: perftest_flat::Storage| {
                let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                    __up.call_void(|env| {
                        let a0 = ::prebindgen_jni_runtime::new_handle(__a0);
                        ::core::result::Result::Ok(
                            ::std::vec![
                                ::prebindgen_jni_runtime::jni::sys::jvalue { j : a0 }
                            ],
                        )
                    })?;
                    ::core::result::Result::Ok(())
                })();
                if let ::core::result::Result::Err(__err) = __res {
                    ::prebindgen_jni_runtime::report_callback_error(
                        "callback impl Fn (Storage) + Send + Sync + 'static",
                        &__err,
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
    let __result = perftest_flat::storage_handler_new(f);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageEmit<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    n: ::prebindgen_jni_runtime::jni::sys::jlong,
    h: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) {
    let mut __env = __env;
    let env = &mut __env;
    let n = n;
    let h = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::StorageHandler>(h)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ();
        }
    };
    let __result = perftest_flat::storage_emit(n, h);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageTotalLen<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a: ::prebindgen_jni_runtime::jni::sys::jlong,
    b: ::prebindgen_jni_runtime::jni::sys::jlong,
    c: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Storage>(a)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let b = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Storage>(b)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let c = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            ::prebindgen_jni_runtime::borrow_handle::<perftest_flat::Storage>(c)?,
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::storage_total_len(a, b, c);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storageLabels<'a>(
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
    let __result = perftest_flat::storage_labels(s);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let (r__n, r) = {
            let __items: ::std::vec::Vec<_> = ::core::iter::IntoIterator::into_iter(
                    __result,
                )
                .collect();
            let __n = __items.len();
            let __c1_0 = ::prebindgen_jni_runtime::new_object_array(env, __n)?;
            for (__i, __x1) in __items.into_iter().enumerate() {
                let __e1 = ::prebindgen_jni_runtime::new_string(
                    env,
                    ::core::convert::AsRef::<str>::as_ref(&__x1),
                )?;
                ::prebindgen_jni_runtime::object_array_set(env, &__c1_0, __i, __e1)?;
            }
            (__n as i32, __c1_0)
        };
        static __S: ::prebindgen_jni_runtime::CachedIfaceMethod = ::prebindgen_jni_runtime::CachedIfaceMethod::new();
        let __o = __S
            .call_object(
                env,
                "io/prebindgen/covertest/__Sink_Vec_String",
                "run",
                "(I[Ljava/lang/Object;)Ljava/lang/Object;",
                &__sink,
                &[
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        i: r__n,
                    },
                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                        l: r.as_raw(),
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_storagePutOpt<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::sys::jlong,
    p__present: ::prebindgen_jni_runtime::jni::sys::jboolean,
    p_id: ::prebindgen_jni_runtime::jni::sys::jlong,
    p_seq: ::prebindgen_jni_runtime::jni::sys::jint,
    p_value: ::prebindgen_jni_runtime::jni::sys::jdouble,
    p_flag: ::prebindgen_jni_runtime::jni::sys::jboolean,
    p_label: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jboolean {
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
            return 0;
        }
    };
    let p = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(
            if (p__present != 0) {
                ::core::option::Option::Some(perftest_flat::Payload {
                    id: p_id,
                    seq: p_seq,
                    value: p_value,
                    flag: (p_flag != 0),
                    label: if !p_label.is_null() {
                        ::core::option::Option::Some(
                            ::std::boxed::Box::new(
                                ::prebindgen_jni_runtime::read_string(env, &p_label)?,
                            ),
                        )
                    } else {
                        ::core::option::Option::None
                    },
                })
            } else {
                ::core::option::Option::None
            },
        )
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return 0;
        }
    };
    let __result = perftest_flat::storage_put_opt(s, p);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jboolean,
        ::std::string::String,
    > = (|| {
        let r = (__result as u8);
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_millisAdd<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    a: ::prebindgen_jni_runtime::jni::sys::jlong,
    b: ::prebindgen_jni_runtime::jni::sys::jlong,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jlong {
    let mut __env = __env;
    let env = &mut __env;
    let a = {
        let __r0 = a;
        cov_helpers::millis_from_long(__r0)
    };
    let b = {
        let __r0 = b;
        cov_helpers::millis_from_long(__r0)
    };
    let __result = perftest_flat::millis_add(a, b);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jlong,
        ::std::string::String,
    > = (|| {
        let r = {
            let __r1 = cov_helpers::millis_value(&__result);
            (__r1 as ::prebindgen_jni_runtime::jni::sys::jlong)
        };
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
pub unsafe extern "system" fn Java_io_prebindgen_covertest_CovNative_stringNew<'a>(
    __env: ::prebindgen_jni_runtime::jni::JNIEnv<'a>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'a>,
    s: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
    __error_sink: ::prebindgen_jni_runtime::jni::objects::JObject<'a>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    let mut __env = __env;
    let env = &mut __env;
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(::prebindgen_jni_runtime::read_string(env, &s)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            __jni_signal(env, &__error_sink, &__err);
            return ::core::ptr::null_mut();
        }
    };
    let __result = perftest_flat::string_new(&s);
    let __r: ::core::result::Result<
        ::prebindgen_jni_runtime::jni::sys::jobject,
        ::std::string::String,
    > = (|| {
        let r = ::prebindgen_jni_runtime::new_string(
            env,
            ::core::convert::AsRef::<str>::as_ref(&__result),
        )?;
        ::core::result::Result::Ok(r.into_raw())
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
const _: () = {
    konst::assertc_eq!(
        perftest_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
const _: () = {
    konst::assertc_eq!(
        cov_helpers::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
