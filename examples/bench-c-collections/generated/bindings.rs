#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_string_len(value: &String) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_string_view(value: &String) -> &[u8] {
    value.as_bytes()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_vec_c_collection_flat_resource_1887aa00ecbbafc7_len(
    value: &Vec<Vec<c_collection_flat::Resource>>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_c_collection_flat_buffersinput_81ed43e9857f6d9e_len(
    value: &Vec<c_collection_flat::BuffersInput>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_c_collection_flat_bytesinput_4663df5690fff942_len(
    value: &Vec<c_collection_flat::BytesInput>,
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
fn __pb_owned_owned_vec_c_collection_flat_textinput_242aaf40bf165058_len(
    value: &Vec<c_collection_flat::TextInput>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_c_collection_flat_wideinput_b588c8309edafb5c_len(
    value: &Vec<c_collection_flat::WideInput>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_u64_009441151d595b1a_len(value: &Vec<u64>) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_u8_672e72343c1b4cb4_len(value: &Vec<u8>) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_u8_672e72343c1b4cb4_view(value: &Vec<u8>) -> &[u8] {
    value.as_slice()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_u8_672e72343c1b4cb4_view_mut(value: &mut Vec<u8>) -> &mut [u8] {
    value.as_mut_slice()
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
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
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
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
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
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
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
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
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
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
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
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
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
pub struct bench_resource_new_result {
    pub v0: *mut c_collection_flat::Resource,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_resource_new(
    p0_w1: u64,
) -> *mut c_collection_flat::Resource {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_resource_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: u64 = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::resource_new(__prebindgen_argument_0);
            let ret_w0: *mut c_collection_flat::Resource = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = bench_resource_new_result {
                v0: ret_w0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0_owner);
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
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_take_buffers_p0_w1 {
    pub v0: *mut c_collection_flat::Resource,
    pub v1: __PbBuffer,
    pub v2: __PbBuffer,
}
#[repr(C)]
pub struct bench_take_buffers_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_buffers(p0_w1: __PbRows) -> u64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_take_buffers_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::DynamicInputs::default();
            let _: () = {};
            let mut p0_w0: Vec<c_collection_flat::BuffersInput> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_take_buffers_p0_w1,
                    p0_w2,
                )?;
                let p0_w1_rows: &[__PbRow_take_buffers_p0_w1] = if p0_w2 == 0 {
                    &[]
                } else {
                    unsafe {
                        ::core::slice::from_raw_parts(
                            p0_w1.data as *const __PbRow_take_buffers_p0_w1,
                            p0_w2,
                        )
                    }
                };
                let mut p0_w1_accesses = __prebindgen_input_aliases
                    .begin_group(p0_w2.saturating_mul(1usize));
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = p0_w1_rows[p0_w3].v0;
                    #[allow(unused_variables)]
                    let p0_w8 = p0_w1_rows[p0_w3].v1;
                    #[allow(unused_variables)]
                    let p0_w10 = p0_w1_rows[p0_w3].v2;
                    #[allow(unused_variables, clippy::redundant_closure_call)]
                    let checked = (|| {
                        let p0_w6_pointer = if true {
                            (p0_w6) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                        }
                        if true {
                            for address in ::core::iter::once(
                                (p0_w6_pointer.addr()) as usize,
                            ) {
                                p0_w1_accesses.push((address, true));
                            }
                        }
                        Ok::<_, String>(())
                    })();
                    checked?;
                }
                __prebindgen_input_aliases.accept_group(p0_w1_accesses)?;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = p0_w1_rows[p0_w3].v0;
                    #[allow(unused_variables)]
                    let p0_w8 = p0_w1_rows[p0_w3].v1;
                    #[allow(unused_variables)]
                    let p0_w10 = p0_w1_rows[p0_w3].v2;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::BuffersInput,
                        String,
                    > {
                        let p0_w6_pointer = if true {
                            (p0_w6) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                        }
                        if true {
                            __prebindgen_input_consumed
                                .set(
                                    __prebindgen_input_consumed.get()
                                        + (::core::iter::once((p0_w6_pointer.addr()) as usize))
                                            .count(),
                                );
                        }
                        let p0_w5 = if true {
                            Some(unsafe { Box::from_raw(p0_w6_pointer) })
                        } else {
                            None
                        };
                        #[allow(unused_mut, unused_assignments)]
                        let mut p0_w7: Option<&str> = None;
                        #[allow(unused_mut, unused_assignments)]
                        let mut p0_w9: Option<&[u8]> = None;
                        let p0_w5: c_collection_flat::Resource = *p0_w5
                            .expect("active allocation");
                        p0_w7 = if true {
                            Some(
                                std::str::from_utf8(unsafe { __pb_read(p0_w8)? })
                                    .map_err(|_| "invalid UTF-8")?,
                            )
                        } else {
                            None
                        };
                        let p0_w7: String = ::core::convert::Into::into(
                            p0_w7.expect("active value"),
                        );
                        p0_w9 = if true {
                            Some(unsafe { __pb_read(p0_w10)? })
                        } else {
                            None
                        };
                        let p0_w9: Vec<u8> = ::core::convert::Into::into(
                            p0_w9.expect("active value"),
                        );
                        let p0_w4: c_collection_flat::BuffersInput = c_collection_flat::buffers_new(
                            p0_w5,
                            p0_w7,
                            p0_w9,
                        );
                        let p0_w0 = p0_w4;
                        Ok(p0_w0)
                    })();
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true { Some(p0_w0) } else { None };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::take_buffers(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = bench_take_buffers_result {
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_take_bytes_p0_w1 {
    pub v0: *mut c_collection_flat::Resource,
    pub v1: __PbBuffer,
}
#[repr(C)]
pub struct bench_take_bytes_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_bytes(p0_w1: __PbRows) -> u64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_take_bytes_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::DynamicInputs::default();
            let _: () = {};
            let mut p0_w0: Vec<c_collection_flat::BytesInput> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_take_bytes_p0_w1,
                    p0_w2,
                )?;
                let p0_w1_rows: &[__PbRow_take_bytes_p0_w1] = if p0_w2 == 0 {
                    &[]
                } else {
                    unsafe {
                        ::core::slice::from_raw_parts(
                            p0_w1.data as *const __PbRow_take_bytes_p0_w1,
                            p0_w2,
                        )
                    }
                };
                let mut p0_w1_accesses = __prebindgen_input_aliases
                    .begin_group(p0_w2.saturating_mul(1usize));
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = p0_w1_rows[p0_w3].v0;
                    #[allow(unused_variables)]
                    let p0_w8 = p0_w1_rows[p0_w3].v1;
                    #[allow(unused_variables, clippy::redundant_closure_call)]
                    let checked = (|| {
                        let p0_w6_pointer = if true {
                            (p0_w6) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                        }
                        if true {
                            for address in ::core::iter::once(
                                (p0_w6_pointer.addr()) as usize,
                            ) {
                                p0_w1_accesses.push((address, true));
                            }
                        }
                        Ok::<_, String>(())
                    })();
                    checked?;
                }
                __prebindgen_input_aliases.accept_group(p0_w1_accesses)?;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = p0_w1_rows[p0_w3].v0;
                    #[allow(unused_variables)]
                    let p0_w8 = p0_w1_rows[p0_w3].v1;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::BytesInput,
                        String,
                    > {
                        let p0_w6_pointer = if true {
                            (p0_w6) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                        }
                        if true {
                            __prebindgen_input_consumed
                                .set(
                                    __prebindgen_input_consumed.get()
                                        + (::core::iter::once((p0_w6_pointer.addr()) as usize))
                                            .count(),
                                );
                        }
                        let p0_w5 = if true {
                            Some(unsafe { Box::from_raw(p0_w6_pointer) })
                        } else {
                            None
                        };
                        #[allow(unused_mut, unused_assignments)]
                        let mut p0_w7: Option<&[u8]> = None;
                        let p0_w5: c_collection_flat::Resource = *p0_w5
                            .expect("active allocation");
                        p0_w7 = if true {
                            Some(unsafe { __pb_read(p0_w8)? })
                        } else {
                            None
                        };
                        let p0_w7: Vec<u8> = ::core::convert::Into::into(
                            p0_w7.expect("active value"),
                        );
                        let p0_w4: c_collection_flat::BytesInput = c_collection_flat::bytes_new(
                            p0_w5,
                            p0_w7,
                        );
                        let p0_w0 = p0_w4;
                        Ok(p0_w0)
                    })();
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true { Some(p0_w0) } else { None };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::take_bytes(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = bench_take_bytes_result {
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_take_handles_p0_w1 {
    pub v0: *mut c_collection_flat::Resource,
}
#[repr(C)]
pub struct bench_take_handles_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_handles(p0_w1: __PbRows) -> u64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_take_handles_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::DynamicInputs::default();
            let _: () = {};
            let mut p0_w0: Vec<c_collection_flat::Resource> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_take_handles_p0_w1,
                    p0_w2,
                )?;
                let p0_w1_rows: &[__PbRow_take_handles_p0_w1] = if p0_w2 == 0 {
                    &[]
                } else {
                    unsafe {
                        ::core::slice::from_raw_parts(
                            p0_w1.data as *const __PbRow_take_handles_p0_w1,
                            p0_w2,
                        )
                    }
                };
                let mut p0_w1_accesses = __prebindgen_input_aliases
                    .begin_group(p0_w2.saturating_mul(1usize));
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = p0_w1_rows[p0_w3].v0;
                    #[allow(unused_variables, clippy::redundant_closure_call)]
                    let checked = (|| {
                        let p0_w5_pointer = if true {
                            (p0_w5) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w5_pointer)?;
                        }
                        if true {
                            for address in ::core::iter::once(
                                (p0_w5_pointer.addr()) as usize,
                            ) {
                                p0_w1_accesses.push((address, true));
                            }
                        }
                        Ok::<_, String>(())
                    })();
                    checked?;
                }
                __prebindgen_input_aliases.accept_group(p0_w1_accesses)?;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = p0_w1_rows[p0_w3].v0;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::Resource,
                        String,
                    > {
                        let p0_w5_pointer = if true {
                            (p0_w5) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w5_pointer)?;
                        }
                        if true {
                            __prebindgen_input_consumed
                                .set(
                                    __prebindgen_input_consumed.get()
                                        + (::core::iter::once((p0_w5_pointer.addr()) as usize))
                                            .count(),
                                );
                        }
                        let p0_w4 = if true {
                            Some(unsafe { Box::from_raw(p0_w5_pointer) })
                        } else {
                            None
                        };
                        let p0_w4: c_collection_flat::Resource = *p0_w4
                            .expect("active allocation");
                        let p0_w0 = p0_w4;
                        Ok(p0_w0)
                    })();
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true { Some(p0_w0) } else { None };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::take_handles(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = bench_take_handles_result {
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_take_numbers_p0_w1 {
    pub v0: u64,
}
#[repr(C)]
pub struct bench_take_numbers_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_numbers(p0_w1: __PbRows) -> u64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_take_numbers_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<u64> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_take_numbers_p0_w1,
                    p0_w2,
                )?;
                let p0_w1_rows: &[__PbRow_take_numbers_p0_w1] = if p0_w2 == 0 {
                    &[]
                } else {
                    unsafe {
                        ::core::slice::from_raw_parts(
                            p0_w1.data as *const __PbRow_take_numbers_p0_w1,
                            p0_w2,
                        )
                    }
                };
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = p0_w1_rows[p0_w3].v0;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<u64, String> {
                        let p0_w4 = if true { Some(p0_w5) } else { None };
                        let p0_w4: u64 = ::core::convert::Into::into(
                            p0_w4.expect("active value"),
                        );
                        let p0_w0 = p0_w4;
                        Ok(p0_w0)
                    })();
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true { Some(p0_w0) } else { None };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::take_numbers(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = bench_take_numbers_result {
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_take_records_p0_w1 {
    pub v0: *mut c_collection_flat::Resource,
    pub v1: u64,
    pub v2: *mut c_collection_flat::Resource,
}
#[repr(C)]
pub struct bench_take_records_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_records(p0_w1: __PbRows) -> u64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_take_records_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::DynamicInputs::default();
            let _: () = {};
            let mut p0_w0: Vec<c_collection_flat::Record> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_take_records_p0_w1,
                    p0_w2,
                )?;
                let p0_w1_rows: &[__PbRow_take_records_p0_w1] = if p0_w2 == 0 {
                    &[]
                } else {
                    unsafe {
                        ::core::slice::from_raw_parts(
                            p0_w1.data as *const __PbRow_take_records_p0_w1,
                            p0_w2,
                        )
                    }
                };
                let mut p0_w1_accesses = __prebindgen_input_aliases
                    .begin_group(p0_w2.saturating_mul(2usize));
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = p0_w1_rows[p0_w3].v0;
                    let p0_w8 = p0_w1_rows[p0_w3].v1;
                    let p0_w10 = p0_w1_rows[p0_w3].v2;
                    #[allow(unused_variables, clippy::redundant_closure_call)]
                    let checked = (|| {
                        let p0_w6_pointer = if true {
                            (p0_w6) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                        }
                        let p0_w7 = if true { Some(p0_w8) } else { None };
                        let p0_w10_pointer = if true {
                            (p0_w10) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w10_pointer)?;
                        }
                        if true {
                            for address in ::core::iter::once(
                                (p0_w6_pointer.addr()) as usize,
                            ) {
                                p0_w1_accesses.push((address, true));
                            }
                        }
                        if true {
                            for address in ::core::iter::once(
                                (p0_w10_pointer.addr()) as usize,
                            ) {
                                p0_w1_accesses.push((address, true));
                            }
                        }
                        Ok::<_, String>(())
                    })();
                    checked?;
                }
                __prebindgen_input_aliases.accept_group(p0_w1_accesses)?;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = p0_w1_rows[p0_w3].v0;
                    let p0_w8 = p0_w1_rows[p0_w3].v1;
                    let p0_w10 = p0_w1_rows[p0_w3].v2;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::Record,
                        String,
                    > {
                        let p0_w6_pointer = if true {
                            (p0_w6) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                        }
                        let p0_w7 = if true { Some(p0_w8) } else { None };
                        let p0_w10_pointer = if true {
                            (p0_w10) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w10_pointer)?;
                        }
                        if true {
                            __prebindgen_input_consumed
                                .set(
                                    __prebindgen_input_consumed.get()
                                        + (::core::iter::once((p0_w6_pointer.addr()) as usize))
                                            .count(),
                                );
                        }
                        if true {
                            __prebindgen_input_consumed
                                .set(
                                    __prebindgen_input_consumed.get()
                                        + (::core::iter::once((p0_w10_pointer.addr()) as usize))
                                            .count(),
                                );
                        }
                        let p0_w5 = if true {
                            Some(unsafe { Box::from_raw(p0_w6_pointer) })
                        } else {
                            None
                        };
                        let p0_w9 = if true {
                            Some(unsafe { Box::from_raw(p0_w10_pointer) })
                        } else {
                            None
                        };
                        let p0_w5: c_collection_flat::Resource = *p0_w5
                            .expect("active allocation");
                        let p0_w7: u64 = ::core::convert::Into::into(
                            p0_w7.expect("active value"),
                        );
                        let p0_w9: c_collection_flat::Resource = *p0_w9
                            .expect("active allocation");
                        let p0_w4: c_collection_flat::Record = c_collection_flat::record_new(
                            p0_w5,
                            p0_w7,
                            p0_w9,
                        );
                        let p0_w0 = p0_w4;
                        Ok(p0_w0)
                    })();
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true { Some(p0_w0) } else { None };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::take_records(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = bench_take_records_result {
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_take_text_p0_w1 {
    pub v0: *mut c_collection_flat::Resource,
    pub v1: __PbBuffer,
}
#[repr(C)]
pub struct bench_take_text_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_text(p0_w1: __PbRows) -> u64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_take_text_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::DynamicInputs::default();
            let _: () = {};
            let mut p0_w0: Vec<c_collection_flat::TextInput> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_take_text_p0_w1,
                    p0_w2,
                )?;
                let p0_w1_rows: &[__PbRow_take_text_p0_w1] = if p0_w2 == 0 {
                    &[]
                } else {
                    unsafe {
                        ::core::slice::from_raw_parts(
                            p0_w1.data as *const __PbRow_take_text_p0_w1,
                            p0_w2,
                        )
                    }
                };
                let mut p0_w1_accesses = __prebindgen_input_aliases
                    .begin_group(p0_w2.saturating_mul(1usize));
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = p0_w1_rows[p0_w3].v0;
                    #[allow(unused_variables)]
                    let p0_w8 = p0_w1_rows[p0_w3].v1;
                    #[allow(unused_variables, clippy::redundant_closure_call)]
                    let checked = (|| {
                        let p0_w6_pointer = if true {
                            (p0_w6) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                        }
                        if true {
                            for address in ::core::iter::once(
                                (p0_w6_pointer.addr()) as usize,
                            ) {
                                p0_w1_accesses.push((address, true));
                            }
                        }
                        Ok::<_, String>(())
                    })();
                    checked?;
                }
                __prebindgen_input_aliases.accept_group(p0_w1_accesses)?;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = p0_w1_rows[p0_w3].v0;
                    #[allow(unused_variables)]
                    let p0_w8 = p0_w1_rows[p0_w3].v1;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::TextInput,
                        String,
                    > {
                        let p0_w6_pointer = if true {
                            (p0_w6) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                        }
                        if true {
                            __prebindgen_input_consumed
                                .set(
                                    __prebindgen_input_consumed.get()
                                        + (::core::iter::once((p0_w6_pointer.addr()) as usize))
                                            .count(),
                                );
                        }
                        let p0_w5 = if true {
                            Some(unsafe { Box::from_raw(p0_w6_pointer) })
                        } else {
                            None
                        };
                        #[allow(unused_mut, unused_assignments)]
                        let mut p0_w7: Option<&str> = None;
                        let p0_w5: c_collection_flat::Resource = *p0_w5
                            .expect("active allocation");
                        p0_w7 = if true {
                            Some(
                                std::str::from_utf8(unsafe { __pb_read(p0_w8)? })
                                    .map_err(|_| "invalid UTF-8")?,
                            )
                        } else {
                            None
                        };
                        let p0_w7: String = ::core::convert::Into::into(
                            p0_w7.expect("active value"),
                        );
                        let p0_w4: c_collection_flat::TextInput = c_collection_flat::text_new(
                            p0_w5,
                            p0_w7,
                        );
                        let p0_w0 = p0_w4;
                        Ok(p0_w0)
                    })();
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true { Some(p0_w0) } else { None };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::take_text(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = bench_take_text_result {
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
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_take_wide_p0_w1 {
    pub v0: *mut c_collection_flat::Resource,
    pub v1: u64,
    pub v2: u64,
    pub v3: u64,
    pub v4: u64,
    pub v5: u64,
    pub v6: u64,
    pub v7: u64,
}
#[repr(C)]
pub struct bench_take_wide_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_wide(p0_w1: __PbRows) -> u64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<bench_take_wide_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::DynamicInputs::default();
            let _: () = {};
            let mut p0_w0: Vec<c_collection_flat::WideInput> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_take_wide_p0_w1,
                    p0_w2,
                )?;
                let p0_w1_rows: &[__PbRow_take_wide_p0_w1] = if p0_w2 == 0 {
                    &[]
                } else {
                    unsafe {
                        ::core::slice::from_raw_parts(
                            p0_w1.data as *const __PbRow_take_wide_p0_w1,
                            p0_w2,
                        )
                    }
                };
                let mut p0_w1_accesses = __prebindgen_input_aliases
                    .begin_group(p0_w2.saturating_mul(1usize));
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = p0_w1_rows[p0_w3].v0;
                    let p0_w8 = p0_w1_rows[p0_w3].v1;
                    let p0_w10 = p0_w1_rows[p0_w3].v2;
                    let p0_w12 = p0_w1_rows[p0_w3].v3;
                    let p0_w14 = p0_w1_rows[p0_w3].v4;
                    let p0_w16 = p0_w1_rows[p0_w3].v5;
                    let p0_w18 = p0_w1_rows[p0_w3].v6;
                    let p0_w20 = p0_w1_rows[p0_w3].v7;
                    #[allow(unused_variables, clippy::redundant_closure_call)]
                    let checked = (|| {
                        let p0_w6_pointer = if true {
                            (p0_w6) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                        }
                        let p0_w7 = if true { Some(p0_w8) } else { None };
                        let p0_w9 = if true { Some(p0_w10) } else { None };
                        let p0_w11 = if true { Some(p0_w12) } else { None };
                        let p0_w13 = if true { Some(p0_w14) } else { None };
                        let p0_w15 = if true { Some(p0_w16) } else { None };
                        let p0_w17 = if true { Some(p0_w18) } else { None };
                        let p0_w19 = if true { Some(p0_w20) } else { None };
                        if true {
                            for address in ::core::iter::once(
                                (p0_w6_pointer.addr()) as usize,
                            ) {
                                p0_w1_accesses.push((address, true));
                            }
                        }
                        Ok::<_, String>(())
                    })();
                    checked?;
                }
                __prebindgen_input_aliases.accept_group(p0_w1_accesses)?;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = p0_w1_rows[p0_w3].v0;
                    let p0_w8 = p0_w1_rows[p0_w3].v1;
                    let p0_w10 = p0_w1_rows[p0_w3].v2;
                    let p0_w12 = p0_w1_rows[p0_w3].v3;
                    let p0_w14 = p0_w1_rows[p0_w3].v4;
                    let p0_w16 = p0_w1_rows[p0_w3].v5;
                    let p0_w18 = p0_w1_rows[p0_w3].v6;
                    let p0_w20 = p0_w1_rows[p0_w3].v7;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::WideInput,
                        String,
                    > {
                        let p0_w6_pointer = if true {
                            (p0_w6) as *mut c_collection_flat::Resource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                        }
                        let p0_w7 = if true { Some(p0_w8) } else { None };
                        let p0_w9 = if true { Some(p0_w10) } else { None };
                        let p0_w11 = if true { Some(p0_w12) } else { None };
                        let p0_w13 = if true { Some(p0_w14) } else { None };
                        let p0_w15 = if true { Some(p0_w16) } else { None };
                        let p0_w17 = if true { Some(p0_w18) } else { None };
                        let p0_w19 = if true { Some(p0_w20) } else { None };
                        if true {
                            __prebindgen_input_consumed
                                .set(
                                    __prebindgen_input_consumed.get()
                                        + (::core::iter::once((p0_w6_pointer.addr()) as usize))
                                            .count(),
                                );
                        }
                        let p0_w5 = if true {
                            Some(unsafe { Box::from_raw(p0_w6_pointer) })
                        } else {
                            None
                        };
                        let p0_w5: c_collection_flat::Resource = *p0_w5
                            .expect("active allocation");
                        let p0_w7: u64 = ::core::convert::Into::into(
                            p0_w7.expect("active value"),
                        );
                        let p0_w9: u64 = ::core::convert::Into::into(
                            p0_w9.expect("active value"),
                        );
                        let p0_w11: u64 = ::core::convert::Into::into(
                            p0_w11.expect("active value"),
                        );
                        let p0_w13: u64 = ::core::convert::Into::into(
                            p0_w13.expect("active value"),
                        );
                        let p0_w15: u64 = ::core::convert::Into::into(
                            p0_w15.expect("active value"),
                        );
                        let p0_w17: u64 = ::core::convert::Into::into(
                            p0_w17.expect("active value"),
                        );
                        let p0_w19: u64 = ::core::convert::Into::into(
                            p0_w19.expect("active value"),
                        );
                        let p0_w4: c_collection_flat::WideInput = c_collection_flat::wide_new(
                            p0_w5,
                            p0_w7,
                            p0_w9,
                            p0_w11,
                            p0_w13,
                            p0_w15,
                            p0_w17,
                            p0_w19,
                        );
                        let p0_w0 = p0_w4;
                        Ok(p0_w0)
                    })();
                    let element = p0_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p0_w0.len();
                        p0_w0.as_mut_ptr().add(initialized).write(element);
                        p0_w0.set_len(initialized + 1);
                    }
                }
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true { Some(p0_w0) } else { None };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = c_collection_flat::take_wide(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = bench_take_wide_result {
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
    ::core::mem::size_of:: < c_collection_flat::BuffersInput > () != 0,
    "flat object `BuffersInput` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::BytesInput > () != 0,
    "flat object `BytesInput` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::Record > () != 0,
    "flat object `Record` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::Resource > () != 0,
    "flat object `Resource` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::TextInput > () != 0,
    "flat object `TextInput` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::WideInput > () != 0,
    "flat object `WideInput` must have nonzero size"
);
