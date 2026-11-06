#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_string_len(value: &String) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_string_view(value: &String) -> &[u8] {
    value.as_bytes()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_option_perftest_flat_bufferinputrow_99c0a6a07204d20f_len(
    value: &Vec<Option<perftest_flat::BufferInputRow>>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_option_perftest_flat_fixedcleanuprecord_b5cc62725b548e0e_len(
    value: &Vec<Option<perftest_flat::FixedCleanupRecord>>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_vec_perftest_flat_fixedcleanuprecord_017e4e0f3b039b33_len(
    value: &Vec<Vec<perftest_flat::FixedCleanupRecord>>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_vec_perftest_flat_ownedresource_bc62184754e77cad_len(
    value: &Vec<Vec<perftest_flat::OwnedResource>>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_i64_8f793c725f29723e_len(value: &Vec<i64>) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_perftest_flat_bufferinputrow_f62bf6f2ce9e50d2_len(
    value: &Vec<perftest_flat::BufferInputRow>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_perftest_flat_fixedcleanuprecord_9c0a6c041d1b7f27_len(
    value: &Vec<perftest_flat::FixedCleanupRecord>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_perftest_flat_ownedresource_0e74a0b28242dcd5_len(
    value: &Vec<perftest_flat::OwnedResource>,
) -> usize {
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
pub unsafe extern "C" fn fast_free_error(error: *mut std::ffi::c_char) {
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
pub unsafe extern "C" fn fast_free_buffer(b: __PbBuffer) {
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
pub unsafe extern "C" fn fast_drop_owned_resource(
    ptr: *mut perftest_flat::OwnedResource,
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
pub unsafe extern "C" fn fast_drop_alias_handler(ptr: *mut perftest_flat::AliasHandler) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
#[allow(non_camel_case_types)]
#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct __PbOwned_owned_string {
    storage: ::core::mem::MaybeUninit<[u8; 24usize]>,
}
const _: () = {
    assert!(
        ::core::mem::size_of:: < __PbOwned_owned_string > () == ::core::mem::size_of:: <
        String > (), "opaque owner size differs from its Rust type"
    );
    assert!(
        ::core::mem::align_of:: < __PbOwned_owned_string > () == ::core::mem::align_of::
        < String > (), "opaque owner alignment differs from its Rust type"
    );
};
#[allow(non_camel_case_types)]
struct __PbOwnedGuard_owned_string(__PbOwned_owned_string);
impl Drop for __PbOwnedGuard_owned_string {
    fn drop(&mut self) {
        unsafe {
            ::core::ptr::drop_in_place(
                &mut self.0 as *mut __PbOwned_owned_string as *mut String,
            );
        }
    }
}
#[allow(dead_code)]
impl __PbOwned_owned_string {
    fn from_rust(value: String) -> Self {
        let mut storage = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write(storage.as_mut_ptr().cast::<String>(), value);
            storage.assume_init()
        }
    }
    fn guard(value: String) -> __PbOwnedGuard_owned_string {
        __PbOwnedGuard_owned_string(Self::from_rust(value))
    }
    unsafe fn as_rust(&self) -> &String {
        &*(self as *const Self as *const String)
    }
    unsafe fn as_rust_mut(&mut self) -> &mut String {
        &mut *(self as *mut Self as *mut String)
    }
    unsafe fn into_rust(self) -> String {
        ::core::ptr::read(&self as *const Self as *const String)
    }
    fn empty() -> Self {
        Self::from_rust(<String as ::core::default::Default>::default())
    }
}
/**# Safety
The pointer must be null or point to one live owner of this exact type. The owner is replaced by its empty value; previous views become invalid.*/
#[no_mangle]
pub unsafe extern "C" fn fast_drop_owned_string(value: *mut __PbOwned_owned_string) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !value.is_null() {
            drop(
                ::core::ptr::replace(
                    value as *mut String,
                    <String as ::core::default::Default>::default(),
                ),
            );
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
/**# Safety
The owner must be live, aligned, and of this exact type.*/
#[no_mangle]
pub unsafe extern "C" fn fast_owned_string_len(
    value: *const __PbOwned_owned_string,
) -> usize {
    (*value).as_rust().len()
}
/**# Safety
The owner must remain live and unmodified while its returned view is used. Obtaining a mutable view invalidates earlier views, even if no bytes are changed.*/
#[no_mangle]
pub unsafe extern "C" fn fast_owned_string_view(
    value: *const __PbOwned_owned_string,
) -> __PbBuffer {
    let bytes = __pb_owned_owned_string_view((*value).as_rust());
    __PbBuffer {
        data: bytes.as_ptr(),
        len: bytes.len(),
    }
}
struct __PbCallbackResult<T: Copy> {
    value: T,
    context: *mut ::core::ffi::c_void,
    release: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void, *const T)>,
}
impl<T: Copy> Drop for __PbCallbackResult<T> {
    fn drop(&mut self) {
        if let Some(release) = self.release {
            unsafe { release(self.context, &self.value) }
        }
    }
}
struct __PbCallbackOwner {
    context: *mut ::core::ffi::c_void,
    drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
unsafe impl Send for __PbCallbackOwner {}
unsafe impl Sync for __PbCallbackOwner {}
impl __PbCallbackOwner {
    fn new(
        context: *mut ::core::ffi::c_void,
        drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    ) -> Self {
        Self { context, drop }
    }
    fn context(&self) -> *mut ::core::ffi::c_void {
        self.context
    }
}
impl Drop for __PbCallbackOwner {
    fn drop(&mut self) {
        if let Some(drop) = self.drop {
            unsafe { drop(self.context) }
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_0605b38a2f053a56_callback_return_w3 {
    pub v0: *mut perftest_flat::OwnedResource,
    pub v1: u32,
    pub v2: __PbBuffer,
    pub v3: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct fast_callback_0605b38a2f053a56_result {
    pub v0: i32,
    pub v1: __PbRows,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct fast_callback_0605b38a2f053a56 {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
        ) -> fast_callback_0605b38a2f053a56_result,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            *const fast_callback_0605b38a2f053a56_result,
        ),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct fast_callback_396f37c4ec343c74_result {
    pub v0: *mut perftest_flat::OwnedResource,
    pub v1: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct fast_callback_396f37c4ec343c74 {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
        ) -> fast_callback_396f37c4ec343c74_result,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_5b6f807c27356f4b_callback_return_w8 {
    pub v0: *mut perftest_flat::OwnedResource,
    pub v1: u32,
    pub v2: __PbBuffer,
    pub v3: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_5b6f807c27356f4b_callback_return_w4 {
    pub v0: __PbRows,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct fast_callback_5b6f807c27356f4b_result {
    pub v0: *mut perftest_flat::OwnedResource,
    pub v1: __PbRows,
    pub v2: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct fast_callback_5b6f807c27356f4b {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
        ) -> fast_callback_5b6f807c27356f4b_result,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            *const fast_callback_5b6f807c27356f4b_result,
        ),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_8735da2fee137b57_callback_return_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct fast_callback_8735da2fee137b57 {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> __PbRows>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbRows),
    >,
}
#[repr(C)]
pub struct fast_alias_callback_rows_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn fast_alias_callback_rows(
    p0_callback: fast_callback_8735da2fee137b57,
    out0: *mut i64,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<fast_alias_callback_rows_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_callback_value_built = if true {
                let p0_callback_release_result = p0_callback.release_result;
                let p0_callback_owner = if true {
                    {
                        (&ownership_accepted).set((&ownership_accepted).get() + 1);
                        Some(
                            __PbCallbackOwner::new(p0_callback.context, p0_callback.drop),
                        )
                    }
                } else {
                    None
                };
                Some({
                    let p0_callback_run = p0_callback_run
                        .expect("validated active callback");
                    let p0_callback_owner = p0_callback_owner
                        .expect("validated active callback");
                    let p0_callback_value = move || {
                        let result = ::prebindgen_runtime::boundary(|| {
                            let __pb_callback_consumed = ::core::cell::Cell::new(0usize);
                            let __prebindgen_input_consumed = &__pb_callback_consumed;
                            let raw_result = unsafe {
                                p0_callback_run(p0_callback_owner.context())
                            };
                            let callback_result = __PbCallbackResult {
                                value: raw_result,
                                context: p0_callback_owner.context(),
                                release: p0_callback_release_result,
                            };
                            let raw_result = callback_result.value;
                            let callback_return_w1 = raw_result;
                            let mut callback_return_cleanup = ::prebindgen_runtime::CallbackReturnCleanup::new(||
                            {
                                let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                    __prebindgen_input_consumed,
                                );
                                match ::prebindgen_runtime::validate_slice(
                                    callback_return_w1.data
                                        as *const __PbRow_callback_8735da2fee137b57_callback_return_w1,
                                    callback_return_w1.len,
                                ) {
                                    Ok(()) => {
                                        for index in 0..callback_return_w1.len {
                                            let callback_return_w5 = unsafe {
                                                (*((callback_return_w1.data
                                                    as *const __PbRow_callback_8735da2fee137b57_callback_return_w1)
                                                    .add(index)))
                                                    .v0
                                            };
                                            unsafe {
                                                callback_return_owners
                                                    .receive(
                                                        callback_return_w5 as *mut perftest_flat::OwnedResource,
                                                    );
                                            }
                                        }
                                    }
                                    Err(error) => {
                                        ::prebindgen_runtime::report_callback_cleanup_error(&error)
                                    }
                                }
                                Ok(())
                            });
                            {
                                let callback_outcome: ::prebindgen_runtime::Result<_> = {
                                    let mut callback_return_w0: Vec<
                                        perftest_flat::OwnedResource,
                                    > = Vec::new();
                                    if true {
                                        let callback_return_w2 = callback_return_w1.len;
                                        ::prebindgen_runtime::validate_slice(
                                            callback_return_w1.data
                                                as *const __PbRow_callback_8735da2fee137b57_callback_return_w1,
                                            callback_return_w2,
                                        )?;
                                        let callback_return_w1_rows: &[__PbRow_callback_8735da2fee137b57_callback_return_w1] = if callback_return_w2
                                            == 0
                                        {
                                            &[]
                                        } else {
                                            unsafe {
                                                ::core::slice::from_raw_parts(
                                                    callback_return_w1.data
                                                        as *const __PbRow_callback_8735da2fee137b57_callback_return_w1,
                                                    callback_return_w2,
                                                )
                                            }
                                        };
                                        callback_return_w0.reserve_exact(callback_return_w2);
                                        for callback_return_w3 in 0..callback_return_w2 {
                                            let callback_return_w5 = callback_return_w1_rows[callback_return_w3]
                                                .v0;
                                            #[allow(clippy::redundant_closure_call)]
                                            let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                                perftest_flat::OwnedResource,
                                                String,
                                            > {
                                                let callback_return_w5_pointer = if true {
                                                    (callback_return_w5) as *mut perftest_flat::OwnedResource
                                                } else {
                                                    ::core::ptr::null_mut()
                                                };
                                                if true {
                                                    ::prebindgen_runtime::validate_pointer(
                                                        callback_return_w5_pointer,
                                                    )?;
                                                }
                                                if true {
                                                    __prebindgen_input_consumed
                                                        .set(
                                                            __prebindgen_input_consumed.get()
                                                                + (::core::iter::once(
                                                                    (callback_return_w5_pointer.addr()) as usize,
                                                                ))
                                                                    .count(),
                                                        );
                                                }
                                                let callback_return_w4 = if true {
                                                    Some(unsafe { Box::from_raw(callback_return_w5_pointer) })
                                                } else {
                                                    None
                                                };
                                                let callback_return_w4: perftest_flat::OwnedResource = *callback_return_w4
                                                    .expect("active allocation");
                                                let callback_return_w0 = callback_return_w4;
                                                Ok(callback_return_w0)
                                            })();
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
                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                    let callback_return_w0_built = if true {
                                        Some(callback_return_w0)
                                    } else {
                                        None
                                    };
                                    let callback_return_value = callback_return_w0_built
                                        .expect("active input");
                                    Ok(callback_return_value)
                                };
                                let callback_decoded = callback_outcome?;
                                callback_return_cleanup.commit();
                                Ok(callback_decoded)
                            }
                        });
                        result
                            .unwrap_or_else(|error| {
                                ::prebindgen_runtime::report_callback_error(
                                    "fast_callback_8735da2fee137b57",
                                    "()->Vec < perftest_flat :: OwnedResource >",
                                    &error,
                                );
                                panic!("callback failed: {}", error)
                            })
                    };
                    p0_callback_value
                })
            } else {
                None
            };
            let _: () = {};
            let __prebindgen_argument_0 = p0_callback_value_built.expect("active input");
            let source_value = perftest_flat::alias_callback_rows(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = ret_w0;
            let result = fast_alias_callback_rows_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted.get());
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct fast_alias_fixed_result {
    pub v0: i64,
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
pub unsafe extern "C" fn fast_alias_fixed(
    p0_w1: *mut perftest_flat::OwnedResource,
    p1_w1: *const perftest_flat::OwnedResource,
) -> i64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<fast_alias_fixed_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::OwnedResource
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: &mut perftest_flat::OwnedResource = unsafe {
                        &mut (*p0_w1_pointer)
                    };
                    p0_w0
                })
            } else {
                None
            };
            let p1_w1_pointer = if true {
                p1_w1 as *mut perftest_flat::OwnedResource
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p1_w1_pointer)?;
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p1_w0_built = if true {
                Some({
                    let p1_w0: &perftest_flat::OwnedResource = unsafe {
                        &(*p1_w1_pointer)
                    };
                    p1_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let __prebindgen_argument_1 = p1_w0_built.expect("active input");
            let source_value = perftest_flat::alias_fixed(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = source_value;
            let result = fast_alias_fixed_result {
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
pub struct fast_alias_handler_call_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn fast_alias_handler_call(
    p0_w1: *const perftest_flat::AliasHandler,
    out0: *mut i64,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<fast_alias_handler_call_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let __prebindgen_input_consumed = &ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::AliasHandler
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
                    let p0_w0: &perftest_flat::AliasHandler = unsafe {
                        &(*p0_w1_pointer)
                    };
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::alias_handler_call(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = ret_w0;
            let result = fast_alias_handler_call_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct fast_alias_handler_new_result {
    pub v0: *mut perftest_flat::AliasHandler,
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
pub unsafe extern "C" fn fast_alias_handler_new(
    p0_callback: fast_callback_396f37c4ec343c74,
) -> *mut perftest_flat::AliasHandler {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<fast_alias_handler_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_callback_value_built = if true {
                let p0_callback_owner = if true {
                    {
                        (&_ownership_accepted).set((&_ownership_accepted).get() + 1);
                        Some(
                            __PbCallbackOwner::new(p0_callback.context, p0_callback.drop),
                        )
                    }
                } else {
                    None
                };
                Some({
                    let p0_callback_run = p0_callback_run
                        .expect("validated active callback");
                    let p0_callback_owner = p0_callback_owner
                        .expect("validated active callback");
                    let p0_callback_value = move || {
                        let result = ::prebindgen_runtime::boundary(|| {
                            let __pb_callback_consumed = ::core::cell::Cell::new(0usize);
                            let __prebindgen_input_consumed = &__pb_callback_consumed;
                            let raw_result = unsafe {
                                p0_callback_run(p0_callback_owner.context())
                            };
                            let callback_return_w2 = raw_result.v0;
                            let callback_return_w4 = raw_result.v1;
                            let mut callback_return_owners = ::prebindgen_runtime::FixedCallbackReturnAllocations::<
                                2usize,
                            >::tracking(__prebindgen_input_consumed);
                            unsafe {
                                callback_return_owners
                                    .receive(
                                        callback_return_w2 as *mut perftest_flat::OwnedResource,
                                    );
                            }
                            unsafe {
                                callback_return_owners
                                    .receive(
                                        callback_return_w4 as *mut perftest_flat::OwnedResource,
                                    );
                            }
                            callback_return_owners.check()?;
                            {
                                let callback_return_w2_pointer = if true {
                                    callback_return_w2 as *mut perftest_flat::OwnedResource
                                } else {
                                    ::core::ptr::null_mut()
                                };
                                if true {
                                    ::prebindgen_runtime::validate_pointer(
                                        callback_return_w2_pointer,
                                    )?;
                                }
                                #[allow(clippy::let_and_return, clippy::unit_arg)]
                                let callback_return_w1_built = if true {
                                    let callback_return_w1_built_transferred_0 = if true {
                                        1usize
                                    } else {
                                        0usize
                                    };
                                    let callback_return_w1 = if true {
                                        Some(unsafe { Box::from_raw(callback_return_w2_pointer) })
                                    } else {
                                        None
                                    };
                                    __prebindgen_input_consumed
                                        .set(
                                            __prebindgen_input_consumed.get()
                                                + callback_return_w1_built_transferred_0,
                                        );
                                    Some({
                                        let callback_return_w1: perftest_flat::OwnedResource = *callback_return_w1
                                            .expect("active allocation");
                                        callback_return_w1
                                    })
                                } else {
                                    None
                                };
                                let callback_return_w4_pointer = if true {
                                    callback_return_w4 as *mut perftest_flat::OwnedResource
                                } else {
                                    ::core::ptr::null_mut()
                                };
                                if true {
                                    ::prebindgen_runtime::validate_pointer(
                                        callback_return_w4_pointer,
                                    )?;
                                }
                                #[allow(clippy::let_and_return, clippy::unit_arg)]
                                let callback_return_w3_built = if true {
                                    let callback_return_w3_built_transferred_0 = if true {
                                        1usize
                                    } else {
                                        0usize
                                    };
                                    let callback_return_w3 = if true {
                                        Some(unsafe { Box::from_raw(callback_return_w4_pointer) })
                                    } else {
                                        None
                                    };
                                    __prebindgen_input_consumed
                                        .set(
                                            __prebindgen_input_consumed.get()
                                                + callback_return_w3_built_transferred_0,
                                        );
                                    Some({
                                        let callback_return_w3: perftest_flat::OwnedResource = *callback_return_w3
                                            .expect("active allocation");
                                        callback_return_w3
                                    })
                                } else {
                                    None
                                };
                                #[allow(clippy::let_and_return, clippy::unit_arg)]
                                let callback_return_w0_built = if true {
                                    Some({
                                        let callback_return_w0: perftest_flat::AliasPair = perftest_flat::alias_pair_new(
                                            callback_return_w1_built.expect("active input"),
                                            callback_return_w3_built.expect("active input"),
                                        );
                                        callback_return_w0
                                    })
                                } else {
                                    None
                                };
                                let callback_return_value = callback_return_w0_built
                                    .expect("active input");
                                Ok(callback_return_value)
                            }
                        });
                        result
                            .unwrap_or_else(|error| {
                                ::prebindgen_runtime::report_callback_error(
                                    "fast_callback_396f37c4ec343c74",
                                    "()->perftest_flat :: AliasPair",
                                    &error,
                                );
                                panic!("callback failed: {}", error)
                            })
                    };
                    p0_callback_value
                })
            } else {
                None
            };
            let _: () = {};
            let __prebindgen_argument_0 = p0_callback_value_built.expect("active input");
            let source_value = perftest_flat::alias_handler_new(__prebindgen_argument_0);
            let ret_w0: *mut perftest_flat::AliasHandler = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = fast_alias_handler_new_result {
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
#[derive(Clone, Copy)]
pub struct __PbRow_alias_nested_sum_p0_w5 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_alias_nested_sum_p0_w1 {
    pub v0: __PbRows,
}
#[repr(C)]
pub struct fast_alias_nested_sum_result {
    pub v0: i64,
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
pub unsafe extern "C" fn fast_alias_nested_sum(p0_w1: __PbRows) -> i64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<fast_alias_nested_sum_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let _: () = {};
            let mut p0_w0: Vec<Vec<perftest_flat::OwnedResource>> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_alias_nested_sum_p0_w1,
                    p0_w2,
                )?;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_alias_nested_sum_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        Vec<perftest_flat::OwnedResource>,
                        String,
                    > {
                        let mut p0_w4: Vec<perftest_flat::OwnedResource> = Vec::new();
                        if true {
                            let p0_w6 = (p0_w5).len;
                            ::prebindgen_runtime::validate_slice(
                                (p0_w5).data as *const __PbRow_alias_nested_sum_p0_w5,
                                p0_w6,
                            )?;
                            let p0_w5_rows: &[__PbRow_alias_nested_sum_p0_w5] = if p0_w6
                                == 0
                            {
                                &[]
                            } else {
                                unsafe {
                                    ::core::slice::from_raw_parts(
                                        (p0_w5).data as *const __PbRow_alias_nested_sum_p0_w5,
                                        p0_w6,
                                    )
                                }
                            };
                            p0_w4.reserve_exact(p0_w6);
                            for p0_w7 in 0..p0_w6 {
                                let p0_w9 = p0_w5_rows[p0_w7].v0;
                                #[allow(clippy::redundant_closure_call)]
                                let p0_w5_element_built = (|| -> ::core::result::Result<
                                    perftest_flat::OwnedResource,
                                    String,
                                > {
                                    let p0_w9_pointer = if true {
                                        (p0_w9) as *mut perftest_flat::OwnedResource
                                    } else {
                                        ::core::ptr::null_mut()
                                    };
                                    if true {
                                        ::prebindgen_runtime::validate_pointer(p0_w9_pointer)?;
                                    }
                                    if true {
                                        __prebindgen_input_consumed
                                            .set(
                                                __prebindgen_input_consumed.get()
                                                    + (::core::iter::once((p0_w9_pointer.addr()) as usize))
                                                        .count(),
                                            );
                                    }
                                    let p0_w8 = if true {
                                        Some(unsafe { Box::from_raw(p0_w9_pointer) })
                                    } else {
                                        None
                                    };
                                    let p0_w8: perftest_flat::OwnedResource = *p0_w8
                                        .expect("active allocation");
                                    let p0_w4 = p0_w8;
                                    Ok(p0_w4)
                                })();
                                let element = p0_w5_element_built?;
                                #[allow(clippy::zst_offset)]
                                unsafe {
                                    let initialized = p0_w4.len();
                                    p0_w4.as_mut_ptr().add(initialized).write(element);
                                    p0_w4.set_len(initialized + 1);
                                }
                            }
                        }
                        #[allow(clippy::let_and_return, clippy::unit_arg)]
                        let p0_w4_built = if true { Some(p0_w4) } else { None };
                        let p0_w0 = p0_w4_built.expect("active input");
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
            let source_value = perftest_flat::alias_nested_sum(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = fast_alias_nested_sum_result {
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
pub struct __PbRow_buffer_rows_enum_p0_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
    pub v1: i32,
    pub v2: i32,
    pub v3: __PbBuffer,
    pub v4: i32,
    pub v5: __PbBuffer,
}
#[repr(C)]
pub struct fast_buffer_rows_enum_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn fast_buffer_rows_enum(
    p0_w1: __PbRows,
    out0: *mut i64,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<fast_buffer_rows_enum_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let __prebindgen_input_consumed = &ownership_accepted;
            let _: () = {};
            let mut p0_w0: Vec<i64> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_buffer_rows_enum_p0_w1,
                    p0_w2,
                )?;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe {
                        (*((p0_w1.data as *const __PbRow_buffer_rows_enum_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    let p0_w8 = unsafe {
                        (*((p0_w1.data as *const __PbRow_buffer_rows_enum_p0_w1)
                            .add(p0_w3)))
                            .v1
                    };
                    let p0_w10 = unsafe {
                        (*((p0_w1.data as *const __PbRow_buffer_rows_enum_p0_w1)
                            .add(p0_w3)))
                            .v2
                    };
                    let p0_w12 = unsafe {
                        (*((p0_w1.data as *const __PbRow_buffer_rows_enum_p0_w1)
                            .add(p0_w3)))
                            .v3
                    };
                    let p0_w14 = unsafe {
                        (*((p0_w1.data as *const __PbRow_buffer_rows_enum_p0_w1)
                            .add(p0_w3)))
                            .v4
                    };
                    let p0_w16 = unsafe {
                        (*((p0_w1.data as *const __PbRow_buffer_rows_enum_p0_w1)
                            .add(p0_w3)))
                            .v5
                    };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<i64, String> {
                        let p0_w6_pointer = if true {
                            (p0_w6) as *mut perftest_flat::OwnedResource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w6_pointer)?;
                        }
                        let p0_w7 = if true {
                            Some(
                                match p0_w8 {
                                    -2147483648i32 => perftest_flat::SparseCode::Bottom,
                                    -2i32 => perftest_flat::SparseCode::MinusTwo,
                                    -1i32 => perftest_flat::SparseCode::MinusOne,
                                    0i32 => perftest_flat::SparseCode::Zero,
                                    7i32 => perftest_flat::SparseCode::Seven,
                                    8i32 => perftest_flat::SparseCode::Eight,
                                    2147483647i32 => perftest_flat::SparseCode::Top,
                                    _ => return Err("invalid enum discriminant".into()),
                                },
                            )
                        } else {
                            None
                        };
                        if (p0_w10) != 0 && (p0_w10) != 1 {
                            return Err("invalid option presence".into());
                        }
                        if (p0_w14) != 0 && (p0_w14) != 1 {
                            return Err("invalid option presence".into());
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
                        let mut p0_w11: Option<String> = None;
                        #[allow(unused_mut, unused_assignments)]
                        let mut p0_w15: Option<Vec<u8>> = None;
                        let p0_w5: perftest_flat::OwnedResource = *p0_w5
                            .expect("active allocation");
                        let p0_w7 = p0_w7.expect("active enum");
                        let p0_w9 = if (p0_w10) == 1i32 {
                            p0_w11 = if (p0_w10) == 1i32 {
                                Some(
                                    ::core::convert::Into::<
                                        String,
                                    >::into(
                                        std::str::from_utf8(unsafe { __pb_read(p0_w12)? })
                                            .map_err(|_| "invalid UTF-8")?,
                                    ),
                                )
                            } else {
                                None
                            };
                            Some(p0_w11.as_ref().expect("active value"))
                        } else {
                            None
                        };
                        let p0_w13 = if (p0_w14) == 1i32 {
                            p0_w15 = if (p0_w14) == 1i32 {
                                Some(
                                    ::core::convert::Into::<
                                        Vec<u8>,
                                    >::into(unsafe { __pb_read(p0_w16)? }),
                                )
                            } else {
                                None
                            };
                            Some(p0_w15.as_ref().expect("active value"))
                        } else {
                            None
                        };
                        let p0_w4: i64 = perftest_flat::buffer_row_enum(
                            p0_w5,
                            p0_w7,
                            p0_w9,
                            p0_w13,
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
            let source_value = perftest_flat::buffer_rows_enum(__prebindgen_argument_0);
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = ret_w0;
            let result = fast_buffer_rows_enum_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted.get());
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_buffer_rows_optional_p0_w3 {
    pub v0: *mut perftest_flat::OwnedResource,
    pub v1: __PbBuffer,
    pub v2: __PbBuffer,
}
#[repr(C)]
pub struct fast_buffer_rows_optional_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn fast_buffer_rows_optional(
    p0_w1: i32,
    p0_w3: __PbRows,
    out0: *mut i64,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        fast_buffer_rows_optional_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let __prebindgen_input_consumed = &ownership_accepted;
            let _: () = {};
            if p0_w1 != 0 && p0_w1 != 1 {
                return Err("invalid option presence".into());
            }
            let mut p0_w2: Vec<Option<perftest_flat::BufferInputRow>> = Vec::new();
            if p0_w1 == 1i32 {
                let p0_w4 = p0_w3.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w3.data as *const __PbRow_buffer_rows_optional_p0_w3,
                    p0_w4,
                )?;
                let p0_w3_rows: &[__PbRow_buffer_rows_optional_p0_w3] = if p0_w4 == 0 {
                    &[]
                } else {
                    unsafe {
                        ::core::slice::from_raw_parts(
                            p0_w3.data as *const __PbRow_buffer_rows_optional_p0_w3,
                            p0_w4,
                        )
                    }
                };
                p0_w2.reserve_exact(p0_w4);
                for p0_w5 in 0..p0_w4 {
                    let p0_w10 = p0_w3_rows[p0_w5].v0;
                    #[allow(unused_variables)]
                    let p0_w12 = p0_w3_rows[p0_w5].v1;
                    #[allow(unused_variables)]
                    let p0_w14 = p0_w3_rows[p0_w5].v2;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w3_element_built = (|| -> ::core::result::Result<
                        Option<perftest_flat::BufferInputRow>,
                        String,
                    > {
                        let p0_w7: i32 = i32::from(((p0_w10) as usize) != 0);
                        let p0_w10_pointer = if p0_w7 == 1i32 {
                            (p0_w10) as *mut perftest_flat::OwnedResource
                        } else {
                            ::core::ptr::null_mut()
                        };
                        if p0_w7 == 1i32 {
                            ::prebindgen_runtime::validate_pointer(p0_w10_pointer)?;
                        }
                        if p0_w7 == 1i32 {
                            __prebindgen_input_consumed
                                .set(
                                    __prebindgen_input_consumed.get()
                                        + (::core::iter::once((p0_w10_pointer.addr()) as usize))
                                            .count(),
                                );
                        }
                        let p0_w9 = if p0_w7 == 1i32 {
                            Some(unsafe { Box::from_raw(p0_w10_pointer) })
                        } else {
                            None
                        };
                        #[allow(unused_mut, unused_assignments)]
                        let mut p0_w11: Option<&str> = None;
                        #[allow(unused_mut, unused_assignments)]
                        let mut p0_w13: Option<&[u8]> = None;
                        let p0_w6 = if p0_w7 == 1i32 {
                            let p0_w9: perftest_flat::OwnedResource = *p0_w9
                                .expect("active allocation");
                            p0_w11 = if p0_w7 == 1i32 {
                                Some(
                                    std::str::from_utf8(unsafe { __pb_read(p0_w12)? })
                                        .map_err(|_| "invalid UTF-8")?,
                                )
                            } else {
                                None
                            };
                            let p0_w11: String = ::core::convert::Into::into(
                                p0_w11.expect("active value"),
                            );
                            p0_w13 = if p0_w7 == 1i32 {
                                Some(unsafe { __pb_read(p0_w14)? })
                            } else {
                                None
                            };
                            let p0_w13: Vec<u8> = ::core::convert::Into::into(
                                p0_w13.expect("active value"),
                            );
                            let p0_w8: perftest_flat::BufferInputRow = perftest_flat::buffer_row_new(
                                p0_w9,
                                p0_w11,
                                p0_w13,
                            );
                            Some(p0_w8)
                        } else {
                            None
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
            let p0_w2_built = if p0_w1 == 1i32 { Some(p0_w2) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0 = if p0_w1 == 1i32 {
                        Some(p0_w2_built.expect("active input"))
                    } else {
                        None
                    };
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::buffer_rows_optional(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = ret_w0;
            let result = fast_buffer_rows_optional_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted.get());
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_buffer_rows_sum_p0_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
    pub v1: __PbBuffer,
    pub v2: __PbBuffer,
}
#[repr(C)]
pub struct fast_buffer_rows_sum_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn fast_buffer_rows_sum(
    p0_w1: __PbRows,
    out0: *mut i64,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<fast_buffer_rows_sum_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let __prebindgen_input_consumed = &ownership_accepted;
            let _: () = {};
            let mut p0_w0: Vec<perftest_flat::BufferInputRow> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_buffer_rows_sum_p0_w1,
                    p0_w2,
                )?;
                let p0_w1_rows: &[__PbRow_buffer_rows_sum_p0_w1] = if p0_w2 == 0 {
                    &[]
                } else {
                    unsafe {
                        ::core::slice::from_raw_parts(
                            p0_w1.data as *const __PbRow_buffer_rows_sum_p0_w1,
                            p0_w2,
                        )
                    }
                };
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = p0_w1_rows[p0_w3].v0;
                    #[allow(unused_variables)]
                    let p0_w8 = p0_w1_rows[p0_w3].v1;
                    #[allow(unused_variables)]
                    let p0_w10 = p0_w1_rows[p0_w3].v2;
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        perftest_flat::BufferInputRow,
                        String,
                    > {
                        let p0_w6_pointer = if true {
                            (p0_w6) as *mut perftest_flat::OwnedResource
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
                        let p0_w5: perftest_flat::OwnedResource = *p0_w5
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
                        let p0_w4: perftest_flat::BufferInputRow = perftest_flat::buffer_row_new(
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
            let source_value = perftest_flat::buffer_rows_sum(__prebindgen_argument_0);
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = ret_w0;
            let result = fast_buffer_rows_sum_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted.get());
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct fast_cleanup_group_call_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn fast_cleanup_group_call(
    p0_callback: fast_callback_5b6f807c27356f4b,
    out0: *mut i64,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<fast_cleanup_group_call_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_callback_value_built = if true {
                let p0_callback_release_result = p0_callback.release_result;
                let p0_callback_owner = if true {
                    {
                        (&ownership_accepted).set((&ownership_accepted).get() + 1);
                        Some(
                            __PbCallbackOwner::new(p0_callback.context, p0_callback.drop),
                        )
                    }
                } else {
                    None
                };
                Some({
                    let p0_callback_run = p0_callback_run
                        .expect("validated active callback");
                    let p0_callback_owner = p0_callback_owner
                        .expect("validated active callback");
                    let p0_callback_value = move || {
                        let result = ::prebindgen_runtime::boundary(|| {
                            let __pb_callback_consumed = ::core::cell::Cell::new(0usize);
                            let __prebindgen_input_consumed = &__pb_callback_consumed;
                            let raw_result = unsafe {
                                p0_callback_run(p0_callback_owner.context())
                            };
                            let callback_result = __PbCallbackResult {
                                value: raw_result,
                                context: p0_callback_owner.context(),
                                release: p0_callback_release_result,
                            };
                            let raw_result = callback_result.value;
                            let callback_return_w2 = raw_result.v0;
                            let callback_return_w4 = raw_result.v1;
                            let callback_return_w23 = raw_result.v2;
                            let mut callback_return_cleanup = ::prebindgen_runtime::CallbackReturnCleanup::new(||
                            {
                                let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                    __prebindgen_input_consumed,
                                );
                                unsafe {
                                    callback_return_owners
                                        .receive(
                                            callback_return_w2 as *mut perftest_flat::OwnedResource,
                                        );
                                }
                                match ::prebindgen_runtime::validate_slice(
                                    callback_return_w4.data
                                        as *const __PbRow_callback_5b6f807c27356f4b_callback_return_w4,
                                    callback_return_w4.len,
                                ) {
                                    Ok(()) => {
                                        for index in 0..callback_return_w4.len {
                                            let callback_return_w8 = unsafe {
                                                (*((callback_return_w4.data
                                                    as *const __PbRow_callback_5b6f807c27356f4b_callback_return_w4)
                                                    .add(index)))
                                                    .v0
                                            };
                                            match ::prebindgen_runtime::validate_slice(
                                                callback_return_w8.data
                                                    as *const __PbRow_callback_5b6f807c27356f4b_callback_return_w8,
                                                callback_return_w8.len,
                                            ) {
                                                Ok(()) => {
                                                    for index in 0..callback_return_w8.len {
                                                        let callback_return_w13 = unsafe {
                                                            (*((callback_return_w8.data
                                                                as *const __PbRow_callback_5b6f807c27356f4b_callback_return_w8)
                                                                .add(index)))
                                                                .v0
                                                        };
                                                        let callback_return_w15 = unsafe {
                                                            (*((callback_return_w8.data
                                                                as *const __PbRow_callback_5b6f807c27356f4b_callback_return_w8)
                                                                .add(index)))
                                                                .v1
                                                        };
                                                        let callback_return_w17 = unsafe {
                                                            (*((callback_return_w8.data
                                                                as *const __PbRow_callback_5b6f807c27356f4b_callback_return_w8)
                                                                .add(index)))
                                                                .v2
                                                        };
                                                        let callback_return_w21 = unsafe {
                                                            (*((callback_return_w8.data
                                                                as *const __PbRow_callback_5b6f807c27356f4b_callback_return_w8)
                                                                .add(index)))
                                                                .v3
                                                        };
                                                        unsafe {
                                                            callback_return_owners
                                                                .receive(
                                                                    callback_return_w13 as *mut perftest_flat::OwnedResource,
                                                                );
                                                        }
                                                        if (callback_return_w21 as usize) != 0 {
                                                            unsafe {
                                                                callback_return_owners
                                                                    .receive(
                                                                        callback_return_w21 as *mut perftest_flat::OwnedResource,
                                                                    );
                                                            }
                                                        }
                                                    }
                                                }
                                                Err(error) => {
                                                    ::prebindgen_runtime::report_callback_cleanup_error(&error)
                                                }
                                            }
                                        }
                                    }
                                    Err(error) => {
                                        ::prebindgen_runtime::report_callback_cleanup_error(&error)
                                    }
                                }
                                unsafe {
                                    callback_return_owners
                                        .receive(
                                            callback_return_w23 as *mut perftest_flat::OwnedResource,
                                        );
                                }
                                Ok(())
                            });
                            {
                                let callback_outcome: ::prebindgen_runtime::Result<_> = {
                                    let callback_return_w2_pointer = if true {
                                        callback_return_w2 as *mut perftest_flat::OwnedResource
                                    } else {
                                        ::core::ptr::null_mut()
                                    };
                                    if true {
                                        ::prebindgen_runtime::validate_pointer(
                                            callback_return_w2_pointer,
                                        )?;
                                    }
                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                    let callback_return_w1_built = if true {
                                        let callback_return_w1_built_transferred_0 = if true {
                                            1usize
                                        } else {
                                            0usize
                                        };
                                        let callback_return_w1 = if true {
                                            Some(unsafe { Box::from_raw(callback_return_w2_pointer) })
                                        } else {
                                            None
                                        };
                                        __prebindgen_input_consumed
                                            .set(
                                                __prebindgen_input_consumed.get()
                                                    + callback_return_w1_built_transferred_0,
                                            );
                                        Some({
                                            let callback_return_w1: perftest_flat::OwnedResource = *callback_return_w1
                                                .expect("active allocation");
                                            callback_return_w1
                                        })
                                    } else {
                                        None
                                    };
                                    let mut callback_return_w3: Vec<
                                        Vec<perftest_flat::FixedCleanupRecord>,
                                    > = Vec::new();
                                    if true {
                                        let callback_return_w5 = callback_return_w4.len;
                                        ::prebindgen_runtime::validate_slice(
                                            callback_return_w4.data
                                                as *const __PbRow_callback_5b6f807c27356f4b_callback_return_w4,
                                            callback_return_w5,
                                        )?;
                                        callback_return_w3.reserve_exact(callback_return_w5);
                                        for callback_return_w6 in 0..callback_return_w5 {
                                            let callback_return_w8 = unsafe {
                                                (*((callback_return_w4.data
                                                    as *const __PbRow_callback_5b6f807c27356f4b_callback_return_w4)
                                                    .add(callback_return_w6)))
                                                    .v0
                                            };
                                            #[allow(clippy::redundant_closure_call)]
                                            let callback_return_w4_element_built = (|| -> ::core::result::Result<
                                                Vec<perftest_flat::FixedCleanupRecord>,
                                                String,
                                            > {
                                                let mut callback_return_w7: Vec<
                                                    perftest_flat::FixedCleanupRecord,
                                                > = Vec::new();
                                                if true {
                                                    let callback_return_w9 = (callback_return_w8).len;
                                                    ::prebindgen_runtime::validate_slice(
                                                        (callback_return_w8).data
                                                            as *const __PbRow_callback_5b6f807c27356f4b_callback_return_w8,
                                                        callback_return_w9,
                                                    )?;
                                                    let callback_return_w8_rows: &[__PbRow_callback_5b6f807c27356f4b_callback_return_w8] = if callback_return_w9
                                                        == 0
                                                    {
                                                        &[]
                                                    } else {
                                                        unsafe {
                                                            ::core::slice::from_raw_parts(
                                                                (callback_return_w8).data
                                                                    as *const __PbRow_callback_5b6f807c27356f4b_callback_return_w8,
                                                                callback_return_w9,
                                                            )
                                                        }
                                                    };
                                                    callback_return_w7.reserve_exact(callback_return_w9);
                                                    for callback_return_w10 in 0..callback_return_w9 {
                                                        let callback_return_w13 = callback_return_w8_rows[callback_return_w10]
                                                            .v0;
                                                        let callback_return_w15 = callback_return_w8_rows[callback_return_w10]
                                                            .v1;
                                                        #[allow(unused_variables)]
                                                        let callback_return_w17 = callback_return_w8_rows[callback_return_w10]
                                                            .v2;
                                                        let callback_return_w21 = callback_return_w8_rows[callback_return_w10]
                                                            .v3;
                                                        #[allow(clippy::redundant_closure_call)]
                                                        let callback_return_w8_element_built = (|| -> ::core::result::Result<
                                                            perftest_flat::FixedCleanupRecord,
                                                            String,
                                                        > {
                                                            let callback_return_w13_pointer = if true {
                                                                (callback_return_w13) as *mut perftest_flat::OwnedResource
                                                            } else {
                                                                ::core::ptr::null_mut()
                                                            };
                                                            if true {
                                                                ::prebindgen_runtime::validate_pointer(
                                                                    callback_return_w13_pointer,
                                                                )?;
                                                            }
                                                            let callback_return_w14 = if true {
                                                                Some(
                                                                    char::from_u32(callback_return_w15)
                                                                        .ok_or("invalid Unicode scalar")?,
                                                                )
                                                            } else {
                                                                None
                                                            };
                                                            let callback_return_w19: i32 = i32::from(
                                                                ((callback_return_w21) as usize) != 0,
                                                            );
                                                            let callback_return_w21_pointer = if callback_return_w19
                                                                == 1i32
                                                            {
                                                                (callback_return_w21) as *mut perftest_flat::OwnedResource
                                                            } else {
                                                                ::core::ptr::null_mut()
                                                            };
                                                            if callback_return_w19 == 1i32 {
                                                                ::prebindgen_runtime::validate_pointer(
                                                                    callback_return_w21_pointer,
                                                                )?;
                                                            }
                                                            if true {
                                                                __prebindgen_input_consumed
                                                                    .set(
                                                                        __prebindgen_input_consumed.get()
                                                                            + (::core::iter::once(
                                                                                (callback_return_w13_pointer.addr()) as usize,
                                                                            ))
                                                                                .count(),
                                                                    );
                                                            }
                                                            if callback_return_w19 == 1i32 {
                                                                __prebindgen_input_consumed
                                                                    .set(
                                                                        __prebindgen_input_consumed.get()
                                                                            + (::core::iter::once(
                                                                                (callback_return_w21_pointer.addr()) as usize,
                                                                            ))
                                                                                .count(),
                                                                    );
                                                            }
                                                            let callback_return_w12 = if true {
                                                                Some(unsafe { Box::from_raw(callback_return_w13_pointer) })
                                                            } else {
                                                                None
                                                            };
                                                            #[allow(unused_mut, unused_assignments)]
                                                            let mut callback_return_w16: Option<&str> = None;
                                                            let callback_return_w20 = if callback_return_w19 == 1i32 {
                                                                Some(unsafe { Box::from_raw(callback_return_w21_pointer) })
                                                            } else {
                                                                None
                                                            };
                                                            let callback_return_w12: perftest_flat::OwnedResource = *callback_return_w12
                                                                .expect("active allocation");
                                                            let callback_return_w14: char = ::core::convert::Into::into(
                                                                callback_return_w14.expect("active value"),
                                                            );
                                                            callback_return_w16 = if true {
                                                                Some(
                                                                    std::str::from_utf8(unsafe {
                                                                            __pb_read(callback_return_w17)?
                                                                        })
                                                                        .map_err(|_| "invalid UTF-8")?,
                                                                )
                                                            } else {
                                                                None
                                                            };
                                                            let callback_return_w16: String = ::core::convert::Into::into(
                                                                callback_return_w16.expect("active value"),
                                                            );
                                                            let callback_return_w18 = if callback_return_w19 == 1i32 {
                                                                let callback_return_w20: perftest_flat::OwnedResource = *callback_return_w20
                                                                    .expect("active allocation");
                                                                Some(callback_return_w20)
                                                            } else {
                                                                None
                                                            };
                                                            let callback_return_w11: perftest_flat::FixedCleanupRecord = perftest_flat::fixed_cleanup_new(
                                                                callback_return_w12,
                                                                callback_return_w14,
                                                                callback_return_w16,
                                                                callback_return_w18,
                                                            );
                                                            let callback_return_w7 = callback_return_w11;
                                                            Ok(callback_return_w7)
                                                        })();
                                                        let element = callback_return_w8_element_built?;
                                                        #[allow(clippy::zst_offset)]
                                                        unsafe {
                                                            let initialized = callback_return_w7.len();
                                                            callback_return_w7
                                                                .as_mut_ptr()
                                                                .add(initialized)
                                                                .write(element);
                                                            callback_return_w7.set_len(initialized + 1);
                                                        }
                                                    }
                                                }
                                                #[allow(clippy::let_and_return, clippy::unit_arg)]
                                                let callback_return_w7_built = if true {
                                                    Some(callback_return_w7)
                                                } else {
                                                    None
                                                };
                                                let callback_return_w3 = callback_return_w7_built
                                                    .expect("active input");
                                                Ok(callback_return_w3)
                                            })();
                                            let element = callback_return_w4_element_built?;
                                            #[allow(clippy::zst_offset)]
                                            unsafe {
                                                let initialized = callback_return_w3.len();
                                                callback_return_w3
                                                    .as_mut_ptr()
                                                    .add(initialized)
                                                    .write(element);
                                                callback_return_w3.set_len(initialized + 1);
                                            }
                                        }
                                    }
                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                    let callback_return_w3_built = if true {
                                        Some(callback_return_w3)
                                    } else {
                                        None
                                    };
                                    let callback_return_w23_pointer = if true {
                                        callback_return_w23 as *mut perftest_flat::OwnedResource
                                    } else {
                                        ::core::ptr::null_mut()
                                    };
                                    if true {
                                        ::prebindgen_runtime::validate_pointer(
                                            callback_return_w23_pointer,
                                        )?;
                                    }
                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                    let callback_return_w22_built = if true {
                                        let callback_return_w22_built_transferred_0 = if true {
                                            1usize
                                        } else {
                                            0usize
                                        };
                                        let callback_return_w22 = if true {
                                            Some(unsafe { Box::from_raw(callback_return_w23_pointer) })
                                        } else {
                                            None
                                        };
                                        __prebindgen_input_consumed
                                            .set(
                                                __prebindgen_input_consumed.get()
                                                    + callback_return_w22_built_transferred_0,
                                            );
                                        Some({
                                            let callback_return_w22: perftest_flat::OwnedResource = *callback_return_w22
                                                .expect("active allocation");
                                            callback_return_w22
                                        })
                                    } else {
                                        None
                                    };
                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                    let callback_return_w0_built = if true {
                                        Some({
                                            let callback_return_w0: perftest_flat::CleanupGroup = perftest_flat::cleanup_group_new(
                                                callback_return_w1_built.expect("active input"),
                                                callback_return_w3_built.expect("active input"),
                                                callback_return_w22_built.expect("active input"),
                                            );
                                            callback_return_w0
                                        })
                                    } else {
                                        None
                                    };
                                    let callback_return_value = callback_return_w0_built
                                        .expect("active input");
                                    Ok(callback_return_value)
                                };
                                let callback_decoded = callback_outcome?;
                                callback_return_cleanup.commit();
                                Ok(callback_decoded)
                            }
                        });
                        result
                            .unwrap_or_else(|error| {
                                ::prebindgen_runtime::report_callback_error(
                                    "fast_callback_5b6f807c27356f4b",
                                    "()->perftest_flat :: CleanupGroup",
                                    &error,
                                );
                                panic!("callback failed: {}", error)
                            })
                    };
                    p0_callback_value
                })
            } else {
                None
            };
            let _: () = {};
            let __prebindgen_argument_0 = p0_callback_value_built.expect("active input");
            let source_value = perftest_flat::cleanup_group_call(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = ret_w0;
            let result = fast_cleanup_group_call_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted.get());
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct fast_cleanup_optional_call_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn fast_cleanup_optional_call(
    p0_callback: fast_callback_0605b38a2f053a56,
    out0: *mut i64,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        fast_cleanup_optional_call_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if ownership.is_null() {
            return Err("null ownership-output pointer".into());
        }
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let p0_callback_run = if true {
                Some(p0_callback.run.ok_or("null callback function")?)
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_callback_value_built = if true {
                let p0_callback_release_result = p0_callback.release_result;
                let p0_callback_owner = if true {
                    {
                        (&ownership_accepted).set((&ownership_accepted).get() + 1);
                        Some(
                            __PbCallbackOwner::new(p0_callback.context, p0_callback.drop),
                        )
                    }
                } else {
                    None
                };
                Some({
                    let p0_callback_run = p0_callback_run
                        .expect("validated active callback");
                    let p0_callback_owner = p0_callback_owner
                        .expect("validated active callback");
                    let p0_callback_value = move || {
                        let result = ::prebindgen_runtime::boundary(|| {
                            let __pb_callback_consumed = ::core::cell::Cell::new(0usize);
                            let __prebindgen_input_consumed = &__pb_callback_consumed;
                            let raw_result = unsafe {
                                p0_callback_run(p0_callback_owner.context())
                            };
                            let callback_result = __PbCallbackResult {
                                value: raw_result,
                                context: p0_callback_owner.context(),
                                release: p0_callback_release_result,
                            };
                            let raw_result = callback_result.value;
                            let callback_return_w1 = raw_result.v0;
                            let callback_return_w3 = raw_result.v1;
                            let mut callback_return_cleanup = ::prebindgen_runtime::CallbackReturnCleanup::new(||
                            {
                                let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                    __prebindgen_input_consumed,
                                );
                                if callback_return_w1 == 1 {
                                    match ::prebindgen_runtime::validate_slice(
                                        callback_return_w3.data
                                            as *const __PbRow_callback_0605b38a2f053a56_callback_return_w3,
                                        callback_return_w3.len,
                                    ) {
                                        Ok(()) => {
                                            for index in 0..callback_return_w3.len {
                                                let callback_return_w10 = unsafe {
                                                    (*((callback_return_w3.data
                                                        as *const __PbRow_callback_0605b38a2f053a56_callback_return_w3)
                                                        .add(index)))
                                                        .v0
                                                };
                                                let callback_return_w12 = unsafe {
                                                    (*((callback_return_w3.data
                                                        as *const __PbRow_callback_0605b38a2f053a56_callback_return_w3)
                                                        .add(index)))
                                                        .v1
                                                };
                                                let callback_return_w14 = unsafe {
                                                    (*((callback_return_w3.data
                                                        as *const __PbRow_callback_0605b38a2f053a56_callback_return_w3)
                                                        .add(index)))
                                                        .v2
                                                };
                                                let callback_return_w18 = unsafe {
                                                    (*((callback_return_w3.data
                                                        as *const __PbRow_callback_0605b38a2f053a56_callback_return_w3)
                                                        .add(index)))
                                                        .v3
                                                };
                                                if (callback_return_w10 as usize) != 0 {
                                                    unsafe {
                                                        callback_return_owners
                                                            .receive(
                                                                callback_return_w10 as *mut perftest_flat::OwnedResource,
                                                            );
                                                    }
                                                    if (callback_return_w18 as usize) != 0 {
                                                        unsafe {
                                                            callback_return_owners
                                                                .receive(
                                                                    callback_return_w18 as *mut perftest_flat::OwnedResource,
                                                                );
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(error) => {
                                            ::prebindgen_runtime::report_callback_cleanup_error(&error)
                                        }
                                    }
                                }
                                Ok(())
                            });
                            {
                                let callback_outcome: ::prebindgen_runtime::Result<_> = {
                                    if callback_return_w1 != 0 && callback_return_w1 != 1 {
                                        return Err("invalid option presence".into());
                                    }
                                    let mut callback_return_w2: Vec<
                                        Option<perftest_flat::FixedCleanupRecord>,
                                    > = Vec::new();
                                    if callback_return_w1 == 1i32 {
                                        let callback_return_w4 = callback_return_w3.len;
                                        ::prebindgen_runtime::validate_slice(
                                            callback_return_w3.data
                                                as *const __PbRow_callback_0605b38a2f053a56_callback_return_w3,
                                            callback_return_w4,
                                        )?;
                                        let callback_return_w3_rows: &[__PbRow_callback_0605b38a2f053a56_callback_return_w3] = if callback_return_w4
                                            == 0
                                        {
                                            &[]
                                        } else {
                                            unsafe {
                                                ::core::slice::from_raw_parts(
                                                    callback_return_w3.data
                                                        as *const __PbRow_callback_0605b38a2f053a56_callback_return_w3,
                                                    callback_return_w4,
                                                )
                                            }
                                        };
                                        callback_return_w2.reserve_exact(callback_return_w4);
                                        for callback_return_w5 in 0..callback_return_w4 {
                                            let callback_return_w10 = callback_return_w3_rows[callback_return_w5]
                                                .v0;
                                            let callback_return_w12 = callback_return_w3_rows[callback_return_w5]
                                                .v1;
                                            #[allow(unused_variables)]
                                            let callback_return_w14 = callback_return_w3_rows[callback_return_w5]
                                                .v2;
                                            let callback_return_w18 = callback_return_w3_rows[callback_return_w5]
                                                .v3;
                                            #[allow(clippy::redundant_closure_call)]
                                            let callback_return_w3_element_built = (|| -> ::core::result::Result<
                                                Option<perftest_flat::FixedCleanupRecord>,
                                                String,
                                            > {
                                                let callback_return_w7: i32 = i32::from(
                                                    ((callback_return_w10) as usize) != 0,
                                                );
                                                let callback_return_w10_pointer = if callback_return_w7
                                                    == 1i32
                                                {
                                                    (callback_return_w10) as *mut perftest_flat::OwnedResource
                                                } else {
                                                    ::core::ptr::null_mut()
                                                };
                                                if callback_return_w7 == 1i32 {
                                                    ::prebindgen_runtime::validate_pointer(
                                                        callback_return_w10_pointer,
                                                    )?;
                                                }
                                                let callback_return_w11 = if callback_return_w7 == 1i32 {
                                                    Some(
                                                        char::from_u32(callback_return_w12)
                                                            .ok_or("invalid Unicode scalar")?,
                                                    )
                                                } else {
                                                    None
                                                };
                                                let callback_return_w16: i32 = i32::from(
                                                    ((callback_return_w18) as usize) != 0,
                                                );
                                                let callback_return_w18_pointer = if (callback_return_w7
                                                    == 1i32) && (callback_return_w16 == 1i32)
                                                {
                                                    (callback_return_w18) as *mut perftest_flat::OwnedResource
                                                } else {
                                                    ::core::ptr::null_mut()
                                                };
                                                if (callback_return_w7 == 1i32)
                                                    && (callback_return_w16 == 1i32)
                                                {
                                                    ::prebindgen_runtime::validate_pointer(
                                                        callback_return_w18_pointer,
                                                    )?;
                                                }
                                                if callback_return_w7 == 1i32 {
                                                    __prebindgen_input_consumed
                                                        .set(
                                                            __prebindgen_input_consumed.get()
                                                                + (::core::iter::once(
                                                                    (callback_return_w10_pointer.addr()) as usize,
                                                                ))
                                                                    .count(),
                                                        );
                                                }
                                                if (callback_return_w7 == 1i32)
                                                    && (callback_return_w16 == 1i32)
                                                {
                                                    __prebindgen_input_consumed
                                                        .set(
                                                            __prebindgen_input_consumed.get()
                                                                + (::core::iter::once(
                                                                    (callback_return_w18_pointer.addr()) as usize,
                                                                ))
                                                                    .count(),
                                                        );
                                                }
                                                let callback_return_w9 = if callback_return_w7 == 1i32 {
                                                    Some(unsafe { Box::from_raw(callback_return_w10_pointer) })
                                                } else {
                                                    None
                                                };
                                                #[allow(unused_mut, unused_assignments)]
                                                let mut callback_return_w13: Option<&str> = None;
                                                let callback_return_w17 = if (callback_return_w7 == 1i32)
                                                    && (callback_return_w16 == 1i32)
                                                {
                                                    Some(unsafe { Box::from_raw(callback_return_w18_pointer) })
                                                } else {
                                                    None
                                                };
                                                let callback_return_w6 = if callback_return_w7 == 1i32 {
                                                    let callback_return_w9: perftest_flat::OwnedResource = *callback_return_w9
                                                        .expect("active allocation");
                                                    let callback_return_w11: char = ::core::convert::Into::into(
                                                        callback_return_w11.expect("active value"),
                                                    );
                                                    callback_return_w13 = if callback_return_w7 == 1i32 {
                                                        Some(
                                                            std::str::from_utf8(unsafe {
                                                                    __pb_read(callback_return_w14)?
                                                                })
                                                                .map_err(|_| "invalid UTF-8")?,
                                                        )
                                                    } else {
                                                        None
                                                    };
                                                    let callback_return_w13: String = ::core::convert::Into::into(
                                                        callback_return_w13.expect("active value"),
                                                    );
                                                    let callback_return_w15 = if (callback_return_w7 == 1i32)
                                                        && (callback_return_w16 == 1i32)
                                                    {
                                                        let callback_return_w17: perftest_flat::OwnedResource = *callback_return_w17
                                                            .expect("active allocation");
                                                        Some(callback_return_w17)
                                                    } else {
                                                        None
                                                    };
                                                    let callback_return_w8: perftest_flat::FixedCleanupRecord = perftest_flat::fixed_cleanup_new(
                                                        callback_return_w9,
                                                        callback_return_w11,
                                                        callback_return_w13,
                                                        callback_return_w15,
                                                    );
                                                    Some(callback_return_w8)
                                                } else {
                                                    None
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
                                    let callback_return_w2_built = if callback_return_w1 == 1i32
                                    {
                                        Some(callback_return_w2)
                                    } else {
                                        None
                                    };
                                    #[allow(clippy::let_and_return, clippy::unit_arg)]
                                    let callback_return_w0_built = if true {
                                        Some({
                                            let callback_return_w0 = if callback_return_w1 == 1i32 {
                                                Some(callback_return_w2_built.expect("active input"))
                                            } else {
                                                None
                                            };
                                            callback_return_w0
                                        })
                                    } else {
                                        None
                                    };
                                    let callback_return_value = callback_return_w0_built
                                        .expect("active input");
                                    Ok(callback_return_value)
                                };
                                let callback_decoded = callback_outcome?;
                                callback_return_cleanup.commit();
                                Ok(callback_decoded)
                            }
                        });
                        result
                            .unwrap_or_else(|error| {
                                ::prebindgen_runtime::report_callback_error(
                                    "fast_callback_0605b38a2f053a56",
                                    "()->Option < Vec < Option < perftest_flat :: FixedCleanupRecord > > >",
                                    &error,
                                );
                                panic!("callback failed: {}", error)
                            })
                    };
                    p0_callback_value
                })
            } else {
                None
            };
            let _: () = {};
            let __prebindgen_argument_0 = p0_callback_value_built.expect("active input");
            let source_value = perftest_flat::cleanup_optional_call(
                __prebindgen_argument_0,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::result_error_message(ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = ret_w0;
            let result = fast_cleanup_optional_call_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    if !ownership.is_null() {
        std::ptr::write(ownership, ownership_accepted.get());
    }
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_collection_alias_p0_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
pub struct fast_collection_alias_result {
    pub v0: i64,
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
pub unsafe extern "C" fn fast_collection_alias(
    p0_w1: __PbRows,
    p1_w1: *const perftest_flat::OwnedResource,
) -> i64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<fast_collection_alias_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::DynamicInputs::default();
            let _: () = {};
            let mut p0_w0: Vec<perftest_flat::OwnedResource> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_collection_alias_p0_w1,
                    p0_w2,
                )?;
                let p0_w1_rows: &[__PbRow_collection_alias_p0_w1] = if p0_w2 == 0 {
                    &[]
                } else {
                    unsafe {
                        ::core::slice::from_raw_parts(
                            p0_w1.data as *const __PbRow_collection_alias_p0_w1,
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
                            (p0_w5) as *mut perftest_flat::OwnedResource
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
                        perftest_flat::OwnedResource,
                        String,
                    > {
                        let p0_w5_pointer = if true {
                            (p0_w5) as *mut perftest_flat::OwnedResource
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
                        let p0_w4: perftest_flat::OwnedResource = *p0_w4
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
            let p1_w1_pointer = if true {
                p1_w1 as *mut perftest_flat::OwnedResource
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p1_w1_pointer)?;
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p1_w0_built = if true {
                if true {
                    for address in ::core::iter::once((p1_w1_pointer.addr()) as usize) {
                        __prebindgen_input_aliases.access(address, false)?;
                    }
                }
                Some({
                    let p1_w0: &perftest_flat::OwnedResource = unsafe {
                        &(*p1_w1_pointer)
                    };
                    p1_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let __prebindgen_argument_1 = p1_w0_built.expect("active input");
            let source_value = perftest_flat::collection_alias(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = source_value;
            let result = fast_collection_alias_result {
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
pub struct fast_collection_resource_new_result {
    pub v0: *mut perftest_flat::OwnedResource,
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
pub unsafe extern "C" fn fast_collection_resource_new(
    p0_w1: i64,
) -> *mut perftest_flat::OwnedResource {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        fast_collection_resource_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
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
            let source_value = perftest_flat::collection_resource_new(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::OwnedResource = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = fast_collection_resource_new_result {
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
pub struct fast_option_resource_id_result {
    pub v0: i64,
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
pub unsafe extern "C" fn fast_option_resource_id(
    p0_w3: *const perftest_flat::OwnedResource,
) -> i64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<fast_option_resource_id_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let p0_w1: i32 = i32::from((p0_w3 as usize) != 0);
            let p0_w3_pointer = if p0_w1 == 1i32 {
                p0_w3 as *mut perftest_flat::OwnedResource
            } else {
                ::core::ptr::null_mut()
            };
            if p0_w1 == 1i32 {
                ::prebindgen_runtime::validate_pointer(p0_w3_pointer)?;
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w2_built = if p0_w1 == 1i32 {
                Some({
                    let p0_w2: &perftest_flat::OwnedResource = unsafe {
                        &(*p0_w3_pointer)
                    };
                    p0_w2
                })
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0 = if p0_w1 == 1i32 {
                        Some(p0_w2_built.expect("active input"))
                    } else {
                        None
                    };
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::option_resource_id(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = fast_option_resource_id_result {
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
pub struct fast_owned_resource_drops_result {
    pub v0: usize,
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
pub unsafe extern "C" fn fast_owned_resource_drops() -> usize {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        fast_owned_resource_drops_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::owned_resource_drops();
            let ret_w0 = source_value;
            let result = fast_owned_resource_drops_result {
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
        perftest_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::AliasHandler > () != 0,
    "flat object `AliasHandler` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::AliasPair > () != 0,
    "flat object `AliasPair` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::BufferInputRow > () != 0,
    "flat object `BufferInputRow` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::CleanupGroup > () != 0,
    "flat object `CleanupGroup` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::FixedCleanupRecord > () != 0,
    "flat object `FixedCleanupRecord` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::OwnedResource > () != 0,
    "flat object `OwnedResource` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::ResultFailure > () != 0,
    "flat object `ResultFailure` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::SparseCode > () != 0,
    "flat object `SparseCode` must have nonzero size"
);
const _: () = assert!(
    perftest_flat::SparseCode::Bottom as i128 == - 2147483648i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::MinusTwo as i128 == - 2i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::MinusOne as i128 == - 1i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::Zero as i128 == 0i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::Seven as i128 == 7i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::Eight as i128 == 8i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::SparseCode::Top as i128 == 2147483647i32 as i128,
    "enum discriminant differs from captured source"
);
