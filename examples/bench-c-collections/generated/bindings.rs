#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_vec_c_collection_flat_resource_1887aa00ecbbafc7_len(
    value: &Vec<Vec<c_collection_flat::Resource>>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_c_collection_flat_record_17cedf618ebcf11e_len(
    value: &Vec<c_collection_flat::Record>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_c_collection_flat_resource_d71c20b2b8f27a13_len(
    value: &Vec<c_collection_flat::Resource>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_u64_009441151d595b1a_len(value: &Vec<u64>) -> usize {
    value.len()
}
#[allow(dead_code)]
struct __PbOutputOwner<T: ?Sized>(*mut T);
impl<T: ?Sized> Drop for __PbOutputOwner<T> {
    fn drop(&mut self) {
        unsafe {
            drop(Box::from_raw(self.0));
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __Pb128 {
    pub hi: u64,
    pub lo: u64,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRows {
    pub data: *const ::core::ffi::c_void,
    pub len: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbBuffer {
    pub data: *const u8,
    pub len: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbBufferMut {
    pub data: *mut u8,
    pub len: usize,
}
unsafe fn __pb_report_message(message: String, error: *mut *mut std::ffi::c_char) {
    if !error.is_null() {
        std::ptr::write(
            error,
            std::ffi::CString::new(message.replace('\0', "?")).unwrap().into_raw(),
        );
    }
}
/**# Safety
The pointer must be null or an unfreed error string returned by this library.*/
#[no_mangle]
pub unsafe extern "C" fn bench_free_error(error: *mut std::ffi::c_char) {
    if !error.is_null() {
        drop(std::ffi::CString::from_raw(error));
    }
}
unsafe fn __pb_read<'a>(b: __PbBuffer) -> Result<&'a [u8], String> {
    if b.len == 0 {
        return Ok(&[]);
    }
    if b.data.is_null() || b.len > isize::MAX as usize {
        return Err("invalid buffer".into());
    }
    Ok(std::slice::from_raw_parts(b.data, b.len))
}
/**# Safety
    The buffer must have been returned by this library and not already freed.*/
#[no_mangle]
pub unsafe extern "C" fn bench_free_buffer(b: __PbBuffer) {
    if !b.data.is_null() {
        drop(
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(b.data as *mut u8, b.len)),
        );
    }
}
const _: () = {
    assert!(
        cfg!(all(target_arch = "x86_64", target_vendor = "unknown", target_os = "linux",
        target_env = "gnu")) && usize::BITS == 64u32,
        "C binding target differs from the Rust compilation target"
    );
};
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn bench_drop_resource(ptr: *mut c_collection_flat::Resource) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
#[repr(C)]
pub struct bench_control_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_control(p0_w1: usize) -> u64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_control_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: usize = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::control(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = bench_control_result { v0: ret_w0 };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_make_handles_ret_w0 {
    pub v0: *mut c_collection_flat::Resource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn bench_make_handles_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_make_handles_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct bench_make_handles_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_make_handles(p0_w1: usize) -> __PbRows {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_make_handles_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: usize = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::make_handles(__prebindgen_argument_0);
            let ret_w0_source: ::std::vec::Vec<c_collection_flat::Resource> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            unsafe fn ret_w0_release_row(row: &__PbRow_make_handles_ret_w0) {
                if !(row.v0).is_null() {
                    let _ = ::prebindgen_runtime::boundary(|| {
                        drop(
                            Box::from_raw((row.v0) as *mut c_collection_flat::Resource),
                        );
                        Ok(())
                    });
                }
            }
            let mut ret_w0 = unsafe {
                ::prebindgen_runtime::OutputRows::<
                    __PbRow_make_handles_ret_w0,
                >::new(ret_w2, ret_w0_release_row)
            };
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w4: *mut c_collection_flat::Resource = Box::into_raw(
                    Box::new(ret_w1),
                );
                let ret_w4_owner = __PbOutputOwner(ret_w4);
                unsafe {
                    ret_w0
                        .write_next(__PbRow_make_handles_ret_w0 {
                            v0: ret_w4,
                        });
                }
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    ::core::mem::forget(ret_w4_owner);
                })();
                unsafe {
                    ret_w0.commit_row();
                }
            }
            let result = bench_make_handles_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ret_w0.commit();
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_make_many_ret_w4 {
    pub v0: *mut c_collection_flat::Resource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn bench_make_many_ret_w4_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_make_many_ret_w4,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_make_many_ret_w0 {
    pub v0: __PbRows,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn bench_make_many_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_make_many_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct bench_make_many_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_make_many(p0_w1: usize) -> __PbRows {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_make_many_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: usize = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::make_many(__prebindgen_argument_0);
            let ret_w0_source: ::std::vec::Vec<Vec<c_collection_flat::Resource>> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            unsafe fn ret_w0_release_row(row: &__PbRow_make_many_ret_w0) {
                if (row.v0).len != 0 {
                    let rows = Box::from_raw(
                        std::ptr::slice_from_raw_parts_mut(
                            (row.v0).data as *mut __PbRow_make_many_ret_w4,
                            (row.v0).len,
                        ),
                    );
                    for row in rows.iter() {
                        if !(row.v0).is_null() {
                            let _ = ::prebindgen_runtime::boundary(|| {
                                drop(
                                    Box::from_raw((row.v0) as *mut c_collection_flat::Resource),
                                );
                                Ok(())
                            });
                        }
                    }
                }
            }
            let mut ret_w0 = unsafe {
                ::prebindgen_runtime::OutputRows::<
                    __PbRow_make_many_ret_w0,
                >::new(ret_w2, ret_w0_release_row)
            };
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w4_source: ::std::vec::Vec<c_collection_flat::Resource> = ret_w1;
                let ret_w6 = ::std::vec::Vec::len(&ret_w4_source);
                unsafe fn ret_w4_release_row(row: &__PbRow_make_many_ret_w4) {
                    if !(row.v0).is_null() {
                        let _ = ::prebindgen_runtime::boundary(|| {
                            drop(
                                Box::from_raw((row.v0) as *mut c_collection_flat::Resource),
                            );
                            Ok(())
                        });
                    }
                }
                let mut ret_w4 = unsafe {
                    ::prebindgen_runtime::OutputRows::<
                        __PbRow_make_many_ret_w4,
                    >::new(ret_w6, ret_w4_release_row)
                };
                for (ret_w7, ret_w5) in ::core::iter::Iterator::enumerate(
                    ::core::iter::IntoIterator::into_iter(ret_w4_source),
                ) {
                    let ret_w8: *mut c_collection_flat::Resource = Box::into_raw(
                        Box::new(ret_w5),
                    );
                    let ret_w8_owner = __PbOutputOwner(ret_w8);
                    unsafe {
                        ret_w4
                            .write_next(__PbRow_make_many_ret_w4 {
                                v0: ret_w8,
                            });
                    }
                    #[allow(
                        clippy::redundant_closure_call,
                        clippy::let_unit_value,
                        clippy::unused_unit
                    )]
                    let _: () = (|| -> () {
                        ::core::mem::forget(ret_w8_owner);
                    })();
                    unsafe {
                        ret_w4.commit_row();
                    }
                }
                unsafe {
                    ret_w0
                        .write_next(__PbRow_make_many_ret_w0 {
                            v0: __PbRows {
                                data: if ret_w4.is_empty() {
                                    ::core::ptr::null()
                                } else {
                                    ret_w4.as_ptr().cast()
                                },
                                len: ret_w4.len(),
                            },
                        });
                }
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    ret_w4.commit();
                })();
                unsafe {
                    ret_w0.commit_row();
                }
            }
            let result = bench_make_many_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ret_w0.commit();
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_make_nested_ret_w4 {
    pub v0: *mut c_collection_flat::Resource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn bench_make_nested_ret_w4_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_make_nested_ret_w4,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_make_nested_ret_w0 {
    pub v0: __PbRows,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn bench_make_nested_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_make_nested_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct bench_make_nested_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_make_nested(p0_w1: usize) -> __PbRows {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_make_nested_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: usize = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::make_nested(__prebindgen_argument_0);
            let ret_w0_source: ::std::vec::Vec<Vec<c_collection_flat::Resource>> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            unsafe fn ret_w0_release_row(row: &__PbRow_make_nested_ret_w0) {
                if (row.v0).len != 0 {
                    let rows = Box::from_raw(
                        std::ptr::slice_from_raw_parts_mut(
                            (row.v0).data as *mut __PbRow_make_nested_ret_w4,
                            (row.v0).len,
                        ),
                    );
                    for row in rows.iter() {
                        if !(row.v0).is_null() {
                            let _ = ::prebindgen_runtime::boundary(|| {
                                drop(
                                    Box::from_raw((row.v0) as *mut c_collection_flat::Resource),
                                );
                                Ok(())
                            });
                        }
                    }
                }
            }
            let mut ret_w0 = unsafe {
                ::prebindgen_runtime::OutputRows::<
                    __PbRow_make_nested_ret_w0,
                >::new(ret_w2, ret_w0_release_row)
            };
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w4_source: ::std::vec::Vec<c_collection_flat::Resource> = ret_w1;
                let ret_w6 = ::std::vec::Vec::len(&ret_w4_source);
                unsafe fn ret_w4_release_row(row: &__PbRow_make_nested_ret_w4) {
                    if !(row.v0).is_null() {
                        let _ = ::prebindgen_runtime::boundary(|| {
                            drop(
                                Box::from_raw((row.v0) as *mut c_collection_flat::Resource),
                            );
                            Ok(())
                        });
                    }
                }
                let mut ret_w4 = unsafe {
                    ::prebindgen_runtime::OutputRows::<
                        __PbRow_make_nested_ret_w4,
                    >::new(ret_w6, ret_w4_release_row)
                };
                for (ret_w7, ret_w5) in ::core::iter::Iterator::enumerate(
                    ::core::iter::IntoIterator::into_iter(ret_w4_source),
                ) {
                    let ret_w8: *mut c_collection_flat::Resource = Box::into_raw(
                        Box::new(ret_w5),
                    );
                    let ret_w8_owner = __PbOutputOwner(ret_w8);
                    unsafe {
                        ret_w4
                            .write_next(__PbRow_make_nested_ret_w4 {
                                v0: ret_w8,
                            });
                    }
                    #[allow(
                        clippy::redundant_closure_call,
                        clippy::let_unit_value,
                        clippy::unused_unit
                    )]
                    let _: () = (|| -> () {
                        ::core::mem::forget(ret_w8_owner);
                    })();
                    unsafe {
                        ret_w4.commit_row();
                    }
                }
                unsafe {
                    ret_w0
                        .write_next(__PbRow_make_nested_ret_w0 {
                            v0: __PbRows {
                                data: if ret_w4.is_empty() {
                                    ::core::ptr::null()
                                } else {
                                    ret_w4.as_ptr().cast()
                                },
                                len: ret_w4.len(),
                            },
                        });
                }
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    ret_w4.commit();
                })();
                unsafe {
                    ret_w0.commit_row();
                }
            }
            let result = bench_make_nested_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ret_w0.commit();
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_make_numbers_ret_w0 {
    pub v0: u64,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn bench_make_numbers_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_make_numbers_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct bench_make_numbers_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_make_numbers(p0_w1: usize) -> __PbRows {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_make_numbers_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: usize = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::make_numbers(__prebindgen_argument_0);
            let ret_w0_source: ::std::vec::Vec<u64> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            unsafe fn ret_w0_release_row(row: &__PbRow_make_numbers_ret_w0) {}
            let mut ret_w0 = unsafe {
                ::prebindgen_runtime::OutputRows::<
                    __PbRow_make_numbers_ret_w0,
                >::new(ret_w2, ret_w0_release_row)
            };
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w4 = ret_w1;
                unsafe {
                    ret_w0
                        .write_next(__PbRow_make_numbers_ret_w0 {
                            v0: ret_w4,
                        });
                }
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {})();
                unsafe {
                    ret_w0.commit_row();
                }
            }
            let result = bench_make_numbers_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ret_w0.commit();
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_make_records_ret_w0 {
    pub v0: *mut c_collection_flat::Resource,
    pub v1: u64,
    pub v2: *mut c_collection_flat::Resource,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn bench_make_records_ret_w0_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_make_records_ret_w0,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct bench_make_records_result {
    pub v0: __PbRows,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_make_records(p0_w1: usize) -> __PbRows {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_make_records_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: usize = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::make_records(__prebindgen_argument_0);
            let ret_w0_source: ::std::vec::Vec<c_collection_flat::Record> = source_value;
            let ret_w2 = ::std::vec::Vec::len(&ret_w0_source);
            unsafe fn ret_w0_release_row(row: &__PbRow_make_records_ret_w0) {
                if !(row.v0).is_null() {
                    let _ = ::prebindgen_runtime::boundary(|| {
                        drop(
                            Box::from_raw((row.v0) as *mut c_collection_flat::Resource),
                        );
                        Ok(())
                    });
                }
                if !(row.v2).is_null() {
                    let _ = ::prebindgen_runtime::boundary(|| {
                        drop(
                            Box::from_raw((row.v2) as *mut c_collection_flat::Resource),
                        );
                        Ok(())
                    });
                }
            }
            let mut ret_w0 = unsafe {
                ::prebindgen_runtime::OutputRows::<
                    __PbRow_make_records_ret_w0,
                >::new(ret_w2, ret_w0_release_row)
            };
            for (ret_w3, ret_w1) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w0_source),
            ) {
                let ret_w4 = c_collection_flat::record_into(ret_w1);
                let ret_w5 = ret_w4;
                let ret_w6: *mut c_collection_flat::Resource = Box::into_raw(
                    Box::new(ret_w5.first),
                );
                let ret_w6_owner = __PbOutputOwner(ret_w6);
                let ret_w7 = ret_w5.tag;
                let ret_w8: *mut c_collection_flat::Resource = Box::into_raw(
                    Box::new(ret_w5.second),
                );
                let ret_w8_owner = __PbOutputOwner(ret_w8);
                unsafe {
                    ret_w0
                        .write_next(__PbRow_make_records_ret_w0 {
                            v0: ret_w6,
                            v1: ret_w7,
                            v2: ret_w8,
                        });
                }
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    ::core::mem::forget(ret_w6_owner);
                    ::core::mem::forget(ret_w8_owner);
                })();
                unsafe {
                    ret_w0.commit_row();
                }
            }
            let result = bench_make_records_result {
                v0: __PbRows {
                    data: if ret_w0.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w0.as_ptr().cast()
                    },
                    len: ret_w0.len(),
                },
            };
            std::ptr::write(output, result);
            let _: () = {
                ret_w0.commit();
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
#[repr(C)]
pub struct bench_resource_value_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_resource_value(
    p0_w1: *const c_collection_flat::Resource,
) -> u64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_resource_value_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let p0_w1_pointer = if true {
                p0_w1 as *mut c_collection_flat::Resource
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                if true {
                    for address in ::core::iter::once((p0_w1_pointer.addr()) as usize) {
                        __prebindgen_input_aliases.access(address, false)?;
                    }
                }
                Some({
                    let p0_w0: &c_collection_flat::Resource = unsafe {
                        &(*p0_w1_pointer)
                    };
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::resource_value(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = bench_resource_value_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    value.v0
}
const _: () = {
    konst::assertc_eq!(
        c_collection_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::Record > () != 0,
    "flat object `Record` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::Resource > () != 0,
    "flat object `Resource` must have nonzero size"
);
