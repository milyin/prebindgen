/**# Safety
Only generated Kotlin ownership release paths may call this entry point. The pointer must be a live allocation of the declared type with no outstanding native access; an opted-in Cleaner may release an unreachable wrapper without its monitor.*/
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
/**# Safety
Only generated Kotlin ownership release paths may call this entry point. The pointer must be a live allocation of the declared type with no outstanding native access; an opted-in Cleaner may release an unreachable wrapper without its monitor.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_drop_1InputHandle_17cc67535bbd4a8ee(
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
pub unsafe extern "system" fn Java_bench_bulk_JNINative_drop_1RowReturnHandler_10972ff7cb1556afa(
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
pub unsafe extern "system" fn Java_bench_bulk_JNINative_drop_1WideRowReturnHandler_1ed750f04931eb9ca(
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
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_i64_i64_06de1c5aa99699f6 {
    pub v0: i64,
    pub v1: i64,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_Range_1800d3dd3eceec6d {
    pub v0: ::prebindgen_jni_runtime::bulk::Range,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_Range_49da87bb4c377b9a {
    pub v0: ::prebindgen_jni_runtime::bulk::Range,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_Range_50871c2a8b72df4b {
    pub v0: ::prebindgen_jni_runtime::bulk::Range,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_Range_5e24e8a1a6fd1960 {
    pub v0: ::prebindgen_jni_runtime::bulk::Range,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_i64_i64_633c6cf82d3acfc1 {
    pub v0: i64,
    pub v1: i64,
    pub v2: i64,
    pub v3: i64,
    pub v4: i64,
    pub v5: i64,
    pub v6: i64,
    pub v7: i64,
    pub v8: i64,
    pub v9: i64,
    pub v10: i64,
    pub v11: i64,
    pub v12: i64,
    pub v13: i64,
    pub v14: i64,
    pub v15: i64,
    pub v16: i64,
    pub v17: i64,
    pub v18: i64,
    pub v19: i64,
    pub v20: i64,
    pub v21: i64,
    pub v22: i64,
    pub v23: i64,
    pub v24: i64,
    pub v25: i64,
    pub v26: i64,
    pub v27: i64,
    pub v28: i64,
    pub v29: i64,
    pub v30: i64,
    pub v31: i64,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_Range_71300b84e922dbba {
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
pub struct __PbBulk_i64_Range_9cc3e9775849e9ab {
    pub v0: i64,
    pub v1: ::prebindgen_jni_runtime::bulk::Range,
    pub v2: ::prebindgen_jni_runtime::bulk::Range,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_i64_a60d3960993b9a77 {
    pub v0: i64,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_i64_b2426f0d5f2715c0 {
    pub v0: i64,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_i64_i64_b3c08eb92fbbc74c {
    pub v0: i64,
    pub v1: i64,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_Range_cb0e46a0a248e38c {
    pub v0: ::prebindgen_jni_runtime::bulk::Range,
}
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_i64_d159346acd2a185e {
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
#[allow(non_camel_case_types, dead_code)]
#[repr(C)]
pub struct __PbBulk_Range_f738f16d97296060 {
    pub v0: ::prebindgen_jni_runtime::bulk::Range,
}
#[no_mangle]
pub extern "system" fn Java_bench_bulk_JNINative_bulkLayouts(
    mut env: ::prebindgen_jni_runtime::jni::JNIEnv,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass,
) -> ::prebindgen_jni_runtime::jni::sys::jlongArray {
    let result = (|| -> Result<_, String> {
        let mut data = vec![
            1i64, 6650375479507191082i64, if cfg!(target_endian = "big") { 1 } else { 0
            }, ::core::mem::size_of:: < usize > () as i64
        ];
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_i64_i64_06de1c5aa99699f6>(),
                ::core::mem::align_of::<__PbBulk_i64_i64_06de1c5aa99699f6>(),
                ::core::mem::offset_of!(__PbBulk_i64_i64_06de1c5aa99699f6, v0),
                ::core::mem::offset_of!(__PbBulk_i64_i64_06de1c5aa99699f6, v1),
            ])
                .into_iter()
                .map(|v| v as i64),
        );
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
                ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>(),
                ::core::mem::align_of::<__PbBulk_Range_49da87bb4c377b9a>(),
                ::core::mem::offset_of!(__PbBulk_Range_49da87bb4c377b9a, v0),
            ])
                .into_iter()
                .map(|v| v as i64),
        );
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_Range_50871c2a8b72df4b>(),
                ::core::mem::align_of::<__PbBulk_Range_50871c2a8b72df4b>(),
                ::core::mem::offset_of!(__PbBulk_Range_50871c2a8b72df4b, v0),
            ])
                .into_iter()
                .map(|v| v as i64),
        );
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_Range_5e24e8a1a6fd1960>(),
                ::core::mem::align_of::<__PbBulk_Range_5e24e8a1a6fd1960>(),
                ::core::mem::offset_of!(__PbBulk_Range_5e24e8a1a6fd1960, v0),
            ])
                .into_iter()
                .map(|v| v as i64),
        );
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_i64_i64_633c6cf82d3acfc1>(),
                ::core::mem::align_of::<__PbBulk_i64_i64_633c6cf82d3acfc1>(),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v0),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v1),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v2),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v3),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v4),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v5),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v6),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v7),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v8),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v9),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v10),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v11),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v12),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v13),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v14),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v15),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v16),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v17),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v18),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v19),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v20),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v21),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v22),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v23),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v24),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v25),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v26),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v27),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v28),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v29),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v30),
                ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v31),
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
                ::core::mem::size_of::<__PbBulk_i64_Range_9cc3e9775849e9ab>(),
                ::core::mem::align_of::<__PbBulk_i64_Range_9cc3e9775849e9ab>(),
                ::core::mem::offset_of!(__PbBulk_i64_Range_9cc3e9775849e9ab, v0),
                ::core::mem::offset_of!(__PbBulk_i64_Range_9cc3e9775849e9ab, v1),
                ::core::mem::offset_of!(__PbBulk_i64_Range_9cc3e9775849e9ab, v2),
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
                ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                ::core::mem::align_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                ::core::mem::offset_of!(__PbBulk_i64_b2426f0d5f2715c0, v0),
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
                ::core::mem::size_of::<__PbBulk_Range_cb0e46a0a248e38c>(),
                ::core::mem::align_of::<__PbBulk_Range_cb0e46a0a248e38c>(),
                ::core::mem::offset_of!(__PbBulk_Range_cb0e46a0a248e38c, v0),
            ])
                .into_iter()
                .map(|v| v as i64),
        );
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_i64_d159346acd2a185e>(),
                ::core::mem::align_of::<__PbBulk_i64_d159346acd2a185e>(),
                ::core::mem::offset_of!(__PbBulk_i64_d159346acd2a185e, v0),
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
        data.extend(
            ([
                ::core::mem::size_of::<__PbBulk_Range_f738f16d97296060>(),
                ::core::mem::align_of::<__PbBulk_Range_f738f16d97296060>(),
                ::core::mem::offset_of!(__PbBulk_Range_f738f16d97296060, v0),
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
unsafe fn __pb_cleanup_native_InputHandle_6935c4b287d7a13d(
    reader: &::prebindgen_jni_runtime::bulk::Reader<'_>,
    base: usize,
) {
    let _ = ::prebindgen_runtime::boundary(|| {
        let raw = reader
            .read::<
                i64,
            >(base + ::core::mem::offset_of!(__PbBulk_i64_b2426f0d5f2715c0, v0))?;
        if raw != 0 {
            let _ = ::prebindgen_runtime::boundary(|| unsafe {
                ::prebindgen_jni_runtime::drop_pointer::<bench_flat::InputHandle>(raw)
            });
        }
        Ok(())
    });
}
#[allow(dead_code, non_snake_case)]
unsafe fn __pb_cleanup_native_InputHandle_f2ac588fb3eed77d(
    reader: &::prebindgen_jni_runtime::bulk::Reader<'_>,
    base: usize,
) {
    let _ = ::prebindgen_runtime::boundary(|| {
        let raw = reader
            .read::<
                i64,
            >(base + ::core::mem::offset_of!(__PbBulk_i64_Range_9cc3e9775849e9ab, v0))?;
        if raw != 0 {
            let _ = ::prebindgen_runtime::boundary(|| unsafe {
                ::prebindgen_jni_runtime::drop_pointer::<bench_flat::InputHandle>(raw)
            });
        }
        Ok(())
    });
}
#[allow(dead_code, non_snake_case)]
unsafe fn __pb_cleanup_native_Rows_ca08f1b833732d9d(
    reader: &::prebindgen_jni_runtime::bulk::Reader<'_>,
    base: usize,
) {
    let _ = ::prebindgen_runtime::boundary(|| {
        if let Some(range) = reader
            .range(base + ::core::mem::offset_of!(__PbBulk_Range_49da87bb4c377b9a, v0))?
        {
            let (start, length) = reader
                .rows(
                    Some(range),
                    ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                )?;
            for index in 0..length {
                unsafe {
                    __pb_cleanup_native_InputHandle_6935c4b287d7a13d(
                        reader,
                        start
                            + index
                                * ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                    );
                }
            }
        }
        Ok(())
    });
}
#[allow(
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1input_1handle_1new_1d7aa473c3cabcfa5<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: i64 = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1input_1handle_1value_1ae1ea4e2f311cdb7<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &__pb_input_consumed;
            let p0_w1_pointer = if true {
                p0_w1
                    as *mut ::prebindgen_jni_runtime::Allocation<bench_flat::InputHandle>
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: &bench_flat::InputHandle = unsafe {
                        &(*p0_w1_pointer).value
                    };
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1buffer_1sizes_109330eed51da0791<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                Some(::prebindgen_jni_runtime::text(&mut jni_environment, &p0_w1)?)
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(
                    ::core::convert::AsRef::<
                        str,
                    >::as_ref(p0_w0.as_ref().expect("active value")),
                )
            } else {
                None
            };
            let p1_w0 = if true {
                Some(::prebindgen_jni_runtime::bytes(&mut jni_environment, &p1_w1)?)
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p1_w0_built = if true {
                Some(
                    ::core::convert::AsRef::<
                        [u8],
                    >::as_ref(p1_w0.as_ref().expect("active value")),
                )
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let __prebindgen_argument_1 = p1_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1selected_1handle_15e0482e736d8e05c<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &__pb_input_consumed;
            if !(0..2i32).contains(&p0_w1) {
                return Err("invalid selector".into());
            }
            let p0_w2 = if p0_w1 == 0i32 { Some(p0_w3) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if p0_w1 == 0i32 {
                Some({
                    let p0_w2: i64 = ::core::convert::Into::into(
                        p0_w2.expect("active value"),
                    );
                    p0_w2
                })
            } else {
                None
            };
            let p0_w6_pointer = if p0_w1 == 1i32 {
                p0_w6
                    as *mut ::prebindgen_jni_runtime::Allocation<bench_flat::InputHandle>
            } else {
                ::core::ptr::null_mut()
            };
            if p0_w1 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w5_built = if p0_w1 == 1i32 {
                Some({
                    let p0_w5: &bench_flat::InputHandle = unsafe {
                        &(*p0_w6_pointer).value
                    };
                    p0_w5
                })
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w4_built = if p0_w1 == 1i32 {
                Some({
                    let p0_w4: i64 = bench_flat::input_handle_value(
                        p0_w5_built.expect("active input"),
                    );
                    p0_w4
                })
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: i64 = match p0_w1 {
                        0i32 => p0_w2_built.expect("active input"),
                        1i32 => p0_w4_built.expect("active input"),
                        _ => unreachable!("validated selector"),
                    };
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1handle_1values_1sum_10c275022668af098<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &__pb_input_consumed;
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_Range_f738f16d97296060>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3 = if true {
                p0_w1_reader
                    .range(::core::mem::offset_of!(__PbBulk_Range_f738f16d97296060, v0))?
            } else {
                None
            };
            let mut p0_w2: Vec<i64> = Vec::new();
            if true {
                let (p0_w3_start, p0_w4) = p0_w1_reader
                    .rows(
                        p0_w3,
                        ::core::mem::size_of::<__PbBulk_i64_d159346acd2a185e>(),
                    )?;
                p0_w2.reserve_exact(p0_w4);
                for p0_w5 in 0..p0_w4 {
                    let p0_w9 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<__PbBulk_i64_d159346acd2a185e>())
                                + ::core::mem::offset_of!(__PbBulk_i64_d159346acd2a185e, v0),
                        )?;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w3_element_built = (|| -> ::core::result::Result<i64, String> {
                        let p0_w9_pointer = if true {
                            (p0_w9)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w9_pointer)?;
                        }
                        let p0_w8: &bench_flat::InputHandle = unsafe {
                            &(*p0_w9_pointer).value
                        };
                        let p0_w7: i64 = bench_flat::input_handle_value(p0_w8);
                        let p0_w2 = p0_w7;
                        Ok(p0_w2)
                    })();
                    let element = p0_w3_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w2.len();
                        p0_w2.as_mut_ptr().add(initialized).write(element);
                        p0_w2.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true { Some(p0_w2) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1handle_1records_1sum_14d3c2822396924df<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &__pb_input_consumed;
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_Range_cb0e46a0a248e38c>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3 = if true {
                p0_w1_reader
                    .range(::core::mem::offset_of!(__PbBulk_Range_cb0e46a0a248e38c, v0))?
            } else {
                None
            };
            let mut p0_w2: Vec<bench_flat::HandleRecord> = Vec::new();
            if true {
                let (p0_w3_start, p0_w4) = p0_w1_reader
                    .rows(
                        p0_w3,
                        ::core::mem::size_of::<__PbBulk_i64_i64_06de1c5aa99699f6>(),
                    )?;
                p0_w2.reserve_exact(p0_w4);
                for p0_w5 in 0..p0_w4 {
                    let p0_w9 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_06de1c5aa99699f6,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_06de1c5aa99699f6, v0
                                ),
                        )?;
                    let p0_w7_f1 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_06de1c5aa99699f6,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_06de1c5aa99699f6, v1
                                ),
                        )?;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w3_element_built = (|| -> ::core::result::Result<
                        bench_flat::HandleRecord,
                        String,
                    > {
                        let p0_w9_pointer = if true {
                            (p0_w9)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w9_pointer)?;
                        }
                        let p0_w6_f1 = if true { Some(p0_w7_f1) } else { None };
                        let p0_w8: &bench_flat::InputHandle = unsafe {
                            &(*p0_w9_pointer).value
                        };
                        let p0_w6_f0: bench_flat::InputHandle = bench_flat::input_handle_copy(
                            p0_w8,
                        );
                        let p0_w6_f1: i64 = ::core::convert::Into::into(
                            p0_w6_f1.expect("active value"),
                        );
                        let p0_w6: bench_flat::HandleRecord = bench_flat::HandleRecord {
                            handle: p0_w6_f0,
                            tag: p0_w6_f1,
                        };
                        let p0_w2 = p0_w6;
                        Ok(p0_w2)
                    })();
                    let element = p0_w3_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w2.len();
                        p0_w2.as_mut_ptr().add(initialized).write(element);
                        p0_w2.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true { Some(p0_w2) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1owned_1handles_1echo_1e97ab382b6bce5df<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JByteArray<'local>,
    p0_w1_length: i32,
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
        {
            let __prebindgen_input_consumed = &__pb_input_consumed;
            let _: () = {
                if jni_environment
                    .get_array_length(&ownership_accepted)
                    .map_err(|e| e.to_string())? != 1
                {
                    return Err("invalid ownership acknowledgement".into());
                }
                __pb_receipt_valid.set(true);
            };
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3 = if true {
                p0_w1_reader
                    .range(::core::mem::offset_of!(__PbBulk_Range_49da87bb4c377b9a, v0))?
            } else {
                None
            };
            let mut p0_w2: Vec<bench_flat::InputHandle> = Vec::new();
            if true {
                let (p0_w3_start, p0_w4) = p0_w1_reader
                    .rows(
                        p0_w3,
                        ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                    )?;
                p0_w2.reserve_exact(p0_w4);
                for p0_w5 in 0..p0_w4 {
                    let p0_w8 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>())
                                + ::core::mem::offset_of!(__PbBulk_i64_b2426f0d5f2715c0, v0),
                        )?;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w3_element_built = (|| -> ::core::result::Result<
                        bench_flat::InputHandle,
                        String,
                    > {
                        let p0_w8_pointer = if true {
                            (p0_w8)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w8_pointer)?;
                        }
                        if true {
                            __prebindgen_input_consumed
                                .set(__prebindgen_input_consumed.get() + (1usize));
                        }
                        let p0_w7 = if true {
                            Some(unsafe { Box::from_raw(p0_w8_pointer) })
                        } else {
                            None
                        };
                        let p0_w7: bench_flat::InputHandle = p0_w7
                            .expect("active allocation")
                            .value;
                        let p0_w2 = p0_w7;
                        Ok(p0_w2)
                    })();
                    let element = p0_w3_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w2.len();
                        p0_w2.as_mut_ptr().add(initialized).write(element);
                        p0_w2.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true { Some(p0_w2) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = bench_flat::owned_handles_echo(__prebindgen_argument_0);
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>(),
            );
            let ret_w1_source: ::std::vec::Vec<bench_flat::InputHandle> = source_value;
            let ret_w3 = ::std::vec::Vec::len(&ret_w1_source);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                    ::core::mem::align_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                )?;
            let ret_w1_owners = unsafe {
                ret_w0_writer
                    .own_rows(
                        ret_w1,
                        ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                        __pb_cleanup_native_InputHandle_6935c4b287d7a13d,
                    )
            };
            for (ret_w4, ret_w2) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w1_source),
            ) {
                let mut ret_w5 = Box::new(::prebindgen_jni_runtime::Allocation {
                    value: ret_w2,
                });
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>())
                            + ::core::mem::offset_of!(__PbBulk_i64_b2426f0d5f2715c0, v0),
                        ((&mut *ret_w5)
                            as *mut ::prebindgen_jni_runtime::Allocation<
                                bench_flat::InputHandle,
                            >) as i64,
                    )?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    let _ = Box::into_raw(ret_w5).expose_provenance();
                })();
                unsafe {
                    ret_w0_writer.commit_rows(ret_w1_owners, ret_w4 + 1);
                }
            }
            let ret_w1 = Some(ret_w1);
            ret_w0_writer
                .range(
                    ::core::mem::offset_of!(__PbBulk_Range_49da87bb4c377b9a, v0),
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1owned_1handle_1records_1echo_17223d199891bf713<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JByteArray<'local>,
    p0_w1_length: i32,
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
        {
            let __prebindgen_input_consumed = &__pb_input_consumed;
            let _: () = {
                if jni_environment
                    .get_array_length(&ownership_accepted)
                    .map_err(|e| e.to_string())? != 1
                {
                    return Err("invalid ownership acknowledgement".into());
                }
                __pb_receipt_valid.set(true);
            };
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_Range_71300b84e922dbba>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3 = if true {
                p0_w1_reader
                    .range(::core::mem::offset_of!(__PbBulk_Range_71300b84e922dbba, v0))?
            } else {
                None
            };
            let mut p0_w2: Vec<bench_flat::HandleRecord> = Vec::new();
            if true {
                let (p0_w3_start, p0_w4) = p0_w1_reader
                    .rows(
                        p0_w3,
                        ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                    )?;
                p0_w2.reserve_exact(p0_w4);
                for p0_w5 in 0..p0_w4 {
                    let p0_w7_f0 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_b3c08eb92fbbc74c, v0
                                ),
                        )?;
                    let p0_w7_f1 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_i64_b3c08eb92fbbc74c, v1
                                ),
                        )?;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w3_element_built = (|| -> ::core::result::Result<
                        bench_flat::HandleRecord,
                        String,
                    > {
                        let p0_w7_f0_pointer = if true {
                            (p0_w7_f0)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w7_f0_pointer)?;
                        }
                        let p0_w6_f1 = if true { Some(p0_w7_f1) } else { None };
                        if true {
                            __prebindgen_input_consumed
                                .set(__prebindgen_input_consumed.get() + (1usize));
                        }
                        let p0_w6_f0 = if true {
                            Some(unsafe { Box::from_raw(p0_w7_f0_pointer) })
                        } else {
                            None
                        };
                        let p0_w6_f0: bench_flat::InputHandle = p0_w6_f0
                            .expect("active allocation")
                            .value;
                        let p0_w6_f1: i64 = ::core::convert::Into::into(
                            p0_w6_f1.expect("active value"),
                        );
                        let p0_w6: bench_flat::HandleRecord = bench_flat::HandleRecord {
                            handle: p0_w6_f0,
                            tag: p0_w6_f1,
                        };
                        let p0_w2 = p0_w6;
                        Ok(p0_w2)
                    })();
                    let element = p0_w3_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w2.len();
                        p0_w2.as_mut_ptr().add(initialized).write(element);
                        p0_w2.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true { Some(p0_w2) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = bench_flat::owned_handle_records_echo(
                __prebindgen_argument_0,
            );
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_71300b84e922dbba>(),
            );
            let ret_w1_source: ::std::vec::Vec<bench_flat::HandleRecord> = source_value;
            let ret_w3 = ::std::vec::Vec::len(&ret_w1_source);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                    ::core::mem::align_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                )?;
            let ret_w1_owners = unsafe {
                ret_w0_writer
                    .own_rows(
                        ret_w1,
                        ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                        __pb_cleanup_native_InputHandle_567ed8299898558e,
                    )
            };
            for (ret_w4, ret_w2) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w1_source),
            ) {
                let ret_w5_record = ret_w2;
                let mut ret_w5_f0 = Box::new(::prebindgen_jni_runtime::Allocation {
                    value: ret_w5_record.handle,
                });
                let ret_w5_f1 = ret_w5_record.tag;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_b3c08eb92fbbc74c, v0
                            ),
                        ((&mut *ret_w5_f0)
                            as *mut ::prebindgen_jni_runtime::Allocation<
                                bench_flat::InputHandle,
                            >) as i64,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_b3c08eb92fbbc74c, v1
                            ),
                        ret_w5_f1,
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
                    ::core::mem::offset_of!(__PbBulk_Range_71300b84e922dbba, v0),
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1callback_1owned_1records_159a5e85070935754<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "()[Ljava/lang/Object;",
                "bench.bulk.OwnedRecordsCallback",
            )?;
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_callback_value_built = if true {
                Some({
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
                                            let callback_return_w1 = ::prebindgen_jni_runtime::row_bytes(
                                                &mut callback_environment,
                                                &(callback_result_rows),
                                                0usize,
                                            )?;
                                            let callback_return_w1_length = ::prebindgen_jni_runtime::row_i32(
                                                &mut callback_environment,
                                                &(callback_result_rows),
                                                1usize,
                                            )?;
                                            let callback_return_w1_cached_bytes = if true {
                                                ::prebindgen_jni_runtime::bulk::input(
                                                    &mut callback_environment,
                                                    &callback_return_w1,
                                                    callback_return_w1_length,
                                                )?
                                            } else {
                                                Vec::new()
                                            };
                                            let mut callback_return_cleanup = ::prebindgen_runtime::CallbackReturnCleanup::new(||
                                            {
                                                let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                                    __prebindgen_input_consumed,
                                                );
                                                if true {
                                                    let callback_return_w1_cleanup_reader = ::prebindgen_jni_runtime::bulk::Reader::new(
                                                        &callback_return_w1_cached_bytes,
                                                    );
                                                    if let Err(error) = (|| -> ::prebindgen_runtime::Result<()> {
                                                        let (
                                                            callback_return_w3_cleanup_start,
                                                            callback_return_w3_cleanup_count,
                                                        ) = callback_return_w1_cleanup_reader
                                                            .rows(
                                                                callback_return_w1_cleanup_reader
                                                                    .range(
                                                                        ::core::mem::offset_of!(__PbBulk_Range_71300b84e922dbba, v0),
                                                                    )?,
                                                                ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                                                            )?;
                                                        for callback_return_w3_cleanup_index in 0..callback_return_w3_cleanup_count {
                                                            if let Err(error) = (|| -> ::prebindgen_runtime::Result<()> {
                                                                unsafe {
                                                                    callback_return_owners
                                                                        .receive(
                                                                            (callback_return_w1_cleanup_reader
                                                                                .read::<
                                                                                    i64,
                                                                                >(
                                                                                    (callback_return_w3_cleanup_start
                                                                                        + callback_return_w3_cleanup_index
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
                                                Ok(())
                                            });
                                            {
                                                let callback_return_decoded = ({
                                                    let callback_return_w1_bytes = if true {
                                                        Ok::<_, String>(callback_return_w1_cached_bytes.as_slice())?
                                                    } else {
                                                        ::core::default::Default::default()
                                                    };
                                                    let callback_return_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                                                        &callback_return_w1_bytes,
                                                    );
                                                    if callback_return_w1_bytes.len()
                                                        < ::core::mem::size_of::<__PbBulk_Range_71300b84e922dbba>()
                                                    {
                                                        return Err("truncated bulk root record".into());
                                                    }
                                                    let callback_return_w3 = if true {
                                                        callback_return_w1_reader
                                                            .range(
                                                                ::core::mem::offset_of!(__PbBulk_Range_71300b84e922dbba, v0),
                                                            )?
                                                    } else {
                                                        None
                                                    };
                                                    let mut callback_return_w2: Vec<bench_flat::HandleRecord> = Vec::new();
                                                    if true {
                                                        let (callback_return_w3_start, callback_return_w4) = callback_return_w1_reader
                                                            .rows(
                                                                callback_return_w3,
                                                                ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                                                            )?;
                                                        callback_return_w2.reserve_exact(callback_return_w4);
                                                        for callback_return_w5 in 0..callback_return_w4 {
                                                            let callback_return_w7_f0 = callback_return_w1_reader
                                                                .read::<
                                                                    i64,
                                                                >(
                                                                    (callback_return_w3_start
                                                                        + callback_return_w5
                                                                            * ::core::mem::size_of::<
                                                                                __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                                                            >())
                                                                        + ::core::mem::offset_of!(
                                                                            __PbBulk_i64_i64_b3c08eb92fbbc74c, v0
                                                                        ),
                                                                )?;
                                                            let callback_return_w7_f1 = callback_return_w1_reader
                                                                .read::<
                                                                    i64,
                                                                >(
                                                                    (callback_return_w3_start
                                                                        + callback_return_w5
                                                                            * ::core::mem::size_of::<
                                                                                __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                                                            >())
                                                                        + ::core::mem::offset_of!(
                                                                            __PbBulk_i64_i64_b3c08eb92fbbc74c, v1
                                                                        ),
                                                                )?;
                                                            #[allow(clippy::redundant_closure_call)]
                                                            let callback_return_w3_element_built = (|| -> ::core::result::Result<
                                                                bench_flat::HandleRecord,
                                                                String,
                                                            > {
                                                                let callback_return_w7_f0_pointer = if true {
                                                                    (callback_return_w7_f0)
                                                                        as *mut ::prebindgen_jni_runtime::Allocation<
                                                                            bench_flat::InputHandle,
                                                                        >
                                                                } else {
                                                                    ::core::ptr::null_mut()
                                                                };
                                                                if true {
                                                                    ::prebindgen_runtime::validate_pointer(
                                                                        callback_return_w7_f0_pointer,
                                                                    )?;
                                                                }
                                                                let callback_return_w6_f1 = if true {
                                                                    Some(callback_return_w7_f1)
                                                                } else {
                                                                    None
                                                                };
                                                                if true {
                                                                    __prebindgen_input_consumed
                                                                        .set(__prebindgen_input_consumed.get() + (1usize));
                                                                }
                                                                let callback_return_w6_f0 = if true {
                                                                    Some(unsafe {
                                                                        Box::from_raw(callback_return_w7_f0_pointer)
                                                                    })
                                                                } else {
                                                                    None
                                                                };
                                                                let callback_return_w6_f0: bench_flat::InputHandle = callback_return_w6_f0
                                                                    .expect("active allocation")
                                                                    .value;
                                                                let callback_return_w6_f1: i64 = ::core::convert::Into::into(
                                                                    callback_return_w6_f1.expect("active value"),
                                                                );
                                                                let callback_return_w6: bench_flat::HandleRecord = bench_flat::HandleRecord {
                                                                    handle: callback_return_w6_f0,
                                                                    tag: callback_return_w6_f1,
                                                                };
                                                                let callback_return_w2 = callback_return_w6;
                                                                Ok(callback_return_w2)
                                                            })();
                                                            let element = callback_return_w3_element_built?;
                                                            #[allow(clippy::zst_offset)]
                                                            unsafe {
                                                                let initialized = callback_return_w2.len();
                                                                callback_return_w2
                                                                    .as_mut_ptr()
                                                                    .add(initialized)
                                                                    .write(element);
                                                                callback_return_w2.set_len(initialized + 1);
                                                            }
                                                        }
                                                    }
                                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                                    let callback_return_w2_built = if true {
                                                        Some(callback_return_w2)
                                                    } else {
                                                        None
                                                    };
                                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                                    let callback_return_w0_built = if true {
                                                        Some(callback_return_w2_built.expect("active input"))
                                                    } else {
                                                        None
                                                    };
                                                    let callback_return_value = callback_return_w0_built
                                                        .expect("active input");
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
                    p0_callback_value
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_callback_value_built.expect("active input");
            let source_value = bench_flat::callback_owned_records(
                __prebindgen_argument_0,
            );
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_71300b84e922dbba>(),
            );
            let ret_w1_source: ::std::vec::Vec<bench_flat::HandleRecord> = source_value;
            let ret_w3 = ::std::vec::Vec::len(&ret_w1_source);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                    ::core::mem::align_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                )?;
            let ret_w1_owners = unsafe {
                ret_w0_writer
                    .own_rows(
                        ret_w1,
                        ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
                        __pb_cleanup_native_InputHandle_567ed8299898558e,
                    )
            };
            for (ret_w4, ret_w2) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w1_source),
            ) {
                let ret_w5_record = ret_w2;
                let mut ret_w5_f0 = Box::new(::prebindgen_jni_runtime::Allocation {
                    value: ret_w5_record.handle,
                });
                let ret_w5_f1 = ret_w5_record.tag;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_b3c08eb92fbbc74c, v0
                            ),
                        ((&mut *ret_w5_f0)
                            as *mut ::prebindgen_jni_runtime::Allocation<
                                bench_flat::InputHandle,
                            >) as i64,
                    )?;
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_i64_b3c08eb92fbbc74c, v1
                            ),
                        ret_w5_f1,
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
                    ::core::mem::offset_of!(__PbBulk_Range_71300b84e922dbba, v0),
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1callback_1owned_1record_1c829b73c0af8c75b<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "()[Ljava/lang/Object;",
                "bench.bulk.OwnedRecordCallback",
            )?;
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_callback_value_built = if true {
                Some({
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
                                            let callback_return_w1 = ::prebindgen_jni_runtime::row_bytes(
                                                &mut callback_environment,
                                                &(callback_result_rows),
                                                0usize,
                                            )?;
                                            let callback_return_w1_length = ::prebindgen_jni_runtime::row_i32(
                                                &mut callback_environment,
                                                &(callback_result_rows),
                                                1usize,
                                            )?;
                                            let callback_return_w1_cached_bytes = if true {
                                                ::prebindgen_jni_runtime::bulk::input(
                                                    &mut callback_environment,
                                                    &callback_return_w1,
                                                    callback_return_w1_length,
                                                )?
                                            } else {
                                                Vec::new()
                                            };
                                            let mut callback_return_cleanup = ::prebindgen_runtime::CallbackReturnCleanup::new(||
                                            {
                                                let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                                    __prebindgen_input_consumed,
                                                );
                                                if true {
                                                    let callback_return_w1_cleanup_reader = ::prebindgen_jni_runtime::bulk::Reader::new(
                                                        &callback_return_w1_cached_bytes,
                                                    );
                                                    if let Err(error) = (|| -> ::prebindgen_runtime::Result<()> {
                                                        unsafe {
                                                            callback_return_owners
                                                                .receive(
                                                                    (callback_return_w1_cleanup_reader
                                                                        .read::<
                                                                            i64,
                                                                        >(
                                                                            ::core::mem::offset_of!(
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
                                            });
                                            {
                                                let callback_return_decoded = ({
                                                    let callback_return_w1_bytes = if true {
                                                        Ok::<_, String>(callback_return_w1_cached_bytes.as_slice())?
                                                    } else {
                                                        ::core::default::Default::default()
                                                    };
                                                    let callback_return_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                                                        &callback_return_w1_bytes,
                                                    );
                                                    if callback_return_w1_bytes.len()
                                                        < ::core::mem::size_of::<
                                                            __PbBulk_i64_i64_b3c08eb92fbbc74c,
                                                        >()
                                                    {
                                                        return Err("truncated bulk root record".into());
                                                    }
                                                    let callback_return_w3_f0 = if true {
                                                        callback_return_w1_reader
                                                            .read::<
                                                                i64,
                                                            >(
                                                                ::core::mem::offset_of!(
                                                                    __PbBulk_i64_i64_b3c08eb92fbbc74c, v0
                                                                ),
                                                            )?
                                                    } else {
                                                        0
                                                    };
                                                    let callback_return_w3_f1 = if true {
                                                        callback_return_w1_reader
                                                            .read::<
                                                                i64,
                                                            >(
                                                                ::core::mem::offset_of!(
                                                                    __PbBulk_i64_i64_b3c08eb92fbbc74c, v1
                                                                ),
                                                            )?
                                                    } else {
                                                        0
                                                    };
                                                    let callback_return_w3_f0_pointer = if true {
                                                        callback_return_w3_f0
                                                            as *mut ::prebindgen_jni_runtime::Allocation<
                                                                bench_flat::InputHandle,
                                                            >
                                                    } else {
                                                        ::core::ptr::null_mut()
                                                    };
                                                    if true {
                                                        ::prebindgen_runtime::validate_pointer(
                                                            callback_return_w3_f0_pointer,
                                                        )?;
                                                    }
                                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                                    let callback_return_w2_f0_built = if true {
                                                        let callback_return_w2_f0 = if true {
                                                            Some(unsafe {
                                                                Box::from_raw(callback_return_w3_f0_pointer)
                                                            })
                                                        } else {
                                                            None
                                                        };
                                                        if true {
                                                            __prebindgen_input_consumed
                                                                .set(__prebindgen_input_consumed.get() + (1usize));
                                                        }
                                                        Some({
                                                            let callback_return_w2_f0: bench_flat::InputHandle = callback_return_w2_f0
                                                                .expect("active allocation")
                                                                .value;
                                                            callback_return_w2_f0
                                                        })
                                                    } else {
                                                        None
                                                    };
                                                    let callback_return_w2_f1 = if true {
                                                        Some(callback_return_w3_f1)
                                                    } else {
                                                        None
                                                    };
                                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                                    let callback_return_w2_f1_built = if true {
                                                        Some({
                                                            let callback_return_w2_f1: i64 = ::core::convert::Into::into(
                                                                callback_return_w2_f1.expect("active value"),
                                                            );
                                                            callback_return_w2_f1
                                                        })
                                                    } else {
                                                        None
                                                    };
                                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                                    let callback_return_w2_built = if true {
                                                        Some({
                                                            let callback_return_w2: bench_flat::HandleRecord = bench_flat::HandleRecord {
                                                                handle: callback_return_w2_f0_built.expect("active input"),
                                                                tag: callback_return_w2_f1_built.expect("active input"),
                                                            };
                                                            callback_return_w2
                                                        })
                                                    } else {
                                                        None
                                                    };
                                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                                    let callback_return_w0_built = if true {
                                                        Some(callback_return_w2_built.expect("active input"))
                                                    } else {
                                                        None
                                                    };
                                                    let callback_return_value = callback_return_w0_built
                                                        .expect("active input");
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
                    p0_callback_value
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_callback_value_built.expect("active input");
            let source_value = bench_flat::callback_owned_record(
                __prebindgen_argument_0,
            );
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_i64_i64_b3c08eb92fbbc74c>(),
            );
            let ret_w1_record = source_value;
            let mut ret_w1_f0 = Box::new(::prebindgen_jni_runtime::Allocation {
                value: ret_w1_record.handle,
            });
            let ret_w1_f1 = ret_w1_record.tag;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_b3c08eb92fbbc74c, v0),
                    ((&mut *ret_w1_f0)
                        as *mut ::prebindgen_jni_runtime::Allocation<
                            bench_flat::InputHandle,
                        >) as i64,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_b3c08eb92fbbc74c, v1),
                    ret_w1_f1,
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
                let _ = Box::into_raw(ret_w1_f0).expose_provenance();
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1owned_1buffer_1records_1echo_132801b2928e6b02f<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JByteArray<'local>,
    p0_w1_length: i32,
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
        {
            let __prebindgen_input_consumed = &__pb_input_consumed;
            let _: () = {
                if jni_environment
                    .get_array_length(&ownership_accepted)
                    .map_err(|e| e.to_string())? != 1
                {
                    return Err("invalid ownership acknowledgement".into());
                }
                __pb_receipt_valid.set(true);
            };
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_Range_50871c2a8b72df4b>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3 = if true {
                p0_w1_reader
                    .range(::core::mem::offset_of!(__PbBulk_Range_50871c2a8b72df4b, v0))?
            } else {
                None
            };
            let mut p0_w2: Vec<bench_flat::BufferRecord> = Vec::new();
            if true {
                let (p0_w3_start, p0_w4) = p0_w1_reader
                    .rows(
                        p0_w3,
                        ::core::mem::size_of::<__PbBulk_i64_Range_9cc3e9775849e9ab>(),
                    )?;
                p0_w2.reserve_exact(p0_w4);
                for p0_w5 in 0..p0_w4 {
                    let p0_w7_f0 = p0_w1_reader
                        .read::<
                            i64,
                        >(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_Range_9cc3e9775849e9ab,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_Range_9cc3e9775849e9ab, v0
                                ),
                        )?;
                    let p0_w7_f1 = p0_w1_reader
                        .range(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_Range_9cc3e9775849e9ab,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_Range_9cc3e9775849e9ab, v1
                                ),
                        )?;
                    let p0_w7_f2 = p0_w1_reader
                        .range(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<
                                        __PbBulk_i64_Range_9cc3e9775849e9ab,
                                    >())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_Range_9cc3e9775849e9ab, v2
                                ),
                        )?;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w3_element_built = (|| -> ::core::result::Result<
                        bench_flat::BufferRecord,
                        String,
                    > {
                        let p0_w7_f0_pointer = if true {
                            (p0_w7_f0)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w7_f0_pointer)?;
                        }
                        if true {
                            __prebindgen_input_consumed
                                .set(__prebindgen_input_consumed.get() + (1usize));
                        }
                        let p0_w6_f0 = if true {
                            Some(unsafe { Box::from_raw(p0_w7_f0_pointer) })
                        } else {
                            None
                        };
                        #[allow(unused_mut, unused_assignments)]
                        let mut p0_w6_f1: Option<&str> = None;
                        #[allow(unused_mut, unused_assignments)]
                        let mut p0_w6_f2: Option<&[u8]> = None;
                        let p0_w6_f0: bench_flat::InputHandle = p0_w6_f0
                            .expect("active allocation")
                            .value;
                        p0_w6_f1 = if true {
                            Some(
                                ::core::str::from_utf8(
                                        p0_w1_reader
                                            .buffer_range(p0_w7_f1)?
                                            .ok_or("null bulk text")?,
                                    )
                                    .map_err(|_| "invalid UTF-8 in bulk text")?,
                            )
                        } else {
                            None
                        };
                        let p0_w6_f1: String = ::core::convert::Into::into(
                            p0_w6_f1.expect("active value"),
                        );
                        p0_w6_f2 = if true {
                            Some(
                                p0_w1_reader
                                    .buffer_range(p0_w7_f2)?
                                    .ok_or("null bulk bytes")?,
                            )
                        } else {
                            None
                        };
                        let p0_w6_f2: Vec<u8> = ::core::convert::Into::into(
                            p0_w6_f2.expect("active value"),
                        );
                        let p0_w6: bench_flat::BufferRecord = bench_flat::BufferRecord {
                            handle: p0_w6_f0,
                            text: p0_w6_f1,
                            bytes: p0_w6_f2,
                        };
                        let p0_w2 = p0_w6;
                        Ok(p0_w2)
                    })();
                    let element = p0_w3_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w2.len();
                        p0_w2.as_mut_ptr().add(initialized).write(element);
                        p0_w2.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true { Some(p0_w2) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = bench_flat::owned_buffer_records_echo(
                __prebindgen_argument_0,
            );
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_50871c2a8b72df4b>(),
            );
            let ret_w1_source: ::std::vec::Vec<bench_flat::BufferRecord> = source_value;
            let ret_w3 = ::std::vec::Vec::len(&ret_w1_source);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_Range_9cc3e9775849e9ab>(),
                    ::core::mem::align_of::<__PbBulk_i64_Range_9cc3e9775849e9ab>(),
                )?;
            let ret_w1_owners = unsafe {
                ret_w0_writer
                    .own_rows(
                        ret_w1,
                        ::core::mem::size_of::<__PbBulk_i64_Range_9cc3e9775849e9ab>(),
                        __pb_cleanup_native_InputHandle_f2ac588fb3eed77d,
                    )
            };
            for (ret_w4, ret_w2) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w1_source),
            ) {
                let ret_w5_record = ret_w2;
                let mut ret_w5_f0 = Box::new(::prebindgen_jni_runtime::Allocation {
                    value: ret_w5_record.handle,
                });
                let ret_w5_f1_source = ret_w5_record.text;
                let ret_w5_f1 = Some(
                    ret_w0_writer.append_buffer(ret_w5_f1_source.as_bytes())?,
                );
                let ret_w5_f2_source = ret_w5_record.bytes;
                let ret_w5_f2 = Some(
                    ret_w0_writer.append_buffer(ret_w5_f2_source.as_ref())?,
                );
                ret_w0_writer
                    .write(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_Range_9cc3e9775849e9ab,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_Range_9cc3e9775849e9ab, v0
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
                                    __PbBulk_i64_Range_9cc3e9775849e9ab,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_Range_9cc3e9775849e9ab, v1
                            ),
                        ret_w5_f1,
                    )?;
                ret_w0_writer
                    .range(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<
                                    __PbBulk_i64_Range_9cc3e9775849e9ab,
                                >())
                            + ::core::mem::offset_of!(
                                __PbBulk_i64_Range_9cc3e9775849e9ab, v2
                            ),
                        ret_w5_f2,
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
                    ::core::mem::offset_of!(__PbBulk_Range_50871c2a8b72df4b, v0),
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1nested_1owned_1handles_1echo_1656072b3182d8e7b<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: ::prebindgen_jni_runtime::jni::objects::JByteArray<'local>,
    p0_w1_length: i32,
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
        {
            let __prebindgen_input_consumed = &__pb_input_consumed;
            let _: () = {
                if jni_environment
                    .get_array_length(&ownership_accepted)
                    .map_err(|e| e.to_string())? != 1
                {
                    return Err("invalid ownership acknowledgement".into());
                }
                __pb_receipt_valid.set(true);
            };
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_Range_5e24e8a1a6fd1960>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3 = if true {
                p0_w1_reader
                    .range(::core::mem::offset_of!(__PbBulk_Range_5e24e8a1a6fd1960, v0))?
            } else {
                None
            };
            let mut p0_w2: Vec<Vec<bench_flat::InputHandle>> = Vec::new();
            if true {
                let (p0_w3_start, p0_w4) = p0_w1_reader
                    .rows(
                        p0_w3,
                        ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>(),
                    )?;
                p0_w2.reserve_exact(p0_w4);
                for p0_w5 in 0..p0_w4 {
                    let p0_w8 = p0_w1_reader
                        .range(
                            (p0_w3_start
                                + p0_w5
                                    * ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>())
                                + ::core::mem::offset_of!(
                                    __PbBulk_Range_49da87bb4c377b9a, v0
                                ),
                        )?;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w3_element_built = (|| -> ::core::result::Result<
                        Vec<bench_flat::InputHandle>,
                        String,
                    > {
                        let mut p0_w7: Vec<bench_flat::InputHandle> = Vec::new();
                        if true {
                            let (p0_w8_start, p0_w9) = p0_w1_reader
                                .rows(
                                    p0_w8,
                                    ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                                )?;
                            p0_w7.reserve_exact(p0_w9);
                            for p0_w10 in 0..p0_w9 {
                                let p0_w13 = p0_w1_reader
                                    .read::<
                                        i64,
                                    >(
                                        (p0_w8_start
                                            + p0_w10
                                                * ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>())
                                            + ::core::mem::offset_of!(__PbBulk_i64_b2426f0d5f2715c0, v0),
                                    )?;
                                #[allow(clippy::redundant_closure_call)]
                                let p0_w8_element_built = (|| -> ::core::result::Result<
                                    bench_flat::InputHandle,
                                    String,
                                > {
                                    let p0_w13_pointer = if true {
                                        (p0_w13)
                                            as *mut ::prebindgen_jni_runtime::Allocation<
                                                bench_flat::InputHandle,
                                            >
                                    } else {
                                        ::core::ptr::null_mut()
                                    };
                                    if true {
                                        ::prebindgen_runtime::validate_pointer(p0_w13_pointer)?;
                                    }
                                    if true {
                                        __prebindgen_input_consumed
                                            .set(__prebindgen_input_consumed.get() + (1usize));
                                    }
                                    let p0_w12 = if true {
                                        Some(unsafe { Box::from_raw(p0_w13_pointer) })
                                    } else {
                                        None
                                    };
                                    let p0_w12: bench_flat::InputHandle = p0_w12
                                        .expect("active allocation")
                                        .value;
                                    let p0_w7 = p0_w12;
                                    Ok(p0_w7)
                                })();
                                let element = p0_w8_element_built?;
                                #[allow(clippy::zst_offset)]
                                unsafe {
                                    let initialized = p0_w7.len();
                                    p0_w7.as_mut_ptr().add(initialized).write(element);
                                    p0_w7.set_len(initialized + 1);
                                }
                            }
                        }
                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                        let p0_w7_built = if true { Some(p0_w7) } else { None };
                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                        let p0_w6_built = if true {
                            Some(p0_w7_built.expect("active input"))
                        } else {
                            None
                        };
                        let p0_w2 = p0_w6_built.expect("active input");
                        Ok(p0_w2)
                    })();
                    let element = p0_w3_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w2.len();
                        p0_w2.as_mut_ptr().add(initialized).write(element);
                        p0_w2.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true { Some(p0_w2) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = bench_flat::nested_owned_handles_echo(
                __prebindgen_argument_0,
            );
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_5e24e8a1a6fd1960>(),
            );
            let ret_w1_source: ::std::vec::Vec<Vec<bench_flat::InputHandle>> = source_value;
            let ret_w3 = ::std::vec::Vec::len(&ret_w1_source);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>(),
                    ::core::mem::align_of::<__PbBulk_Range_49da87bb4c377b9a>(),
                )?;
            let ret_w1_owners = unsafe {
                ret_w0_writer
                    .own_rows(
                        ret_w1,
                        ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>(),
                        __pb_cleanup_native_Rows_ca08f1b833732d9d,
                    )
            };
            for (ret_w4, ret_w2) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w1_source),
            ) {
                let ret_w5_source: ::std::vec::Vec<bench_flat::InputHandle> = ret_w2;
                let ret_w7 = ::std::vec::Vec::len(&ret_w5_source);
                let ret_w5 = ret_w0_writer
                    .rows(
                        ret_w7,
                        ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                        ::core::mem::align_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                    )?;
                let ret_w5_owners = unsafe {
                    ret_w0_writer
                        .own_rows(
                            ret_w5,
                            ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                            __pb_cleanup_native_InputHandle_6935c4b287d7a13d,
                        )
                };
                for (ret_w8, ret_w6) in ::core::iter::Iterator::enumerate(
                    ::core::iter::IntoIterator::into_iter(ret_w5_source),
                ) {
                    let mut ret_w9 = Box::new(::prebindgen_jni_runtime::Allocation {
                        value: ret_w6,
                    });
                    ret_w0_writer
                        .write(
                            (ret_w5.offset as usize
                                + ret_w8
                                    * ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_b2426f0d5f2715c0, v0
                                ),
                            ((&mut *ret_w9)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >) as i64,
                        )?;
                    #[allow(
                        clippy::redundant_closure_call,
                        clippy::let_unit_value,
                        clippy::unused_unit
                    )]
                    let _: () = (|| -> () {
                        let _ = Box::into_raw(ret_w9).expose_provenance();
                    })();
                    unsafe {
                        ret_w0_writer.commit_rows(ret_w5_owners, ret_w8 + 1);
                    }
                }
                let ret_w5 = Some(ret_w5);
                ret_w0_writer
                    .range(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>())
                            + ::core::mem::offset_of!(
                                __PbBulk_Range_49da87bb4c377b9a, v0
                            ),
                        ret_w5,
                    )?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {})();
                unsafe {
                    ret_w0_writer.commit_rows(ret_w1_owners, ret_w4 + 1);
                }
            }
            let ret_w1 = Some(ret_w1);
            ret_w0_writer
                .range(
                    ::core::mem::offset_of!(__PbBulk_Range_5e24e8a1a6fd1960, v0),
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1callback_1nested_1owned_1handles_145191dd02ebf0e07<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "()[Ljava/lang/Object;",
                "bench.bulk.OwnedNestedCallback",
            )?;
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_callback_value_built = if true {
                Some({
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
                                            let callback_return_w1 = ::prebindgen_jni_runtime::row_bytes(
                                                &mut callback_environment,
                                                &(callback_result_rows),
                                                0usize,
                                            )?;
                                            let callback_return_w1_length = ::prebindgen_jni_runtime::row_i32(
                                                &mut callback_environment,
                                                &(callback_result_rows),
                                                1usize,
                                            )?;
                                            let callback_return_w1_cached_bytes = if true {
                                                ::prebindgen_jni_runtime::bulk::input(
                                                    &mut callback_environment,
                                                    &callback_return_w1,
                                                    callback_return_w1_length,
                                                )?
                                            } else {
                                                Vec::new()
                                            };
                                            let mut callback_return_cleanup = ::prebindgen_runtime::CallbackReturnCleanup::new(||
                                            {
                                                let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                                    __prebindgen_input_consumed,
                                                );
                                                if true {
                                                    let callback_return_w1_cleanup_reader = ::prebindgen_jni_runtime::bulk::Reader::new(
                                                        &callback_return_w1_cached_bytes,
                                                    );
                                                    if let Err(error) = (|| -> ::prebindgen_runtime::Result<()> {
                                                        let (
                                                            callback_return_w3_cleanup_start,
                                                            callback_return_w3_cleanup_count,
                                                        ) = callback_return_w1_cleanup_reader
                                                            .rows(
                                                                callback_return_w1_cleanup_reader
                                                                    .range(
                                                                        ::core::mem::offset_of!(__PbBulk_Range_5e24e8a1a6fd1960, v0),
                                                                    )?,
                                                                ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>(),
                                                            )?;
                                                        for callback_return_w3_cleanup_index in 0..callback_return_w3_cleanup_count {
                                                            if let Err(error) = (|| -> ::prebindgen_runtime::Result<()> {
                                                                let (
                                                                    callback_return_w7_cleanup_start,
                                                                    callback_return_w7_cleanup_count,
                                                                ) = callback_return_w1_cleanup_reader
                                                                    .rows(
                                                                        callback_return_w1_cleanup_reader
                                                                            .range(
                                                                                (callback_return_w3_cleanup_start
                                                                                    + callback_return_w3_cleanup_index
                                                                                        * ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>())
                                                                                    + ::core::mem::offset_of!(
                                                                                        __PbBulk_Range_49da87bb4c377b9a, v0
                                                                                    ),
                                                                            )?,
                                                                        ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                                                                    )?;
                                                                for callback_return_w7_cleanup_index in 0..callback_return_w7_cleanup_count {
                                                                    if let Err(error) = (|| -> ::prebindgen_runtime::Result<()> {
                                                                        unsafe {
                                                                            callback_return_owners
                                                                                .receive(
                                                                                    (callback_return_w1_cleanup_reader
                                                                                        .read::<
                                                                                            i64,
                                                                                        >(
                                                                                            (callback_return_w7_cleanup_start
                                                                                                + callback_return_w7_cleanup_index
                                                                                                    * ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>())
                                                                                                + ::core::mem::offset_of!(__PbBulk_i64_b2426f0d5f2715c0, v0),
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
                                                        Ok(())
                                                    })() {
                                                        ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                                    }
                                                }
                                                Ok(())
                                            });
                                            {
                                                let callback_return_decoded = ({
                                                    let callback_return_w1_bytes = if true {
                                                        Ok::<_, String>(callback_return_w1_cached_bytes.as_slice())?
                                                    } else {
                                                        ::core::default::Default::default()
                                                    };
                                                    let callback_return_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                                                        &callback_return_w1_bytes,
                                                    );
                                                    if callback_return_w1_bytes.len()
                                                        < ::core::mem::size_of::<__PbBulk_Range_5e24e8a1a6fd1960>()
                                                    {
                                                        return Err("truncated bulk root record".into());
                                                    }
                                                    let callback_return_w3 = if true {
                                                        callback_return_w1_reader
                                                            .range(
                                                                ::core::mem::offset_of!(__PbBulk_Range_5e24e8a1a6fd1960, v0),
                                                            )?
                                                    } else {
                                                        None
                                                    };
                                                    let mut callback_return_w2: Vec<
                                                        Vec<bench_flat::InputHandle>,
                                                    > = Vec::new();
                                                    if true {
                                                        let (callback_return_w3_start, callback_return_w4) = callback_return_w1_reader
                                                            .rows(
                                                                callback_return_w3,
                                                                ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>(),
                                                            )?;
                                                        callback_return_w2.reserve_exact(callback_return_w4);
                                                        for callback_return_w5 in 0..callback_return_w4 {
                                                            let callback_return_w7 = callback_return_w1_reader
                                                                .range(
                                                                    (callback_return_w3_start
                                                                        + callback_return_w5
                                                                            * ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>())
                                                                        + ::core::mem::offset_of!(
                                                                            __PbBulk_Range_49da87bb4c377b9a, v0
                                                                        ),
                                                                )?;
                                                            #[allow(clippy::redundant_closure_call)]
                                                            let callback_return_w3_element_built = (|| -> ::core::result::Result<
                                                                Vec<bench_flat::InputHandle>,
                                                                String,
                                                            > {
                                                                let mut callback_return_w6: Vec<bench_flat::InputHandle> = Vec::new();
                                                                if true {
                                                                    let (callback_return_w7_start, callback_return_w8) = callback_return_w1_reader
                                                                        .rows(
                                                                            callback_return_w7,
                                                                            ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                                                                        )?;
                                                                    callback_return_w6.reserve_exact(callback_return_w8);
                                                                    for callback_return_w9 in 0..callback_return_w8 {
                                                                        let callback_return_w12 = callback_return_w1_reader
                                                                            .read::<
                                                                                i64,
                                                                            >(
                                                                                (callback_return_w7_start
                                                                                    + callback_return_w9
                                                                                        * ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>())
                                                                                    + ::core::mem::offset_of!(__PbBulk_i64_b2426f0d5f2715c0, v0),
                                                                            )?;
                                                                        #[allow(clippy::redundant_closure_call)]
                                                                        let callback_return_w7_element_built = (|| -> ::core::result::Result<
                                                                            bench_flat::InputHandle,
                                                                            String,
                                                                        > {
                                                                            let callback_return_w12_pointer = if true {
                                                                                (callback_return_w12)
                                                                                    as *mut ::prebindgen_jni_runtime::Allocation<
                                                                                        bench_flat::InputHandle,
                                                                                    >
                                                                            } else {
                                                                                ::core::ptr::null_mut()
                                                                            };
                                                                            if true {
                                                                                ::prebindgen_runtime::validate_pointer(
                                                                                    callback_return_w12_pointer,
                                                                                )?;
                                                                            }
                                                                            if true {
                                                                                __prebindgen_input_consumed
                                                                                    .set(__prebindgen_input_consumed.get() + (1usize));
                                                                            }
                                                                            let callback_return_w11 = if true {
                                                                                Some(unsafe { Box::from_raw(callback_return_w12_pointer) })
                                                                            } else {
                                                                                None
                                                                            };
                                                                            let callback_return_w11: bench_flat::InputHandle = callback_return_w11
                                                                                .expect("active allocation")
                                                                                .value;
                                                                            let callback_return_w6 = callback_return_w11;
                                                                            Ok(callback_return_w6)
                                                                        })();
                                                                        let element = callback_return_w7_element_built?;
                                                                        #[allow(clippy::zst_offset)]
                                                                        unsafe {
                                                                            let initialized = callback_return_w6.len();
                                                                            callback_return_w6
                                                                                .as_mut_ptr()
                                                                                .add(initialized)
                                                                                .write(element);
                                                                            callback_return_w6.set_len(initialized + 1);
                                                                        }
                                                                    }
                                                                }
                                                                #[allow(clippy::let_and_return, clippy::unit_arg)]
                                                                let callback_return_w6_built = if true {
                                                                    Some(callback_return_w6)
                                                                } else {
                                                                    None
                                                                };
                                                                let callback_return_w2 = callback_return_w6_built
                                                                    .expect("active input");
                                                                Ok(callback_return_w2)
                                                            })();
                                                            let element = callback_return_w3_element_built?;
                                                            #[allow(clippy::zst_offset)]
                                                            unsafe {
                                                                let initialized = callback_return_w2.len();
                                                                callback_return_w2
                                                                    .as_mut_ptr()
                                                                    .add(initialized)
                                                                    .write(element);
                                                                callback_return_w2.set_len(initialized + 1);
                                                            }
                                                        }
                                                    }
                                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                                    let callback_return_w2_built = if true {
                                                        Some(callback_return_w2)
                                                    } else {
                                                        None
                                                    };
                                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                                    let callback_return_w0_built = if true {
                                                        Some(callback_return_w2_built.expect("active input"))
                                                    } else {
                                                        None
                                                    };
                                                    let callback_return_value = callback_return_w0_built
                                                        .expect("active input");
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
                    p0_callback_value
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_callback_value_built.expect("active input");
            let source_value = bench_flat::callback_nested_owned_handles(
                __prebindgen_argument_0,
            );
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_5e24e8a1a6fd1960>(),
            );
            let ret_w1_source: ::std::vec::Vec<Vec<bench_flat::InputHandle>> = source_value;
            let ret_w3 = ::std::vec::Vec::len(&ret_w1_source);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>(),
                    ::core::mem::align_of::<__PbBulk_Range_49da87bb4c377b9a>(),
                )?;
            let ret_w1_owners = unsafe {
                ret_w0_writer
                    .own_rows(
                        ret_w1,
                        ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>(),
                        __pb_cleanup_native_Rows_ca08f1b833732d9d,
                    )
            };
            for (ret_w4, ret_w2) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w1_source),
            ) {
                let ret_w5_source: ::std::vec::Vec<bench_flat::InputHandle> = ret_w2;
                let ret_w7 = ::std::vec::Vec::len(&ret_w5_source);
                let ret_w5 = ret_w0_writer
                    .rows(
                        ret_w7,
                        ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                        ::core::mem::align_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                    )?;
                let ret_w5_owners = unsafe {
                    ret_w0_writer
                        .own_rows(
                            ret_w5,
                            ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>(),
                            __pb_cleanup_native_InputHandle_6935c4b287d7a13d,
                        )
                };
                for (ret_w8, ret_w6) in ::core::iter::Iterator::enumerate(
                    ::core::iter::IntoIterator::into_iter(ret_w5_source),
                ) {
                    let mut ret_w9 = Box::new(::prebindgen_jni_runtime::Allocation {
                        value: ret_w6,
                    });
                    ret_w0_writer
                        .write(
                            (ret_w5.offset as usize
                                + ret_w8
                                    * ::core::mem::size_of::<__PbBulk_i64_b2426f0d5f2715c0>())
                                + ::core::mem::offset_of!(
                                    __PbBulk_i64_b2426f0d5f2715c0, v0
                                ),
                            ((&mut *ret_w9)
                                as *mut ::prebindgen_jni_runtime::Allocation<
                                    bench_flat::InputHandle,
                                >) as i64,
                        )?;
                    #[allow(
                        clippy::redundant_closure_call,
                        clippy::let_unit_value,
                        clippy::unused_unit
                    )]
                    let _: () = (|| -> () {
                        let _ = Box::into_raw(ret_w9).expose_provenance();
                    })();
                    unsafe {
                        ret_w0_writer.commit_rows(ret_w5_owners, ret_w8 + 1);
                    }
                }
                let ret_w5 = Some(ret_w5);
                ret_w0_writer
                    .range(
                        (ret_w1.offset as usize
                            + ret_w4
                                * ::core::mem::size_of::<__PbBulk_Range_49da87bb4c377b9a>())
                            + ::core::mem::offset_of!(
                                __PbBulk_Range_49da87bb4c377b9a, v0
                            ),
                        ret_w5,
                    )?;
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {})();
                unsafe {
                    ret_w0_writer.commit_rows(ret_w1_owners, ret_w4 + 1);
                }
            }
            let ret_w1 = Some(ret_w1);
            ret_w0_writer
                .range(
                    ::core::mem::offset_of!(__PbBulk_Range_5e24e8a1a6fd1960, v0),
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1row_1make_1bcd43e7b136151dd<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: i64 = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = bench_flat::row_make(__prebindgen_argument_0);
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
            );
            let ret_w1_record = source_value;
            let ret_w1_f0 = ret_w1_record.a;
            let ret_w1_f1 = ret_w1_record.b;
            let ret_w1_f2 = ret_w1_record.c;
            let ret_w1_f3 = ret_w1_record.d;
            let ret_w1_f4 = ret_w1_record.e;
            let ret_w1_f5 = ret_w1_record.f;
            let ret_w1_f6 = ret_w1_record.g;
            let ret_w1_f7 = ret_w1_record.h;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v0),
                    ret_w1_f0,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v1),
                    ret_w1_f1,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v2),
                    ret_w1_f2,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v3),
                    ret_w1_f3,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v4),
                    ret_w1_f4,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v5),
                    ret_w1_f5,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v6),
                    ret_w1_f6,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_789cd54eb117685f, v7),
                    ret_w1_f7,
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1wide_1row_1make_11dbd8ca2fd4616ac<
    'local,
>(
    mut jni_environment: ::prebindgen_jni_runtime::jni::JNIEnv<'local>,
    _class: ::prebindgen_jni_runtime::jni::objects::JClass<'local>,
    p0_w1: i64,
    __pb_return: ::prebindgen_jni_runtime::jni::objects::JObject<'local>,
) -> ::prebindgen_jni_runtime::jni::sys::jobject {
    #[allow(unused_mut)]
    let mut domain_output: Option<::prebindgen_jni_runtime::jni::sys::jobjectArray> = None;
    let mut success_transferred = false;
    let mut direct_output: ::prebindgen_jni_runtime::jni::sys::jobject = std::ptr::null_mut();
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: i64 = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = bench_flat::wide_row_make(__prebindgen_argument_0);
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_i64_i64_633c6cf82d3acfc1>(),
            );
            let ret_w1_record = source_value;
            let ret_w1_f0 = ret_w1_record.f0;
            let ret_w1_f1 = ret_w1_record.f1;
            let ret_w1_f2 = ret_w1_record.f2;
            let ret_w1_f3 = ret_w1_record.f3;
            let ret_w1_f4 = ret_w1_record.f4;
            let ret_w1_f5 = ret_w1_record.f5;
            let ret_w1_f6 = ret_w1_record.f6;
            let ret_w1_f7 = ret_w1_record.f7;
            let ret_w1_f8 = ret_w1_record.f8;
            let ret_w1_f9 = ret_w1_record.f9;
            let ret_w1_f10 = ret_w1_record.f10;
            let ret_w1_f11 = ret_w1_record.f11;
            let ret_w1_f12 = ret_w1_record.f12;
            let ret_w1_f13 = ret_w1_record.f13;
            let ret_w1_f14 = ret_w1_record.f14;
            let ret_w1_f15 = ret_w1_record.f15;
            let ret_w1_f16 = ret_w1_record.f16;
            let ret_w1_f17 = ret_w1_record.f17;
            let ret_w1_f18 = ret_w1_record.f18;
            let ret_w1_f19 = ret_w1_record.f19;
            let ret_w1_f20 = ret_w1_record.f20;
            let ret_w1_f21 = ret_w1_record.f21;
            let ret_w1_f22 = ret_w1_record.f22;
            let ret_w1_f23 = ret_w1_record.f23;
            let ret_w1_f24 = ret_w1_record.f24;
            let ret_w1_f25 = ret_w1_record.f25;
            let ret_w1_f26 = ret_w1_record.f26;
            let ret_w1_f27 = ret_w1_record.f27;
            let ret_w1_f28 = ret_w1_record.f28;
            let ret_w1_f29 = ret_w1_record.f29;
            let ret_w1_f30 = ret_w1_record.f30;
            let ret_w1_f31 = ret_w1_record.f31;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v0),
                    ret_w1_f0,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v1),
                    ret_w1_f1,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v2),
                    ret_w1_f2,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v3),
                    ret_w1_f3,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v4),
                    ret_w1_f4,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v5),
                    ret_w1_f5,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v6),
                    ret_w1_f6,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v7),
                    ret_w1_f7,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v8),
                    ret_w1_f8,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v9),
                    ret_w1_f9,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v10),
                    ret_w1_f10,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v11),
                    ret_w1_f11,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v12),
                    ret_w1_f12,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v13),
                    ret_w1_f13,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v14),
                    ret_w1_f14,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v15),
                    ret_w1_f15,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v16),
                    ret_w1_f16,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v17),
                    ret_w1_f17,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v18),
                    ret_w1_f18,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v19),
                    ret_w1_f19,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v20),
                    ret_w1_f20,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v21),
                    ret_w1_f21,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v22),
                    ret_w1_f22,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v23),
                    ret_w1_f23,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v24),
                    ret_w1_f24,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v25),
                    ret_w1_f25,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v26),
                    ret_w1_f26,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v27),
                    ret_w1_f27,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v28),
                    ret_w1_f28,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v29),
                    ret_w1_f29,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v30),
                    ret_w1_f30,
                )?;
            ret_w0_writer
                .write(
                    ::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v31),
                    ret_w1_f31,
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1wide_1row_1sum_1861bdfef124a25ad<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
            if p0_w1_bytes.len()
                < ::core::mem::size_of::<__PbBulk_i64_i64_633c6cf82d3acfc1>()
            {
                return Err("truncated bulk root record".into());
            }
            let p0_w3_f0 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v0))?
            } else {
                0
            };
            let p0_w3_f1 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v1))?
            } else {
                0
            };
            let p0_w3_f2 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v2))?
            } else {
                0
            };
            let p0_w3_f3 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v3))?
            } else {
                0
            };
            let p0_w3_f4 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v4))?
            } else {
                0
            };
            let p0_w3_f5 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v5))?
            } else {
                0
            };
            let p0_w3_f6 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v6))?
            } else {
                0
            };
            let p0_w3_f7 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v7))?
            } else {
                0
            };
            let p0_w3_f8 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v8))?
            } else {
                0
            };
            let p0_w3_f9 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v9))?
            } else {
                0
            };
            let p0_w3_f10 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v10))?
            } else {
                0
            };
            let p0_w3_f11 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v11))?
            } else {
                0
            };
            let p0_w3_f12 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v12))?
            } else {
                0
            };
            let p0_w3_f13 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v13))?
            } else {
                0
            };
            let p0_w3_f14 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v14))?
            } else {
                0
            };
            let p0_w3_f15 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v15))?
            } else {
                0
            };
            let p0_w3_f16 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v16))?
            } else {
                0
            };
            let p0_w3_f17 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v17))?
            } else {
                0
            };
            let p0_w3_f18 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v18))?
            } else {
                0
            };
            let p0_w3_f19 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v19))?
            } else {
                0
            };
            let p0_w3_f20 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v20))?
            } else {
                0
            };
            let p0_w3_f21 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v21))?
            } else {
                0
            };
            let p0_w3_f22 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v22))?
            } else {
                0
            };
            let p0_w3_f23 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v23))?
            } else {
                0
            };
            let p0_w3_f24 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v24))?
            } else {
                0
            };
            let p0_w3_f25 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v25))?
            } else {
                0
            };
            let p0_w3_f26 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v26))?
            } else {
                0
            };
            let p0_w3_f27 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v27))?
            } else {
                0
            };
            let p0_w3_f28 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v28))?
            } else {
                0
            };
            let p0_w3_f29 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v29))?
            } else {
                0
            };
            let p0_w3_f30 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v30))?
            } else {
                0
            };
            let p0_w3_f31 = if true {
                p0_w1_reader
                    .read::<
                        i64,
                    >(::core::mem::offset_of!(__PbBulk_i64_i64_633c6cf82d3acfc1, v31))?
            } else {
                0
            };
            let p0_w2_f0 = if true { Some(p0_w3_f0) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f0_built = if true {
                Some({
                    let p0_w2_f0: i64 = ::core::convert::Into::into(
                        p0_w2_f0.expect("active value"),
                    );
                    p0_w2_f0
                })
            } else {
                None
            };
            let p0_w2_f1 = if true { Some(p0_w3_f1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f1_built = if true {
                Some({
                    let p0_w2_f1: i64 = ::core::convert::Into::into(
                        p0_w2_f1.expect("active value"),
                    );
                    p0_w2_f1
                })
            } else {
                None
            };
            let p0_w2_f2 = if true { Some(p0_w3_f2) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f2_built = if true {
                Some({
                    let p0_w2_f2: i64 = ::core::convert::Into::into(
                        p0_w2_f2.expect("active value"),
                    );
                    p0_w2_f2
                })
            } else {
                None
            };
            let p0_w2_f3 = if true { Some(p0_w3_f3) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f3_built = if true {
                Some({
                    let p0_w2_f3: i64 = ::core::convert::Into::into(
                        p0_w2_f3.expect("active value"),
                    );
                    p0_w2_f3
                })
            } else {
                None
            };
            let p0_w2_f4 = if true { Some(p0_w3_f4) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f4_built = if true {
                Some({
                    let p0_w2_f4: i64 = ::core::convert::Into::into(
                        p0_w2_f4.expect("active value"),
                    );
                    p0_w2_f4
                })
            } else {
                None
            };
            let p0_w2_f5 = if true { Some(p0_w3_f5) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f5_built = if true {
                Some({
                    let p0_w2_f5: i64 = ::core::convert::Into::into(
                        p0_w2_f5.expect("active value"),
                    );
                    p0_w2_f5
                })
            } else {
                None
            };
            let p0_w2_f6 = if true { Some(p0_w3_f6) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f6_built = if true {
                Some({
                    let p0_w2_f6: i64 = ::core::convert::Into::into(
                        p0_w2_f6.expect("active value"),
                    );
                    p0_w2_f6
                })
            } else {
                None
            };
            let p0_w2_f7 = if true { Some(p0_w3_f7) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f7_built = if true {
                Some({
                    let p0_w2_f7: i64 = ::core::convert::Into::into(
                        p0_w2_f7.expect("active value"),
                    );
                    p0_w2_f7
                })
            } else {
                None
            };
            let p0_w2_f8 = if true { Some(p0_w3_f8) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f8_built = if true {
                Some({
                    let p0_w2_f8: i64 = ::core::convert::Into::into(
                        p0_w2_f8.expect("active value"),
                    );
                    p0_w2_f8
                })
            } else {
                None
            };
            let p0_w2_f9 = if true { Some(p0_w3_f9) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f9_built = if true {
                Some({
                    let p0_w2_f9: i64 = ::core::convert::Into::into(
                        p0_w2_f9.expect("active value"),
                    );
                    p0_w2_f9
                })
            } else {
                None
            };
            let p0_w2_f10 = if true { Some(p0_w3_f10) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f10_built = if true {
                Some({
                    let p0_w2_f10: i64 = ::core::convert::Into::into(
                        p0_w2_f10.expect("active value"),
                    );
                    p0_w2_f10
                })
            } else {
                None
            };
            let p0_w2_f11 = if true { Some(p0_w3_f11) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f11_built = if true {
                Some({
                    let p0_w2_f11: i64 = ::core::convert::Into::into(
                        p0_w2_f11.expect("active value"),
                    );
                    p0_w2_f11
                })
            } else {
                None
            };
            let p0_w2_f12 = if true { Some(p0_w3_f12) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f12_built = if true {
                Some({
                    let p0_w2_f12: i64 = ::core::convert::Into::into(
                        p0_w2_f12.expect("active value"),
                    );
                    p0_w2_f12
                })
            } else {
                None
            };
            let p0_w2_f13 = if true { Some(p0_w3_f13) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f13_built = if true {
                Some({
                    let p0_w2_f13: i64 = ::core::convert::Into::into(
                        p0_w2_f13.expect("active value"),
                    );
                    p0_w2_f13
                })
            } else {
                None
            };
            let p0_w2_f14 = if true { Some(p0_w3_f14) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f14_built = if true {
                Some({
                    let p0_w2_f14: i64 = ::core::convert::Into::into(
                        p0_w2_f14.expect("active value"),
                    );
                    p0_w2_f14
                })
            } else {
                None
            };
            let p0_w2_f15 = if true { Some(p0_w3_f15) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f15_built = if true {
                Some({
                    let p0_w2_f15: i64 = ::core::convert::Into::into(
                        p0_w2_f15.expect("active value"),
                    );
                    p0_w2_f15
                })
            } else {
                None
            };
            let p0_w2_f16 = if true { Some(p0_w3_f16) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f16_built = if true {
                Some({
                    let p0_w2_f16: i64 = ::core::convert::Into::into(
                        p0_w2_f16.expect("active value"),
                    );
                    p0_w2_f16
                })
            } else {
                None
            };
            let p0_w2_f17 = if true { Some(p0_w3_f17) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f17_built = if true {
                Some({
                    let p0_w2_f17: i64 = ::core::convert::Into::into(
                        p0_w2_f17.expect("active value"),
                    );
                    p0_w2_f17
                })
            } else {
                None
            };
            let p0_w2_f18 = if true { Some(p0_w3_f18) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f18_built = if true {
                Some({
                    let p0_w2_f18: i64 = ::core::convert::Into::into(
                        p0_w2_f18.expect("active value"),
                    );
                    p0_w2_f18
                })
            } else {
                None
            };
            let p0_w2_f19 = if true { Some(p0_w3_f19) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f19_built = if true {
                Some({
                    let p0_w2_f19: i64 = ::core::convert::Into::into(
                        p0_w2_f19.expect("active value"),
                    );
                    p0_w2_f19
                })
            } else {
                None
            };
            let p0_w2_f20 = if true { Some(p0_w3_f20) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f20_built = if true {
                Some({
                    let p0_w2_f20: i64 = ::core::convert::Into::into(
                        p0_w2_f20.expect("active value"),
                    );
                    p0_w2_f20
                })
            } else {
                None
            };
            let p0_w2_f21 = if true { Some(p0_w3_f21) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f21_built = if true {
                Some({
                    let p0_w2_f21: i64 = ::core::convert::Into::into(
                        p0_w2_f21.expect("active value"),
                    );
                    p0_w2_f21
                })
            } else {
                None
            };
            let p0_w2_f22 = if true { Some(p0_w3_f22) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f22_built = if true {
                Some({
                    let p0_w2_f22: i64 = ::core::convert::Into::into(
                        p0_w2_f22.expect("active value"),
                    );
                    p0_w2_f22
                })
            } else {
                None
            };
            let p0_w2_f23 = if true { Some(p0_w3_f23) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f23_built = if true {
                Some({
                    let p0_w2_f23: i64 = ::core::convert::Into::into(
                        p0_w2_f23.expect("active value"),
                    );
                    p0_w2_f23
                })
            } else {
                None
            };
            let p0_w2_f24 = if true { Some(p0_w3_f24) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f24_built = if true {
                Some({
                    let p0_w2_f24: i64 = ::core::convert::Into::into(
                        p0_w2_f24.expect("active value"),
                    );
                    p0_w2_f24
                })
            } else {
                None
            };
            let p0_w2_f25 = if true { Some(p0_w3_f25) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f25_built = if true {
                Some({
                    let p0_w2_f25: i64 = ::core::convert::Into::into(
                        p0_w2_f25.expect("active value"),
                    );
                    p0_w2_f25
                })
            } else {
                None
            };
            let p0_w2_f26 = if true { Some(p0_w3_f26) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f26_built = if true {
                Some({
                    let p0_w2_f26: i64 = ::core::convert::Into::into(
                        p0_w2_f26.expect("active value"),
                    );
                    p0_w2_f26
                })
            } else {
                None
            };
            let p0_w2_f27 = if true { Some(p0_w3_f27) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f27_built = if true {
                Some({
                    let p0_w2_f27: i64 = ::core::convert::Into::into(
                        p0_w2_f27.expect("active value"),
                    );
                    p0_w2_f27
                })
            } else {
                None
            };
            let p0_w2_f28 = if true { Some(p0_w3_f28) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f28_built = if true {
                Some({
                    let p0_w2_f28: i64 = ::core::convert::Into::into(
                        p0_w2_f28.expect("active value"),
                    );
                    p0_w2_f28
                })
            } else {
                None
            };
            let p0_w2_f29 = if true { Some(p0_w3_f29) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f29_built = if true {
                Some({
                    let p0_w2_f29: i64 = ::core::convert::Into::into(
                        p0_w2_f29.expect("active value"),
                    );
                    p0_w2_f29
                })
            } else {
                None
            };
            let p0_w2_f30 = if true { Some(p0_w3_f30) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f30_built = if true {
                Some({
                    let p0_w2_f30: i64 = ::core::convert::Into::into(
                        p0_w2_f30.expect("active value"),
                    );
                    p0_w2_f30
                })
            } else {
                None
            };
            let p0_w2_f31 = if true { Some(p0_w3_f31) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f31_built = if true {
                Some({
                    let p0_w2_f31: i64 = ::core::convert::Into::into(
                        p0_w2_f31.expect("active value"),
                    );
                    p0_w2_f31
                })
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true {
                Some({
                    let p0_w2: bench_flat::WideRow = bench_flat::WideRow {
                        f0: p0_w2_f0_built.expect("active input"),
                        f1: p0_w2_f1_built.expect("active input"),
                        f2: p0_w2_f2_built.expect("active input"),
                        f3: p0_w2_f3_built.expect("active input"),
                        f4: p0_w2_f4_built.expect("active input"),
                        f5: p0_w2_f5_built.expect("active input"),
                        f6: p0_w2_f6_built.expect("active input"),
                        f7: p0_w2_f7_built.expect("active input"),
                        f8: p0_w2_f8_built.expect("active input"),
                        f9: p0_w2_f9_built.expect("active input"),
                        f10: p0_w2_f10_built.expect("active input"),
                        f11: p0_w2_f11_built.expect("active input"),
                        f12: p0_w2_f12_built.expect("active input"),
                        f13: p0_w2_f13_built.expect("active input"),
                        f14: p0_w2_f14_built.expect("active input"),
                        f15: p0_w2_f15_built.expect("active input"),
                        f16: p0_w2_f16_built.expect("active input"),
                        f17: p0_w2_f17_built.expect("active input"),
                        f18: p0_w2_f18_built.expect("active input"),
                        f19: p0_w2_f19_built.expect("active input"),
                        f20: p0_w2_f20_built.expect("active input"),
                        f21: p0_w2_f21_built.expect("active input"),
                        f22: p0_w2_f22_built.expect("active input"),
                        f23: p0_w2_f23_built.expect("active input"),
                        f24: p0_w2_f24_built.expect("active input"),
                        f25: p0_w2_f25_built.expect("active input"),
                        f26: p0_w2_f26_built.expect("active input"),
                        f27: p0_w2_f27_built.expect("active input"),
                        f28: p0_w2_f28_built.expect("active input"),
                        f29: p0_w2_f29_built.expect("active input"),
                        f30: p0_w2_f30_built.expect("active input"),
                        f31: p0_w2_f31_built.expect("active input"),
                    };
                    p0_w2
                })
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1row_1return_1handler_1new_19a8fcadcab7b9640<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "(J)[Ljava/lang/Object;",
                "bench.bulk.RowReturnCallback",
            )?;
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_callback_value_built = if true {
                Some({
                    #[allow(unused_mut, clippy::collapsible_if)]
                    let p0_callback_value = move |callback_source_0: i64| {
                        p0_callback_retained
                            .with_env_result(
                                20i32,
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
                                        let callback_return_w1 = ::prebindgen_jni_runtime::row_bytes(
                                            &mut callback_environment,
                                            &(callback_result_rows),
                                            0usize,
                                        )?;
                                        let callback_return_w1_length = ::prebindgen_jni_runtime::row_i32(
                                            &mut callback_environment,
                                            &(callback_result_rows),
                                            1usize,
                                        )?;
                                        let callback_return_w1_cached_bytes = if true {
                                            ::prebindgen_jni_runtime::bulk::input(
                                                &mut callback_environment,
                                                &callback_return_w1,
                                                callback_return_w1_length,
                                            )?
                                        } else {
                                            Vec::new()
                                        };
                                        let callback_return_w1_bytes = if true {
                                            Ok::<_, String>(callback_return_w1_cached_bytes.as_slice())?
                                        } else {
                                            ::core::default::Default::default()
                                        };
                                        let callback_return_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                                            &callback_return_w1_bytes,
                                        );
                                        if callback_return_w1_bytes.len()
                                            < ::core::mem::size_of::<
                                                __PbBulk_i64_i64_789cd54eb117685f,
                                            >()
                                        {
                                            return Err("truncated bulk root record".into());
                                        }
                                        let callback_return_w3_f0 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_789cd54eb117685f, v0
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f1 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_789cd54eb117685f, v1
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f2 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_789cd54eb117685f, v2
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f3 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_789cd54eb117685f, v3
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f4 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_789cd54eb117685f, v4
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f5 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_789cd54eb117685f, v5
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f6 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_789cd54eb117685f, v6
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f7 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_789cd54eb117685f, v7
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w2_f0 = if true {
                                            Some(callback_return_w3_f0)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f0_built = if true {
                                            Some({
                                                let callback_return_w2_f0: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f0.expect("active value"),
                                                );
                                                callback_return_w2_f0
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f1 = if true {
                                            Some(callback_return_w3_f1)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f1_built = if true {
                                            Some({
                                                let callback_return_w2_f1: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f1.expect("active value"),
                                                );
                                                callback_return_w2_f1
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f2 = if true {
                                            Some(callback_return_w3_f2)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f2_built = if true {
                                            Some({
                                                let callback_return_w2_f2: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f2.expect("active value"),
                                                );
                                                callback_return_w2_f2
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f3 = if true {
                                            Some(callback_return_w3_f3)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f3_built = if true {
                                            Some({
                                                let callback_return_w2_f3: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f3.expect("active value"),
                                                );
                                                callback_return_w2_f3
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f4 = if true {
                                            Some(callback_return_w3_f4)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f4_built = if true {
                                            Some({
                                                let callback_return_w2_f4: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f4.expect("active value"),
                                                );
                                                callback_return_w2_f4
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f5 = if true {
                                            Some(callback_return_w3_f5)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f5_built = if true {
                                            Some({
                                                let callback_return_w2_f5: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f5.expect("active value"),
                                                );
                                                callback_return_w2_f5
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f6 = if true {
                                            Some(callback_return_w3_f6)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f6_built = if true {
                                            Some({
                                                let callback_return_w2_f6: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f6.expect("active value"),
                                                );
                                                callback_return_w2_f6
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f7 = if true {
                                            Some(callback_return_w3_f7)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f7_built = if true {
                                            Some({
                                                let callback_return_w2_f7: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f7.expect("active value"),
                                                );
                                                callback_return_w2_f7
                                            })
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_built = if true {
                                            Some({
                                                let callback_return_w2: bench_flat::Row = bench_flat::Row {
                                                    a: callback_return_w2_f0_built.expect("active input"),
                                                    b: callback_return_w2_f1_built.expect("active input"),
                                                    c: callback_return_w2_f2_built.expect("active input"),
                                                    d: callback_return_w2_f3_built.expect("active input"),
                                                    e: callback_return_w2_f4_built.expect("active input"),
                                                    f: callback_return_w2_f5_built.expect("active input"),
                                                    g: callback_return_w2_f6_built.expect("active input"),
                                                    h: callback_return_w2_f7_built.expect("active input"),
                                                };
                                                callback_return_w2
                                            })
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w0_built = if true {
                                            Some(callback_return_w2_built.expect("active input"))
                                        } else {
                                            None
                                        };
                                        let callback_return_value = callback_return_w0_built
                                            .expect("active input");
                                        Ok::<_, String>(callback_return_value)
                                    }
                                },
                            )
                            .unwrap_or_else(|error| panic!("callback failed: {}", error))
                    };
                    p0_callback_value
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_callback_value_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1wide_1row_1return_1handler_1new_167f38d7168020371<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "(J)[Ljava/lang/Object;",
                "bench.bulk.WideRowReturnCallback",
            )?;
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_callback_value_built = if true {
                Some({
                    #[allow(unused_mut, clippy::collapsible_if)]
                    let p0_callback_value = move |callback_source_0: i64| {
                        p0_callback_retained
                            .with_env_result(
                                20i32,
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
                                        let callback_return_w1 = ::prebindgen_jni_runtime::row_bytes(
                                            &mut callback_environment,
                                            &(callback_result_rows),
                                            0usize,
                                        )?;
                                        let callback_return_w1_length = ::prebindgen_jni_runtime::row_i32(
                                            &mut callback_environment,
                                            &(callback_result_rows),
                                            1usize,
                                        )?;
                                        let callback_return_w1_cached_bytes = if true {
                                            ::prebindgen_jni_runtime::bulk::input(
                                                &mut callback_environment,
                                                &callback_return_w1,
                                                callback_return_w1_length,
                                            )?
                                        } else {
                                            Vec::new()
                                        };
                                        let callback_return_w1_bytes = if true {
                                            Ok::<_, String>(callback_return_w1_cached_bytes.as_slice())?
                                        } else {
                                            ::core::default::Default::default()
                                        };
                                        let callback_return_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                                            &callback_return_w1_bytes,
                                        );
                                        if callback_return_w1_bytes.len()
                                            < ::core::mem::size_of::<
                                                __PbBulk_i64_i64_633c6cf82d3acfc1,
                                            >()
                                        {
                                            return Err("truncated bulk root record".into());
                                        }
                                        let callback_return_w3_f0 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v0
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f1 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v1
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f2 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v2
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f3 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v3
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f4 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v4
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f5 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v5
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f6 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v6
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f7 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v7
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f8 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v8
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f9 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v9
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f10 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v10
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f11 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v11
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f12 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v12
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f13 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v13
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f14 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v14
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f15 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v15
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f16 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v16
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f17 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v17
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f18 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v18
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f19 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v19
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f20 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v20
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f21 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v21
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f22 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v22
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f23 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v23
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f24 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v24
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f25 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v25
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f26 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v26
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f27 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v27
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f28 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v28
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f29 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v29
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f30 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v30
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w3_f31 = if true {
                                            callback_return_w1_reader
                                                .read::<
                                                    i64,
                                                >(
                                                    ::core::mem::offset_of!(
                                                        __PbBulk_i64_i64_633c6cf82d3acfc1, v31
                                                    ),
                                                )?
                                        } else {
                                            0
                                        };
                                        let callback_return_w2_f0 = if true {
                                            Some(callback_return_w3_f0)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f0_built = if true {
                                            Some({
                                                let callback_return_w2_f0: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f0.expect("active value"),
                                                );
                                                callback_return_w2_f0
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f1 = if true {
                                            Some(callback_return_w3_f1)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f1_built = if true {
                                            Some({
                                                let callback_return_w2_f1: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f1.expect("active value"),
                                                );
                                                callback_return_w2_f1
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f2 = if true {
                                            Some(callback_return_w3_f2)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f2_built = if true {
                                            Some({
                                                let callback_return_w2_f2: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f2.expect("active value"),
                                                );
                                                callback_return_w2_f2
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f3 = if true {
                                            Some(callback_return_w3_f3)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f3_built = if true {
                                            Some({
                                                let callback_return_w2_f3: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f3.expect("active value"),
                                                );
                                                callback_return_w2_f3
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f4 = if true {
                                            Some(callback_return_w3_f4)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f4_built = if true {
                                            Some({
                                                let callback_return_w2_f4: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f4.expect("active value"),
                                                );
                                                callback_return_w2_f4
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f5 = if true {
                                            Some(callback_return_w3_f5)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f5_built = if true {
                                            Some({
                                                let callback_return_w2_f5: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f5.expect("active value"),
                                                );
                                                callback_return_w2_f5
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f6 = if true {
                                            Some(callback_return_w3_f6)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f6_built = if true {
                                            Some({
                                                let callback_return_w2_f6: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f6.expect("active value"),
                                                );
                                                callback_return_w2_f6
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f7 = if true {
                                            Some(callback_return_w3_f7)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f7_built = if true {
                                            Some({
                                                let callback_return_w2_f7: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f7.expect("active value"),
                                                );
                                                callback_return_w2_f7
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f8 = if true {
                                            Some(callback_return_w3_f8)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f8_built = if true {
                                            Some({
                                                let callback_return_w2_f8: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f8.expect("active value"),
                                                );
                                                callback_return_w2_f8
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f9 = if true {
                                            Some(callback_return_w3_f9)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f9_built = if true {
                                            Some({
                                                let callback_return_w2_f9: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f9.expect("active value"),
                                                );
                                                callback_return_w2_f9
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f10 = if true {
                                            Some(callback_return_w3_f10)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f10_built = if true {
                                            Some({
                                                let callback_return_w2_f10: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f10.expect("active value"),
                                                );
                                                callback_return_w2_f10
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f11 = if true {
                                            Some(callback_return_w3_f11)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f11_built = if true {
                                            Some({
                                                let callback_return_w2_f11: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f11.expect("active value"),
                                                );
                                                callback_return_w2_f11
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f12 = if true {
                                            Some(callback_return_w3_f12)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f12_built = if true {
                                            Some({
                                                let callback_return_w2_f12: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f12.expect("active value"),
                                                );
                                                callback_return_w2_f12
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f13 = if true {
                                            Some(callback_return_w3_f13)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f13_built = if true {
                                            Some({
                                                let callback_return_w2_f13: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f13.expect("active value"),
                                                );
                                                callback_return_w2_f13
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f14 = if true {
                                            Some(callback_return_w3_f14)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f14_built = if true {
                                            Some({
                                                let callback_return_w2_f14: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f14.expect("active value"),
                                                );
                                                callback_return_w2_f14
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f15 = if true {
                                            Some(callback_return_w3_f15)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f15_built = if true {
                                            Some({
                                                let callback_return_w2_f15: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f15.expect("active value"),
                                                );
                                                callback_return_w2_f15
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f16 = if true {
                                            Some(callback_return_w3_f16)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f16_built = if true {
                                            Some({
                                                let callback_return_w2_f16: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f16.expect("active value"),
                                                );
                                                callback_return_w2_f16
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f17 = if true {
                                            Some(callback_return_w3_f17)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f17_built = if true {
                                            Some({
                                                let callback_return_w2_f17: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f17.expect("active value"),
                                                );
                                                callback_return_w2_f17
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f18 = if true {
                                            Some(callback_return_w3_f18)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f18_built = if true {
                                            Some({
                                                let callback_return_w2_f18: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f18.expect("active value"),
                                                );
                                                callback_return_w2_f18
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f19 = if true {
                                            Some(callback_return_w3_f19)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f19_built = if true {
                                            Some({
                                                let callback_return_w2_f19: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f19.expect("active value"),
                                                );
                                                callback_return_w2_f19
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f20 = if true {
                                            Some(callback_return_w3_f20)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f20_built = if true {
                                            Some({
                                                let callback_return_w2_f20: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f20.expect("active value"),
                                                );
                                                callback_return_w2_f20
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f21 = if true {
                                            Some(callback_return_w3_f21)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f21_built = if true {
                                            Some({
                                                let callback_return_w2_f21: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f21.expect("active value"),
                                                );
                                                callback_return_w2_f21
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f22 = if true {
                                            Some(callback_return_w3_f22)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f22_built = if true {
                                            Some({
                                                let callback_return_w2_f22: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f22.expect("active value"),
                                                );
                                                callback_return_w2_f22
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f23 = if true {
                                            Some(callback_return_w3_f23)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f23_built = if true {
                                            Some({
                                                let callback_return_w2_f23: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f23.expect("active value"),
                                                );
                                                callback_return_w2_f23
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f24 = if true {
                                            Some(callback_return_w3_f24)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f24_built = if true {
                                            Some({
                                                let callback_return_w2_f24: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f24.expect("active value"),
                                                );
                                                callback_return_w2_f24
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f25 = if true {
                                            Some(callback_return_w3_f25)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f25_built = if true {
                                            Some({
                                                let callback_return_w2_f25: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f25.expect("active value"),
                                                );
                                                callback_return_w2_f25
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f26 = if true {
                                            Some(callback_return_w3_f26)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f26_built = if true {
                                            Some({
                                                let callback_return_w2_f26: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f26.expect("active value"),
                                                );
                                                callback_return_w2_f26
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f27 = if true {
                                            Some(callback_return_w3_f27)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f27_built = if true {
                                            Some({
                                                let callback_return_w2_f27: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f27.expect("active value"),
                                                );
                                                callback_return_w2_f27
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f28 = if true {
                                            Some(callback_return_w3_f28)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f28_built = if true {
                                            Some({
                                                let callback_return_w2_f28: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f28.expect("active value"),
                                                );
                                                callback_return_w2_f28
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f29 = if true {
                                            Some(callback_return_w3_f29)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f29_built = if true {
                                            Some({
                                                let callback_return_w2_f29: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f29.expect("active value"),
                                                );
                                                callback_return_w2_f29
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f30 = if true {
                                            Some(callback_return_w3_f30)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f30_built = if true {
                                            Some({
                                                let callback_return_w2_f30: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f30.expect("active value"),
                                                );
                                                callback_return_w2_f30
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_w2_f31 = if true {
                                            Some(callback_return_w3_f31)
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_f31_built = if true {
                                            Some({
                                                let callback_return_w2_f31: i64 = ::core::convert::Into::into(
                                                    callback_return_w2_f31.expect("active value"),
                                                );
                                                callback_return_w2_f31
                                            })
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w2_built = if true {
                                            Some({
                                                let callback_return_w2: bench_flat::WideRow = bench_flat::WideRow {
                                                    f0: callback_return_w2_f0_built.expect("active input"),
                                                    f1: callback_return_w2_f1_built.expect("active input"),
                                                    f2: callback_return_w2_f2_built.expect("active input"),
                                                    f3: callback_return_w2_f3_built.expect("active input"),
                                                    f4: callback_return_w2_f4_built.expect("active input"),
                                                    f5: callback_return_w2_f5_built.expect("active input"),
                                                    f6: callback_return_w2_f6_built.expect("active input"),
                                                    f7: callback_return_w2_f7_built.expect("active input"),
                                                    f8: callback_return_w2_f8_built.expect("active input"),
                                                    f9: callback_return_w2_f9_built.expect("active input"),
                                                    f10: callback_return_w2_f10_built.expect("active input"),
                                                    f11: callback_return_w2_f11_built.expect("active input"),
                                                    f12: callback_return_w2_f12_built.expect("active input"),
                                                    f13: callback_return_w2_f13_built.expect("active input"),
                                                    f14: callback_return_w2_f14_built.expect("active input"),
                                                    f15: callback_return_w2_f15_built.expect("active input"),
                                                    f16: callback_return_w2_f16_built.expect("active input"),
                                                    f17: callback_return_w2_f17_built.expect("active input"),
                                                    f18: callback_return_w2_f18_built.expect("active input"),
                                                    f19: callback_return_w2_f19_built.expect("active input"),
                                                    f20: callback_return_w2_f20_built.expect("active input"),
                                                    f21: callback_return_w2_f21_built.expect("active input"),
                                                    f22: callback_return_w2_f22_built.expect("active input"),
                                                    f23: callback_return_w2_f23_built.expect("active input"),
                                                    f24: callback_return_w2_f24_built.expect("active input"),
                                                    f25: callback_return_w2_f25_built.expect("active input"),
                                                    f26: callback_return_w2_f26_built.expect("active input"),
                                                    f27: callback_return_w2_f27_built.expect("active input"),
                                                    f28: callback_return_w2_f28_built.expect("active input"),
                                                    f29: callback_return_w2_f29_built.expect("active input"),
                                                    f30: callback_return_w2_f30_built.expect("active input"),
                                                    f31: callback_return_w2_f31_built.expect("active input"),
                                                };
                                                callback_return_w2
                                            })
                                        } else {
                                            None
                                        };
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w0_built = if true {
                                            Some(callback_return_w2_built.expect("active input"))
                                        } else {
                                            None
                                        };
                                        let callback_return_value = callback_return_w0_built
                                            .expect("active input");
                                        Ok::<_, String>(callback_return_value)
                                    }
                                },
                            )
                            .unwrap_or_else(|error| panic!("callback failed: {}", error))
                    };
                    p0_callback_value
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_callback_value_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1row_1return_1fire_1434e8dbfa624de44<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &__pb_input_consumed;
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
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: &bench_flat::RowReturnHandler = unsafe {
                        &(*p0_w1_pointer).value
                    };
                    p0_w0
                })
            } else {
                None
            };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p1_w0_built = if true {
                Some({
                    let p1_w0: i64 = ::core::convert::Into::into(
                        p1_w0.expect("active value"),
                    );
                    p1_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let __prebindgen_argument_1 = p1_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1wide_1row_1return_1fire_1ea0468a8c78895fe<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &__pb_input_consumed;
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
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: &bench_flat::WideRowReturnHandler = unsafe {
                        &(*p0_w1_pointer).value
                    };
                    p0_w0
                })
            } else {
                None
            };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p1_w0_built = if true {
                Some({
                    let p1_w0: i64 = ::core::convert::Into::into(
                        p1_w0.expect("active value"),
                    );
                    p1_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let __prebindgen_argument_1 = p1_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1control_1b0073ee32096caea<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: i64 = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
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
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true {
                Some({
                    let p0_w2: i64 = ::core::convert::Into::into(
                        p0_w2.expect("active value"),
                    );
                    p0_w2
                })
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
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
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f0_built = if true {
                Some({
                    let p0_w2_f0: i64 = ::core::convert::Into::into(
                        p0_w2_f0.expect("active value"),
                    );
                    p0_w2_f0
                })
            } else {
                None
            };
            let p0_w2_f1 = if true { Some(p0_w3_f1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f1_built = if true {
                Some({
                    let p0_w2_f1: i64 = ::core::convert::Into::into(
                        p0_w2_f1.expect("active value"),
                    );
                    p0_w2_f1
                })
            } else {
                None
            };
            let p0_w2_f2 = if true { Some(p0_w3_f2) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f2_built = if true {
                Some({
                    let p0_w2_f2: i64 = ::core::convert::Into::into(
                        p0_w2_f2.expect("active value"),
                    );
                    p0_w2_f2
                })
            } else {
                None
            };
            let p0_w2_f3 = if true { Some(p0_w3_f3) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f3_built = if true {
                Some({
                    let p0_w2_f3: i64 = ::core::convert::Into::into(
                        p0_w2_f3.expect("active value"),
                    );
                    p0_w2_f3
                })
            } else {
                None
            };
            let p0_w2_f4 = if true { Some(p0_w3_f4) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f4_built = if true {
                Some({
                    let p0_w2_f4: i64 = ::core::convert::Into::into(
                        p0_w2_f4.expect("active value"),
                    );
                    p0_w2_f4
                })
            } else {
                None
            };
            let p0_w2_f5 = if true { Some(p0_w3_f5) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f5_built = if true {
                Some({
                    let p0_w2_f5: i64 = ::core::convert::Into::into(
                        p0_w2_f5.expect("active value"),
                    );
                    p0_w2_f5
                })
            } else {
                None
            };
            let p0_w2_f6 = if true { Some(p0_w3_f6) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f6_built = if true {
                Some({
                    let p0_w2_f6: i64 = ::core::convert::Into::into(
                        p0_w2_f6.expect("active value"),
                    );
                    p0_w2_f6
                })
            } else {
                None
            };
            let p0_w2_f7 = if true { Some(p0_w3_f7) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f7_built = if true {
                Some({
                    let p0_w2_f7: i64 = ::core::convert::Into::into(
                        p0_w2_f7.expect("active value"),
                    );
                    p0_w2_f7
                })
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true {
                Some({
                    let p0_w2: bench_flat::Row = bench_flat::Row {
                        a: p0_w2_f0_built.expect("active input"),
                        b: p0_w2_f1_built.expect("active input"),
                        c: p0_w2_f2_built.expect("active input"),
                        d: p0_w2_f3_built.expect("active input"),
                        e: p0_w2_f4_built.expect("active input"),
                        f: p0_w2_f5_built.expect("active input"),
                        g: p0_w2_f6_built.expect("active input"),
                        h: p0_w2_f7_built.expect("active input"),
                    };
                    p0_w2
                })
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
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
            let mut p0_w2: Vec<i64> = Vec::new();
            if true {
                let (p0_w3_start, p0_w4) = p0_w1_reader
                    .rows(
                        p0_w3,
                        ::core::mem::size_of::<__PbBulk_i64_a60d3960993b9a77>(),
                    )?;
                p0_w2.reserve_exact(p0_w4);
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
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w3_element_built = (|| -> ::core::result::Result<i64, String> {
                        let p0_w6 = if true { Some(p0_w7) } else { None };
                        let p0_w6: i64 = ::core::convert::Into::into(
                            p0_w6.expect("active value"),
                        );
                        let p0_w2 = p0_w6;
                        Ok(p0_w2)
                    })();
                    let element = p0_w3_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w2.len();
                        p0_w2.as_mut_ptr().add(initialized).write(element);
                        p0_w2.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true { Some(p0_w2) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
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
            let mut p0_w2: Vec<bench_flat::Row> = Vec::new();
            if true {
                let (p0_w3_start, p0_w4) = p0_w1_reader
                    .rows(
                        p0_w3,
                        ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                    )?;
                p0_w2.reserve_exact(p0_w4);
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
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w3_element_built = (|| -> ::core::result::Result<
                        bench_flat::Row,
                        String,
                    > {
                        let p0_w6_f0 = if true { Some(p0_w7_f0) } else { None };
                        let p0_w6_f1 = if true { Some(p0_w7_f1) } else { None };
                        let p0_w6_f2 = if true { Some(p0_w7_f2) } else { None };
                        let p0_w6_f3 = if true { Some(p0_w7_f3) } else { None };
                        let p0_w6_f4 = if true { Some(p0_w7_f4) } else { None };
                        let p0_w6_f5 = if true { Some(p0_w7_f5) } else { None };
                        let p0_w6_f6 = if true { Some(p0_w7_f6) } else { None };
                        let p0_w6_f7 = if true { Some(p0_w7_f7) } else { None };
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
                    })();
                    let element = p0_w3_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w2.len();
                        p0_w2.as_mut_ptr().add(initialized).write(element);
                        p0_w2.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true { Some(p0_w2.as_slice()) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: i64 = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p1_w0_built = if true {
                Some({
                    let p1_w0: i64 = ::core::convert::Into::into(
                        p1_w0.expect("active value"),
                    );
                    p1_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let __prebindgen_argument_1 = p1_w0_built.expect("active input");
            let source_value = bench_flat::rows_make(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_f4d5524a08b9867b>(),
            );
            let ret_w1_source: ::std::vec::Vec<bench_flat::Row> = source_value;
            let ret_w3 = ::std::vec::Vec::len(&ret_w1_source);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                    ::core::mem::align_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                )?;
            for (ret_w4, ret_w2) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w1_source),
            ) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: i64 = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p1_w0_built = if true {
                Some({
                    let p1_w0: i64 = ::core::convert::Into::into(
                        p1_w0.expect("active value"),
                    );
                    p1_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let __prebindgen_argument_1 = p1_w0_built.expect("active input");
            let source_value = bench_flat::rows_for_reduce(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_f4d5524a08b9867b>(),
            );
            let ret_w1_source: ::std::vec::Vec<bench_flat::Row> = source_value;
            let ret_w3 = ::std::vec::Vec::len(&ret_w1_source);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                    ::core::mem::align_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                )?;
            for (ret_w4, ret_w2) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w1_source),
            ) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
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
            let mut p0_w2: Vec<bench_flat::Row> = Vec::new();
            if true {
                let (p0_w3_start, p0_w4) = p0_w1_reader
                    .rows(
                        p0_w3,
                        ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                    )?;
                p0_w2.reserve_exact(p0_w4);
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
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w3_element_built = (|| -> ::core::result::Result<
                        bench_flat::Row,
                        String,
                    > {
                        let p0_w6_f0 = if true { Some(p0_w7_f0) } else { None };
                        let p0_w6_f1 = if true { Some(p0_w7_f1) } else { None };
                        let p0_w6_f2 = if true { Some(p0_w7_f2) } else { None };
                        let p0_w6_f3 = if true { Some(p0_w7_f3) } else { None };
                        let p0_w6_f4 = if true { Some(p0_w7_f4) } else { None };
                        let p0_w6_f5 = if true { Some(p0_w7_f5) } else { None };
                        let p0_w6_f6 = if true { Some(p0_w7_f6) } else { None };
                        let p0_w6_f7 = if true { Some(p0_w7_f7) } else { None };
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
                    })();
                    let element = p0_w3_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w2.len();
                        p0_w2.as_mut_ptr().add(initialized).write(element);
                        p0_w2.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true { Some(p0_w2) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = bench_flat::rows_echo(__prebindgen_argument_0);
            let mut ret_w0_writer = ::prebindgen_jni_runtime::bulk::Writer::new(
                ::core::mem::size_of::<__PbBulk_Range_f4d5524a08b9867b>(),
            );
            let ret_w1_source: ::std::vec::Vec<bench_flat::Row> = source_value;
            let ret_w3 = ::std::vec::Vec::len(&ret_w1_source);
            let ret_w1 = ret_w0_writer
                .rows(
                    ret_w3,
                    ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                    ::core::mem::align_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                )?;
            for (ret_w4, ret_w2) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w1_source),
            ) {
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_bytes = if true {
                ::prebindgen_jni_runtime::bulk::input(
                    &mut jni_environment,
                    &p0_w1,
                    p0_w1_length,
                )?
            } else {
                ::core::default::Default::default()
            };
            let p0_w1_reader = &::prebindgen_jni_runtime::bulk::Reader::new(
                &p0_w1_bytes,
            );
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
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f0_f0_built = if true {
                Some({
                    let p0_w2_f0_f0: i64 = ::core::convert::Into::into(
                        p0_w2_f0_f0.expect("active value"),
                    );
                    p0_w2_f0_f0
                })
            } else {
                None
            };
            let p0_w2_f0_f1 = if true { Some(p0_w3_f0_f1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f0_f1_built = if true {
                Some({
                    let p0_w2_f0_f1: i64 = ::core::convert::Into::into(
                        p0_w2_f0_f1.expect("active value"),
                    );
                    p0_w2_f0_f1
                })
            } else {
                None
            };
            let p0_w2_f0_f2 = if true { Some(p0_w3_f0_f2) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f0_f2_built = if true {
                Some({
                    let p0_w2_f0_f2: i64 = ::core::convert::Into::into(
                        p0_w2_f0_f2.expect("active value"),
                    );
                    p0_w2_f0_f2
                })
            } else {
                None
            };
            let p0_w2_f0_f3 = if true { Some(p0_w3_f0_f3) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f0_f3_built = if true {
                Some({
                    let p0_w2_f0_f3: i64 = ::core::convert::Into::into(
                        p0_w2_f0_f3.expect("active value"),
                    );
                    p0_w2_f0_f3
                })
            } else {
                None
            };
            let p0_w2_f0_f4 = if true { Some(p0_w3_f0_f4) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f0_f4_built = if true {
                Some({
                    let p0_w2_f0_f4: i64 = ::core::convert::Into::into(
                        p0_w2_f0_f4.expect("active value"),
                    );
                    p0_w2_f0_f4
                })
            } else {
                None
            };
            let p0_w2_f0_f5 = if true { Some(p0_w3_f0_f5) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f0_f5_built = if true {
                Some({
                    let p0_w2_f0_f5: i64 = ::core::convert::Into::into(
                        p0_w2_f0_f5.expect("active value"),
                    );
                    p0_w2_f0_f5
                })
            } else {
                None
            };
            let p0_w2_f0_f6 = if true { Some(p0_w3_f0_f6) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f0_f6_built = if true {
                Some({
                    let p0_w2_f0_f6: i64 = ::core::convert::Into::into(
                        p0_w2_f0_f6.expect("active value"),
                    );
                    p0_w2_f0_f6
                })
            } else {
                None
            };
            let p0_w2_f0_f7 = if true { Some(p0_w3_f0_f7) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f0_f7_built = if true {
                Some({
                    let p0_w2_f0_f7: i64 = ::core::convert::Into::into(
                        p0_w2_f0_f7.expect("active value"),
                    );
                    p0_w2_f0_f7
                })
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f0_built = if true {
                Some({
                    let p0_w2_f0: bench_flat::Row = bench_flat::Row {
                        a: p0_w2_f0_f0_built.expect("active input"),
                        b: p0_w2_f0_f1_built.expect("active input"),
                        c: p0_w2_f0_f2_built.expect("active input"),
                        d: p0_w2_f0_f3_built.expect("active input"),
                        e: p0_w2_f0_f4_built.expect("active input"),
                        f: p0_w2_f0_f5_built.expect("active input"),
                        g: p0_w2_f0_f6_built.expect("active input"),
                        h: p0_w2_f0_f7_built.expect("active input"),
                    };
                    p0_w2_f0
                })
            } else {
                None
            };
            let p0_w2_f1 = if true {
                Some(p0_w1_reader.buffer_range(p0_w3_f1)?.ok_or("null bulk bytes")?)
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_f1_built = if true {
                Some({
                    let p0_w2_f1: Vec<u8> = ::core::convert::Into::into(
                        p0_w2_f1.expect("active value"),
                    );
                    p0_w2_f1
                })
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if true {
                Some({
                    let p0_w2: bench_flat::Packet = bench_flat::Packet {
                        header: p0_w2_f0_built.expect("active input"),
                        bytes: p0_w2_f1_built.expect("active input"),
                    };
                    p0_w2
                })
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some(p0_w2_built.expect("active input"))
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_retained = ::prebindgen_jni_runtime::Callback::new_named(
                &mut jni_environment,
                &p0_callback,
                "([B)J",
                "bench.bulk.RowsCallback",
            )?;
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_callback_value_built = if true {
                Some({
                    #[allow(unused_mut, clippy::collapsible_if)]
                    let p0_callback_value = move |
                        callback_source_0: Vec<bench_flat::Row>|
                    {
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
                                        let callback_arg_0_w1_source: ::std::vec::Vec<
                                            bench_flat::Row,
                                        > = callback_value_0;
                                        let callback_arg_0_w3 = ::std::vec::Vec::len(
                                            &callback_arg_0_w1_source,
                                        );
                                        let callback_arg_0_w1 = callback_arg_0_w0_writer
                                            .rows(
                                                callback_arg_0_w3,
                                                ::core::mem::size_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                                                ::core::mem::align_of::<__PbBulk_i64_i64_789cd54eb117685f>(),
                                            )?;
                                        for (callback_arg_0_w4, callback_arg_0_w2) in ::core::iter::Iterator::enumerate(
                                            ::core::iter::IntoIterator::into_iter(
                                                callback_arg_0_w1_source,
                                            ),
                                        ) {
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
                                            #[allow(
                                                clippy::redundant_closure_call,
                                                clippy::let_unit_value,
                                                clippy::unused_unit
                                            )]
                                            let _: () = (|| -> () {})();
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
                                            callback_arg_0_w0_writer.commit();
                                        }
                                        let __pb_input_consumed = ::core::cell::Cell::new(0usize);
                                        let __prebindgen_input_consumed = &__pb_input_consumed;
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
                                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                                        let callback_return_w0_built = if true {
                                            Some({
                                                let callback_return_w0: i64 = ::core::convert::Into::into(
                                                    callback_return_w0.expect("active value"),
                                                );
                                                callback_return_w0
                                            })
                                        } else {
                                            None
                                        };
                                        let callback_return_value = callback_return_w0_built
                                            .expect("active input");
                                        Ok::<_, String>(callback_return_value)
                                    }
                                },
                            )
                            .unwrap_or_else(|error| panic!("callback failed: {}", error))
                    };
                    p0_callback_value
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_callback_value_built.expect("active input");
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
    clippy::needless_borrow,
    clippy::borrow_deref_ref,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
/**# Safety
The caller must uphold the generated wrapper's locking, preparation and ownership protocol, with live pointers of the declared types. Complete Kotlin handle preflight before this entry point; native decoding does not check input aliases.*/
#[no_mangle]
pub unsafe extern "system" fn Java_bench_bulk_JNINative_call_1fire_1ff69607332471b89<
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
    let __pb_input_consumed = ::core::cell::Cell::new(0usize);
    let __pb_receipt_valid = ::core::cell::Cell::new(false);
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &__pb_input_consumed;
            let p0_w1_pointer = if true {
                p0_w1 as *mut ::prebindgen_jni_runtime::Allocation<bench_flat::Handler>
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: &bench_flat::Handler = unsafe { &(*p0_w1_pointer).value };
                    p0_w0
                })
            } else {
                None
            };
            let p1_w0 = if true { Some(p1_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p1_w0_built = if true {
                Some({
                    let p1_w0: i64 = ::core::convert::Into::into(
                        p1_w0.expect("active value"),
                    );
                    p1_w0
                })
            } else {
                None
            };
            let p2_w0 = if true { Some(p2_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p2_w0_built = if true {
                Some({
                    let p2_w0: i64 = ::core::convert::Into::into(
                        p2_w0.expect("active value"),
                    );
                    p2_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let __prebindgen_argument_1 = p1_w0_built.expect("active input");
            let __prebindgen_argument_2 = p2_w0_built.expect("active input");
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
