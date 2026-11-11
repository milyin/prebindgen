/**# Safety
Only generated Kotlin ownership release paths may call this entry point. The pointer must be a live allocation of the declared type with no outstanding native access; an opted-in Cleaner may release an unreachable wrapper without its monitor.*/
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
Only generated Kotlin ownership release paths may call this entry point. The pointer must be a live allocation of the declared type with no outstanding native access; an opted-in Cleaner may release an unreachable wrapper without its monitor.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_plain_JNINative_drop_1InputHandle_17cc67535bbd4a8ee(
    mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,
    id: i64,
) {
    if let Err(e) = ::prebindgen_runtime::boundary(|| unsafe {
        ::prebindgen_jni_runtime::drop_pointer::<bench_flat::InputHandle>(id)
    }) {
        ::prebindgen_jni_runtime::fail(&mut env, &e);
    }
}
/**# Safety
Only generated Kotlin ownership release paths may call this entry point. The pointer must be a live allocation of the declared type with no outstanding native access; an opted-in Cleaner may release an unreachable wrapper without its monitor.*/
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
Only generated Kotlin ownership release paths may call this entry point. The pointer must be a live allocation of the declared type with no outstanding native access; an opted-in Cleaner may release an unreachable wrapper without its monitor.*/
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
#[allow(non_upper_case_globals)]
static __pb_row_type_SequenceRow_Array_Any___1_00f39593b4ea53ea: ::prebindgen_jni_runtime::OutputRowType = ::prebindgen_jni_runtime::OutputRowType::new();
#[allow(non_upper_case_globals)]
static __pb_row_type_SequenceRow_Long_1_127a3c24b600dbe0: ::prebindgen_jni_runtime::OutputRowType = ::prebindgen_jni_runtime::OutputRowType::new();
#[allow(non_upper_case_globals)]
static __pb_row_type_SequenceRow_Long_Long_2_c3e8677e51c52ca2: ::prebindgen_jni_runtime::OutputRowType = ::prebindgen_jni_runtime::OutputRowType::new();
#[allow(non_upper_case_globals)]
static __pb_row_type_SequenceRow_Long_Long_Long_8_3346a47ef3bb7c4a: ::prebindgen_jni_runtime::OutputRowType = ::prebindgen_jni_runtime::OutputRowType::new();
#[allow(non_upper_case_globals)]
static __pb_row_type_SequenceRow_Long_String_ByteArray_3_e5ce0d0d57155c04: ::prebindgen_jni_runtime::OutputRowType = ::prebindgen_jni_runtime::OutputRowType::new();
#[no_mangle]
pub extern "system" fn Java_bench_plain_JNINative_init_1OutputRows_1776ecb9d0c3754e8(
    mut env: ::prebindgen_jni_runtime::jni::JNIEnv<'_>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'_>,
    classes: ::prebindgen_jni_runtime::jni::objects::JObjectArray<'_>,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if env.get_array_length(&classes).map_err(|e| e.to_string())? != 5i32 {
            return Err("output row type count differs".into());
        }
        let class = env
            .get_object_array_element(&classes, 0i32)
            .map_err(|e| e.to_string())?;
        let class = env
            .auto_local(::prebindgen_jni_runtime::jni::objects::JClass::from(class));
        __pb_row_type_SequenceRow_Array_Any___1_00f39593b4ea53ea
            .initialize(&mut env, &class, &["[Ljava/lang/Object;"])?;
        let class = env
            .get_object_array_element(&classes, 1i32)
            .map_err(|e| e.to_string())?;
        let class = env
            .auto_local(::prebindgen_jni_runtime::jni::objects::JClass::from(class));
        __pb_row_type_SequenceRow_Long_1_127a3c24b600dbe0
            .initialize(&mut env, &class, &["J"])?;
        let class = env
            .get_object_array_element(&classes, 2i32)
            .map_err(|e| e.to_string())?;
        let class = env
            .auto_local(::prebindgen_jni_runtime::jni::objects::JClass::from(class));
        __pb_row_type_SequenceRow_Long_Long_2_c3e8677e51c52ca2
            .initialize(&mut env, &class, &["J", "J"])?;
        let class = env
            .get_object_array_element(&classes, 3i32)
            .map_err(|e| e.to_string())?;
        let class = env
            .auto_local(::prebindgen_jni_runtime::jni::objects::JClass::from(class));
        __pb_row_type_SequenceRow_Long_Long_Long_8_3346a47ef3bb7c4a
            .initialize(&mut env, &class, &["J", "J", "J", "J", "J", "J", "J", "J"])?;
        let class = env
            .get_object_array_element(&classes, 4i32)
            .map_err(|e| e.to_string())?;
        let class = env
            .auto_local(::prebindgen_jni_runtime::jni::objects::JClass::from(class));
        __pb_row_type_SequenceRow_Long_String_ByteArray_3_e5ce0d0d57155c04
            .initialize(&mut env, &class, &["J", "Ljava/lang/String;", "[B"])?;
        Ok(())
    }) {
        ::prebindgen_jni_runtime::fail(&mut env, &error);
    }
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_i64_Range_0643563939dde3e9 {
    pub v0: i64,
    pub v1: ::prebindgen_jni_runtime::bulk::Range,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_Range_71300b84e922dbba {
    pub v0: ::prebindgen_jni_runtime::bulk::Range,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_i64_i64_b3c08eb92fbbc74c {
    pub v0: i64,
    pub v1: i64,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_Range_d5ee2af5a2eda167 {
    pub v0: ::prebindgen_jni_runtime::bulk::Range,
}
#[no_mangle]
pub extern "system" fn Java_bench_plain_JNINative_bulkLayouts(
    mut env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
) -> ::prebindgen_jni_runtime::jni::sys::jlongArray {
    let result = (|| -> Result<_, String> {
        let mut data = vec![
            1i64, 3176938622011698635i64, if cfg!(target_endian = "big") { 1 } else { 0
            }, ::core::mem::size_of:: < usize > () as i64
        ];
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_i64_Range_0643563939dde3e9>(),
                ::core::mem::align_of::<__PbBulk_i64_Range_0643563939dde3e9>(),
                ::core::mem::offset_of!(__PbBulk_i64_Range_0643563939dde3e9, v0),
                ::core::mem::offset_of!(__PbBulk_i64_Range_0643563939dde3e9, v1),
            ])
                .into_iter()
                .map(|v| v as i64),
        );
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_Range_71300b84e922dbba>(),
                ::core::mem::align_of::<__PbBulk_Range_71300b84e922dbba>(),
                ::core::mem::offset_of!(__PbBulk_Range_71300b84e922dbba, v0),
            ])
                .into_iter()
                .map(|v| v as i64),
        );
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                ::core::mem::align_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                ::core::mem::offset_of!(__PbBulk_i64_i64_b3c08eb92fbbc74c, v0),
                ::core::mem::offset_of!(__PbBulk_i64_i64_b3c08eb92fbbc74c, v1),
            ])
                .into_iter()
                .map(|v| v as i64),
        );
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_Range_d5ee2af5a2eda167>(),
                ::core::mem::align_of::<__PbBulk_Range_d5ee2af5a2eda167>(),
                ::core::mem::offset_of!(__PbBulk_Range_d5ee2af5a2eda167, v0),
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
#[allow(dead_code, non_snake_case)]
unsafe fn __pb_cleanup_native_InputHandle_567ed8299898558e(
    reader: &::prebindgen_jni_runtime::bulk::Reader<'_>,
    base: usize,
) {
    let _ = ::prebindgen_runtime::boundary(|| {
        let raw = reader
            .read::<
                i64,
            >(base + ::core::mem::offset_of!(__PbBulk_i64_i64_b3c08eb92fbbc74c, v0))?;
        if raw != 0 {
            let _ = ::prebindgen_runtime::boundary(|| unsafe {
                ::prebindgen_jni_runtime::drop_pointer::<bench_flat::InputHandle>(raw)
            });
        }
        Ok(())
    });
}
#[allow(dead_code, non_snake_case)]
unsafe fn __pb_cleanup_native_InputHandle_Rows_64034ad291d2d0cd(
    reader: &::prebindgen_jni_runtime::bulk::Reader<'_>,
    base: usize,
) {
    let _ = ::prebindgen_runtime::boundary(|| {
        let raw = reader
            .read::<
                i64,
            >(base + ::core::mem::offset_of!(__PbBulk_i64_Range_0643563939dde3e9, v0))?;
        if raw != 0 {
            let _ = ::prebindgen_runtime::boundary(|| unsafe {
                ::prebindgen_jni_runtime::drop_pointer::<bench_flat::InputHandle>(raw)
            });
        }
        Ok(())
    });
    let _ = ::prebindgen_runtime::boundary(|| {
        if let Some(range) = reader
            .range(
                base + ::core::mem::offset_of!(__PbBulk_i64_Range_0643563939dde3e9, v1),
            )?
        {
            let (start, length) = reader
                .rows(
                    Some(range),
                    ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                )?;
            for index in 0..length {
                unsafe {
                    __pb_cleanup_native_InputHandle_567ed8299898558e(
                        reader,
                        start
                            + index
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                >(),
                    );
                }
            }
        }
        Ok(())
    });
}
#[allow(dead_code, non_snake_case)]
fn __pb_cleanup_typed_InputHandle_68cf0461fb68be7c<'local>(
    env: &mut ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    row: &::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) {
    ::prebindgen_jni_runtime::reclaim_output(
        env,
        |env| {
            let raw = env
                .get_field_unchecked(
                    row,
                    __pb_row_type_SequenceRow_Long_String_ByteArray_3_e5ce0d0d57155c04
                        .get()?
                        .field(0usize),
                    ::prebindgen_jni_runtime::jni::signature::ReturnType::Primitive(
                        ::prebindgen_jni_runtime::jni::signature::Primitive::Long,
                    ),
                )
                .and_then(|v| v.j())
                .map_err(|e| e.to_string())?;
            if raw != 0 {
                let _ = ::prebindgen_runtime::boundary(|| unsafe {
                    ::prebindgen_jni_runtime::drop_pointer::<
                        bench_flat::InputHandle,
                    >(raw)
                });
            }
            Ok(())
        },
    );
}
#[allow(dead_code, non_snake_case)]
fn __pb_cleanup_typed_InputHandle_a3d87635baf7af7c<'local>(
    env: &mut ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    row: &::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) {
    ::prebindgen_jni_runtime::reclaim_output(
        env,
        |env| {
            let raw = env
                .get_field_unchecked(
                    row,
                    __pb_row_type_SequenceRow_Long_Long_2_c3e8677e51c52ca2
                        .get()?
                        .field(0usize),
                    ::prebindgen_jni_runtime::jni::signature::ReturnType::Primitive(
                        ::prebindgen_jni_runtime::jni::signature::Primitive::Long,
                    ),
                )
                .and_then(|v| v.j())
                .map_err(|e| e.to_string())?;
            if raw != 0 {
                let _ = ::prebindgen_runtime::boundary(|| unsafe {
                    ::prebindgen_jni_runtime::drop_pointer::<
                        bench_flat::InputHandle,
                    >(raw)
                });
            }
            Ok(())
        },
    );
}
#[allow(dead_code, non_snake_case)]
fn __pb_cleanup_typed_InputHandle_b207c5d0d65e1d12<'local>(
    env: &mut ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    row: &::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) {
    ::prebindgen_jni_runtime::reclaim_output(
        env,
        |env| {
            let raw = env
                .get_field_unchecked(
                    row,
                    __pb_row_type_SequenceRow_Long_1_127a3c24b600dbe0
                        .get()?
                        .field(0usize),
                    ::prebindgen_jni_runtime::jni::signature::ReturnType::Primitive(
                        ::prebindgen_jni_runtime::jni::signature::Primitive::Long,
                    ),
                )
                .and_then(|v| v.j())
                .map_err(|e| e.to_string())?;
            if raw != 0 {
                let _ = ::prebindgen_runtime::boundary(|| unsafe {
                    ::prebindgen_jni_runtime::drop_pointer::<
                        bench_flat::InputHandle,
                    >(raw)
                });
            }
            Ok(())
        },
    );
}
#[allow(dead_code, non_snake_case)]
fn __pb_cleanup_typed_Rows_fe4a4c800c91ea9c<'local>(
    env: &mut ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    row: &::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) {
    ::prebindgen_jni_runtime::reclaim_output(
        env,
        |env| {
            let rows = env
                .get_field_unchecked(
                    row,
                    __pb_row_type_SequenceRow_Array_Any___1_00f39593b4ea53ea
                        .get()?
                        .field(0usize),
                    ::prebindgen_jni_runtime::jni::signature::ReturnType::Object,
                )
                .and_then(|v| v.l())
                .map_err(|e| e.to_string())?;
            let rows = env
                .auto_local(
                    ::prebindgen_jni_runtime::jni::objects::JObjectArray::from(rows),
                );
            if !rows.is_null() {
                let length = env.get_array_length(&*rows).map_err(|e| e.to_string())?
                    as usize;
                unsafe {
                    ::prebindgen_jni_runtime::reclaim_objects(
                        env,
                        rows.as_raw(),
                        length,
                        __pb_cleanup_typed_InputHandle_b207c5d0d65e1d12,
                    );
                }
            }
            Ok(())
        },
    );
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1callback_1batches_1owned_1handles_133b783abb5780e95<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "()[Ljava/lang/Object;",
                "bench.plain.OwnedBatchesCallback",
            )?;
            #[allow(unused_mut, clippy::collapsible_if)]
            let p0_callback_value = move || {
                p0_callback_retained
                    .with_env_result(
                        17i32,
                        |callback_environment| {
                            let mut callback_environment = callback_environment;
                            {
                                let arguments = [];
                                {
                                    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
                                    let __prebindgen_input_consumed = &__pb_input_consumed;
                                    let callback_result = unsafe {
                                        p0_callback_retained
                                            .invoke_return(
                                                callback_environment,
                                                ::prebindgen_jni_runtime::jni::signature::ReturnType::Object,
                                                &arguments,
                                            )
                                    }?;
                                    let callback_return_w1 = callback_result
                                        .l()
                                        .map_err(|e| e.to_string())
                                        .map(
                                            ::prebindgen_jni_runtime::jni::objects::JObjectArray::from,
                                        )?;
                                    {
                                        let callback_return_decoded = ::prebindgen_runtime::boundary(||
                                        {
                                            {
                                                let mut callback_return_w0: Vec<bench_flat::HandleBatch> = Vec::new();
                                                if true {
                                                    if callback_return_w1.is_null() {
                                                        return Err("null collection".into());
                                                    }
                                                    let callback_return_w2 = callback_environment
                                                        .get_array_length(&callback_return_w1)
                                                        .map_err(|e| e.to_string())? as usize;
                                                    callback_return_w0.reserve_exact(callback_return_w2);
                                                    for callback_return_w3 in 0..callback_return_w2 {
                                                        let callback_return_w1_frame = unsafe {
                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                &callback_environment,
                                                                10i32,
                                                            )
                                                        }?;
                                                        let callback_return_w1_row = ::prebindgen_jni_runtime::row(
                                                            &mut callback_environment,
                                                            &callback_return_w1,
                                                            callback_return_w3,
                                                            2usize,
                                                        )?;
                                                        let callback_return_w5_f0 = ::prebindgen_jni_runtime::row_i64(
                                                            &mut callback_environment,
                                                            &(callback_return_w1_row),
                                                            0usize,
                                                        );
                                                        if callback_return_w5_f0.is_err()
                                                            && callback_environment
                                                                .exception_check()
                                                                .map_err(|e| e.to_string())?
                                                        {
                                                            return Err("JNI collection field decoding failed".into());
                                                        }
                                                        let callback_return_w6 = ::prebindgen_jni_runtime::row_rows(
                                                            &mut callback_environment,
                                                            &(callback_return_w1_row),
                                                            1usize,
                                                        );
                                                        if callback_return_w6.is_err()
                                                            && callback_environment
                                                                .exception_check()
                                                                .map_err(|e| e.to_string())?
                                                        {
                                                            return Err("JNI collection field decoding failed".into());
                                                        }
                                                        #[allow(clippy::redundant_closure_call)]
                                                        let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                                            bench_flat::HandleBatch,
                                                            String,
                                                        > {
                                                            {
                                                                let callback_return_w5_f0_pointer = (*callback_return_w5_f0
                                                                    .as_ref()
                                                                    .map_err(|e| e.clone())?)
                                                                    as *mut ::prebindgen_jni_runtime::Allocation<
                                                                        bench_flat::InputHandle,
                                                                    >;
                                                                ::prebindgen_runtime::validate_pointer(
                                                                    callback_return_w5_f0_pointer,
                                                                )?;
                                                                let callback_return_w4_f0: bench_flat::InputHandle = {
                                                                    let callback_return_w4_f0 = unsafe {
                                                                        Box::from_raw(callback_return_w5_f0_pointer)
                                                                    };
                                                                    {
                                                                        __pb_input_consumed
                                                                            .set(__pb_input_consumed.get() + (1usize));
                                                                    }
                                                                    callback_return_w4_f0.value
                                                                };
                                                                let callback_return_w4_field_0 = callback_return_w4_f0;
                                                                let mut callback_return_w4_f1: Vec<
                                                                    bench_flat::HandleRecord,
                                                                > = Vec::new();
                                                                if true {
                                                                    if (*callback_return_w6.as_ref().map_err(|e| e.clone())?)
                                                                        .is_null()
                                                                    {
                                                                        return Err("null collection".into());
                                                                    }
                                                                    let callback_return_w7 = callback_environment
                                                                        .get_array_length(
                                                                            &(*callback_return_w6.as_ref().map_err(|e| e.clone())?),
                                                                        )
                                                                        .map_err(|e| e.to_string())? as usize;
                                                                    callback_return_w4_f1.reserve_exact(callback_return_w7);
                                                                    for callback_return_w8 in 0..callback_return_w7 {
                                                                        let callback_return_w6_frame = unsafe {
                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                &callback_environment,
                                                                                10i32,
                                                                            )
                                                                        }?;
                                                                        let callback_return_w6_row = ::prebindgen_jni_runtime::row(
                                                                            &mut callback_environment,
                                                                            &(*callback_return_w6.as_ref().map_err(|e| e.clone())?),
                                                                            callback_return_w8,
                                                                            2usize,
                                                                        )?;
                                                                        let callback_return_w10_f0 = ::prebindgen_jni_runtime::row_i64(
                                                                            &mut callback_environment,
                                                                            &(callback_return_w6_row),
                                                                            0usize,
                                                                        );
                                                                        if callback_return_w10_f0.is_err()
                                                                            && callback_environment
                                                                                .exception_check()
                                                                                .map_err(|e| e.to_string())?
                                                                        {
                                                                            return Err("JNI collection field decoding failed".into());
                                                                        }
                                                                        let callback_return_w10_f1 = ::prebindgen_jni_runtime::row_i64(
                                                                            &mut callback_environment,
                                                                            &(callback_return_w6_row),
                                                                            1usize,
                                                                        );
                                                                        if callback_return_w10_f1.is_err()
                                                                            && callback_environment
                                                                                .exception_check()
                                                                                .map_err(|e| e.to_string())?
                                                                        {
                                                                            return Err("JNI collection field decoding failed".into());
                                                                        }
                                                                        #[allow(clippy::redundant_closure_call)]
                                                                        let callback_return_w6_element_built = (|| -> ::core::result::Result<
                                                                            bench_flat::HandleRecord,
                                                                            String,
                                                                        > {
                                                                            {
                                                                                let callback_return_w10_f0_pointer = (*callback_return_w10_f0
                                                                                    .as_ref()
                                                                                    .map_err(|e| e.clone())?)
                                                                                    as *mut ::prebindgen_jni_runtime::Allocation<
                                                                                        bench_flat::InputHandle,
                                                                                    >;
                                                                                ::prebindgen_runtime::validate_pointer(
                                                                                    callback_return_w10_f0_pointer,
                                                                                )?;
                                                                                let callback_return_w9_f0: bench_flat::InputHandle = {
                                                                                    let callback_return_w9_f0 = unsafe {
                                                                                        Box::from_raw(callback_return_w10_f0_pointer)
                                                                                    };
                                                                                    {
                                                                                        __pb_input_consumed
                                                                                            .set(__pb_input_consumed.get() + (1usize));
                                                                                    }
                                                                                    callback_return_w9_f0.value
                                                                                };
                                                                                let callback_return_w9_field_0 = callback_return_w9_f0;
                                                                                let callback_return_w9_f1: i64 = ::core::convert::Into::into(
                                                                                    *callback_return_w10_f1.as_ref().map_err(|e| e.clone())?,
                                                                                );
                                                                                let callback_return_w9_field_1 = callback_return_w9_f1;
                                                                                let callback_return_w9: bench_flat::HandleRecord = bench_flat::HandleRecord {
                                                                                    handle: callback_return_w9_field_0,
                                                                                    tag: callback_return_w9_field_1,
                                                                                };
                                                                                let callback_return_w6_row_value = callback_return_w9;
                                                                                Ok(callback_return_w6_row_value)
                                                                            }
                                                                        })();
                                                                        drop(callback_return_w6_frame);
                                                                        let element = callback_return_w6_element_built?;
                                                                        #[allow(clippy::zst_offset)]
                                                                        unsafe {
                                                                            let initialized = callback_return_w4_f1.len();
                                                                            callback_return_w4_f1
                                                                                .as_mut_ptr()
                                                                                .add(initialized)
                                                                                .write(element);
                                                                            callback_return_w4_f1.set_len(initialized + 1);
                                                                        }
                                                                    }
                                                                }
                                                                let callback_return_w4_field_1 = callback_return_w4_f1;
                                                                let callback_return_w4: bench_flat::HandleBatch = bench_flat::HandleBatch {
                                                                    guard: callback_return_w4_field_0,
                                                                    rows: callback_return_w4_field_1,
                                                                };
                                                                let callback_return_w1_row_value = callback_return_w4;
                                                                Ok(callback_return_w1_row_value)
                                                            }
                                                        })();
                                                        drop(callback_return_w1_frame);
                                                        let element = callback_return_w1_element_built?;
                                                        #[allow(clippy::zst_offset)]
                                                        unsafe {
                                                            let initialized = callback_return_w0.len();
                                                            callback_return_w0
                                                                .as_mut_ptr()
                                                                .add(initialized)
                                                                .write(element);
                                                            callback_return_w0.set_len(initialized + 1);
                                                        }
                                                    }
                                                }
                                                let callback_return_value = callback_return_w0;
                                                Ok::<_, String>(callback_return_value)
                                            }
                                        });
                                        if callback_return_decoded.is_err() {
                                            ::prebindgen_jni_runtime::reclaim_output(
                                                &mut callback_environment,
                                                |callback_environment| {
                                                    let mut callback_environment = callback_environment;
                                                    let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                                        __prebindgen_input_consumed,
                                                    );
                                                    if !callback_return_w1.is_null() {
                                                        let length = callback_environment
                                                            .get_array_length(&callback_return_w1)
                                                            .map_err(|e| e.to_string())? as usize;
                                                        for callback_return_w1_cleanup_index in 0..length {
                                                            let frame = unsafe {
                                                                ::prebindgen_jni_runtime::LocalFrame::new(
                                                                    &callback_environment,
                                                                    10i32,
                                                                )
                                                            }?;
                                                            ::prebindgen_jni_runtime::reclaim_output(
                                                                &mut callback_environment,
                                                                |callback_environment| {
                                                                    let mut callback_environment = callback_environment;
                                                                    let cleanup_read_frame = unsafe {
                                                                        ::prebindgen_jni_runtime::LocalFrame::new(
                                                                            &callback_environment,
                                                                            8,
                                                                        )
                                                                    }?;
                                                                    let cleanup_read = ::prebindgen_jni_runtime::row_rows(
                                                                        &mut callback_environment,
                                                                        &callback_return_w1,
                                                                        callback_return_w1_cleanup_index,
                                                                    );
                                                                    match cleanup_read {
                                                                        Ok(callback_return_w1_cleanup_row) => {
                                                                            if !callback_return_w1_cleanup_row.is_null() {
                                                                                ::prebindgen_jni_runtime::reclaim_output(
                                                                                    &mut callback_environment,
                                                                                    |callback_environment| {
                                                                                        let mut callback_environment = callback_environment;
                                                                                        let cleanup_read_frame = unsafe {
                                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                &callback_environment,
                                                                                                8,
                                                                                            )
                                                                                        }?;
                                                                                        let cleanup_read = if 0usize
                                                                                            < callback_environment
                                                                                                .get_array_length(&callback_return_w1_cleanup_row)
                                                                                                .map_err(|e| e.to_string())? as usize
                                                                                        {
                                                                                            ::prebindgen_jni_runtime::row_i64(
                                                                                                &mut callback_environment,
                                                                                                &(callback_return_w1_cleanup_row),
                                                                                                0usize,
                                                                                            )
                                                                                        } else {
                                                                                            Err("callback cleanup field is absent".to_owned())
                                                                                        };
                                                                                        match cleanup_read {
                                                                                            Ok(callback_return_w5_f0) => {
                                                                                                unsafe {
                                                                                                    callback_return_owners
                                                                                                        .receive(
                                                                                                            callback_return_w5_f0
                                                                                                                as *mut ::prebindgen_jni_runtime::Allocation<
                                                                                                                    bench_flat::InputHandle,
                                                                                                                >,
                                                                                                        );
                                                                                                }
                                                                                            }
                                                                                            Err(error) => {
                                                                                                if callback_environment
                                                                                                    .exception_check()
                                                                                                    .map_err(|e| e.to_string())?
                                                                                                {
                                                                                                    return Err(error);
                                                                                                }
                                                                                                ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                            }
                                                                                        }
                                                                                        drop(cleanup_read_frame);
                                                                                        Ok(())
                                                                                    },
                                                                                );
                                                                                ::prebindgen_jni_runtime::reclaim_output(
                                                                                    &mut callback_environment,
                                                                                    |callback_environment| {
                                                                                        let mut callback_environment = callback_environment;
                                                                                        let cleanup_read_frame = unsafe {
                                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                &callback_environment,
                                                                                                8,
                                                                                            )
                                                                                        }?;
                                                                                        let cleanup_read = if 1usize
                                                                                            < callback_environment
                                                                                                .get_array_length(&callback_return_w1_cleanup_row)
                                                                                                .map_err(|e| e.to_string())? as usize
                                                                                        {
                                                                                            ::prebindgen_jni_runtime::row_rows(
                                                                                                &mut callback_environment,
                                                                                                &(callback_return_w1_cleanup_row),
                                                                                                1usize,
                                                                                            )
                                                                                        } else {
                                                                                            Err("callback cleanup field is absent".to_owned())
                                                                                        };
                                                                                        match cleanup_read {
                                                                                            Ok(callback_return_w6) => {
                                                                                                if !callback_return_w6.is_null() {
                                                                                                    let length = callback_environment
                                                                                                        .get_array_length(&callback_return_w6)
                                                                                                        .map_err(|e| e.to_string())? as usize;
                                                                                                    for callback_return_w6_cleanup_index in 0..length {
                                                                                                        let frame = unsafe {
                                                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                                &callback_environment,
                                                                                                                10i32,
                                                                                                            )
                                                                                                        }?;
                                                                                                        ::prebindgen_jni_runtime::reclaim_output(
                                                                                                            &mut callback_environment,
                                                                                                            |callback_environment| {
                                                                                                                let mut callback_environment = callback_environment;
                                                                                                                let cleanup_read_frame = unsafe {
                                                                                                                    ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                                        &callback_environment,
                                                                                                                        8,
                                                                                                                    )
                                                                                                                }?;
                                                                                                                let cleanup_read = ::prebindgen_jni_runtime::row_rows(
                                                                                                                    &mut callback_environment,
                                                                                                                    &callback_return_w6,
                                                                                                                    callback_return_w6_cleanup_index,
                                                                                                                );
                                                                                                                match cleanup_read {
                                                                                                                    Ok(callback_return_w6_cleanup_row) => {
                                                                                                                        if !callback_return_w6_cleanup_row.is_null() {
                                                                                                                            ::prebindgen_jni_runtime::reclaim_output(
                                                                                                                                &mut callback_environment,
                                                                                                                                |callback_environment| {
                                                                                                                                    let mut callback_environment = callback_environment;
                                                                                                                                    let cleanup_read_frame = unsafe {
                                                                                                                                        ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                                                            &callback_environment,
                                                                                                                                            8,
                                                                                                                                        )
                                                                                                                                    }?;
                                                                                                                                    let cleanup_read = if 0usize
                                                                                                                                        < callback_environment
                                                                                                                                            .get_array_length(&callback_return_w6_cleanup_row)
                                                                                                                                            .map_err(|e| e.to_string())? as usize
                                                                                                                                    {
                                                                                                                                        ::prebindgen_jni_runtime::row_i64(
                                                                                                                                            &mut callback_environment,
                                                                                                                                            &(callback_return_w6_cleanup_row),
                                                                                                                                            0usize,
                                                                                                                                        )
                                                                                                                                    } else {
                                                                                                                                        Err("callback cleanup field is absent".to_owned())
                                                                                                                                    };
                                                                                                                                    match cleanup_read {
                                                                                                                                        Ok(callback_return_w10_f0) => {
                                                                                                                                            unsafe {
                                                                                                                                                callback_return_owners
                                                                                                                                                    .receive(
                                                                                                                                                        callback_return_w10_f0
                                                                                                                                                            as *mut ::prebindgen_jni_runtime::Allocation<
                                                                                                                                                                bench_flat::InputHandle,
                                                                                                                                                            >,
                                                                                                                                                    );
                                                                                                                                            }
                                                                                                                                        }
                                                                                                                                        Err(error) => {
                                                                                                                                            if callback_environment
                                                                                                                                                .exception_check()
                                                                                                                                                .map_err(|e| e.to_string())?
                                                                                                                                            {
                                                                                                                                                return Err(error);
                                                                                                                                            }
                                                                                                                                            ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                                                                        }
                                                                                                                                    }
                                                                                                                                    drop(cleanup_read_frame);
                                                                                                                                    Ok(())
                                                                                                                                },
                                                                                                                            );
                                                                                                                        }
                                                                                                                    }
                                                                                                                    Err(error) => {
                                                                                                                        if callback_environment
                                                                                                                            .exception_check()
                                                                                                                            .map_err(|e| e.to_string())?
                                                                                                                        {
                                                                                                                            return Err(error);
                                                                                                                        }
                                                                                                                        ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                                                    }
                                                                                                                }
                                                                                                                drop(cleanup_read_frame);
                                                                                                                Ok(())
                                                                                                            },
                                                                                                        );
                                                                                                        drop(frame);
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                            Err(error) => {
                                                                                                if callback_environment
                                                                                                    .exception_check()
                                                                                                    .map_err(|e| e.to_string())?
                                                                                                {
                                                                                                    return Err(error);
                                                                                                }
                                                                                                ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                            }
                                                                                        }
                                                                                        drop(cleanup_read_frame);
                                                                                        Ok(())
                                                                                    },
                                                                                );
                                                                            }
                                                                        }
                                                                        Err(error) => {
                                                                            if callback_environment
                                                                                .exception_check()
                                                                                .map_err(|e| e.to_string())?
                                                                            {
                                                                                return Err(error);
                                                                            }
                                                                            ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                        }
                                                                    }
                                                                    drop(cleanup_read_frame);
                                                                    Ok(())
                                                                },
                                                            );
                                                            drop(frame);
                                                        }
                                                    }
                                                    Ok(())
                                                },
                                            );
                                        }
                                        callback_return_decoded
                                    }
                                }
                            }
                        },
                    )
                    .unwrap_or_else(|error| panic!("callback failed: {}", error))
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = bench_flat::callback_batches_owned_handles(
                __prebindgen_argument_0,
            );
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_d5ee2af5a2eda167>(),
            );
            let ret_w1_source: ::std::vec::Vec<bench_flat::HandleBatch> = source_value;
            let ret_w3 = ::std::vec::Vec::len(&ret_w1_source);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_Range_0643563939dde3e9>(),
                    ::core::mem::align_of::<__PbBulk_i64_Range_0643563939dde3e9>(),
                )?;
            let ret_w1_owners = unsafe {
                ret_w0_writer
                    .own_rows(
                        ret_w1,
                        ::core::mem::size_of::<__PbBulk_i64_Range_0643563939dde3e9>(),
                        __pb_cleanup_native_InputHandle_Rows_64034ad291d2d0cd,
                    )
            };
            for (ret_w4, ret_w2) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w1_source),
            ) {
                let ret_w5_record = ret_w2;
                let mut ret_w5_f0 = Box::new(::prebindgen_jni_runtime::Allocation {
                    value: ret_w5_record.guard,
                });
                let ret_w6_source: ::std::vec::Vec<bench_flat::HandleRecord> = ret_w5_record
                    .rows;
                let ret_w8 = ::std::vec::Vec::len(&ret_w6_source);
                let ret_w6 = ret_w0_writer
                    .rows(
                        ret_w8,
                        ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                        ::core::mem::align_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                    )?;
                let ret_w6_owners = unsafe {
                    ret_w0_writer
                        .own_rows(
                            ret_w6,
                            ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                            __pb_cleanup_native_InputHandle_567ed8299898558e,
                        )
                };
                for (ret_w9, ret_w7) in ::core::iter::Iterator::enumerate(
                    ::core::iter::IntoIterator::into_iter(ret_w6_source),
                ) {
                    let ret_w10_record = ret_w7;
                    let mut ret_w10_f0 = Box::new(::prebindgen_jni_runtime::Allocation {
                        value: ret_w10_record.handle,
                    });
                    let ret_w10_f1 = ret_w10_record.tag;
                    ret_w0_writer
                        .write(
                            (ret_w6.offset as usize
                                + ret_w9
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_b3c08eb92fbbc74c, v0
                                ),
                            ((&mut *ret_w10_f0)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >) as i64,
                        )?;
                    ret_w0_writer
                        .write(
                            (ret_w6.offset as usize
                                + ret_w9
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_b3c08eb92fbbc74c, v1
                                ),
                            ret_w10_f1,
                        )?;
                    #[allow(
                        clippy::redundant_closure_call,
                        clippy::let_unit_value,
                        clippy::unused_unit
                    )]
                    let _: () = (|| -> () {
                        let _ = Box::into_raw(ret_w10_f0).expose_provenance();
                    })();
                    unsafe {
                        ret_w0_writer.commit_rows(ret_w6_owners, ret_w9 + 1);
                    }
                }
                let ret_w6 = Some(ret_w6);
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_Range_0643563939dde3e9,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_Range_0643563939dde3e9, v0
                            ),
                        ((&mut *ret_w5_f0)
                            as *mut ::prebindgen_jni_runtime::Allocation<
                                bench_flat::InputHandle,
                            >) as i64,
                    )?;
                ret_w0_writer
                    .range(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_Range_0643563939dde3e9,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_Range_0643563939dde3e9, v1
                            ),
                        ret_w6,
                    )?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    let _ = Box::into_raw(ret_w5_f0).expose_provenance();
                })();
                unsafe {
                    ret_w0_writer.commit_rows(ret_w1_owners, ret_w4 + 1);
                }
            }
            let ret_w1 = Some(ret_w1);
            ret_w0_writer
                .range(
                    ::core::mem::offset_of!(__PbBulk_Range_d5ee2af5a2eda167, v0),
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1callback_1mixed_1batches_1owned_1handles_12c0ba11b9a082cc3<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "()[Ljava/lang/Object;",
                "bench.plain.OwnedMixedBatchesCallback",
            )?;
            #[allow(unused_mut, clippy::collapsible_if)]
            let p0_callback_value = move || {
                p0_callback_retained
                    .with_env_result(
                        17i32,
                        |callback_environment| {
                            let mut callback_environment = callback_environment;
                            {
                                let arguments = [];
                                {
                                    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
                                    let __prebindgen_input_consumed = &__pb_input_consumed;
                                    let callback_result = unsafe {
                                        p0_callback_retained
                                            .invoke_return(
                                                callback_environment,
                                                ::prebindgen_jni_runtime::jni::signature::ReturnType::Object,
                                                &arguments,
                                            )
                                    }?;
                                    let callback_return_w1 = callback_result
                                        .l()
                                        .map_err(|e| e.to_string())
                                        .map(
                                            ::prebindgen_jni_runtime::jni::objects::JObjectArray::from,
                                        )?;
                                    {
                                        let callback_return_decoded = ::prebindgen_runtime::boundary(||
                                        {
                                            {
                                                let mut callback_return_w0: Vec<bench_flat::HandleBatch> = Vec::new();
                                                if true {
                                                    if callback_return_w1.is_null() {
                                                        return Err("null collection".into());
                                                    }
                                                    let callback_return_w2 = callback_environment
                                                        .get_array_length(&callback_return_w1)
                                                        .map_err(|e| e.to_string())? as usize;
                                                    callback_return_w0.reserve_exact(callback_return_w2);
                                                    for callback_return_w3 in 0..callback_return_w2 {
                                                        let callback_return_w1_frame = unsafe {
                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                &callback_environment,
                                                                11i32,
                                                            )
                                                        }?;
                                                        let callback_return_w1_row = ::prebindgen_jni_runtime::row(
                                                            &mut callback_environment,
                                                            &callback_return_w1,
                                                            callback_return_w3,
                                                            3usize,
                                                        )?;
                                                        let callback_return_w5_f0 = ::prebindgen_jni_runtime::row_i64(
                                                            &mut callback_environment,
                                                            &(callback_return_w1_row),
                                                            0usize,
                                                        );
                                                        if callback_return_w5_f0.is_err()
                                                            && callback_environment
                                                                .exception_check()
                                                                .map_err(|e| e.to_string())?
                                                        {
                                                            return Err("JNI collection field decoding failed".into());
                                                        }
                                                        let callback_return_w6 = ::prebindgen_jni_runtime::row_bytes(
                                                            &mut callback_environment,
                                                            &(callback_return_w1_row),
                                                            1usize,
                                                        );
                                                        if callback_return_w6.is_err()
                                                            && callback_environment
                                                                .exception_check()
                                                                .map_err(|e| e.to_string())?
                                                        {
                                                            return Err("JNI collection field decoding failed".into());
                                                        }
                                                        let callback_return_w6_length = ::prebindgen_jni_runtime::row_i32(
                                                            &mut callback_environment,
                                                            &(callback_return_w1_row),
                                                            2usize,
                                                        );
                                                        if callback_return_w6_length.is_err()
                                                            && callback_environment
                                                                .exception_check()
                                                                .map_err(|e| e.to_string())?
                                                        {
                                                            return Err("JNI collection field decoding failed".into());
                                                        }
                                                        #[allow(clippy::redundant_closure_call)]
                                                        let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                                            bench_flat::HandleBatch,
                                                            String,
                                                        > {
                                                            #[allow(unused_mut, unused_assignments)]
                                                            let mut callback_return_w6_bytes = None;
                                                            {
                                                                let callback_return_w5_f0_pointer = (*callback_return_w5_f0
                                                                    .as_ref()
                                                                    .map_err(|e| e.clone())?)
                                                                    as *mut ::prebindgen_jni_runtime::Allocation<
                                                                        bench_flat::InputHandle,
                                                                    >;
                                                                ::prebindgen_runtime::validate_pointer(
                                                                    callback_return_w5_f0_pointer,
                                                                )?;
                                                                let callback_return_w4_f0: bench_flat::InputHandle = {
                                                                    let callback_return_w4_f0 = unsafe {
                                                                        Box::from_raw(callback_return_w5_f0_pointer)
                                                                    };
                                                                    {
                                                                        __pb_input_consumed
                                                                            .set(__pb_input_consumed.get() + (1usize));
                                                                    }
                                                                    callback_return_w4_f0.value
                                                                };
                                                                let callback_return_w4_field_0 = callback_return_w4_f0;
                                                                #[allow(clippy::redundant_closure_call)]
                                                                let bulk_value = (::prebindgen_jni_runtime::bulk::input)(
                                                                    &mut callback_environment,
                                                                    &(*callback_return_w6.as_ref().map_err(|e| e.clone())?),
                                                                    *callback_return_w6_length.as_ref().map_err(|e| e.clone())?,
                                                                )?;
                                                                callback_return_w6_bytes = Some(bulk_value);
                                                                let callback_return_w6_bytes = callback_return_w6_bytes
                                                                    .as_ref()
                                                                    .expect("active bulk carrier");
                                                                let callback_return_w6_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                                                                    callback_return_w6_bytes,
                                                                );
                                                                if callback_return_w6_bytes.len()
                                                                    < ::core::mem::size_of::<__PbBulk_Range_71300b84e922dbba>()
                                                                {
                                                                    return Err("truncated bulk root record".into());
                                                                }
                                                                let callback_return_w7 = if true {
                                                                    callback_return_w6_reader
                                                                        .range(
                                                                            ::core::mem::offset_of!(__PbBulk_Range_71300b84e922dbba, v0),
                                                                        )?
                                                                } else {
                                                                    None
                                                                };
                                                                let mut callback_return_w4_f1: Vec<
                                                                    bench_flat::HandleRecord,
                                                                > = Vec::new();
                                                                if true {
                                                                    let (callback_return_w7_start, callback_return_w8) = callback_return_w6_reader
                                                                        .rows(
                                                                            callback_return_w7,
                                                                            ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                                                                        )?;
                                                                    callback_return_w4_f1.reserve_exact(callback_return_w8);
                                                                    for callback_return_w9 in 0..callback_return_w8 {
                                                                        let callback_return_w11_f0 = callback_return_w6_reader
                                                                            .read::<
                                                                                i64,
                                                                            >(
                                                                                (callback_return_w7_start
                                                                                    + callback_return_w9
                                                                                        * ::core::mem::size_of::<
                                                                                            __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                                                                        >())
                                                                                    + ::core::mem::offset_of!(
                                                                                        __PbBulk_i64_i64_b3c08eb92fbbc74c, v0
                                                                                    ),
                                                                            )?;
                                                                        let callback_return_w11_f1 = callback_return_w6_reader
                                                                            .read::<
                                                                                i64,
                                                                            >(
                                                                                (callback_return_w7_start
                                                                                    + callback_return_w9
                                                                                        * ::core::mem::size_of::<
                                                                                            __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                                                                        >())
                                                                                    + ::core::mem::offset_of!(
                                                                                        __PbBulk_i64_i64_b3c08eb92fbbc74c, v1
                                                                                    ),
                                                                            )?;
                                                                        #[allow(clippy::redundant_closure_call)]
                                                                        let callback_return_w7_element_built = (|| -> ::core::result::Result<
                                                                            bench_flat::HandleRecord,
                                                                            String,
                                                                        > {
                                                                            {
                                                                                let callback_return_w11_f0_pointer = (callback_return_w11_f0)
                                                                                    as *mut ::prebindgen_jni_runtime::Allocation<
                                                                                        bench_flat::InputHandle,
                                                                                    >;
                                                                                ::prebindgen_runtime::validate_pointer(
                                                                                    callback_return_w11_f0_pointer,
                                                                                )?;
                                                                                let callback_return_w10_f0: bench_flat::InputHandle = {
                                                                                    let callback_return_w10_f0 = unsafe {
                                                                                        Box::from_raw(callback_return_w11_f0_pointer)
                                                                                    };
                                                                                    {
                                                                                        __pb_input_consumed
                                                                                            .set(__pb_input_consumed.get() + (1usize));
                                                                                    }
                                                                                    callback_return_w10_f0.value
                                                                                };
                                                                                let callback_return_w10_field_0 = callback_return_w10_f0;
                                                                                let callback_return_w10_f1: i64 = ::core::convert::Into::into(
                                                                                    callback_return_w11_f1,
                                                                                );
                                                                                let callback_return_w10_field_1 = callback_return_w10_f1;
                                                                                let callback_return_w10: bench_flat::HandleRecord = bench_flat::HandleRecord {
                                                                                    handle: callback_return_w10_field_0,
                                                                                    tag: callback_return_w10_field_1,
                                                                                };
                                                                                let callback_return_w7_row_value = callback_return_w10;
                                                                                Ok(callback_return_w7_row_value)
                                                                            }
                                                                        })();
                                                                        let element = callback_return_w7_element_built?;
                                                                        #[allow(clippy::zst_offset)]
                                                                        unsafe {
                                                                            let initialized = callback_return_w4_f1.len();
                                                                            callback_return_w4_f1
                                                                                .as_mut_ptr()
                                                                                .add(initialized)
                                                                                .write(element);
                                                                            callback_return_w4_f1.set_len(initialized + 1);
                                                                        }
                                                                    }
                                                                }
                                                                let callback_return_w4_field_1 = callback_return_w4_f1;
                                                                let callback_return_w4: bench_flat::HandleBatch = bench_flat::HandleBatch {
                                                                    guard: callback_return_w4_field_0,
                                                                    rows: callback_return_w4_field_1,
                                                                };
                                                                let callback_return_w1_row_value = callback_return_w4;
                                                                Ok(callback_return_w1_row_value)
                                                            }
                                                        })();
                                                        drop(callback_return_w1_frame);
                                                        let element = callback_return_w1_element_built?;
                                                        #[allow(clippy::zst_offset)]
                                                        unsafe {
                                                            let initialized = callback_return_w0.len();
                                                            callback_return_w0
                                                                .as_mut_ptr()
                                                                .add(initialized)
                                                                .write(element);
                                                            callback_return_w0.set_len(initialized + 1);
                                                        }
                                                    }
                                                }
                                                let callback_return_value = callback_return_w0;
                                                Ok::<_, String>(callback_return_value)
                                            }
                                        });
                                        if callback_return_decoded.is_err() {
                                            ::prebindgen_jni_runtime::reclaim_output(
                                                &mut callback_environment,
                                                |callback_environment| {
                                                    let mut callback_environment = callback_environment;
                                                    let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                                        __prebindgen_input_consumed,
                                                    );
                                                    if !callback_return_w1.is_null() {
                                                        let length = callback_environment
                                                            .get_array_length(&callback_return_w1)
                                                            .map_err(|e| e.to_string())? as usize;
                                                        for callback_return_w1_cleanup_index in 0..length {
                                                            let frame = unsafe {
                                                                ::prebindgen_jni_runtime::LocalFrame::new(
                                                                    &callback_environment,
                                                                    11i32,
                                                                )
                                                            }?;
                                                            ::prebindgen_jni_runtime::reclaim_output(
                                                                &mut callback_environment,
                                                                |callback_environment| {
                                                                    let mut callback_environment = callback_environment;
                                                                    let cleanup_read_frame = unsafe {
                                                                        ::prebindgen_jni_runtime::LocalFrame::new(
                                                                            &callback_environment,
                                                                            8,
                                                                        )
                                                                    }?;
                                                                    let cleanup_read = ::prebindgen_jni_runtime::row_rows(
                                                                        &mut callback_environment,
                                                                        &callback_return_w1,
                                                                        callback_return_w1_cleanup_index,
                                                                    );
                                                                    match cleanup_read {
                                                                        Ok(callback_return_w1_cleanup_row) => {
                                                                            if !callback_return_w1_cleanup_row.is_null() {
                                                                                ::prebindgen_jni_runtime::reclaim_output(
                                                                                    &mut callback_environment,
                                                                                    |callback_environment| {
                                                                                        let mut callback_environment = callback_environment;
                                                                                        let cleanup_read_frame = unsafe {
                                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                &callback_environment,
                                                                                                8,
                                                                                            )
                                                                                        }?;
                                                                                        let cleanup_read = if 0usize
                                                                                            < callback_environment
                                                                                                .get_array_length(&callback_return_w1_cleanup_row)
                                                                                                .map_err(|e| e.to_string())? as usize
                                                                                        {
                                                                                            ::prebindgen_jni_runtime::row_i64(
                                                                                                &mut callback_environment,
                                                                                                &(callback_return_w1_cleanup_row),
                                                                                                0usize,
                                                                                            )
                                                                                        } else {
                                                                                            Err("callback cleanup field is absent".to_owned())
                                                                                        };
                                                                                        match cleanup_read {
                                                                                            Ok(callback_return_w5_f0) => {
                                                                                                unsafe {
                                                                                                    callback_return_owners
                                                                                                        .receive(
                                                                                                            callback_return_w5_f0
                                                                                                                as *mut ::prebindgen_jni_runtime::Allocation<
                                                                                                                    bench_flat::InputHandle,
                                                                                                                >,
                                                                                                        );
                                                                                                }
                                                                                            }
                                                                                            Err(error) => {
                                                                                                if callback_environment
                                                                                                    .exception_check()
                                                                                                    .map_err(|e| e.to_string())?
                                                                                                {
                                                                                                    return Err(error);
                                                                                                }
                                                                                                ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                            }
                                                                                        }
                                                                                        drop(cleanup_read_frame);
                                                                                        Ok(())
                                                                                    },
                                                                                );
                                                                                ::prebindgen_jni_runtime::reclaim_output(
                                                                                    &mut callback_environment,
                                                                                    |callback_environment| {
                                                                                        let mut callback_environment = callback_environment;
                                                                                        let cleanup_read_frame = unsafe {
                                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                &callback_environment,
                                                                                                8,
                                                                                            )
                                                                                        }?;
                                                                                        let cleanup_read = if 1usize
                                                                                            < callback_environment
                                                                                                .get_array_length(&callback_return_w1_cleanup_row)
                                                                                                .map_err(|e| e.to_string())? as usize
                                                                                        {
                                                                                            ::prebindgen_jni_runtime::row_bytes(
                                                                                                &mut callback_environment,
                                                                                                &(callback_return_w1_cleanup_row),
                                                                                                1usize,
                                                                                            )
                                                                                        } else {
                                                                                            Err("callback cleanup field is absent".to_owned())
                                                                                        };
                                                                                        match cleanup_read {
                                                                                            Ok(callback_return_w6) => {
                                                                                                if !callback_return_w6.is_null() {
                                                                                                    ::prebindgen_jni_runtime::reclaim_output(
                                                                                                        &mut callback_environment,
                                                                                                        |callback_environment| {
                                                                                                            let mut callback_environment = callback_environment;
                                                                                                            let cleanup_read_frame = unsafe {
                                                                                                                ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                                    &callback_environment,
                                                                                                                    8,
                                                                                                                )
                                                                                                            }?;
                                                                                                            let cleanup_read = {
                                                                                                                let actual = callback_environment
                                                                                                                    .get_array_length(&callback_return_w6)
                                                                                                                    .map_err(|e| e.to_string())?;
                                                                                                                ::prebindgen_jni_runtime::bulk::input(
                                                                                                                    &mut callback_environment,
                                                                                                                    &callback_return_w6,
                                                                                                                    actual,
                                                                                                                )
                                                                                                            };
                                                                                                            match cleanup_read {
                                                                                                                Ok(callback_return_w6_cleanup_bytes) => {
                                                                                                                    let callback_return_w6_cleanup_bytes_reader = ::prebindgen_jni_runtime::bulk::Reader::new(
                                                                                                                        &callback_return_w6_cleanup_bytes,
                                                                                                                    );
                                                                                                                    if let Err(error) = (|| -> ::prebindgen_runtime::Result<()> {
                                                                                                                        let (
                                                                                                                            callback_return_w7_cleanup_start,
                                                                                                                            callback_return_w7_cleanup_count,
                                                                                                                        ) = callback_return_w6_cleanup_bytes_reader
                                                                                                                            .rows(
                                                                                                                                callback_return_w6_cleanup_bytes_reader
                                                                                                                                    .range(
                                                                                                                                        ::core::mem::offset_of!(__PbBulk_Range_71300b84e922dbba, v0),
                                                                                                                                    )?,
                                                                                                                                ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                                                                                                                            )?;
                                                                                                                        for callback_return_w7_cleanup_index in 0..callback_return_w7_cleanup_count {
                                                                                                                            if let Err(error) = (|| -> ::prebindgen_runtime::Result<()> {
                                                                                                                                unsafe {
                                                                                                                                    callback_return_owners
                                                                                                                                        .receive(
                                                                                                                                            (callback_return_w6_cleanup_bytes_reader
                                                                                                                                                .read::<
                                                                                                                                                    i64,
                                                                                                                                                >(
                                                                                                                                                    (callback_return_w7_cleanup_start
                                                                                                                                                        + callback_return_w7_cleanup_index
                                                                                                                                                            * ::core::mem::size_of::<
                                                                                                                                                                __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                                                                                                                                            >())
                                                                                                                                                        + ::core::mem::offset_of!(
                                                                                                                                                            __PbBulk_i64_i64_b3c08eb92fbbc74c, v0
                                                                                                                                                        ),
                                                                                                                                                )?)
                                                                                                                                                as *mut ::prebindgen_jni_runtime::Allocation<
                                                                                                                                                    bench_flat::InputHandle,
                                                                                                                                                >,
                                                                                                                                        );
                                                                                                                                }
                                                                                                                                Ok(())
                                                                                                                            })() {
                                                                                                                                ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                                                            }
                                                                                                                        }
                                                                                                                        Ok(())
                                                                                                                    })() {
                                                                                                                        ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                                                    }
                                                                                                                }
                                                                                                                Err(error) => {
                                                                                                                    if callback_environment
                                                                                                                        .exception_check()
                                                                                                                        .map_err(|e| e.to_string())?
                                                                                                                    {
                                                                                                                        return Err(error);
                                                                                                                    }
                                                                                                                    ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                                                }
                                                                                                            }
                                                                                                            drop(cleanup_read_frame);
                                                                                                            Ok(())
                                                                                                        },
                                                                                                    );
                                                                                                }
                                                                                            }
                                                                                            Err(error) => {
                                                                                                if callback_environment
                                                                                                    .exception_check()
                                                                                                    .map_err(|e| e.to_string())?
                                                                                                {
                                                                                                    return Err(error);
                                                                                                }
                                                                                                ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                            }
                                                                                        }
                                                                                        drop(cleanup_read_frame);
                                                                                        Ok(())
                                                                                    },
                                                                                );
                                                                            }
                                                                        }
                                                                        Err(error) => {
                                                                            if callback_environment
                                                                                .exception_check()
                                                                                .map_err(|e| e.to_string())?
                                                                            {
                                                                                return Err(error);
                                                                            }
                                                                            ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                        }
                                                                    }
                                                                    drop(cleanup_read_frame);
                                                                    Ok(())
                                                                },
                                                            );
                                                            drop(frame);
                                                        }
                                                    }
                                                    Ok(())
                                                },
                                            );
                                        }
                                        callback_return_decoded
                                    }
                                }
                            }
                        },
                    )
                    .unwrap_or_else(|error| panic!("callback failed: {}", error))
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = bench_flat::callback_mixed_batches_owned_handles(
                __prebindgen_argument_0,
            );
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_d5ee2af5a2eda167>(),
            );
            let ret_w1_source: ::std::vec::Vec<bench_flat::HandleBatch> = source_value;
            let ret_w3 = ::std::vec::Vec::len(&ret_w1_source);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_Range_0643563939dde3e9>(),
                    ::core::mem::align_of::<__PbBulk_i64_Range_0643563939dde3e9>(),
                )?;
            let ret_w1_owners = unsafe {
                ret_w0_writer
                    .own_rows(
                        ret_w1,
                        ::core::mem::size_of::<__PbBulk_i64_Range_0643563939dde3e9>(),
                        __pb_cleanup_native_InputHandle_Rows_64034ad291d2d0cd,
                    )
            };
            for (ret_w4, ret_w2) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w1_source),
            ) {
                let ret_w5_record = ret_w2;
                let mut ret_w5_f0 = Box::new(::prebindgen_jni_runtime::Allocation {
                    value: ret_w5_record.guard,
                });
                let ret_w6_source: ::std::vec::Vec<bench_flat::HandleRecord> = ret_w5_record
                    .rows;
                let ret_w8 = ::std::vec::Vec::len(&ret_w6_source);
                let ret_w6 = ret_w0_writer
                    .rows(
                        ret_w8,
                        ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                        ::core::mem::align_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                    )?;
                let ret_w6_owners = unsafe {
                    ret_w0_writer
                        .own_rows(
                            ret_w6,
                            ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                            __pb_cleanup_native_InputHandle_567ed8299898558e,
                        )
                };
                for (ret_w9, ret_w7) in ::core::iter::Iterator::enumerate(
                    ::core::iter::IntoIterator::into_iter(ret_w6_source),
                ) {
                    let ret_w10_record = ret_w7;
                    let mut ret_w10_f0 = Box::new(::prebindgen_jni_runtime::Allocation {
                        value: ret_w10_record.handle,
                    });
                    let ret_w10_f1 = ret_w10_record.tag;
                    ret_w0_writer
                        .write(
                            (ret_w6.offset as usize
                                + ret_w9
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_b3c08eb92fbbc74c, v0
                                ),
                            ((&mut *ret_w10_f0)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >) as i64,
                        )?;
                    ret_w0_writer
                        .write(
                            (ret_w6.offset as usize
                                + ret_w9
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_b3c08eb92fbbc74c, v1
                                ),
                            ret_w10_f1,
                        )?;
                    #[allow(
                        clippy::redundant_closure_call,
                        clippy::let_unit_value,
                        clippy::unused_unit
                    )]
                    let _: () = (|| -> () {
                        let _ = Box::into_raw(ret_w10_f0).expose_provenance();
                    })();
                    unsafe {
                        ret_w0_writer.commit_rows(ret_w6_owners, ret_w9 + 1);
                    }
                }
                let ret_w6 = Some(ret_w6);
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_Range_0643563939dde3e9,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_Range_0643563939dde3e9, v0
                            ),
                        ((&mut *ret_w5_f0)
                            as *mut ::prebindgen_jni_runtime::Allocation<
                                bench_flat::InputHandle,
                            >) as i64,
                    )?;
                ret_w0_writer
                    .range(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_Range_0643563939dde3e9,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_Range_0643563939dde3e9, v1
                            ),
                        ret_w6,
                    )?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    let _ = Box::into_raw(ret_w5_f0).expose_provenance();
                })();
                unsafe {
                    ret_w0_writer.commit_rows(ret_w1_owners, ret_w4 + 1);
                }
            }
            let ret_w1 = Some(ret_w1);
            ret_w0_writer
                .range(
                    ::core::mem::offset_of!(__PbBulk_Range_d5ee2af5a2eda167, v0),
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1input_1handle_1new_1d7aa473c3cabcfa5<
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
            let source_value = bench_flat::input_handle_new(__prebindgen_argument_0);
            let mut ret_w0 = Box::new(::prebindgen_jni_runtime::Allocation {
                value: source_value,
            });
            let __pb_direct_result = unsafe {
                (::prebindgen_jni_runtime::jni::sys::jvalue {
                    j: ((&mut *ret_w0)
                        as *mut ::prebindgen_jni_runtime::Allocation<
                            bench_flat::InputHandle,
                        >) as i64,
                })
                    .j
            };
            let _: () = {
                let _ = Box::into_raw(ret_w0).expose_provenance();
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1input_1handle_1value_1ae1ea4e2f311cdb7<
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
            let p0_w1_pointer = p0_w1
                as *mut ::prebindgen_jni_runtime::Allocation<bench_flat::InputHandle>;
            ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            let p0_w0: &bench_flat::InputHandle = unsafe { &(*p0_w1_pointer).value };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::input_handle_value(__prebindgen_argument_0);
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1buffer_1sizes_109330eed51da0791<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JString<'local>,
    p1_w1: ::prebindgen_jni_runtime::jni::objects::JByteArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            #[allow(unused_mut, unused_assignments)]
            let mut p0_w0 = None;
            p0_w0 = Some(::prebindgen_jni_runtime::text(&mut jni_environment, &p0_w1)?);
            let __prebindgen_argument_0 = ::core::convert::AsRef::<
                str,
            >::as_ref(p0_w0.as_ref().expect("active value"));
            #[allow(unused_mut, unused_assignments)]
            let mut p1_w0 = None;
            p1_w0 = Some(::prebindgen_jni_runtime::bytes(&mut jni_environment, &p1_w1)?);
            let __prebindgen_argument_1 = ::core::convert::AsRef::<
                [u8],
            >::as_ref(p1_w0.as_ref().expect("active value"));
            let source_value = bench_flat::buffer_sizes(
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1selected_1handle_15e0482e736d8e05c<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i32,
    p0_w3: i64,
    p0_w6: i64,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i64 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i64 = Default::default();
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            if !(0..2i32).contains(&p0_w1) {
                return Err("invalid selector".into());
            }
            let p0_w0: i64 = match p0_w1 {
                0i32 => {
                    let p0_w2: i64 = ::core::convert::Into::into(p0_w3);
                    let p0_w0_alternative_0 = p0_w2;
                    p0_w0_alternative_0
                }
                1i32 => {
                    let p0_w6_pointer = p0_w6
                        as *mut ::prebindgen_jni_runtime::Allocation<
                            bench_flat::InputHandle,
                        >;
                    ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                    let p0_w5: &bench_flat::InputHandle = unsafe {
                        &(*p0_w6_pointer).value
                    };
                    let p0_w4_argument_0 = p0_w5;
                    let p0_w4: i64 = bench_flat::input_handle_value(p0_w4_argument_0);
                    let p0_w0_alternative_1 = p0_w4;
                    p0_w0_alternative_1
                }
                _ => unreachable!("validated selector"),
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::selected_handle(__prebindgen_argument_0);
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1handle_1values_1sum_10c275022668af098<
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
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
                    let p0_w6 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        0usize,
                    );
                    if p0_w6.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<i64, String> {
                        {
                            let p0_w6_pointer = (*p0_w6.as_ref().map_err(|e| e.clone())?)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >;
                            ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                            let p0_w5: &bench_flat::InputHandle = unsafe {
                                &(*p0_w6_pointer).value
                            };
                            let p0_w4_argument_0 = p0_w5;
                            let p0_w4: i64 = bench_flat::input_handle_value(
                                p0_w4_argument_0,
                            );
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
                    })();
                    drop(p0_w1_frame);
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::handle_values_sum(__prebindgen_argument_0);
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1handle_1records_1sum_14d3c2822396924df<
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<bench_flat::HandleRecord> = Vec::new();
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
                        ::prebindgen_jni_runtime::LocalFrame::new(
                            &jni_environment,
                            10i32,
                        )
                    }?;
                    let p0_w1_row = ::prebindgen_jni_runtime::row(
                        &mut jni_environment,
                        &p0_w1,
                        p0_w3,
                        2usize,
                    )?;
                    let p0_w7 = ::prebindgen_jni_runtime::row_i64(
                        &mut jni_environment,
                        &(p0_w1_row),
                        0usize,
                    );
                    if p0_w7.is_err()
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
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        bench_flat::HandleRecord,
                        String,
                    > {
                        {
                            let p0_w7_pointer = (*p0_w7.as_ref().map_err(|e| e.clone())?)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >;
                            ::prebindgen_runtime::validate_pointer(p0_w7_pointer)?;
                            let p0_w6: &bench_flat::InputHandle = unsafe {
                                &(*p0_w7_pointer).value
                            };
                            let p0_w4_f0_argument_0 = p0_w6;
                            let p0_w4_f0: bench_flat::InputHandle = bench_flat::input_handle_copy(
                                p0_w4_f0_argument_0,
                            );
                            let p0_w4_field_0 = p0_w4_f0;
                            let p0_w4_f1: i64 = ::core::convert::Into::into(
                                *p0_w5_f1.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_1 = p0_w4_f1;
                            let p0_w4: bench_flat::HandleRecord = bench_flat::HandleRecord {
                                handle: p0_w4_field_0,
                                tag: p0_w4_field_1,
                            };
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
                    })();
                    drop(p0_w1_frame);
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::handle_records_sum(__prebindgen_argument_0);
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1owned_1handles_1echo_1e97ab382b6bce5df<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JObjectArray<'local>,
    ownership_accepted: ::prebindgen_jni_runtime::jni::objects::JLongArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        if jni_environment
            .get_array_length(&ownership_accepted)
            .map_err(|e| e.to_string())? != 1
        {
            return Err("invalid ownership acknowledgement".into());
        }
        __pb_receipt_valid.set(true);
        {
            let mut p0_w0: Vec<bench_flat::InputHandle> = Vec::new();
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
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        bench_flat::InputHandle,
                        String,
                    > {
                        {
                            let p0_w5_pointer = (*p0_w5.as_ref().map_err(|e| e.clone())?)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >;
                            ::prebindgen_runtime::validate_pointer(p0_w5_pointer)?;
                            let p0_w4: bench_flat::InputHandle = {
                                let p0_w4 = unsafe { Box::from_raw(p0_w5_pointer) };
                                {
                                    __pb_input_consumed
                                        .set(__pb_input_consumed.get() + (1usize));
                                }
                                p0_w4.value
                            };
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
                    })();
                    drop(p0_w1_frame);
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::owned_handles_echo(__prebindgen_argument_0);
            let ret_w0_source: ::std::vec::Vec<bench_flat::InputHandle> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            let ret_w0_row_constructor = __pb_row_type_SequenceRow_Long_1_127a3c24b600dbe0
                .get()?;
            let ret_w0 = ::prebindgen_jni_runtime::array(&mut jni_environment, ret_w2)?;
            let mut ret_w0_owner = unsafe {
                ::prebindgen_jni_runtime::OutputRows::new_objects(
                    &mut jni_environment,
                    &ret_w0,
                    __pb_cleanup_typed_InputHandle_b207c5d0d65e1d12,
                )
            };
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let mut ret_w4 = Box::new(::prebindgen_jni_runtime::Allocation {
                    value: ret_w1,
                });
                let ret_w0_row = unsafe {
                    ret_w0_row_constructor
                        .construct(
                            &mut jni_environment,
                            &[
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ((&mut *ret_w4)
                                        as *mut ::prebindgen_jni_runtime::Allocation<
                                            bench_flat::InputHandle,
                                        >) as i64,
                                },
                            ],
                        )
                }?;
                let ret_w0_row = jni_environment.auto_local(ret_w0_row);
                jni_environment
                    .set_object_array_element(&ret_w0, ret_w3 as i32, &ret_w0_row)
                    .map_err(|e| e.to_string())?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    let _ = Box::into_raw(ret_w4).expose_provenance();
                })();
                unsafe {
                    ret_w0_owner.complete_row();
                }
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
                ret_w0_owner.commit();
                direct_output = __pb_direct_result;
                success_transferred = true;
            };
            Ok(__pb_direct_result)
        }
    });
    if __pb_receipt_valid.get() {
        let count = __pb_input_consumed.get() as i64;
        let value = if outcome.is_ok() { count } else { -1 - count };
        ::prebindgen_jni_runtime::write_input_receipt(
            &mut jni_environment,
            &ownership_accepted,
            value,
            outcome.is_err(),
        );
    }
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1owned_1handle_1records_1echo_17223d199891bf713<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JObjectArray<'local>,
    ownership_accepted: ::prebindgen_jni_runtime::jni::objects::JLongArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        if jni_environment
            .get_array_length(&ownership_accepted)
            .map_err(|e| e.to_string())? != 1
        {
            return Err("invalid ownership acknowledgement".into());
        }
        __pb_receipt_valid.set(true);
        {
            let mut p0_w0: Vec<bench_flat::HandleRecord> = Vec::new();
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
                        ::prebindgen_jni_runtime::LocalFrame::new(
                            &jni_environment,
                            10i32,
                        )
                    }?;
                    let p0_w1_row = ::prebindgen_jni_runtime::row(
                        &mut jni_environment,
                        &p0_w1,
                        p0_w3,
                        2usize,
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
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        bench_flat::HandleRecord,
                        String,
                    > {
                        {
                            let p0_w5_f0_pointer = (*p0_w5_f0
                                .as_ref()
                                .map_err(|e| e.clone())?)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >;
                            ::prebindgen_runtime::validate_pointer(p0_w5_f0_pointer)?;
                            let p0_w4_f0: bench_flat::InputHandle = {
                                let p0_w4_f0 = unsafe { Box::from_raw(p0_w5_f0_pointer) };
                                {
                                    __pb_input_consumed
                                        .set(__pb_input_consumed.get() + (1usize));
                                }
                                p0_w4_f0.value
                            };
                            let p0_w4_field_0 = p0_w4_f0;
                            let p0_w4_f1: i64 = ::core::convert::Into::into(
                                *p0_w5_f1.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_1 = p0_w4_f1;
                            let p0_w4: bench_flat::HandleRecord = bench_flat::HandleRecord {
                                handle: p0_w4_field_0,
                                tag: p0_w4_field_1,
                            };
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
                    })();
                    drop(p0_w1_frame);
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::owned_handle_records_echo(
                __prebindgen_argument_0,
            );
            let ret_w0_source: ::std::vec::Vec<bench_flat::HandleRecord> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            let ret_w0_row_constructor = __pb_row_type_SequenceRow_Long_Long_2_c3e8677e51c52ca2
                .get()?;
            let ret_w0 = ::prebindgen_jni_runtime::array(&mut jni_environment, ret_w2)?;
            let mut ret_w0_owner = unsafe {
                ::prebindgen_jni_runtime::OutputRows::new_objects(
                    &mut jni_environment,
                    &ret_w0,
                    __pb_cleanup_typed_InputHandle_a3d87635baf7af7c,
                )
            };
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w4_record = ret_w1;
                let mut ret_w4_f0 = Box::new(::prebindgen_jni_runtime::Allocation {
                    value: ret_w4_record.handle,
                });
                let ret_w4_f1 = ret_w4_record.tag;
                let ret_w0_row = unsafe {
                    ret_w0_row_constructor
                        .construct(
                            &mut jni_environment,
                            &[
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ((&mut *ret_w4_f0)
                                        as *mut ::prebindgen_jni_runtime::Allocation<
                                            bench_flat::InputHandle,
                                        >) as i64,
                                },
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ret_w4_f1,
                                },
                            ],
                        )
                }?;
                let ret_w0_row = jni_environment.auto_local(ret_w0_row);
                jni_environment
                    .set_object_array_element(&ret_w0, ret_w3 as i32, &ret_w0_row)
                    .map_err(|e| e.to_string())?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    let _ = Box::into_raw(ret_w4_f0).expose_provenance();
                })();
                unsafe {
                    ret_w0_owner.complete_row();
                }
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
                ret_w0_owner.commit();
                direct_output = __pb_direct_result;
                success_transferred = true;
            };
            Ok(__pb_direct_result)
        }
    });
    if __pb_receipt_valid.get() {
        let count = __pb_input_consumed.get() as i64;
        let value = if outcome.is_ok() { count } else { -1 - count };
        ::prebindgen_jni_runtime::write_input_receipt(
            &mut jni_environment,
            &ownership_accepted,
            value,
            outcome.is_err(),
        );
    }
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1callback_1owned_1records_159a5e85070935754<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "()[Ljava/lang/Object;",
                "bench.plain.OwnedRecordsCallback",
            )?;
            #[allow(unused_mut, clippy::collapsible_if)]
            let p0_callback_value = move || {
                p0_callback_retained
                    .with_env_result(
                        17i32,
                        |callback_environment| {
                            let mut callback_environment = callback_environment;
                            {
                                let arguments = [];
                                {
                                    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
                                    let __prebindgen_input_consumed = &__pb_input_consumed;
                                    let callback_result = unsafe {
                                        p0_callback_retained
                                            .invoke_return(
                                                callback_environment,
                                                ::prebindgen_jni_runtime::jni::signature::ReturnType::Object,
                                                &arguments,
                                            )
                                    }?;
                                    let callback_return_w1 = callback_result
                                        .l()
                                        .map_err(|e| e.to_string())
                                        .map(
                                            ::prebindgen_jni_runtime::jni::objects::JObjectArray::from,
                                        )?;
                                    {
                                        let callback_return_decoded = ::prebindgen_runtime::boundary(||
                                        {
                                            {
                                                let mut callback_return_w0: Vec<bench_flat::HandleRecord> = Vec::new();
                                                if true {
                                                    if callback_return_w1.is_null() {
                                                        return Err("null collection".into());
                                                    }
                                                    let callback_return_w2 = callback_environment
                                                        .get_array_length(&callback_return_w1)
                                                        .map_err(|e| e.to_string())? as usize;
                                                    callback_return_w0.reserve_exact(callback_return_w2);
                                                    for callback_return_w3 in 0..callback_return_w2 {
                                                        let callback_return_w1_frame = unsafe {
                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                &callback_environment,
                                                                10i32,
                                                            )
                                                        }?;
                                                        let callback_return_w1_row = ::prebindgen_jni_runtime::row(
                                                            &mut callback_environment,
                                                            &callback_return_w1,
                                                            callback_return_w3,
                                                            2usize,
                                                        )?;
                                                        let callback_return_w5_f0 = ::prebindgen_jni_runtime::row_i64(
                                                            &mut callback_environment,
                                                            &(callback_return_w1_row),
                                                            0usize,
                                                        );
                                                        if callback_return_w5_f0.is_err()
                                                            && callback_environment
                                                                .exception_check()
                                                                .map_err(|e| e.to_string())?
                                                        {
                                                            return Err("JNI collection field decoding failed".into());
                                                        }
                                                        let callback_return_w5_f1 = ::prebindgen_jni_runtime::row_i64(
                                                            &mut callback_environment,
                                                            &(callback_return_w1_row),
                                                            1usize,
                                                        );
                                                        if callback_return_w5_f1.is_err()
                                                            && callback_environment
                                                                .exception_check()
                                                                .map_err(|e| e.to_string())?
                                                        {
                                                            return Err("JNI collection field decoding failed".into());
                                                        }
                                                        #[allow(clippy::redundant_closure_call)]
                                                        let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                                            bench_flat::HandleRecord,
                                                            String,
                                                        > {
                                                            {
                                                                let callback_return_w5_f0_pointer = (*callback_return_w5_f0
                                                                    .as_ref()
                                                                    .map_err(|e| e.clone())?)
                                                                    as *mut ::prebindgen_jni_runtime::Allocation<
                                                                        bench_flat::InputHandle,
                                                                    >;
                                                                ::prebindgen_runtime::validate_pointer(
                                                                    callback_return_w5_f0_pointer,
                                                                )?;
                                                                let callback_return_w4_f0: bench_flat::InputHandle = {
                                                                    let callback_return_w4_f0 = unsafe {
                                                                        Box::from_raw(callback_return_w5_f0_pointer)
                                                                    };
                                                                    {
                                                                        __pb_input_consumed
                                                                            .set(__pb_input_consumed.get() + (1usize));
                                                                    }
                                                                    callback_return_w4_f0.value
                                                                };
                                                                let callback_return_w4_field_0 = callback_return_w4_f0;
                                                                let callback_return_w4_f1: i64 = ::core::convert::Into::into(
                                                                    *callback_return_w5_f1.as_ref().map_err(|e| e.clone())?,
                                                                );
                                                                let callback_return_w4_field_1 = callback_return_w4_f1;
                                                                let callback_return_w4: bench_flat::HandleRecord = bench_flat::HandleRecord {
                                                                    handle: callback_return_w4_field_0,
                                                                    tag: callback_return_w4_field_1,
                                                                };
                                                                let callback_return_w1_row_value = callback_return_w4;
                                                                Ok(callback_return_w1_row_value)
                                                            }
                                                        })();
                                                        drop(callback_return_w1_frame);
                                                        let element = callback_return_w1_element_built?;
                                                        #[allow(clippy::zst_offset)]
                                                        unsafe {
                                                            let initialized = callback_return_w0.len();
                                                            callback_return_w0
                                                                .as_mut_ptr()
                                                                .add(initialized)
                                                                .write(element);
                                                            callback_return_w0.set_len(initialized + 1);
                                                        }
                                                    }
                                                }
                                                let callback_return_value = callback_return_w0;
                                                Ok::<_, String>(callback_return_value)
                                            }
                                        });
                                        if callback_return_decoded.is_err() {
                                            ::prebindgen_jni_runtime::reclaim_output(
                                                &mut callback_environment,
                                                |callback_environment| {
                                                    let mut callback_environment = callback_environment;
                                                    let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                                        __prebindgen_input_consumed,
                                                    );
                                                    if !callback_return_w1.is_null() {
                                                        let length = callback_environment
                                                            .get_array_length(&callback_return_w1)
                                                            .map_err(|e| e.to_string())? as usize;
                                                        for callback_return_w1_cleanup_index in 0..length {
                                                            let frame = unsafe {
                                                                ::prebindgen_jni_runtime::LocalFrame::new(
                                                                    &callback_environment,
                                                                    10i32,
                                                                )
                                                            }?;
                                                            ::prebindgen_jni_runtime::reclaim_output(
                                                                &mut callback_environment,
                                                                |callback_environment| {
                                                                    let mut callback_environment = callback_environment;
                                                                    let cleanup_read_frame = unsafe {
                                                                        ::prebindgen_jni_runtime::LocalFrame::new(
                                                                            &callback_environment,
                                                                            8,
                                                                        )
                                                                    }?;
                                                                    let cleanup_read = ::prebindgen_jni_runtime::row_rows(
                                                                        &mut callback_environment,
                                                                        &callback_return_w1,
                                                                        callback_return_w1_cleanup_index,
                                                                    );
                                                                    match cleanup_read {
                                                                        Ok(callback_return_w1_cleanup_row) => {
                                                                            if !callback_return_w1_cleanup_row.is_null() {
                                                                                ::prebindgen_jni_runtime::reclaim_output(
                                                                                    &mut callback_environment,
                                                                                    |callback_environment| {
                                                                                        let mut callback_environment = callback_environment;
                                                                                        let cleanup_read_frame = unsafe {
                                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                &callback_environment,
                                                                                                8,
                                                                                            )
                                                                                        }?;
                                                                                        let cleanup_read = if 0usize
                                                                                            < callback_environment
                                                                                                .get_array_length(&callback_return_w1_cleanup_row)
                                                                                                .map_err(|e| e.to_string())? as usize
                                                                                        {
                                                                                            ::prebindgen_jni_runtime::row_i64(
                                                                                                &mut callback_environment,
                                                                                                &(callback_return_w1_cleanup_row),
                                                                                                0usize,
                                                                                            )
                                                                                        } else {
                                                                                            Err("callback cleanup field is absent".to_owned())
                                                                                        };
                                                                                        match cleanup_read {
                                                                                            Ok(callback_return_w5_f0) => {
                                                                                                unsafe {
                                                                                                    callback_return_owners
                                                                                                        .receive(
                                                                                                            callback_return_w5_f0
                                                                                                                as *mut ::prebindgen_jni_runtime::Allocation<
                                                                                                                    bench_flat::InputHandle,
                                                                                                                >,
                                                                                                        );
                                                                                                }
                                                                                            }
                                                                                            Err(error) => {
                                                                                                if callback_environment
                                                                                                    .exception_check()
                                                                                                    .map_err(|e| e.to_string())?
                                                                                                {
                                                                                                    return Err(error);
                                                                                                }
                                                                                                ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                            }
                                                                                        }
                                                                                        drop(cleanup_read_frame);
                                                                                        Ok(())
                                                                                    },
                                                                                );
                                                                            }
                                                                        }
                                                                        Err(error) => {
                                                                            if callback_environment
                                                                                .exception_check()
                                                                                .map_err(|e| e.to_string())?
                                                                            {
                                                                                return Err(error);
                                                                            }
                                                                            ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                        }
                                                                    }
                                                                    drop(cleanup_read_frame);
                                                                    Ok(())
                                                                },
                                                            );
                                                            drop(frame);
                                                        }
                                                    }
                                                    Ok(())
                                                },
                                            );
                                        }
                                        callback_return_decoded
                                    }
                                }
                            }
                        },
                    )
                    .unwrap_or_else(|error| panic!("callback failed: {}", error))
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = bench_flat::callback_owned_records(
                __prebindgen_argument_0,
            );
            let ret_w0_source: ::std::vec::Vec<bench_flat::HandleRecord> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            let ret_w0_row_constructor = __pb_row_type_SequenceRow_Long_Long_2_c3e8677e51c52ca2
                .get()?;
            let ret_w0 = ::prebindgen_jni_runtime::array(&mut jni_environment, ret_w2)?;
            let mut ret_w0_owner = unsafe {
                ::prebindgen_jni_runtime::OutputRows::new_objects(
                    &mut jni_environment,
                    &ret_w0,
                    __pb_cleanup_typed_InputHandle_a3d87635baf7af7c,
                )
            };
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w4_record = ret_w1;
                let mut ret_w4_f0 = Box::new(::prebindgen_jni_runtime::Allocation {
                    value: ret_w4_record.handle,
                });
                let ret_w4_f1 = ret_w4_record.tag;
                let ret_w0_row = unsafe {
                    ret_w0_row_constructor
                        .construct(
                            &mut jni_environment,
                            &[
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ((&mut *ret_w4_f0)
                                        as *mut ::prebindgen_jni_runtime::Allocation<
                                            bench_flat::InputHandle,
                                        >) as i64,
                                },
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ret_w4_f1,
                                },
                            ],
                        )
                }?;
                let ret_w0_row = jni_environment.auto_local(ret_w0_row);
                jni_environment
                    .set_object_array_element(&ret_w0, ret_w3 as i32, &ret_w0_row)
                    .map_err(|e| e.to_string())?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    let _ = Box::into_raw(ret_w4_f0).expose_provenance();
                })();
                unsafe {
                    ret_w0_owner.complete_row();
                }
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
                ret_w0_owner.commit();
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1callback_1owned_1record_1c829b73c0af8c75b<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "()[Ljava/lang/Object;",
                "bench.plain.OwnedRecordCallback",
            )?;
            #[allow(unused_mut, clippy::collapsible_if)]
            let p0_callback_value = move || {
                p0_callback_retained
                    .with_env_result(
                        19i32,
                        |callback_environment| {
                            let mut callback_environment = callback_environment;
                            {
                                let arguments = [];
                                {
                                    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
                                    let __prebindgen_input_consumed = &__pb_input_consumed;
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
                                    let mut callback_return_cleanup = ::prebindgen_runtime::CallbackReturnCleanup::new(||
                                    {
                                        let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                            __prebindgen_input_consumed,
                                        );
                                        unsafe {
                                            callback_return_owners
                                                .receive(
                                                    callback_return_w1_f0
                                                        as *mut ::prebindgen_jni_runtime::Allocation<
                                                            bench_flat::InputHandle,
                                                        >,
                                                );
                                        }
                                        Ok(())
                                    });
                                    let callback_return_w1_f1 = ::prebindgen_jni_runtime::row_i64(
                                        &mut callback_environment,
                                        &(callback_result_rows),
                                        1usize,
                                    )?;
                                    {
                                        let callback_return_decoded = ({
                                            let callback_return_w1_f0_pointer = callback_return_w1_f0
                                                as *mut ::prebindgen_jni_runtime::Allocation<
                                                    bench_flat::InputHandle,
                                                >;
                                            ::prebindgen_runtime::validate_pointer(
                                                callback_return_w1_f0_pointer,
                                            )?;
                                            let callback_return_w0_f0: bench_flat::InputHandle = {
                                                let callback_return_w0_f0 = unsafe {
                                                    Box::from_raw(callback_return_w1_f0_pointer)
                                                };
                                                {
                                                    __pb_input_consumed
                                                        .set(__pb_input_consumed.get() + (1usize));
                                                }
                                                callback_return_w0_f0.value
                                            };
                                            let callback_return_w0_field_0 = callback_return_w0_f0;
                                            let callback_return_w0_f1: i64 = ::core::convert::Into::into(
                                                callback_return_w1_f1,
                                            );
                                            let callback_return_w0_field_1 = callback_return_w0_f1;
                                            let callback_return_w0: bench_flat::HandleRecord = bench_flat::HandleRecord {
                                                handle: callback_return_w0_field_0,
                                                tag: callback_return_w0_field_1,
                                            };
                                            let callback_return_value = callback_return_w0;
                                            Ok::<_, String>(callback_return_value)
                                        })?;
                                        callback_return_cleanup.commit();
                                        Ok::<_, String>(callback_return_decoded)
                                    }
                                }
                            }
                        },
                    )
                    .unwrap_or_else(|error| panic!("callback failed: {}", error))
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = bench_flat::callback_owned_record(
                __prebindgen_argument_0,
            );
            let ret_w0_record = source_value;
            let mut ret_w0_f0 = Box::new(::prebindgen_jni_runtime::Allocation {
                value: ret_w0_record.handle,
            });
            let ret_w0_f1 = ret_w0_record.tag;
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
                                j: ((&mut *ret_w0_f0)
                                    as *mut ::prebindgen_jni_runtime::Allocation<
                                        bench_flat::InputHandle,
                                    >) as i64,
                            },
                            ::prebindgen_jni_runtime::jni::sys::jvalue {
                                j: ret_w0_f1,
                            },
                        ],
                    )
            }?;
            let _: () = {
                let _ = Box::into_raw(ret_w0_f0).expose_provenance();
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1owned_1buffer_1records_1echo_132801b2928e6b02f<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JObjectArray<'local>,
    ownership_accepted: ::prebindgen_jni_runtime::jni::objects::JLongArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        if jni_environment
            .get_array_length(&ownership_accepted)
            .map_err(|e| e.to_string())? != 1
        {
            return Err("invalid ownership acknowledgement".into());
        }
        __pb_receipt_valid.set(true);
        {
            let mut p0_w0: Vec<bench_flat::BufferRecord> = Vec::new();
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
                        ::prebindgen_jni_runtime::LocalFrame::new(
                            &jni_environment,
                            11i32,
                        )
                    }?;
                    let p0_w1_row = ::prebindgen_jni_runtime::row(
                        &mut jni_environment,
                        &p0_w1,
                        p0_w3,
                        3usize,
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
                    let p0_w5_f1 = ::prebindgen_jni_runtime::row_text(
                        &mut jni_environment,
                        &(p0_w1_row),
                        1usize,
                    );
                    if p0_w5_f1.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    let p0_w5_f2 = ::prebindgen_jni_runtime::row_bytes(
                        &mut jni_environment,
                        &(p0_w1_row),
                        2usize,
                    );
                    if p0_w5_f2.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        bench_flat::BufferRecord,
                        String,
                    > {
                        {
                            let p0_w5_f0_pointer = (*p0_w5_f0
                                .as_ref()
                                .map_err(|e| e.clone())?)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >;
                            ::prebindgen_runtime::validate_pointer(p0_w5_f0_pointer)?;
                            let p0_w4_f0: bench_flat::InputHandle = {
                                let p0_w4_f0 = unsafe { Box::from_raw(p0_w5_f0_pointer) };
                                {
                                    __pb_input_consumed
                                        .set(__pb_input_consumed.get() + (1usize));
                                }
                                p0_w4_f0.value
                            };
                            let p0_w4_field_0 = p0_w4_f0;
                            let p0_w4_f1: String = ::core::convert::Into::into(
                                ::prebindgen_jni_runtime::text(
                                    &mut jni_environment,
                                    &(*p0_w5_f1.as_ref().map_err(|e| e.clone())?),
                                )?,
                            );
                            let p0_w4_field_1 = p0_w4_f1;
                            let p0_w4_f2: Vec<u8> = ::core::convert::Into::into(
                                ::prebindgen_jni_runtime::bytes(
                                    &mut jni_environment,
                                    &(*p0_w5_f2.as_ref().map_err(|e| e.clone())?),
                                )?,
                            );
                            let p0_w4_field_2 = p0_w4_f2;
                            let p0_w4: bench_flat::BufferRecord = bench_flat::BufferRecord {
                                handle: p0_w4_field_0,
                                text: p0_w4_field_1,
                                bytes: p0_w4_field_2,
                            };
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
                    })();
                    drop(p0_w1_frame);
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::owned_buffer_records_echo(
                __prebindgen_argument_0,
            );
            let ret_w0_source: ::std::vec::Vec<bench_flat::BufferRecord> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            let ret_w0_row_constructor = __pb_row_type_SequenceRow_Long_String_ByteArray_3_e5ce0d0d57155c04
                .get()?;
            let ret_w0 = ::prebindgen_jni_runtime::array(&mut jni_environment, ret_w2)?;
            let mut ret_w0_owner = unsafe {
                ::prebindgen_jni_runtime::OutputRows::new_objects(
                    &mut jni_environment,
                    &ret_w0,
                    __pb_cleanup_typed_InputHandle_68cf0461fb68be7c,
                )
            };
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w0_frame = unsafe {
                    ::prebindgen_jni_runtime::LocalFrame::new(&jni_environment, 11i32)
                }?;
                let ret_w4_record = ret_w1;
                let mut ret_w4_f0 = Box::new(::prebindgen_jni_runtime::Allocation {
                    value: ret_w4_record.handle,
                });
                let ret_w4_f1: ::prebindgen_jni_runtime::jni::objects::JObject<'_> = {
                    let ret_w4_f1 = ret_w4_record.text;
                    ::prebindgen_jni_runtime::text_object(
                        &mut jni_environment,
                        ::core::convert::AsRef::<str>::as_ref(&ret_w4_f1),
                    )?
                };
                let ret_w4_f2: ::prebindgen_jni_runtime::jni::objects::JObject<'_> = {
                    let ret_w4_f2 = ret_w4_record.bytes;
                    ::prebindgen_jni_runtime::bytes_object(
                        &mut jni_environment,
                        ::core::convert::AsRef::<[u8]>::as_ref(&ret_w4_f2),
                    )?
                };
                let ret_w0_row_argument_1 = ret_w4_f1;
                let ret_w0_row_argument_2 = ret_w4_f2;
                let ret_w0_row = unsafe {
                    ret_w0_row_constructor
                        .construct(
                            &mut jni_environment,
                            &[
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    j: ((&mut *ret_w4_f0)
                                        as *mut ::prebindgen_jni_runtime::Allocation<
                                            bench_flat::InputHandle,
                                        >) as i64,
                                },
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    l: ret_w0_row_argument_1.as_raw(),
                                },
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    l: ret_w0_row_argument_2.as_raw(),
                                },
                            ],
                        )
                }?;
                jni_environment
                    .set_object_array_element(&ret_w0, ret_w3 as i32, &ret_w0_row)
                    .map_err(|e| e.to_string())?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    let _ = Box::into_raw(ret_w4_f0).expose_provenance();
                })();
                unsafe {
                    ret_w0_owner.complete_row();
                }
                drop(ret_w0_frame);
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
                ret_w0_owner.commit();
                direct_output = __pb_direct_result;
                success_transferred = true;
            };
            Ok(__pb_direct_result)
        }
    });
    if __pb_receipt_valid.get() {
        let count = __pb_input_consumed.get() as i64;
        let value = if outcome.is_ok() { count } else { -1 - count };
        ::prebindgen_jni_runtime::write_input_receipt(
            &mut jni_environment,
            &ownership_accepted,
            value,
            outcome.is_err(),
        );
    }
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1nested_1owned_1handles_1echo_1656072b3182d8e7b<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JObjectArray<'local>,
    ownership_accepted: ::prebindgen_jni_runtime::jni::objects::JLongArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        if jni_environment
            .get_array_length(&ownership_accepted)
            .map_err(|e| e.to_string())? != 1
        {
            return Err("invalid ownership acknowledgement".into());
        }
        __pb_receipt_valid.set(true);
        {
            let mut p0_w0: Vec<Vec<bench_flat::InputHandle>> = Vec::new();
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
                    let p0_w5 = ::prebindgen_jni_runtime::row_rows(
                        &mut jni_environment,
                        &(p0_w1_row),
                        0usize,
                    );
                    if p0_w5.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        Vec<bench_flat::InputHandle>,
                        String,
                    > {
                        {
                            let mut p0_w4: Vec<bench_flat::InputHandle> = Vec::new();
                            if true {
                                if (*p0_w5.as_ref().map_err(|e| e.clone())?).is_null() {
                                    return Err("null collection".into());
                                }
                                let p0_w6 = jni_environment
                                    .get_array_length(
                                        &(*p0_w5.as_ref().map_err(|e| e.clone())?),
                                    )
                                    .map_err(|e| e.to_string())? as usize;
                                p0_w4.reserve_exact(p0_w6);
                                for p0_w7 in 0..p0_w6 {
                                    let p0_w5_frame = unsafe {
                                        ::prebindgen_jni_runtime::LocalFrame::new(
                                            &jni_environment,
                                            9i32,
                                        )
                                    }?;
                                    let p0_w5_row = ::prebindgen_jni_runtime::row(
                                        &mut jni_environment,
                                        &(*p0_w5.as_ref().map_err(|e| e.clone())?),
                                        p0_w7,
                                        1usize,
                                    )?;
                                    let p0_w9 = ::prebindgen_jni_runtime::row_i64(
                                        &mut jni_environment,
                                        &(p0_w5_row),
                                        0usize,
                                    );
                                    if p0_w9.is_err()
                                        && jni_environment
                                            .exception_check()
                                            .map_err(|e| e.to_string())?
                                    {
                                        return Err("JNI collection field decoding failed".into());
                                    }
                                    #[allow(clippy::redundant_closure_call)]
                                    let p0_w5_element_built = (|| -> ::core::result::Result<
                                        bench_flat::InputHandle,
                                        String,
                                    > {
                                        {
                                            let p0_w9_pointer = (*p0_w9
                                                .as_ref()
                                                .map_err(|e| e.clone())?)
                                                as *mut ::prebindgen_jni_runtime::Allocation<
                                                    bench_flat::InputHandle,
                                                >;
                                            ::prebindgen_runtime::validate_pointer(p0_w9_pointer)?;
                                            let p0_w8: bench_flat::InputHandle = {
                                                let p0_w8 = unsafe { Box::from_raw(p0_w9_pointer) };
                                                {
                                                    __pb_input_consumed
                                                        .set(__pb_input_consumed.get() + (1usize));
                                                }
                                                p0_w8.value
                                            };
                                            let p0_w5_row_value = p0_w8;
                                            Ok(p0_w5_row_value)
                                        }
                                    })();
                                    drop(p0_w5_frame);
                                    let element = p0_w5_element_built?;
                                    #[allow(clippy::zst_offset)]
                                    unsafe {
                                        let initialized = p0_w4.len();
                                        p0_w4.as_mut_ptr().add(initialized).write(element);
                                        p0_w4.set_len(initialized + 1);
                                    }
                                }
                            }
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
                    })();
                    drop(p0_w1_frame);
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::nested_owned_handles_echo(
                __prebindgen_argument_0,
            );
            let ret_w0_source: ::std::vec::Vec<Vec<bench_flat::InputHandle>> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            let ret_w0_row_constructor = __pb_row_type_SequenceRow_Array_Any___1_00f39593b4ea53ea
                .get()?;
            let ret_w0 = ::prebindgen_jni_runtime::array(&mut jni_environment, ret_w2)?;
            let mut ret_w0_owner = unsafe {
                ::prebindgen_jni_runtime::OutputRows::new_objects(
                    &mut jni_environment,
                    &ret_w0,
                    __pb_cleanup_typed_Rows_fe4a4c800c91ea9c,
                )
            };
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w0_frame = unsafe {
                    ::prebindgen_jni_runtime::LocalFrame::new(&jni_environment, 9i32)
                }?;
                let ret_w4_source: ::std::vec::Vec<bench_flat::InputHandle> = ret_w1;
                let ret_w6 = ::std::vec::Vec::len(&ret_w4_source);
                let ret_w4_row_constructor = __pb_row_type_SequenceRow_Long_1_127a3c24b600dbe0
                    .get()?;
                let ret_w4 = ::prebindgen_jni_runtime::array(
                    &mut jni_environment,
                    ret_w6,
                )?;
                let mut ret_w4_owner = unsafe {
                    ::prebindgen_jni_runtime::OutputRows::new_objects(
                        &mut jni_environment,
                        &ret_w4,
                        __pb_cleanup_typed_InputHandle_b207c5d0d65e1d12,
                    )
                };
                for (ret_w7, ret_w5) in ::core::iter::Iterator::enumerate(
                    ::core::iter::IntoIterator::into_iter(ret_w4_source),
                ) {
                    let mut ret_w8 = Box::new(::prebindgen_jni_runtime::Allocation {
                        value: ret_w5,
                    });
                    let ret_w4_row = unsafe {
                        ret_w4_row_constructor
                            .construct(
                                &mut jni_environment,
                                &[
                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                        j: ((&mut *ret_w8)
                                            as *mut ::prebindgen_jni_runtime::Allocation<
                                                bench_flat::InputHandle,
                                            >) as i64,
                                    },
                                ],
                            )
                    }?;
                    let ret_w4_row = jni_environment.auto_local(ret_w4_row);
                    jni_environment
                        .set_object_array_element(&ret_w4, ret_w7 as i32, &ret_w4_row)
                        .map_err(|e| e.to_string())?;
                    #[allow(
                        clippy::redundant_closure_call,
                        clippy::let_unit_value,
                        clippy::unused_unit
                    )]
                    let _: () = (|| -> () {
                        let _ = Box::into_raw(ret_w8).expose_provenance();
                    })();
                    unsafe {
                        ret_w4_owner.complete_row();
                    }
                }
                let ret_w4 = Some(ret_w4);
                let ret_w0_row_argument_0 = match (ret_w4).as_ref() {
                    Some(value) => {
                        jni_environment.new_local_ref(value).map_err(|e| e.to_string())?
                    }
                    None => ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                };
                let ret_w0_row = unsafe {
                    ret_w0_row_constructor
                        .construct(
                            &mut jni_environment,
                            &[
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    l: ret_w0_row_argument_0.as_raw(),
                                },
                            ],
                        )
                }?;
                jni_environment
                    .set_object_array_element(&ret_w0, ret_w3 as i32, &ret_w0_row)
                    .map_err(|e| e.to_string())?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    ret_w4_owner.commit();
                })();
                unsafe {
                    ret_w0_owner.complete_row();
                }
                drop(ret_w0_frame);
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
                ret_w0_owner.commit();
                direct_output = __pb_direct_result;
                success_transferred = true;
            };
            Ok(__pb_direct_result)
        }
    });
    if __pb_receipt_valid.get() {
        let count = __pb_input_consumed.get() as i64;
        let value = if outcome.is_ok() { count } else { -1 - count };
        ::prebindgen_jni_runtime::write_input_receipt(
            &mut jni_environment,
            &ownership_accepted,
            value,
            outcome.is_err(),
        );
    }
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1callback_1nested_1owned_1handles_145191dd02ebf0e07<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "()[Ljava/lang/Object;",
                "bench.plain.OwnedNestedCallback",
            )?;
            #[allow(unused_mut, clippy::collapsible_if)]
            let p0_callback_value = move || {
                p0_callback_retained
                    .with_env_result(
                        17i32,
                        |callback_environment| {
                            let mut callback_environment = callback_environment;
                            {
                                let arguments = [];
                                {
                                    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
                                    let __prebindgen_input_consumed = &__pb_input_consumed;
                                    let callback_result = unsafe {
                                        p0_callback_retained
                                            .invoke_return(
                                                callback_environment,
                                                ::prebindgen_jni_runtime::jni::signature::ReturnType::Object,
                                                &arguments,
                                            )
                                    }?;
                                    let callback_return_w1 = callback_result
                                        .l()
                                        .map_err(|e| e.to_string())
                                        .map(
                                            ::prebindgen_jni_runtime::jni::objects::JObjectArray::from,
                                        )?;
                                    {
                                        let callback_return_decoded = ::prebindgen_runtime::boundary(||
                                        {
                                            {
                                                let mut callback_return_w0: Vec<
                                                    Vec<bench_flat::InputHandle>,
                                                > = Vec::new();
                                                if true {
                                                    if callback_return_w1.is_null() {
                                                        return Err("null collection".into());
                                                    }
                                                    let callback_return_w2 = callback_environment
                                                        .get_array_length(&callback_return_w1)
                                                        .map_err(|e| e.to_string())? as usize;
                                                    callback_return_w0.reserve_exact(callback_return_w2);
                                                    for callback_return_w3 in 0..callback_return_w2 {
                                                        let callback_return_w1_frame = unsafe {
                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                &callback_environment,
                                                                9i32,
                                                            )
                                                        }?;
                                                        let callback_return_w1_row = ::prebindgen_jni_runtime::row(
                                                            &mut callback_environment,
                                                            &callback_return_w1,
                                                            callback_return_w3,
                                                            1usize,
                                                        )?;
                                                        let callback_return_w5 = ::prebindgen_jni_runtime::row_rows(
                                                            &mut callback_environment,
                                                            &(callback_return_w1_row),
                                                            0usize,
                                                        );
                                                        if callback_return_w5.is_err()
                                                            && callback_environment
                                                                .exception_check()
                                                                .map_err(|e| e.to_string())?
                                                        {
                                                            return Err("JNI collection field decoding failed".into());
                                                        }
                                                        #[allow(clippy::redundant_closure_call)]
                                                        let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                                            Vec<bench_flat::InputHandle>,
                                                            String,
                                                        > {
                                                            {
                                                                let mut callback_return_w4: Vec<bench_flat::InputHandle> = Vec::new();
                                                                if true {
                                                                    if (*callback_return_w5.as_ref().map_err(|e| e.clone())?)
                                                                        .is_null()
                                                                    {
                                                                        return Err("null collection".into());
                                                                    }
                                                                    let callback_return_w6 = callback_environment
                                                                        .get_array_length(
                                                                            &(*callback_return_w5.as_ref().map_err(|e| e.clone())?),
                                                                        )
                                                                        .map_err(|e| e.to_string())? as usize;
                                                                    callback_return_w4.reserve_exact(callback_return_w6);
                                                                    for callback_return_w7 in 0..callback_return_w6 {
                                                                        let callback_return_w5_frame = unsafe {
                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                &callback_environment,
                                                                                9i32,
                                                                            )
                                                                        }?;
                                                                        let callback_return_w5_row = ::prebindgen_jni_runtime::row(
                                                                            &mut callback_environment,
                                                                            &(*callback_return_w5.as_ref().map_err(|e| e.clone())?),
                                                                            callback_return_w7,
                                                                            1usize,
                                                                        )?;
                                                                        let callback_return_w9 = ::prebindgen_jni_runtime::row_i64(
                                                                            &mut callback_environment,
                                                                            &(callback_return_w5_row),
                                                                            0usize,
                                                                        );
                                                                        if callback_return_w9.is_err()
                                                                            && callback_environment
                                                                                .exception_check()
                                                                                .map_err(|e| e.to_string())?
                                                                        {
                                                                            return Err("JNI collection field decoding failed".into());
                                                                        }
                                                                        #[allow(clippy::redundant_closure_call)]
                                                                        let callback_return_w5_element_built = (|| -> ::core::result::Result<
                                                                            bench_flat::InputHandle,
                                                                            String,
                                                                        > {
                                                                            {
                                                                                let callback_return_w9_pointer = (*callback_return_w9
                                                                                    .as_ref()
                                                                                    .map_err(|e| e.clone())?)
                                                                                    as *mut ::prebindgen_jni_runtime::Allocation<
                                                                                        bench_flat::InputHandle,
                                                                                    >;
                                                                                ::prebindgen_runtime::validate_pointer(
                                                                                    callback_return_w9_pointer,
                                                                                )?;
                                                                                let callback_return_w8: bench_flat::InputHandle = {
                                                                                    let callback_return_w8 = unsafe {
                                                                                        Box::from_raw(callback_return_w9_pointer)
                                                                                    };
                                                                                    {
                                                                                        __pb_input_consumed
                                                                                            .set(__pb_input_consumed.get() + (1usize));
                                                                                    }
                                                                                    callback_return_w8.value
                                                                                };
                                                                                let callback_return_w5_row_value = callback_return_w8;
                                                                                Ok(callback_return_w5_row_value)
                                                                            }
                                                                        })();
                                                                        drop(callback_return_w5_frame);
                                                                        let element = callback_return_w5_element_built?;
                                                                        #[allow(clippy::zst_offset)]
                                                                        unsafe {
                                                                            let initialized = callback_return_w4.len();
                                                                            callback_return_w4
                                                                                .as_mut_ptr()
                                                                                .add(initialized)
                                                                                .write(element);
                                                                            callback_return_w4.set_len(initialized + 1);
                                                                        }
                                                                    }
                                                                }
                                                                let callback_return_w1_row_value = callback_return_w4;
                                                                Ok(callback_return_w1_row_value)
                                                            }
                                                        })();
                                                        drop(callback_return_w1_frame);
                                                        let element = callback_return_w1_element_built?;
                                                        #[allow(clippy::zst_offset)]
                                                        unsafe {
                                                            let initialized = callback_return_w0.len();
                                                            callback_return_w0
                                                                .as_mut_ptr()
                                                                .add(initialized)
                                                                .write(element);
                                                            callback_return_w0.set_len(initialized + 1);
                                                        }
                                                    }
                                                }
                                                let callback_return_value = callback_return_w0;
                                                Ok::<_, String>(callback_return_value)
                                            }
                                        });
                                        if callback_return_decoded.is_err() {
                                            ::prebindgen_jni_runtime::reclaim_output(
                                                &mut callback_environment,
                                                |callback_environment| {
                                                    let mut callback_environment = callback_environment;
                                                    let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                                        __prebindgen_input_consumed,
                                                    );
                                                    if !callback_return_w1.is_null() {
                                                        let length = callback_environment
                                                            .get_array_length(&callback_return_w1)
                                                            .map_err(|e| e.to_string())? as usize;
                                                        for callback_return_w1_cleanup_index in 0..length {
                                                            let frame = unsafe {
                                                                ::prebindgen_jni_runtime::LocalFrame::new(
                                                                    &callback_environment,
                                                                    9i32,
                                                                )
                                                            }?;
                                                            ::prebindgen_jni_runtime::reclaim_output(
                                                                &mut callback_environment,
                                                                |callback_environment| {
                                                                    let mut callback_environment = callback_environment;
                                                                    let cleanup_read_frame = unsafe {
                                                                        ::prebindgen_jni_runtime::LocalFrame::new(
                                                                            &callback_environment,
                                                                            8,
                                                                        )
                                                                    }?;
                                                                    let cleanup_read = ::prebindgen_jni_runtime::row_rows(
                                                                        &mut callback_environment,
                                                                        &callback_return_w1,
                                                                        callback_return_w1_cleanup_index,
                                                                    );
                                                                    match cleanup_read {
                                                                        Ok(callback_return_w1_cleanup_row) => {
                                                                            if !callback_return_w1_cleanup_row.is_null() {
                                                                                ::prebindgen_jni_runtime::reclaim_output(
                                                                                    &mut callback_environment,
                                                                                    |callback_environment| {
                                                                                        let mut callback_environment = callback_environment;
                                                                                        let cleanup_read_frame = unsafe {
                                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                &callback_environment,
                                                                                                8,
                                                                                            )
                                                                                        }?;
                                                                                        let cleanup_read = if 0usize
                                                                                            < callback_environment
                                                                                                .get_array_length(&callback_return_w1_cleanup_row)
                                                                                                .map_err(|e| e.to_string())? as usize
                                                                                        {
                                                                                            ::prebindgen_jni_runtime::row_rows(
                                                                                                &mut callback_environment,
                                                                                                &(callback_return_w1_cleanup_row),
                                                                                                0usize,
                                                                                            )
                                                                                        } else {
                                                                                            Err("callback cleanup field is absent".to_owned())
                                                                                        };
                                                                                        match cleanup_read {
                                                                                            Ok(callback_return_w5) => {
                                                                                                if !callback_return_w5.is_null() {
                                                                                                    let length = callback_environment
                                                                                                        .get_array_length(&callback_return_w5)
                                                                                                        .map_err(|e| e.to_string())? as usize;
                                                                                                    for callback_return_w5_cleanup_index in 0..length {
                                                                                                        let frame = unsafe {
                                                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                                &callback_environment,
                                                                                                                9i32,
                                                                                                            )
                                                                                                        }?;
                                                                                                        ::prebindgen_jni_runtime::reclaim_output(
                                                                                                            &mut callback_environment,
                                                                                                            |callback_environment| {
                                                                                                                let mut callback_environment = callback_environment;
                                                                                                                let cleanup_read_frame = unsafe {
                                                                                                                    ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                                        &callback_environment,
                                                                                                                        8,
                                                                                                                    )
                                                                                                                }?;
                                                                                                                let cleanup_read = ::prebindgen_jni_runtime::row_rows(
                                                                                                                    &mut callback_environment,
                                                                                                                    &callback_return_w5,
                                                                                                                    callback_return_w5_cleanup_index,
                                                                                                                );
                                                                                                                match cleanup_read {
                                                                                                                    Ok(callback_return_w5_cleanup_row) => {
                                                                                                                        if !callback_return_w5_cleanup_row.is_null() {
                                                                                                                            ::prebindgen_jni_runtime::reclaim_output(
                                                                                                                                &mut callback_environment,
                                                                                                                                |callback_environment| {
                                                                                                                                    let mut callback_environment = callback_environment;
                                                                                                                                    let cleanup_read_frame = unsafe {
                                                                                                                                        ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                                                            &callback_environment,
                                                                                                                                            8,
                                                                                                                                        )
                                                                                                                                    }?;
                                                                                                                                    let cleanup_read = if 0usize
                                                                                                                                        < callback_environment
                                                                                                                                            .get_array_length(&callback_return_w5_cleanup_row)
                                                                                                                                            .map_err(|e| e.to_string())? as usize
                                                                                                                                    {
                                                                                                                                        ::prebindgen_jni_runtime::row_i64(
                                                                                                                                            &mut callback_environment,
                                                                                                                                            &(callback_return_w5_cleanup_row),
                                                                                                                                            0usize,
                                                                                                                                        )
                                                                                                                                    } else {
                                                                                                                                        Err("callback cleanup field is absent".to_owned())
                                                                                                                                    };
                                                                                                                                    match cleanup_read {
                                                                                                                                        Ok(callback_return_w9) => {
                                                                                                                                            unsafe {
                                                                                                                                                callback_return_owners
                                                                                                                                                    .receive(
                                                                                                                                                        callback_return_w9
                                                                                                                                                            as *mut ::prebindgen_jni_runtime::Allocation<
                                                                                                                                                                bench_flat::InputHandle,
                                                                                                                                                            >,
                                                                                                                                                    );
                                                                                                                                            }
                                                                                                                                        }
                                                                                                                                        Err(error) => {
                                                                                                                                            if callback_environment
                                                                                                                                                .exception_check()
                                                                                                                                                .map_err(|e| e.to_string())?
                                                                                                                                            {
                                                                                                                                                return Err(error);
                                                                                                                                            }
                                                                                                                                            ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                                                                        }
                                                                                                                                    }
                                                                                                                                    drop(cleanup_read_frame);
                                                                                                                                    Ok(())
                                                                                                                                },
                                                                                                                            );
                                                                                                                        }
                                                                                                                    }
                                                                                                                    Err(error) => {
                                                                                                                        if callback_environment
                                                                                                                            .exception_check()
                                                                                                                            .map_err(|e| e.to_string())?
                                                                                                                        {
                                                                                                                            return Err(error);
                                                                                                                        }
                                                                                                                        ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                                                    }
                                                                                                                }
                                                                                                                drop(cleanup_read_frame);
                                                                                                                Ok(())
                                                                                                            },
                                                                                                        );
                                                                                                        drop(frame);
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                            Err(error) => {
                                                                                                if callback_environment
                                                                                                    .exception_check()
                                                                                                    .map_err(|e| e.to_string())?
                                                                                                {
                                                                                                    return Err(error);
                                                                                                }
                                                                                                ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                            }
                                                                                        }
                                                                                        drop(cleanup_read_frame);
                                                                                        Ok(())
                                                                                    },
                                                                                );
                                                                            }
                                                                        }
                                                                        Err(error) => {
                                                                            if callback_environment
                                                                                .exception_check()
                                                                                .map_err(|e| e.to_string())?
                                                                            {
                                                                                return Err(error);
                                                                            }
                                                                            ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                        }
                                                                    }
                                                                    drop(cleanup_read_frame);
                                                                    Ok(())
                                                                },
                                                            );
                                                            drop(frame);
                                                        }
                                                    }
                                                    Ok(())
                                                },
                                            );
                                        }
                                        callback_return_decoded
                                    }
                                }
                            }
                        },
                    )
                    .unwrap_or_else(|error| panic!("callback failed: {}", error))
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = bench_flat::callback_nested_owned_handles(
                __prebindgen_argument_0,
            );
            let ret_w0_source: ::std::vec::Vec<Vec<bench_flat::InputHandle>> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            let ret_w0_row_constructor = __pb_row_type_SequenceRow_Array_Any___1_00f39593b4ea53ea
                .get()?;
            let ret_w0 = ::prebindgen_jni_runtime::array(&mut jni_environment, ret_w2)?;
            let mut ret_w0_owner = unsafe {
                ::prebindgen_jni_runtime::OutputRows::new_objects(
                    &mut jni_environment,
                    &ret_w0,
                    __pb_cleanup_typed_Rows_fe4a4c800c91ea9c,
                )
            };
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w0_frame = unsafe {
                    ::prebindgen_jni_runtime::LocalFrame::new(&jni_environment, 9i32)
                }?;
                let ret_w4_source: ::std::vec::Vec<bench_flat::InputHandle> = ret_w1;
                let ret_w6 = ::std::vec::Vec::len(&ret_w4_source);
                let ret_w4_row_constructor = __pb_row_type_SequenceRow_Long_1_127a3c24b600dbe0
                    .get()?;
                let ret_w4 = ::prebindgen_jni_runtime::array(
                    &mut jni_environment,
                    ret_w6,
                )?;
                let mut ret_w4_owner = unsafe {
                    ::prebindgen_jni_runtime::OutputRows::new_objects(
                        &mut jni_environment,
                        &ret_w4,
                        __pb_cleanup_typed_InputHandle_b207c5d0d65e1d12,
                    )
                };
                for (ret_w7, ret_w5) in ::core::iter::Iterator::enumerate(
                    ::core::iter::IntoIterator::into_iter(ret_w4_source),
                ) {
                    let mut ret_w8 = Box::new(::prebindgen_jni_runtime::Allocation {
                        value: ret_w5,
                    });
                    let ret_w4_row = unsafe {
                        ret_w4_row_constructor
                            .construct(
                                &mut jni_environment,
                                &[
                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                        j: ((&mut *ret_w8)
                                            as *mut ::prebindgen_jni_runtime::Allocation<
                                                bench_flat::InputHandle,
                                            >) as i64,
                                    },
                                ],
                            )
                    }?;
                    let ret_w4_row = jni_environment.auto_local(ret_w4_row);
                    jni_environment
                        .set_object_array_element(&ret_w4, ret_w7 as i32, &ret_w4_row)
                        .map_err(|e| e.to_string())?;
                    #[allow(
                        clippy::redundant_closure_call,
                        clippy::let_unit_value,
                        clippy::unused_unit
                    )]
                    let _: () = (|| -> () {
                        let _ = Box::into_raw(ret_w8).expose_provenance();
                    })();
                    unsafe {
                        ret_w4_owner.complete_row();
                    }
                }
                let ret_w4 = Some(ret_w4);
                let ret_w0_row_argument_0 = match (ret_w4).as_ref() {
                    Some(value) => {
                        jni_environment.new_local_ref(value).map_err(|e| e.to_string())?
                    }
                    None => ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                };
                let ret_w0_row = unsafe {
                    ret_w0_row_constructor
                        .construct(
                            &mut jni_environment,
                            &[
                                ::prebindgen_jni_runtime::jni::sys::jvalue {
                                    l: ret_w0_row_argument_0.as_raw(),
                                },
                            ],
                        )
                }?;
                jni_environment
                    .set_object_array_element(&ret_w0, ret_w3 as i32, &ret_w0_row)
                    .map_err(|e| e.to_string())?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    ret_w4_owner.commit();
                })();
                unsafe {
                    ret_w0_owner.complete_row();
                }
                drop(ret_w0_frame);
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
                ret_w0_owner.commit();
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: i64 = ::core::convert::Into::into(p0_w1);
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: i64 = ::core::convert::Into::into(p0_w1);
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0_f0: i64 = ::core::convert::Into::into(p0_w1_f0);
            let p0_w0_field_0 = p0_w0_f0;
            let p0_w0_f1: i64 = ::core::convert::Into::into(p0_w1_f1);
            let p0_w0_field_1 = p0_w0_f1;
            let p0_w0_f2: i64 = ::core::convert::Into::into(p0_w1_f2);
            let p0_w0_field_2 = p0_w0_f2;
            let p0_w0_f3: i64 = ::core::convert::Into::into(p0_w1_f3);
            let p0_w0_field_3 = p0_w0_f3;
            let p0_w0_f4: i64 = ::core::convert::Into::into(p0_w1_f4);
            let p0_w0_field_4 = p0_w0_f4;
            let p0_w0_f5: i64 = ::core::convert::Into::into(p0_w1_f5);
            let p0_w0_field_5 = p0_w0_f5;
            let p0_w0_f6: i64 = ::core::convert::Into::into(p0_w1_f6);
            let p0_w0_field_6 = p0_w0_f6;
            let p0_w0_f7: i64 = ::core::convert::Into::into(p0_w1_f7);
            let p0_w0_field_7 = p0_w0_f7;
            let p0_w0_f8: i64 = ::core::convert::Into::into(p0_w1_f8);
            let p0_w0_field_8 = p0_w0_f8;
            let p0_w0_f9: i64 = ::core::convert::Into::into(p0_w1_f9);
            let p0_w0_field_9 = p0_w0_f9;
            let p0_w0_f10: i64 = ::core::convert::Into::into(p0_w1_f10);
            let p0_w0_field_10 = p0_w0_f10;
            let p0_w0_f11: i64 = ::core::convert::Into::into(p0_w1_f11);
            let p0_w0_field_11 = p0_w0_f11;
            let p0_w0_f12: i64 = ::core::convert::Into::into(p0_w1_f12);
            let p0_w0_field_12 = p0_w0_f12;
            let p0_w0_f13: i64 = ::core::convert::Into::into(p0_w1_f13);
            let p0_w0_field_13 = p0_w0_f13;
            let p0_w0_f14: i64 = ::core::convert::Into::into(p0_w1_f14);
            let p0_w0_field_14 = p0_w0_f14;
            let p0_w0_f15: i64 = ::core::convert::Into::into(p0_w1_f15);
            let p0_w0_field_15 = p0_w0_f15;
            let p0_w0_f16: i64 = ::core::convert::Into::into(p0_w1_f16);
            let p0_w0_field_16 = p0_w0_f16;
            let p0_w0_f17: i64 = ::core::convert::Into::into(p0_w1_f17);
            let p0_w0_field_17 = p0_w0_f17;
            let p0_w0_f18: i64 = ::core::convert::Into::into(p0_w1_f18);
            let p0_w0_field_18 = p0_w0_f18;
            let p0_w0_f19: i64 = ::core::convert::Into::into(p0_w1_f19);
            let p0_w0_field_19 = p0_w0_f19;
            let p0_w0_f20: i64 = ::core::convert::Into::into(p0_w1_f20);
            let p0_w0_field_20 = p0_w0_f20;
            let p0_w0_f21: i64 = ::core::convert::Into::into(p0_w1_f21);
            let p0_w0_field_21 = p0_w0_f21;
            let p0_w0_f22: i64 = ::core::convert::Into::into(p0_w1_f22);
            let p0_w0_field_22 = p0_w0_f22;
            let p0_w0_f23: i64 = ::core::convert::Into::into(p0_w1_f23);
            let p0_w0_field_23 = p0_w0_f23;
            let p0_w0_f24: i64 = ::core::convert::Into::into(p0_w1_f24);
            let p0_w0_field_24 = p0_w0_f24;
            let p0_w0_f25: i64 = ::core::convert::Into::into(p0_w1_f25);
            let p0_w0_field_25 = p0_w0_f25;
            let p0_w0_f26: i64 = ::core::convert::Into::into(p0_w1_f26);
            let p0_w0_field_26 = p0_w0_f26;
            let p0_w0_f27: i64 = ::core::convert::Into::into(p0_w1_f27);
            let p0_w0_field_27 = p0_w0_f27;
            let p0_w0_f28: i64 = ::core::convert::Into::into(p0_w1_f28);
            let p0_w0_field_28 = p0_w0_f28;
            let p0_w0_f29: i64 = ::core::convert::Into::into(p0_w1_f29);
            let p0_w0_field_29 = p0_w0_f29;
            let p0_w0_f30: i64 = ::core::convert::Into::into(p0_w1_f30);
            let p0_w0_field_30 = p0_w0_f30;
            let p0_w0_f31: i64 = ::core::convert::Into::into(p0_w1_f31);
            let p0_w0_field_31 = p0_w0_f31;
            let p0_w0: bench_flat::WideRow = bench_flat::WideRow {
                f0: p0_w0_field_0,
                f1: p0_w0_field_1,
                f2: p0_w0_field_2,
                f3: p0_w0_field_3,
                f4: p0_w0_field_4,
                f5: p0_w0_field_5,
                f6: p0_w0_field_6,
                f7: p0_w0_field_7,
                f8: p0_w0_field_8,
                f9: p0_w0_field_9,
                f10: p0_w0_field_10,
                f11: p0_w0_field_11,
                f12: p0_w0_field_12,
                f13: p0_w0_field_13,
                f14: p0_w0_field_14,
                f15: p0_w0_field_15,
                f16: p0_w0_field_16,
                f17: p0_w0_field_17,
                f18: p0_w0_field_18,
                f19: p0_w0_field_19,
                f20: p0_w0_field_20,
                f21: p0_w0_field_21,
                f22: p0_w0_field_22,
                f23: p0_w0_field_23,
                f24: p0_w0_field_24,
                f25: p0_w0_field_25,
                f26: p0_w0_field_26,
                f27: p0_w0_field_27,
                f28: p0_w0_field_28,
                f29: p0_w0_field_29,
                f30: p0_w0_field_30,
                f31: p0_w0_field_31,
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
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
                                let callback_return_w0_f0: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f0,
                                );
                                let callback_return_w0_field_0 = callback_return_w0_f0;
                                let callback_return_w0_f1: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f1,
                                );
                                let callback_return_w0_field_1 = callback_return_w0_f1;
                                let callback_return_w0_f2: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f2,
                                );
                                let callback_return_w0_field_2 = callback_return_w0_f2;
                                let callback_return_w0_f3: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f3,
                                );
                                let callback_return_w0_field_3 = callback_return_w0_f3;
                                let callback_return_w0_f4: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f4,
                                );
                                let callback_return_w0_field_4 = callback_return_w0_f4;
                                let callback_return_w0_f5: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f5,
                                );
                                let callback_return_w0_field_5 = callback_return_w0_f5;
                                let callback_return_w0_f6: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f6,
                                );
                                let callback_return_w0_field_6 = callback_return_w0_f6;
                                let callback_return_w0_f7: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f7,
                                );
                                let callback_return_w0_field_7 = callback_return_w0_f7;
                                let callback_return_w0: bench_flat::Row = bench_flat::Row {
                                    a: callback_return_w0_field_0,
                                    b: callback_return_w0_field_1,
                                    c: callback_return_w0_field_2,
                                    d: callback_return_w0_field_3,
                                    e: callback_return_w0_field_4,
                                    f: callback_return_w0_field_5,
                                    g: callback_return_w0_field_6,
                                    h: callback_return_w0_field_7,
                                };
                                let callback_return_value = callback_return_w0;
                                Ok::<_, String>(callback_return_value)
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
                let _ = Box::into_raw(ret_w0).expose_provenance();
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
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
                                let callback_return_w0_f0: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f0,
                                );
                                let callback_return_w0_field_0 = callback_return_w0_f0;
                                let callback_return_w0_f1: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f1,
                                );
                                let callback_return_w0_field_1 = callback_return_w0_f1;
                                let callback_return_w0_f2: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f2,
                                );
                                let callback_return_w0_field_2 = callback_return_w0_f2;
                                let callback_return_w0_f3: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f3,
                                );
                                let callback_return_w0_field_3 = callback_return_w0_f3;
                                let callback_return_w0_f4: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f4,
                                );
                                let callback_return_w0_field_4 = callback_return_w0_f4;
                                let callback_return_w0_f5: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f5,
                                );
                                let callback_return_w0_field_5 = callback_return_w0_f5;
                                let callback_return_w0_f6: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f6,
                                );
                                let callback_return_w0_field_6 = callback_return_w0_f6;
                                let callback_return_w0_f7: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f7,
                                );
                                let callback_return_w0_field_7 = callback_return_w0_f7;
                                let callback_return_w0_f8: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f8,
                                );
                                let callback_return_w0_field_8 = callback_return_w0_f8;
                                let callback_return_w0_f9: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f9,
                                );
                                let callback_return_w0_field_9 = callback_return_w0_f9;
                                let callback_return_w0_f10: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f10,
                                );
                                let callback_return_w0_field_10 = callback_return_w0_f10;
                                let callback_return_w0_f11: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f11,
                                );
                                let callback_return_w0_field_11 = callback_return_w0_f11;
                                let callback_return_w0_f12: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f12,
                                );
                                let callback_return_w0_field_12 = callback_return_w0_f12;
                                let callback_return_w0_f13: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f13,
                                );
                                let callback_return_w0_field_13 = callback_return_w0_f13;
                                let callback_return_w0_f14: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f14,
                                );
                                let callback_return_w0_field_14 = callback_return_w0_f14;
                                let callback_return_w0_f15: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f15,
                                );
                                let callback_return_w0_field_15 = callback_return_w0_f15;
                                let callback_return_w0_f16: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f16,
                                );
                                let callback_return_w0_field_16 = callback_return_w0_f16;
                                let callback_return_w0_f17: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f17,
                                );
                                let callback_return_w0_field_17 = callback_return_w0_f17;
                                let callback_return_w0_f18: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f18,
                                );
                                let callback_return_w0_field_18 = callback_return_w0_f18;
                                let callback_return_w0_f19: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f19,
                                );
                                let callback_return_w0_field_19 = callback_return_w0_f19;
                                let callback_return_w0_f20: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f20,
                                );
                                let callback_return_w0_field_20 = callback_return_w0_f20;
                                let callback_return_w0_f21: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f21,
                                );
                                let callback_return_w0_field_21 = callback_return_w0_f21;
                                let callback_return_w0_f22: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f22,
                                );
                                let callback_return_w0_field_22 = callback_return_w0_f22;
                                let callback_return_w0_f23: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f23,
                                );
                                let callback_return_w0_field_23 = callback_return_w0_f23;
                                let callback_return_w0_f24: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f24,
                                );
                                let callback_return_w0_field_24 = callback_return_w0_f24;
                                let callback_return_w0_f25: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f25,
                                );
                                let callback_return_w0_field_25 = callback_return_w0_f25;
                                let callback_return_w0_f26: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f26,
                                );
                                let callback_return_w0_field_26 = callback_return_w0_f26;
                                let callback_return_w0_f27: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f27,
                                );
                                let callback_return_w0_field_27 = callback_return_w0_f27;
                                let callback_return_w0_f28: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f28,
                                );
                                let callback_return_w0_field_28 = callback_return_w0_f28;
                                let callback_return_w0_f29: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f29,
                                );
                                let callback_return_w0_field_29 = callback_return_w0_f29;
                                let callback_return_w0_f30: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f30,
                                );
                                let callback_return_w0_field_30 = callback_return_w0_f30;
                                let callback_return_w0_f31: i64 = ::core::convert::Into::into(
                                    callback_return_w1_f31,
                                );
                                let callback_return_w0_field_31 = callback_return_w0_f31;
                                let callback_return_w0: bench_flat::WideRow = bench_flat::WideRow {
                                    f0: callback_return_w0_field_0,
                                    f1: callback_return_w0_field_1,
                                    f2: callback_return_w0_field_2,
                                    f3: callback_return_w0_field_3,
                                    f4: callback_return_w0_field_4,
                                    f5: callback_return_w0_field_5,
                                    f6: callback_return_w0_field_6,
                                    f7: callback_return_w0_field_7,
                                    f8: callback_return_w0_field_8,
                                    f9: callback_return_w0_field_9,
                                    f10: callback_return_w0_field_10,
                                    f11: callback_return_w0_field_11,
                                    f12: callback_return_w0_field_12,
                                    f13: callback_return_w0_field_13,
                                    f14: callback_return_w0_field_14,
                                    f15: callback_return_w0_field_15,
                                    f16: callback_return_w0_field_16,
                                    f17: callback_return_w0_field_17,
                                    f18: callback_return_w0_field_18,
                                    f19: callback_return_w0_field_19,
                                    f20: callback_return_w0_field_20,
                                    f21: callback_return_w0_field_21,
                                    f22: callback_return_w0_field_22,
                                    f23: callback_return_w0_field_23,
                                    f24: callback_return_w0_field_24,
                                    f25: callback_return_w0_field_25,
                                    f26: callback_return_w0_field_26,
                                    f27: callback_return_w0_field_27,
                                    f28: callback_return_w0_field_28,
                                    f29: callback_return_w0_field_29,
                                    f30: callback_return_w0_field_30,
                                    f31: callback_return_w0_field_31,
                                };
                                let callback_return_value = callback_return_w0;
                                Ok::<_, String>(callback_return_value)
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
                let _ = Box::into_raw(ret_w0).expose_provenance();
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1row_1return_1fire_1434e8dbfa624de44<
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1
                as *mut ::prebindgen_jni_runtime::Allocation<
                    bench_flat::RowReturnHandler,
                >;
            ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            let p0_w0: &bench_flat::RowReturnHandler = unsafe {
                &(*p0_w1_pointer).value
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w1);
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1wide_1row_1return_1fire_1ea0468a8c78895fe<
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1
                as *mut ::prebindgen_jni_runtime::Allocation<
                    bench_flat::WideRowReturnHandler,
                >;
            ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            let p0_w0: &bench_flat::WideRowReturnHandler = unsafe {
                &(*p0_w1_pointer).value
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w1);
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: i64 = ::core::convert::Into::into(p0_w1);
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: i64 = ::core::convert::Into::into(p0_w1);
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0_f0: i64 = ::core::convert::Into::into(p0_w1_f0);
            let p0_w0_field_0 = p0_w0_f0;
            let p0_w0_f1: i64 = ::core::convert::Into::into(p0_w1_f1);
            let p0_w0_field_1 = p0_w0_f1;
            let p0_w0_f2: i64 = ::core::convert::Into::into(p0_w1_f2);
            let p0_w0_field_2 = p0_w0_f2;
            let p0_w0_f3: i64 = ::core::convert::Into::into(p0_w1_f3);
            let p0_w0_field_3 = p0_w0_f3;
            let p0_w0_f4: i64 = ::core::convert::Into::into(p0_w1_f4);
            let p0_w0_field_4 = p0_w0_f4;
            let p0_w0_f5: i64 = ::core::convert::Into::into(p0_w1_f5);
            let p0_w0_field_5 = p0_w0_f5;
            let p0_w0_f6: i64 = ::core::convert::Into::into(p0_w1_f6);
            let p0_w0_field_6 = p0_w0_f6;
            let p0_w0_f7: i64 = ::core::convert::Into::into(p0_w1_f7);
            let p0_w0_field_7 = p0_w0_f7;
            let p0_w0: bench_flat::Row = bench_flat::Row {
                a: p0_w0_field_0,
                b: p0_w0_field_1,
                c: p0_w0_field_2,
                d: p0_w0_field_3,
                e: p0_w0_field_4,
                f: p0_w0_field_5,
                g: p0_w0_field_6,
                h: p0_w0_field_7,
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
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
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<i64, String> {
                        {
                            let p0_w4: i64 = ::core::convert::Into::into(
                                *p0_w5.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
                    })();
                    drop(p0_w1_frame);
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<bench_flat::Row> = Vec::new();
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
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        bench_flat::Row,
                        String,
                    > {
                        {
                            let p0_w4_f0: i64 = ::core::convert::Into::into(
                                *p0_w5_f0.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_0 = p0_w4_f0;
                            let p0_w4_f1: i64 = ::core::convert::Into::into(
                                *p0_w5_f1.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_1 = p0_w4_f1;
                            let p0_w4_f2: i64 = ::core::convert::Into::into(
                                *p0_w5_f2.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_2 = p0_w4_f2;
                            let p0_w4_f3: i64 = ::core::convert::Into::into(
                                *p0_w5_f3.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_3 = p0_w4_f3;
                            let p0_w4_f4: i64 = ::core::convert::Into::into(
                                *p0_w5_f4.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_4 = p0_w4_f4;
                            let p0_w4_f5: i64 = ::core::convert::Into::into(
                                *p0_w5_f5.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_5 = p0_w4_f5;
                            let p0_w4_f6: i64 = ::core::convert::Into::into(
                                *p0_w5_f6.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_6 = p0_w4_f6;
                            let p0_w4_f7: i64 = ::core::convert::Into::into(
                                *p0_w5_f7.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_7 = p0_w4_f7;
                            let p0_w4: bench_flat::Row = bench_flat::Row {
                                a: p0_w4_field_0,
                                b: p0_w4_field_1,
                                c: p0_w4_field_2,
                                d: p0_w4_field_3,
                                e: p0_w4_field_4,
                                f: p0_w4_field_5,
                                g: p0_w4_field_6,
                                h: p0_w4_field_7,
                            };
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
                    })();
                    drop(p0_w1_frame);
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: i64 = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w1);
            let __prebindgen_argument_1 = p1_w0;
            let source_value = bench_flat::rows_make(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0_source: ::std::vec::Vec<bench_flat::Row> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            let ret_w0_row_constructor = __pb_row_type_SequenceRow_Long_Long_Long_8_3346a47ef3bb7c4a
                .get()?;
            let ret_w0 = ::prebindgen_jni_runtime::array(&mut jni_environment, ret_w2)?;
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w4_record = ret_w1;
                let ret_w4_f0 = ret_w4_record.a;
                let ret_w4_f1 = ret_w4_record.b;
                let ret_w4_f2 = ret_w4_record.c;
                let ret_w4_f3 = ret_w4_record.d;
                let ret_w4_f4 = ret_w4_record.e;
                let ret_w4_f5 = ret_w4_record.f;
                let ret_w4_f6 = ret_w4_record.g;
                let ret_w4_f7 = ret_w4_record.h;
                let ret_w0_row = unsafe {
                    ret_w0_row_constructor
                        .construct(
                            &mut jni_environment,
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
                }?;
                let ret_w0_row = jni_environment.auto_local(ret_w0_row);
                jni_environment
                    .set_object_array_element(&ret_w0, ret_w3 as i32, &ret_w0_row)
                    .map_err(|e| e.to_string())?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {})();
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<bench_flat::Row> = Vec::new();
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
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        bench_flat::Row,
                        String,
                    > {
                        {
                            let p0_w4_f0: i64 = ::core::convert::Into::into(
                                *p0_w5_f0.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_0 = p0_w4_f0;
                            let p0_w4_f1: i64 = ::core::convert::Into::into(
                                *p0_w5_f1.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_1 = p0_w4_f1;
                            let p0_w4_f2: i64 = ::core::convert::Into::into(
                                *p0_w5_f2.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_2 = p0_w4_f2;
                            let p0_w4_f3: i64 = ::core::convert::Into::into(
                                *p0_w5_f3.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_3 = p0_w4_f3;
                            let p0_w4_f4: i64 = ::core::convert::Into::into(
                                *p0_w5_f4.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_4 = p0_w4_f4;
                            let p0_w4_f5: i64 = ::core::convert::Into::into(
                                *p0_w5_f5.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_5 = p0_w4_f5;
                            let p0_w4_f6: i64 = ::core::convert::Into::into(
                                *p0_w5_f6.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_6 = p0_w4_f6;
                            let p0_w4_f7: i64 = ::core::convert::Into::into(
                                *p0_w5_f7.as_ref().map_err(|e| e.clone())?,
                            );
                            let p0_w4_field_7 = p0_w4_f7;
                            let p0_w4: bench_flat::Row = bench_flat::Row {
                                a: p0_w4_field_0,
                                b: p0_w4_field_1,
                                c: p0_w4_field_2,
                                d: p0_w4_field_3,
                                e: p0_w4_field_4,
                                f: p0_w4_field_5,
                                g: p0_w4_field_6,
                                h: p0_w4_field_7,
                            };
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
                    })();
                    drop(p0_w1_frame);
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::rows_echo(__prebindgen_argument_0);
            let ret_w0_source: ::std::vec::Vec<bench_flat::Row> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            let ret_w0_row_constructor = __pb_row_type_SequenceRow_Long_Long_Long_8_3346a47ef3bb7c4a
                .get()?;
            let ret_w0 = ::prebindgen_jni_runtime::array(&mut jni_environment, ret_w2)?;
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w4_record = ret_w1;
                let ret_w4_f0 = ret_w4_record.a;
                let ret_w4_f1 = ret_w4_record.b;
                let ret_w4_f2 = ret_w4_record.c;
                let ret_w4_f3 = ret_w4_record.d;
                let ret_w4_f4 = ret_w4_record.e;
                let ret_w4_f5 = ret_w4_record.f;
                let ret_w4_f6 = ret_w4_record.g;
                let ret_w4_f7 = ret_w4_record.h;
                let ret_w0_row = unsafe {
                    ret_w0_row_constructor
                        .construct(
                            &mut jni_environment,
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
                }?;
                let ret_w0_row = jni_environment.auto_local(ret_w0_row);
                jni_environment
                    .set_object_array_element(&ret_w0, ret_w3 as i32, &ret_w0_row)
                    .map_err(|e| e.to_string())?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {})();
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0_f0_f0: i64 = ::core::convert::Into::into(p0_w1_f0_f0);
            let p0_w0_f0_field_0 = p0_w0_f0_f0;
            let p0_w0_f0_f1: i64 = ::core::convert::Into::into(p0_w1_f0_f1);
            let p0_w0_f0_field_1 = p0_w0_f0_f1;
            let p0_w0_f0_f2: i64 = ::core::convert::Into::into(p0_w1_f0_f2);
            let p0_w0_f0_field_2 = p0_w0_f0_f2;
            let p0_w0_f0_f3: i64 = ::core::convert::Into::into(p0_w1_f0_f3);
            let p0_w0_f0_field_3 = p0_w0_f0_f3;
            let p0_w0_f0_f4: i64 = ::core::convert::Into::into(p0_w1_f0_f4);
            let p0_w0_f0_field_4 = p0_w0_f0_f4;
            let p0_w0_f0_f5: i64 = ::core::convert::Into::into(p0_w1_f0_f5);
            let p0_w0_f0_field_5 = p0_w0_f0_f5;
            let p0_w0_f0_f6: i64 = ::core::convert::Into::into(p0_w1_f0_f6);
            let p0_w0_f0_field_6 = p0_w0_f0_f6;
            let p0_w0_f0_f7: i64 = ::core::convert::Into::into(p0_w1_f0_f7);
            let p0_w0_f0_field_7 = p0_w0_f0_f7;
            let p0_w0_f0: bench_flat::Row = bench_flat::Row {
                a: p0_w0_f0_field_0,
                b: p0_w0_f0_field_1,
                c: p0_w0_f0_field_2,
                d: p0_w0_f0_field_3,
                e: p0_w0_f0_field_4,
                f: p0_w0_f0_field_5,
                g: p0_w0_f0_field_6,
                h: p0_w0_f0_field_7,
            };
            let p0_w0_field_0 = p0_w0_f0;
            let p0_w0_f1: Vec<u8> = ::core::convert::Into::into(
                ::prebindgen_jni_runtime::bytes(&mut jni_environment, &p0_w1_f1)?,
            );
            let p0_w0_field_1 = p0_w0_f1;
            let p0_w0: bench_flat::Packet = bench_flat::Packet {
                header: p0_w0_field_0,
                bytes: p0_w0_field_1,
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
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
                                let callback_arg_0_w0_source: ::std::vec::Vec<
                                    bench_flat::Row,
                                > = callback_value_0;
                                let callback_arg_0_w2 = ::std::vec::Vec::len(
                                    &callback_arg_0_w0_source,
                                );
                                let callback_arg_0_w0_row_constructor = __pb_row_type_SequenceRow_Long_Long_Long_8_3346a47ef3bb7c4a
                                    .get()?;
                                let callback_arg_0_w0 = ::prebindgen_jni_runtime::array(
                                    &mut callback_environment,
                                    callback_arg_0_w2,
                                )?;
                                for (callback_arg_0_w3, callback_arg_0_w1) in ::core::iter::Iterator::enumerate(
                                    ::core::iter::IntoIterator::into_iter(
                                        callback_arg_0_w0_source,
                                    ),
                                ) {
                                    let callback_arg_0_w4_record = callback_arg_0_w1;
                                    let callback_arg_0_w4_f0 = callback_arg_0_w4_record.a;
                                    let callback_arg_0_w4_f1 = callback_arg_0_w4_record.b;
                                    let callback_arg_0_w4_f2 = callback_arg_0_w4_record.c;
                                    let callback_arg_0_w4_f3 = callback_arg_0_w4_record.d;
                                    let callback_arg_0_w4_f4 = callback_arg_0_w4_record.e;
                                    let callback_arg_0_w4_f5 = callback_arg_0_w4_record.f;
                                    let callback_arg_0_w4_f6 = callback_arg_0_w4_record.g;
                                    let callback_arg_0_w4_f7 = callback_arg_0_w4_record.h;
                                    let callback_arg_0_w0_row = unsafe {
                                        callback_arg_0_w0_row_constructor
                                            .construct(
                                                &mut callback_environment,
                                                &[
                                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                                        j: callback_arg_0_w4_f0,
                                                    },
                                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                                        j: callback_arg_0_w4_f1,
                                                    },
                                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                                        j: callback_arg_0_w4_f2,
                                                    },
                                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                                        j: callback_arg_0_w4_f3,
                                                    },
                                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                                        j: callback_arg_0_w4_f4,
                                                    },
                                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                                        j: callback_arg_0_w4_f5,
                                                    },
                                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                                        j: callback_arg_0_w4_f6,
                                                    },
                                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                                        j: callback_arg_0_w4_f7,
                                                    },
                                                ],
                                            )
                                    }?;
                                    let callback_arg_0_w0_row = callback_environment
                                        .auto_local(callback_arg_0_w0_row);
                                    callback_environment
                                        .set_object_array_element(
                                            &callback_arg_0_w0,
                                            callback_arg_0_w3 as i32,
                                            &callback_arg_0_w0_row,
                                        )
                                        .map_err(|e| e.to_string())?;
                                    #[allow(
                                        clippy::redundant_closure_call,
                                        clippy::let_unit_value,
                                        clippy::unused_unit
                                    )]
                                    let _: () = (|| -> () {})();
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
                let _ = Box::into_raw(ret_w0).expose_provenance();
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1fire_1ff69607332471b89<
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
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1
                as *mut ::prebindgen_jni_runtime::Allocation<bench_flat::Handler>;
            ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            let p0_w0: &bench_flat::Handler = unsafe { &(*p0_w1_pointer).value };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w1);
            let __prebindgen_argument_1 = p1_w0;
            let p2_w0: i64 = ::core::convert::Into::into(p2_w1);
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
pub extern "system" fn Java_bench_plain_JNINative_init_1SequenceCapacity_1e067e94ae7b50e2a(
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1owned_1handles_1echo_1fold_1de46f033d154a724<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JObjectArray<'local>,
    ownership_accepted: ::prebindgen_jni_runtime::jni::objects::JLongArray<'local>,
    __pb_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
    __pb_output_accepted: ::prebindgen_jni_runtime::jni::objects::JBooleanArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i32 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i32 = Default::default();
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
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
        if jni_environment
            .get_array_length(&ownership_accepted)
            .map_err(|e| e.to_string())? != 1
        {
            return Err("invalid ownership acknowledgement".into());
        }
        __pb_receipt_valid.set(true);
        {
            let mut p0_w0: Vec<bench_flat::InputHandle> = Vec::new();
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
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        bench_flat::InputHandle,
                        String,
                    > {
                        {
                            let p0_w5_pointer = (*p0_w5.as_ref().map_err(|e| e.clone())?)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >;
                            ::prebindgen_runtime::validate_pointer(p0_w5_pointer)?;
                            let p0_w4: bench_flat::InputHandle = {
                                let p0_w4 = unsafe { Box::from_raw(p0_w5_pointer) };
                                {
                                    __pb_input_consumed
                                        .set(__pb_input_consumed.get() + (1usize));
                                }
                                p0_w4.value
                            };
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
                    })();
                    drop(p0_w1_frame);
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::owned_handles_echo(__prebindgen_argument_0);
            let ret_w0_source: ::std::vec::Vec<bench_flat::InputHandle> = source_value;
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
                let mut ret_w4 = Box::new(::prebindgen_jni_runtime::Allocation {
                    value: ret_w1,
                });
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
                                    j: ((&mut *ret_w4)
                                        as *mut ::prebindgen_jni_runtime::Allocation<
                                            bench_flat::InputHandle,
                                        >) as i64,
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
                let _: () = (|| -> () {
                    let _ = Box::into_raw(ret_w4).expose_provenance();
                })();
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
    if __pb_receipt_valid.get() {
        let count = __pb_input_consumed.get() as i64;
        let value = if outcome.is_ok() { count } else { -1 - count };
        ::prebindgen_jni_runtime::write_input_receipt(
            &mut jni_environment,
            &ownership_accepted,
            value,
            outcome.is_err(),
        );
    }
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1nested_1owned_1handles_1echo_1fold_1f1a4a186e9c98620<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JObjectArray<'local>,
    ownership_accepted: ::prebindgen_jni_runtime::jni::objects::JLongArray<'local>,
    __pb_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
    __pb_output_accepted: ::prebindgen_jni_runtime::jni::objects::JBooleanArray<'local>,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> i32 {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: i32 = Default::default();
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
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
        if jni_environment
            .get_array_length(&ownership_accepted)
            .map_err(|e| e.to_string())? != 1
        {
            return Err("invalid ownership acknowledgement".into());
        }
        __pb_receipt_valid.set(true);
        {
            let mut p0_w0: Vec<Vec<bench_flat::InputHandle>> = Vec::new();
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
                    let p0_w5 = ::prebindgen_jni_runtime::row_rows(
                        &mut jni_environment,
                        &(p0_w1_row),
                        0usize,
                    );
                    if p0_w5.is_err()
                        && jni_environment.exception_check().map_err(|e| e.to_string())?
                    {
                        return Err("JNI collection field decoding failed".into());
                    }
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        Vec<bench_flat::InputHandle>,
                        String,
                    > {
                        {
                            let mut p0_w4: Vec<bench_flat::InputHandle> = Vec::new();
                            if true {
                                if (*p0_w5.as_ref().map_err(|e| e.clone())?).is_null() {
                                    return Err("null collection".into());
                                }
                                let p0_w6 = jni_environment
                                    .get_array_length(
                                        &(*p0_w5.as_ref().map_err(|e| e.clone())?),
                                    )
                                    .map_err(|e| e.to_string())? as usize;
                                p0_w4.reserve_exact(p0_w6);
                                for p0_w7 in 0..p0_w6 {
                                    let p0_w5_frame = unsafe {
                                        ::prebindgen_jni_runtime::LocalFrame::new(
                                            &jni_environment,
                                            9i32,
                                        )
                                    }?;
                                    let p0_w5_row = ::prebindgen_jni_runtime::row(
                                        &mut jni_environment,
                                        &(*p0_w5.as_ref().map_err(|e| e.clone())?),
                                        p0_w7,
                                        1usize,
                                    )?;
                                    let p0_w9 = ::prebindgen_jni_runtime::row_i64(
                                        &mut jni_environment,
                                        &(p0_w5_row),
                                        0usize,
                                    );
                                    if p0_w9.is_err()
                                        && jni_environment
                                            .exception_check()
                                            .map_err(|e| e.to_string())?
                                    {
                                        return Err("JNI collection field decoding failed".into());
                                    }
                                    #[allow(clippy::redundant_closure_call)]
                                    let p0_w5_element_built = (|| -> ::core::result::Result<
                                        bench_flat::InputHandle,
                                        String,
                                    > {
                                        {
                                            let p0_w9_pointer = (*p0_w9
                                                .as_ref()
                                                .map_err(|e| e.clone())?)
                                                as *mut ::prebindgen_jni_runtime::Allocation<
                                                    bench_flat::InputHandle,
                                                >;
                                            ::prebindgen_runtime::validate_pointer(p0_w9_pointer)?;
                                            let p0_w8: bench_flat::InputHandle = {
                                                let p0_w8 = unsafe { Box::from_raw(p0_w9_pointer) };
                                                {
                                                    __pb_input_consumed
                                                        .set(__pb_input_consumed.get() + (1usize));
                                                }
                                                p0_w8.value
                                            };
                                            let p0_w5_row_value = p0_w8;
                                            Ok(p0_w5_row_value)
                                        }
                                    })();
                                    drop(p0_w5_frame);
                                    let element = p0_w5_element_built?;
                                    #[allow(clippy::zst_offset)]
                                    unsafe {
                                        let initialized = p0_w4.len();
                                        p0_w4.as_mut_ptr().add(initialized).write(element);
                                        p0_w4.set_len(initialized + 1);
                                    }
                                }
                            }
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
                    })();
                    drop(p0_w1_frame);
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            let __prebindgen_argument_0 = p0_w0;
            let source_value = bench_flat::nested_owned_handles_echo(
                __prebindgen_argument_0,
            );
            let ret_w0_source: ::std::vec::Vec<Vec<bench_flat::InputHandle>> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            jni_environment
                .set_boolean_array_region(&__pb_output_accepted, 0, &[1])
                .map_err(|e| e.to_string())?;
            let ret_w0_callback_class = jni_environment
                .get_object_class(&__pb_callback)
                .map_err(|e| e.to_string())?;
            let ret_w0_callback_method = jni_environment
                .get_method_id(&ret_w0_callback_class, "run", "([Ljava/lang/Object;)V")
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
                let ret_w0_callback_frame = unsafe {
                    ::prebindgen_jni_runtime::LocalFrame::new(&jni_environment, 9i32)
                }?;
                let ret_w4_source: ::std::vec::Vec<bench_flat::InputHandle> = ret_w1;
                let ret_w6 = ::std::vec::Vec::len(&ret_w4_source);
                let ret_w4_row_constructor = __pb_row_type_SequenceRow_Long_1_127a3c24b600dbe0
                    .get()?;
                let ret_w4 = ::prebindgen_jni_runtime::array(
                    &mut jni_environment,
                    ret_w6,
                )?;
                let mut ret_w4_owner = unsafe {
                    ::prebindgen_jni_runtime::OutputRows::new_objects(
                        &mut jni_environment,
                        &ret_w4,
                        __pb_cleanup_typed_InputHandle_b207c5d0d65e1d12,
                    )
                };
                for (ret_w7, ret_w5) in ::core::iter::Iterator::enumerate(
                    ::core::iter::IntoIterator::into_iter(ret_w4_source),
                ) {
                    let mut ret_w8 = Box::new(::prebindgen_jni_runtime::Allocation {
                        value: ret_w5,
                    });
                    let ret_w4_row = unsafe {
                        ret_w4_row_constructor
                            .construct(
                                &mut jni_environment,
                                &[
                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                        j: ((&mut *ret_w8)
                                            as *mut ::prebindgen_jni_runtime::Allocation<
                                                bench_flat::InputHandle,
                                            >) as i64,
                                    },
                                ],
                            )
                    }?;
                    let ret_w4_row = jni_environment.auto_local(ret_w4_row);
                    jni_environment
                        .set_object_array_element(&ret_w4, ret_w7 as i32, &ret_w4_row)
                        .map_err(|e| e.to_string())?;
                    #[allow(
                        clippy::redundant_closure_call,
                        clippy::let_unit_value,
                        clippy::unused_unit
                    )]
                    let _: () = (|| -> () {
                        let _ = Box::into_raw(ret_w8).expose_provenance();
                    })();
                    unsafe {
                        ret_w4_owner.complete_row();
                    }
                }
                let ret_w4 = Some(ret_w4);
                let ret_w0_callback_object_0 = match (ret_w4).as_ref() {
                    Some(value) => {
                        jni_environment.new_local_ref(value).map_err(|e| e.to_string())?
                    }
                    None => ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                };
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
                                    l: ret_w0_callback_object_0.as_raw(),
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
                let _: () = (|| -> () {
                    ret_w4_owner.commit();
                })();
                drop(ret_w0_callback_frame);
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
    if __pb_receipt_valid.get() {
        let count = __pb_input_consumed.get() as i64;
        let value = if outcome.is_ok() { count } else { -1 - count };
        ::prebindgen_jni_runtime::write_input_receipt(
            &mut jni_environment,
            &ownership_accepted,
            value,
            outcome.is_err(),
        );
    }
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
pub unsafe extern "system" fn Java_bench_plain_JNINative_call_1callback_1nested_1owned_1handles_1fold_1496cd4317fea585c<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_callback: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
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
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "()[Ljava/lang/Object;",
                "bench.plain.OwnedNestedCallback",
            )?;
            #[allow(unused_mut, clippy::collapsible_if)]
            let p0_callback_value = move || {
                p0_callback_retained
                    .with_env_result(
                        17i32,
                        |callback_environment| {
                            let mut callback_environment = callback_environment;
                            {
                                let arguments = [];
                                {
                                    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
                                    let __prebindgen_input_consumed = &__pb_input_consumed;
                                    let callback_result = unsafe {
                                        p0_callback_retained
                                            .invoke_return(
                                                callback_environment,
                                                ::prebindgen_jni_runtime::jni::signature::ReturnType::Object,
                                                &arguments,
                                            )
                                    }?;
                                    let callback_return_w1 = callback_result
                                        .l()
                                        .map_err(|e| e.to_string())
                                        .map(
                                            ::prebindgen_jni_runtime::jni::objects::JObjectArray::from,
                                        )?;
                                    {
                                        let callback_return_decoded = ::prebindgen_runtime::boundary(||
                                        {
                                            {
                                                let mut callback_return_w0: Vec<
                                                    Vec<bench_flat::InputHandle>,
                                                > = Vec::new();
                                                if true {
                                                    if callback_return_w1.is_null() {
                                                        return Err("null collection".into());
                                                    }
                                                    let callback_return_w2 = callback_environment
                                                        .get_array_length(&callback_return_w1)
                                                        .map_err(|e| e.to_string())? as usize;
                                                    callback_return_w0.reserve_exact(callback_return_w2);
                                                    for callback_return_w3 in 0..callback_return_w2 {
                                                        let callback_return_w1_frame = unsafe {
                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                &callback_environment,
                                                                9i32,
                                                            )
                                                        }?;
                                                        let callback_return_w1_row = ::prebindgen_jni_runtime::row(
                                                            &mut callback_environment,
                                                            &callback_return_w1,
                                                            callback_return_w3,
                                                            1usize,
                                                        )?;
                                                        let callback_return_w5 = ::prebindgen_jni_runtime::row_rows(
                                                            &mut callback_environment,
                                                            &(callback_return_w1_row),
                                                            0usize,
                                                        );
                                                        if callback_return_w5.is_err()
                                                            && callback_environment
                                                                .exception_check()
                                                                .map_err(|e| e.to_string())?
                                                        {
                                                            return Err("JNI collection field decoding failed".into());
                                                        }
                                                        #[allow(clippy::redundant_closure_call)]
                                                        let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                                            Vec<bench_flat::InputHandle>,
                                                            String,
                                                        > {
                                                            {
                                                                let mut callback_return_w4: Vec<bench_flat::InputHandle> = Vec::new();
                                                                if true {
                                                                    if (*callback_return_w5.as_ref().map_err(|e| e.clone())?)
                                                                        .is_null()
                                                                    {
                                                                        return Err("null collection".into());
                                                                    }
                                                                    let callback_return_w6 = callback_environment
                                                                        .get_array_length(
                                                                            &(*callback_return_w5.as_ref().map_err(|e| e.clone())?),
                                                                        )
                                                                        .map_err(|e| e.to_string())? as usize;
                                                                    callback_return_w4.reserve_exact(callback_return_w6);
                                                                    for callback_return_w7 in 0..callback_return_w6 {
                                                                        let callback_return_w5_frame = unsafe {
                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                &callback_environment,
                                                                                9i32,
                                                                            )
                                                                        }?;
                                                                        let callback_return_w5_row = ::prebindgen_jni_runtime::row(
                                                                            &mut callback_environment,
                                                                            &(*callback_return_w5.as_ref().map_err(|e| e.clone())?),
                                                                            callback_return_w7,
                                                                            1usize,
                                                                        )?;
                                                                        let callback_return_w9 = ::prebindgen_jni_runtime::row_i64(
                                                                            &mut callback_environment,
                                                                            &(callback_return_w5_row),
                                                                            0usize,
                                                                        );
                                                                        if callback_return_w9.is_err()
                                                                            && callback_environment
                                                                                .exception_check()
                                                                                .map_err(|e| e.to_string())?
                                                                        {
                                                                            return Err("JNI collection field decoding failed".into());
                                                                        }
                                                                        #[allow(clippy::redundant_closure_call)]
                                                                        let callback_return_w5_element_built = (|| -> ::core::result::Result<
                                                                            bench_flat::InputHandle,
                                                                            String,
                                                                        > {
                                                                            {
                                                                                let callback_return_w9_pointer = (*callback_return_w9
                                                                                    .as_ref()
                                                                                    .map_err(|e| e.clone())?)
                                                                                    as *mut ::prebindgen_jni_runtime::Allocation<
                                                                                        bench_flat::InputHandle,
                                                                                    >;
                                                                                ::prebindgen_runtime::validate_pointer(
                                                                                    callback_return_w9_pointer,
                                                                                )?;
                                                                                let callback_return_w8: bench_flat::InputHandle = {
                                                                                    let callback_return_w8 = unsafe {
                                                                                        Box::from_raw(callback_return_w9_pointer)
                                                                                    };
                                                                                    {
                                                                                        __pb_input_consumed
                                                                                            .set(__pb_input_consumed.get() + (1usize));
                                                                                    }
                                                                                    callback_return_w8.value
                                                                                };
                                                                                let callback_return_w5_row_value = callback_return_w8;
                                                                                Ok(callback_return_w5_row_value)
                                                                            }
                                                                        })();
                                                                        drop(callback_return_w5_frame);
                                                                        let element = callback_return_w5_element_built?;
                                                                        #[allow(clippy::zst_offset)]
                                                                        unsafe {
                                                                            let initialized = callback_return_w4.len();
                                                                            callback_return_w4
                                                                                .as_mut_ptr()
                                                                                .add(initialized)
                                                                                .write(element);
                                                                            callback_return_w4.set_len(initialized + 1);
                                                                        }
                                                                    }
                                                                }
                                                                let callback_return_w1_row_value = callback_return_w4;
                                                                Ok(callback_return_w1_row_value)
                                                            }
                                                        })();
                                                        drop(callback_return_w1_frame);
                                                        let element = callback_return_w1_element_built?;
                                                        #[allow(clippy::zst_offset)]
                                                        unsafe {
                                                            let initialized = callback_return_w0.len();
                                                            callback_return_w0
                                                                .as_mut_ptr()
                                                                .add(initialized)
                                                                .write(element);
                                                            callback_return_w0.set_len(initialized + 1);
                                                        }
                                                    }
                                                }
                                                let callback_return_value = callback_return_w0;
                                                Ok::<_, String>(callback_return_value)
                                            }
                                        });
                                        if callback_return_decoded.is_err() {
                                            ::prebindgen_jni_runtime::reclaim_output(
                                                &mut callback_environment,
                                                |callback_environment| {
                                                    let mut callback_environment = callback_environment;
                                                    let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                                        __prebindgen_input_consumed,
                                                    );
                                                    if !callback_return_w1.is_null() {
                                                        let length = callback_environment
                                                            .get_array_length(&callback_return_w1)
                                                            .map_err(|e| e.to_string())? as usize;
                                                        for callback_return_w1_cleanup_index in 0..length {
                                                            let frame = unsafe {
                                                                ::prebindgen_jni_runtime::LocalFrame::new(
                                                                    &callback_environment,
                                                                    9i32,
                                                                )
                                                            }?;
                                                            ::prebindgen_jni_runtime::reclaim_output(
                                                                &mut callback_environment,
                                                                |callback_environment| {
                                                                    let mut callback_environment = callback_environment;
                                                                    let cleanup_read_frame = unsafe {
                                                                        ::prebindgen_jni_runtime::LocalFrame::new(
                                                                            &callback_environment,
                                                                            8,
                                                                        )
                                                                    }?;
                                                                    let cleanup_read = ::prebindgen_jni_runtime::row_rows(
                                                                        &mut callback_environment,
                                                                        &callback_return_w1,
                                                                        callback_return_w1_cleanup_index,
                                                                    );
                                                                    match cleanup_read {
                                                                        Ok(callback_return_w1_cleanup_row) => {
                                                                            if !callback_return_w1_cleanup_row.is_null() {
                                                                                ::prebindgen_jni_runtime::reclaim_output(
                                                                                    &mut callback_environment,
                                                                                    |callback_environment| {
                                                                                        let mut callback_environment = callback_environment;
                                                                                        let cleanup_read_frame = unsafe {
                                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                &callback_environment,
                                                                                                8,
                                                                                            )
                                                                                        }?;
                                                                                        let cleanup_read = if 0usize
                                                                                            < callback_environment
                                                                                                .get_array_length(&callback_return_w1_cleanup_row)
                                                                                                .map_err(|e| e.to_string())? as usize
                                                                                        {
                                                                                            ::prebindgen_jni_runtime::row_rows(
                                                                                                &mut callback_environment,
                                                                                                &(callback_return_w1_cleanup_row),
                                                                                                0usize,
                                                                                            )
                                                                                        } else {
                                                                                            Err("callback cleanup field is absent".to_owned())
                                                                                        };
                                                                                        match cleanup_read {
                                                                                            Ok(callback_return_w5) => {
                                                                                                if !callback_return_w5.is_null() {
                                                                                                    let length = callback_environment
                                                                                                        .get_array_length(&callback_return_w5)
                                                                                                        .map_err(|e| e.to_string())? as usize;
                                                                                                    for callback_return_w5_cleanup_index in 0..length {
                                                                                                        let frame = unsafe {
                                                                                                            ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                                &callback_environment,
                                                                                                                9i32,
                                                                                                            )
                                                                                                        }?;
                                                                                                        ::prebindgen_jni_runtime::reclaim_output(
                                                                                                            &mut callback_environment,
                                                                                                            |callback_environment| {
                                                                                                                let mut callback_environment = callback_environment;
                                                                                                                let cleanup_read_frame = unsafe {
                                                                                                                    ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                                        &callback_environment,
                                                                                                                        8,
                                                                                                                    )
                                                                                                                }?;
                                                                                                                let cleanup_read = ::prebindgen_jni_runtime::row_rows(
                                                                                                                    &mut callback_environment,
                                                                                                                    &callback_return_w5,
                                                                                                                    callback_return_w5_cleanup_index,
                                                                                                                );
                                                                                                                match cleanup_read {
                                                                                                                    Ok(callback_return_w5_cleanup_row) => {
                                                                                                                        if !callback_return_w5_cleanup_row.is_null() {
                                                                                                                            ::prebindgen_jni_runtime::reclaim_output(
                                                                                                                                &mut callback_environment,
                                                                                                                                |callback_environment| {
                                                                                                                                    let mut callback_environment = callback_environment;
                                                                                                                                    let cleanup_read_frame = unsafe {
                                                                                                                                        ::prebindgen_jni_runtime::LocalFrame::new(
                                                                                                                                            &callback_environment,
                                                                                                                                            8,
                                                                                                                                        )
                                                                                                                                    }?;
                                                                                                                                    let cleanup_read = if 0usize
                                                                                                                                        < callback_environment
                                                                                                                                            .get_array_length(&callback_return_w5_cleanup_row)
                                                                                                                                            .map_err(|e| e.to_string())? as usize
                                                                                                                                    {
                                                                                                                                        ::prebindgen_jni_runtime::row_i64(
                                                                                                                                            &mut callback_environment,
                                                                                                                                            &(callback_return_w5_cleanup_row),
                                                                                                                                            0usize,
                                                                                                                                        )
                                                                                                                                    } else {
                                                                                                                                        Err("callback cleanup field is absent".to_owned())
                                                                                                                                    };
                                                                                                                                    match cleanup_read {
                                                                                                                                        Ok(callback_return_w9) => {
                                                                                                                                            unsafe {
                                                                                                                                                callback_return_owners
                                                                                                                                                    .receive(
                                                                                                                                                        callback_return_w9
                                                                                                                                                            as *mut ::prebindgen_jni_runtime::Allocation<
                                                                                                                                                                bench_flat::InputHandle,
                                                                                                                                                            >,
                                                                                                                                                    );
                                                                                                                                            }
                                                                                                                                        }
                                                                                                                                        Err(error) => {
                                                                                                                                            if callback_environment
                                                                                                                                                .exception_check()
                                                                                                                                                .map_err(|e| e.to_string())?
                                                                                                                                            {
                                                                                                                                                return Err(error);
                                                                                                                                            }
                                                                                                                                            ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                                                                        }
                                                                                                                                    }
                                                                                                                                    drop(cleanup_read_frame);
                                                                                                                                    Ok(())
                                                                                                                                },
                                                                                                                            );
                                                                                                                        }
                                                                                                                    }
                                                                                                                    Err(error) => {
                                                                                                                        if callback_environment
                                                                                                                            .exception_check()
                                                                                                                            .map_err(|e| e.to_string())?
                                                                                                                        {
                                                                                                                            return Err(error);
                                                                                                                        }
                                                                                                                        ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                                                    }
                                                                                                                }
                                                                                                                drop(cleanup_read_frame);
                                                                                                                Ok(())
                                                                                                            },
                                                                                                        );
                                                                                                        drop(frame);
                                                                                                    }
                                                                                                }
                                                                                            }
                                                                                            Err(error) => {
                                                                                                if callback_environment
                                                                                                    .exception_check()
                                                                                                    .map_err(|e| e.to_string())?
                                                                                                {
                                                                                                    return Err(error);
                                                                                                }
                                                                                                ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                                            }
                                                                                        }
                                                                                        drop(cleanup_read_frame);
                                                                                        Ok(())
                                                                                    },
                                                                                );
                                                                            }
                                                                        }
                                                                        Err(error) => {
                                                                            if callback_environment
                                                                                .exception_check()
                                                                                .map_err(|e| e.to_string())?
                                                                            {
                                                                                return Err(error);
                                                                            }
                                                                            ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                                        }
                                                                    }
                                                                    drop(cleanup_read_frame);
                                                                    Ok(())
                                                                },
                                                            );
                                                            drop(frame);
                                                        }
                                                    }
                                                    Ok(())
                                                },
                                            );
                                        }
                                        callback_return_decoded
                                    }
                                }
                            }
                        },
                    )
                    .unwrap_or_else(|error| panic!("callback failed: {}", error))
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = bench_flat::callback_nested_owned_handles(
                __prebindgen_argument_0,
            );
            let ret_w0_source: ::std::vec::Vec<Vec<bench_flat::InputHandle>> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            jni_environment
                .set_boolean_array_region(&__pb_output_accepted, 0, &[1])
                .map_err(|e| e.to_string())?;
            let ret_w0_callback_class = jni_environment
                .get_object_class(&__pb_callback)
                .map_err(|e| e.to_string())?;
            let ret_w0_callback_method = jni_environment
                .get_method_id(&ret_w0_callback_class, "run", "([Ljava/lang/Object;)V")
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
                let ret_w0_callback_frame = unsafe {
                    ::prebindgen_jni_runtime::LocalFrame::new(&jni_environment, 9i32)
                }?;
                let ret_w4_source: ::std::vec::Vec<bench_flat::InputHandle> = ret_w1;
                let ret_w6 = ::std::vec::Vec::len(&ret_w4_source);
                let ret_w4_row_constructor = __pb_row_type_SequenceRow_Long_1_127a3c24b600dbe0
                    .get()?;
                let ret_w4 = ::prebindgen_jni_runtime::array(
                    &mut jni_environment,
                    ret_w6,
                )?;
                let mut ret_w4_owner = unsafe {
                    ::prebindgen_jni_runtime::OutputRows::new_objects(
                        &mut jni_environment,
                        &ret_w4,
                        __pb_cleanup_typed_InputHandle_b207c5d0d65e1d12,
                    )
                };
                for (ret_w7, ret_w5) in ::core::iter::Iterator::enumerate(
                    ::core::iter::IntoIterator::into_iter(ret_w4_source),
                ) {
                    let mut ret_w8 = Box::new(::prebindgen_jni_runtime::Allocation {
                        value: ret_w5,
                    });
                    let ret_w4_row = unsafe {
                        ret_w4_row_constructor
                            .construct(
                                &mut jni_environment,
                                &[
                                    ::prebindgen_jni_runtime::jni::sys::jvalue {
                                        j: ((&mut *ret_w8)
                                            as *mut ::prebindgen_jni_runtime::Allocation<
                                                bench_flat::InputHandle,
                                            >) as i64,
                                    },
                                ],
                            )
                    }?;
                    let ret_w4_row = jni_environment.auto_local(ret_w4_row);
                    jni_environment
                        .set_object_array_element(&ret_w4, ret_w7 as i32, &ret_w4_row)
                        .map_err(|e| e.to_string())?;
                    #[allow(
                        clippy::redundant_closure_call,
                        clippy::let_unit_value,
                        clippy::unused_unit
                    )]
                    let _: () = (|| -> () {
                        let _ = Box::into_raw(ret_w8).expose_provenance();
                    })();
                    unsafe {
                        ret_w4_owner.complete_row();
                    }
                }
                let ret_w4 = Some(ret_w4);
                let ret_w0_callback_object_0 = match (ret_w4).as_ref() {
                    Some(value) => {
                        jni_environment.new_local_ref(value).map_err(|e| e.to_string())?
                    }
                    None => ::prebindgen_jni_runtime::jni::objects::JObject::null(),
                };
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
                                    l: ret_w0_callback_object_0.as_raw(),
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
                let _: () = (|| -> () {
                    ret_w4_owner.commit();
                })();
                drop(ret_w0_callback_frame);
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
            let p0_w0: i64 = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: i64 = ::core::convert::Into::into(p1_w1);
            let __prebindgen_argument_1 = p1_w0;
            let source_value = bench_flat::rows_for_reduce(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0_source: ::std::vec::Vec<bench_flat::Row> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            jni_environment
                .set_boolean_array_region(&__pb_output_accepted, 0, &[1])
                .map_err(|e| e.to_string())?;
            let ret_w0_callback_class = jni_environment
                .get_object_class(&__pb_callback)
                .map_err(|e| e.to_string())?;
            let ret_w0_callback_method = jni_environment
                .get_method_id(&ret_w0_callback_class, "run", "(JJJJJJJJ)V")
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
    ::core::mem::size_of:: < bench_flat::BufferRecord > () != 0,
    "flat object `BufferRecord` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < bench_flat::HandleBatch > () != 0,
    "flat object `HandleBatch` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < bench_flat::HandleRecord > () != 0,
    "flat object `HandleRecord` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < bench_flat::Handler > () != 0,
    "flat object `Handler` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < bench_flat::InputHandle > () != 0,
    "flat object `InputHandle` must have nonzero size"
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
