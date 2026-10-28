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
pub unsafe extern "C" fn example_free_error(error: *mut std::ffi::c_char) {
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
pub unsafe extern "C" fn example_free_buffer(b: __PbBuffer) {
    if !b.data.is_null() {
        drop(
            Box::from_raw(std::ptr::slice_from_raw_parts_mut(b.data as *mut u8, b.len)),
        );
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn example_drop_calculator(ptr: *mut example_flat::Calculator) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn example_drop_local_counter(
    ptr: *mut example_flat::LocalCounter,
) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn example_drop_caption(ptr: *mut example_flat::Caption) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn example_drop_foo(ptr: *mut example_flat::Foo) {
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
pub struct example_calculator_new_result {
    pub v0: *mut example_flat::Calculator,
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
pub unsafe extern "C" fn example_calculator_new() -> *mut example_flat::Calculator {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<example_calculator_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let source_value = example_flat::calculator_new();
            let ret_w0: *mut example_flat::Calculator = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = example_calculator_new_result {
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
pub struct example_caption_new_result {
    pub v0: *mut example_flat::Caption,
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
pub unsafe extern "C" fn example_caption_new(
    p0_w1: u64,
    p1_w1: __PbBuffer,
    p2_w1: u8,
) -> *mut example_flat::Caption {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<example_caption_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let p1_w0 = if true {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p1_w1)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
            let p2_w0 = if true { Some(p2_w1 != 0) } else { None };
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {};
            let _: () = {};
            let _: () = {};
            let p0_w0: u64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let __prebindgen_argument_1 = ::core::convert::AsRef::<
                str,
            >::as_ref(p1_w0.as_ref().expect("active value"));
            let p2_w0: bool = ::core::convert::Into::into(p2_w0.expect("active value"));
            let __prebindgen_argument_2 = p2_w0;
            let source_value = example_flat::caption_new(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
                __prebindgen_argument_2,
            );
            let ret_w0: *mut example_flat::Caption = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = example_caption_new_result {
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
pub struct example_foo_get_id_result {
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
pub unsafe extern "C" fn example_foo_get_id(p0_w1: *mut example_flat::Foo) -> u64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<example_foo_get_id_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut example_flat::Foo
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
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w0 = if true {
                Some(unsafe { Box::from_raw(p0_w1_pointer) })
            } else {
                None
            };
            let p0_w0: example_flat::Foo = *p0_w0.expect("active allocation");
            let __prebindgen_argument_0 = p0_w0;
            let source_value = example_flat::foo_get_id(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = example_foo_get_id_result {
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
pub struct example_foo_new_result {
    pub v0: *mut example_flat::Foo,
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
pub unsafe extern "C" fn example_foo_new(p0_w1: u64) -> *mut example_flat::Foo {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<example_foo_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {};
            let p0_w0: u64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = example_flat::foo_new(__prebindgen_argument_0);
            let ret_w0: *mut example_flat::Foo = Box::into_raw(Box::new(source_value));
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = example_foo_new_result {
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
pub struct example_local_counter_label_result {
    pub v0: *mut example_flat::LocalCounter,
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
pub unsafe extern "C" fn example_local_counter_label(
    p0_w1: *mut example_flat::LocalCounter,
    p1_w1: __PbBuffer,
) -> *mut example_flat::LocalCounter {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        example_local_counter_label_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut example_flat::LocalCounter
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p1_w0 = if true {
                Some(
                    std::str::from_utf8(unsafe { __pb_read(p1_w1)? })
                        .map_err(|_| "invalid UTF-8")?,
                )
            } else {
                None
            };
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
            let _: () = {
                _ownership_accepted = true;
            };
            let p0_w0 = if true {
                Some(unsafe { Box::from_raw(p0_w1_pointer) })
            } else {
                None
            };
            let p0_w0: example_flat::LocalCounter = *p0_w0.expect("active allocation");
            let __prebindgen_argument_0 = p0_w0;
            let __prebindgen_argument_1 = ::core::convert::AsRef::<
                str,
            >::as_ref(p1_w0.as_ref().expect("active value"));
            let source_value = example_flat::local_counter_label(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0: *mut example_flat::LocalCounter = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = example_local_counter_label_result {
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
pub struct example_local_counter_new_result {
    pub v0: *mut example_flat::LocalCounter,
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
pub unsafe extern "C" fn example_local_counter_new(
    p0_w1: u64,
) -> *mut example_flat::LocalCounter {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        example_local_counter_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true { Some(p0_w1) } else { None };
            let mut __prebindgen_resources: Vec<(usize, bool)> = Vec::new();
            let _: () = {};
            let p0_w0: u64 = ::core::convert::Into::into(p0_w0.expect("active value"));
            let __prebindgen_argument_0 = p0_w0;
            let source_value = example_flat::local_counter_new(__prebindgen_argument_0);
            let ret_w0: *mut example_flat::LocalCounter = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = example_local_counter_new_result {
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
pub struct example_local_counter_value_result {
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
pub unsafe extern "C" fn example_local_counter_value(
    p0_w1: *const example_flat::LocalCounter,
) -> u64 {
    #[allow(unused_mut)]
    let mut _ownership_accepted = false;
    let mut storage = std::mem::MaybeUninit::<
        example_local_counter_value_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = if true {
                p0_w1 as *mut example_flat::LocalCounter
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
            let p0_w0: &example_flat::LocalCounter = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = example_flat::local_counter_value(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = example_local_counter_value_result {
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
        example_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
const _: () = assert!(
    ::core::mem::size_of:: < example_flat::Calculator > () != 0,
    "flat object `Calculator` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < example_flat::Caption > () != 0,
    "flat object `Caption` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < example_flat::Foo > () != 0,
    "flat object `Foo` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < example_flat::LocalCounter > () != 0,
    "flat object `LocalCounter` must have nonzero size"
);
