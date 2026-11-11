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
fn __pb_owned_owned_vec_c_collection_flat_nativemixed_418032a4ed10ccc3_len(
    value: &Vec<c_collection_flat::NativeMixed>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_c_collection_flat_nativepacket_af8cbe451c6259a8_len(
    value: &Vec<c_collection_flat::NativePacket>,
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
fn __pb_owned_owned_vec_c_collection_flat_taggedbuffersinput_691f0ce1b29b05f6_len(
    value: &Vec<c_collection_flat::TaggedBuffersInput>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_c_collection_flat_taggedinput_02dfc659771ede25_len(
    value: &Vec<c_collection_flat::TaggedInput>,
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
fn __pb_owned_owned_vec_i64_8f793c725f29723e_len(value: &Vec<i64>) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_i64_8f793c725f29723e_view(value: &Vec<i64>) -> &[i64] {
    value.as_slice()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_u64_009441151d595b1a_len(value: &Vec<u64>) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_u64_009441151d595b1a_view(value: &Vec<u64>) -> &[u64] {
    value.as_slice()
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
unsafe fn __pb_read<'a>(b: __PbBuffer) -> &'a [u8] {
    if b.len == 0 { &[] } else { std::slice::from_raw_parts(b.data, b.len) }
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
#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbSliceView_i64 {
    pub data: *const ::core::primitive::i64,
    pub len: usize,
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
pub unsafe extern "C" fn bench_drop_borrowed_numbers(
    ptr: *mut c_collection_flat::BorrowedNumbers,
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
#[allow(non_camel_case_types)]
#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct __PbOwned_native_packet {
    storage: ::core::mem::MaybeUninit<[u8; 56usize]>,
}
const _: () = {
    assert!(
        ::core::mem::size_of:: < __PbOwned_native_packet > () == ::core::mem::size_of:: <
        c_collection_flat::NativePacket > (),
        "opaque owner size differs from its Rust type"
    );
    assert!(
        ::core::mem::align_of:: < __PbOwned_native_packet > () == ::core::mem::align_of::
        < c_collection_flat::NativePacket > (),
        "opaque owner alignment differs from its Rust type"
    );
};
#[allow(non_camel_case_types)]
struct __PbOwnedGuard_native_packet(__PbOwned_native_packet);
impl Drop for __PbOwnedGuard_native_packet {
    fn drop(&mut self) {
        unsafe {
            ::core::ptr::drop_in_place(
                &mut self.0 as *mut __PbOwned_native_packet
                    as *mut c_collection_flat::NativePacket,
            );
        }
    }
}
#[allow(dead_code)]
impl __PbOwned_native_packet {
    fn from_rust(value: c_collection_flat::NativePacket) -> Self {
        let mut storage = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write(
                storage.as_mut_ptr().cast::<c_collection_flat::NativePacket>(),
                value,
            );
            storage.assume_init()
        }
    }
    fn guard(value: c_collection_flat::NativePacket) -> __PbOwnedGuard_native_packet {
        __PbOwnedGuard_native_packet(Self::from_rust(value))
    }
    unsafe fn as_rust(&self) -> &c_collection_flat::NativePacket {
        &*(self as *const Self as *const c_collection_flat::NativePacket)
    }
    unsafe fn as_rust_mut(&mut self) -> &mut c_collection_flat::NativePacket {
        &mut *(self as *mut Self as *mut c_collection_flat::NativePacket)
    }
    unsafe fn into_rust(self) -> c_collection_flat::NativePacket {
        ::core::ptr::read(&self as *const Self as *const c_collection_flat::NativePacket)
    }
    fn empty() -> Self {
        Self::from_rust(
            <c_collection_flat::NativePacket as ::core::default::Default>::default(),
        )
    }
}
/**# Safety
The pointer must be null or point to one live owner of this exact type. The owner is replaced by its empty value; previous views become invalid.*/
#[no_mangle]
pub unsafe extern "C" fn bench_drop_native_packet(value: *mut __PbOwned_native_packet) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !value.is_null() {
            drop(
                ::core::ptr::replace(
                    value as *mut c_collection_flat::NativePacket,
                    <c_collection_flat::NativePacket as ::core::default::Default>::default(),
                ),
            );
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
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
pub struct bench_callback_3832b9cb035dc725 {
    pub run: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, __PbSliceView_i64) -> (),
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_7fc0734290773f1a_callback_return_w1 {
    pub v0: *mut c_collection_flat::Resource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct bench_callback_7fc0734290773f1a {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> __PbRows>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbRows),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct bench_callback_d4c2b6b7f94cdedf_result {
    pub v0: *mut c_collection_flat::Resource,
    pub v1: u64,
    pub v2: *mut c_collection_flat::Resource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct bench_callback_d4c2b6b7f94cdedf {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
        ) -> bench_callback_d4c2b6b7f94cdedf_result,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_f1755815cfab1b60_callback_return_w1 {
    pub v0: *mut c_collection_flat::Resource,
    pub v1: __PbOwned_native_packet,
    pub v2: *mut c_collection_flat::Resource,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct bench_callback_f1755815cfab1b60 {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> __PbRows>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbRows),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_fc8e455d100e4602_callback_return_w1 {
    pub v0: __PbOwned_native_packet,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct bench_callback_fc8e455d100e4602 {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> __PbRows>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbRows),
    >,
}
#[repr(C)]
pub struct bench_borrowed_emit_result {
    pub unused: u8,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_borrowed_emit(
    p0_w1: *const c_collection_flat::BorrowedNumbers,
    p1_callback: bench_callback_3832b9cb035dc725,
    p2_w1: usize,
) {
    let mut storage = std::mem::MaybeUninit::<bench_borrowed_emit_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut c_collection_flat::BorrowedNumbers;
            let p0_w0: &c_collection_flat::BorrowedNumbers = unsafe {
                &(*p0_w1_pointer)
            };
            let __prebindgen_argument_0 = p0_w0;
            let p1_callback_run = if true {
                Some(unsafe { p1_callback.run.unwrap_unchecked() })
            } else {
                None
            };
            let p1_callback_owner = if true {
                let owner = __PbCallbackOwner::new(
                    p1_callback.context,
                    p1_callback.drop,
                );
                {}
                Some(owner)
            } else {
                None
            };
            let p1_callback_run = p1_callback_run.expect("active callback");
            let p1_callback_owner = p1_callback_owner.expect("active callback");
            let p1_callback_value = move |callback_source_0: &[i64]| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0_source = &(callback_source_0);
                    let callback_arg_0_w0: &[::core::primitive::i64] = ::core::convert::AsRef::<
                        [::core::primitive::i64],
                    >::as_ref(callback_arg_0_w0_source);
                    let callback_arg_0_w0 = __PbSliceView_i64 {
                        data: <[::core::primitive::i64]>::as_ptr(callback_arg_0_w0),
                        len: <[::core::primitive::i64]>::len(callback_arg_0_w0),
                    };
                    let callback_argument_0 = callback_arg_0_w0;
                    {}
                    let raw_result = unsafe {
                        p1_callback_run(p1_callback_owner.context(), callback_argument_0)
                    };
                    Ok(())
                });
                if let Err(error) = result {
                    ::prebindgen_runtime::report_callback_error(
                        "bench_callback_3832b9cb035dc725",
                        "(& [i64])->()",
                        &error,
                    );
                }
            };
            let __prebindgen_argument_1 = p1_callback_value;
            let p2_w0: usize = ::core::convert::Into::into(p2_w1);
            let __prebindgen_argument_2 = p2_w0;
            let source_value = c_collection_flat::borrowed_emit(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
                __prebindgen_argument_2,
            );
            let result = bench_borrowed_emit_result {
                unused: 0,
            };
            std::ptr::write(output, result);
            let _: () = {};
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
}
#[repr(C)]
pub struct bench_borrowed_numbers_new_result {
    pub v0: *mut c_collection_flat::BorrowedNumbers,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_borrowed_numbers_new(
    p0_w1: usize,
) -> *mut c_collection_flat::BorrowedNumbers {
    let mut storage = std::mem::MaybeUninit::<
        bench_borrowed_numbers_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: usize = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
            let source_value = c_collection_flat::borrowed_numbers_new(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut c_collection_flat::BorrowedNumbers = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = bench_borrowed_numbers_new_result {
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
pub struct bench_borrowed_values_result {
    pub v0: __PbSliceView_i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_borrowed_values(
    p0_w1: *const c_collection_flat::BorrowedNumbers,
) -> __PbSliceView_i64 {
    let mut storage = std::mem::MaybeUninit::<bench_borrowed_values_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut c_collection_flat::BorrowedNumbers;
            let p0_w0: &c_collection_flat::BorrowedNumbers = unsafe {
                &(*p0_w1_pointer)
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = c_collection_flat::borrowed_values(
                __prebindgen_argument_0,
            );
            let ret_w0_source = &(source_value);
            let ret_w0: &[::core::primitive::i64] = ::core::convert::AsRef::<
                [::core::primitive::i64],
            >::as_ref(ret_w0_source);
            let ret_w0 = __PbSliceView_i64 {
                data: <[::core::primitive::i64]>::as_ptr(ret_w0),
                len: <[::core::primitive::i64]>::len(ret_w0),
            };
            let result = bench_borrowed_values_result {
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
pub struct bench_callback_fixed_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_callback_fixed(
    p0_callback: bench_callback_d4c2b6b7f94cdedf,
) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_callback_fixed_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(unsafe { p0_callback.run.unwrap_unchecked() })
            } else {
                None
            };
            let p0_callback_owner = if true {
                let owner = __PbCallbackOwner::new(
                    p0_callback.context,
                    p0_callback.drop,
                );
                {}
                Some(owner)
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("active callback");
            let p0_callback_owner = p0_callback_owner.expect("active callback");
            let p0_callback_value = move || {
                let result = ::prebindgen_runtime::boundary(|| {
                    let __pb_callback_consumed = ::core::cell::Cell::new(0usize);
                    let __prebindgen_input_consumed = &__pb_callback_consumed;
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context())
                    };
                    let callback_return_w2 = raw_result.v0;
                    let callback_return_w4 = raw_result.v1;
                    let callback_return_w6 = raw_result.v2;
                    let mut callback_return_cleanup = ::prebindgen_runtime::CallbackReturnCleanup::new(||
                    {
                        let mut __pb_return_ordinal = 0usize;
                        {
                            let ordinal = __pb_return_ordinal;
                            __pb_return_ordinal += 1;
                            if ordinal >= __prebindgen_input_consumed.get() {
                                if let Err(error) = ::prebindgen_runtime::boundary(|| {
                                    unsafe {
                                        drop(
                                            Box::from_raw(
                                                callback_return_w2 as *mut c_collection_flat::Resource,
                                            ),
                                        );
                                    }
                                    Ok(())
                                }) {
                                    ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                }
                            }
                        }
                        {
                            let ordinal = __pb_return_ordinal;
                            __pb_return_ordinal += 1;
                            if ordinal >= __prebindgen_input_consumed.get() {
                                if let Err(error) = ::prebindgen_runtime::boundary(|| {
                                    unsafe {
                                        drop(
                                            Box::from_raw(
                                                callback_return_w6 as *mut c_collection_flat::Resource,
                                            ),
                                        );
                                    }
                                    Ok(())
                                }) {
                                    ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                }
                            }
                        }
                        Ok(())
                    });
                    {
                        let callback_outcome: ::prebindgen_runtime::Result<_> = {
                            let callback_return_w2_pointer = callback_return_w2
                                as *mut c_collection_flat::Resource;
                            let callback_return_w1 = unsafe {
                                Box::from_raw(callback_return_w2_pointer)
                            };
                            {
                                __pb_callback_consumed
                                    .set(__pb_callback_consumed.get() + (1usize));
                            }
                            let callback_return_w1: c_collection_flat::Resource = *callback_return_w1;
                            let callback_return_w0_argument_0 = callback_return_w1;
                            let callback_return_w3: u64 = ::core::convert::Into::into(
                                callback_return_w4,
                            );
                            let callback_return_w0_argument_1 = callback_return_w3;
                            let callback_return_w6_pointer = callback_return_w6
                                as *mut c_collection_flat::Resource;
                            let callback_return_w5 = unsafe {
                                Box::from_raw(callback_return_w6_pointer)
                            };
                            {
                                __pb_callback_consumed
                                    .set(__pb_callback_consumed.get() + (1usize));
                            }
                            let callback_return_w5: c_collection_flat::Resource = *callback_return_w5;
                            let callback_return_w0_argument_2 = callback_return_w5;
                            let callback_return_w0: c_collection_flat::Record = c_collection_flat::record_new(
                                callback_return_w0_argument_0,
                                callback_return_w0_argument_1,
                                callback_return_w0_argument_2,
                            );
                            let callback_return_value = callback_return_w0;
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
                            "bench_callback_d4c2b6b7f94cdedf",
                            "()->c_collection_flat :: Record",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = c_collection_flat::callback_fixed(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = bench_callback_fixed_result {
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
pub struct bench_callback_many_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_callback_many(
    p0_callback: bench_callback_7fc0734290773f1a,
) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_callback_many_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(unsafe { p0_callback.run.unwrap_unchecked() })
            } else {
                None
            };
            let p0_callback_release_result = p0_callback.release_result;
            let p0_callback_owner = if true {
                let owner = __PbCallbackOwner::new(
                    p0_callback.context,
                    p0_callback.drop,
                );
                {}
                Some(owner)
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("active callback");
            let p0_callback_owner = p0_callback_owner.expect("active callback");
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
                        let mut __pb_return_ordinal = 0usize;
                        #[allow(clippy::explicit_counter_loop)]
                        for index in 0..callback_return_w1.len {
                            let callback_return_w5 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_7fc0734290773f1a_callback_return_w1)
                                    .add(index)))
                                    .v0
                            };
                            {
                                let ordinal = __pb_return_ordinal;
                                __pb_return_ordinal += 1;
                                if ordinal >= __prebindgen_input_consumed.get() {
                                    if let Err(error) = ::prebindgen_runtime::boundary(|| {
                                        unsafe {
                                            drop(
                                                Box::from_raw(
                                                    callback_return_w5 as *mut c_collection_flat::Resource,
                                                ),
                                            );
                                        }
                                        Ok(())
                                    }) {
                                        ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                    }
                                }
                            }
                        }
                        Ok(())
                    });
                    {
                        let callback_outcome: ::prebindgen_runtime::Result<_> = {
                            let mut callback_return_w0: Vec<
                                c_collection_flat::Resource,
                            > = Vec::new();
                            if true {
                                let callback_return_w2 = callback_return_w1.len;
                                let callback_return_w1_data = callback_return_w1.data
                                    as *const __PbRow_callback_7fc0734290773f1a_callback_return_w1;
                                callback_return_w0.reserve_exact(callback_return_w2);
                                for callback_return_w3 in 0..callback_return_w2 {
                                    let callback_return_w5 = unsafe {
                                        (*callback_return_w1_data.add(callback_return_w3)).v0
                                    };
                                    #[allow(clippy::redundant_closure_call)]
                                    let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                        c_collection_flat::Resource,
                                        String,
                                    > {
                                        {
                                            let callback_return_w5_pointer = (callback_return_w5)
                                                as *mut c_collection_flat::Resource;
                                            let callback_return_w4 = unsafe {
                                                Box::from_raw(callback_return_w5_pointer)
                                            };
                                            {
                                                __pb_callback_consumed
                                                    .set(__pb_callback_consumed.get() + (1usize));
                                            }
                                            let callback_return_w4: c_collection_flat::Resource = *callback_return_w4;
                                            let callback_return_w1_row_value = callback_return_w4;
                                            Ok(callback_return_w1_row_value)
                                        }
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
                            let callback_return_value = callback_return_w0;
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
                            "bench_callback_7fc0734290773f1a",
                            "()->Vec < c_collection_flat :: Resource >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = c_collection_flat::callback_many(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = bench_callback_many_result {
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
pub struct bench_callback_native_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_callback_native(
    p0_callback: bench_callback_fc8e455d100e4602,
) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_callback_native_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(unsafe { p0_callback.run.unwrap_unchecked() })
            } else {
                None
            };
            let p0_callback_release_result = p0_callback.release_result;
            let p0_callback_owner = if true {
                let owner = __PbCallbackOwner::new(
                    p0_callback.context,
                    p0_callback.drop,
                );
                {}
                Some(owner)
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("active callback");
            let p0_callback_owner = p0_callback_owner.expect("active callback");
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
                        let mut __pb_return_ordinal = 0usize;
                        #[allow(clippy::explicit_counter_loop)]
                        for index in 0..callback_return_w1.len {
                            let callback_return_w5 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_fc8e455d100e4602_callback_return_w1)
                                    .add(index)))
                                    .v0
                            };
                            {
                                let ordinal = __pb_return_ordinal;
                                __pb_return_ordinal += 1;
                                if ordinal >= __prebindgen_input_consumed.get() {
                                    if let Err(error) = ::prebindgen_runtime::boundary(|| {
                                        unsafe {
                                            drop(
                                                ::core::ptr::read(
                                                    (::core::ptr::addr_of!(
                                                        (* ((callback_return_w1.data as * const
                                                        __PbRow_callback_fc8e455d100e4602_callback_return_w1)
                                                        .add(index))).v0
                                                    ) as *const __PbOwned_native_packet)
                                                        .cast::<c_collection_flat::NativePacket>(),
                                                ),
                                            );
                                        }
                                        Ok(())
                                    }) {
                                        ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                    }
                                }
                            }
                        }
                        Ok(())
                    });
                    {
                        let callback_outcome: ::prebindgen_runtime::Result<_> = {
                            let mut callback_return_w0: Vec<
                                c_collection_flat::NativePacket,
                            > = Vec::new();
                            if true {
                                let callback_return_w2 = callback_return_w1.len;
                                let callback_return_w1_data = callback_return_w1.data
                                    as *const __PbRow_callback_fc8e455d100e4602_callback_return_w1;
                                callback_return_w0.reserve_exact(callback_return_w2);
                                for callback_return_w3 in 0..callback_return_w2 {
                                    let callback_return_w5 = unsafe {
                                        (*callback_return_w1_data.add(callback_return_w3)).v0
                                    };
                                    #[allow(clippy::redundant_closure_call)]
                                    let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                        c_collection_flat::NativePacket,
                                        String,
                                    > {
                                        {
                                            #[allow(unused_parens)]
                                            let callback_return_w5_storage = (callback_return_w5);
                                            let callback_return_w4 = if true {
                                                let callback_return_w5_pointer = ::core::ptr::addr_of!(
                                                    callback_return_w5_storage
                                                )
                                                    .cast::<c_collection_flat::NativePacket>();
                                                let value = unsafe {
                                                    ::core::ptr::read(callback_return_w5_pointer)
                                                };
                                                {
                                                    __pb_callback_consumed
                                                        .set(__pb_callback_consumed.get() + (1usize));
                                                }
                                                Some(value)
                                            } else {
                                                None
                                            };
                                            let callback_return_w4: c_collection_flat::NativePacket = callback_return_w4
                                                .expect("active inline owner");
                                            let callback_return_w1_row_value = callback_return_w4;
                                            Ok(callback_return_w1_row_value)
                                        }
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
                            let callback_return_value = callback_return_w0;
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
                            "bench_callback_fc8e455d100e4602",
                            "()->Vec < c_collection_flat :: NativePacket >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = c_collection_flat::callback_native(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = bench_callback_native_result {
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
pub struct bench_callback_native_mixed_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_callback_native_mixed(
    p0_callback: bench_callback_f1755815cfab1b60,
) -> u64 {
    let mut storage = std::mem::MaybeUninit::<
        bench_callback_native_mixed_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_callback_run = if true {
                Some(unsafe { p0_callback.run.unwrap_unchecked() })
            } else {
                None
            };
            let p0_callback_release_result = p0_callback.release_result;
            let p0_callback_owner = if true {
                let owner = __PbCallbackOwner::new(
                    p0_callback.context,
                    p0_callback.drop,
                );
                {}
                Some(owner)
            } else {
                None
            };
            let p0_callback_run = p0_callback_run.expect("active callback");
            let p0_callback_owner = p0_callback_owner.expect("active callback");
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
                        let mut __pb_return_ordinal = 0usize;
                        #[allow(clippy::explicit_counter_loop)]
                        for index in 0..callback_return_w1.len {
                            let callback_return_w6 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_f1755815cfab1b60_callback_return_w1)
                                    .add(index)))
                                    .v0
                            };
                            let callback_return_w8 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_f1755815cfab1b60_callback_return_w1)
                                    .add(index)))
                                    .v1
                            };
                            let callback_return_w10 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_f1755815cfab1b60_callback_return_w1)
                                    .add(index)))
                                    .v2
                            };
                            {
                                let ordinal = __pb_return_ordinal;
                                __pb_return_ordinal += 1;
                                if ordinal >= __prebindgen_input_consumed.get() {
                                    if let Err(error) = ::prebindgen_runtime::boundary(|| {
                                        unsafe {
                                            drop(
                                                Box::from_raw(
                                                    callback_return_w6 as *mut c_collection_flat::Resource,
                                                ),
                                            );
                                        }
                                        Ok(())
                                    }) {
                                        ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                    }
                                }
                            }
                            {
                                let ordinal = __pb_return_ordinal;
                                __pb_return_ordinal += 1;
                                if ordinal >= __prebindgen_input_consumed.get() {
                                    if let Err(error) = ::prebindgen_runtime::boundary(|| {
                                        unsafe {
                                            drop(
                                                ::core::ptr::read(
                                                    (::core::ptr::addr_of!(
                                                        (* ((callback_return_w1.data as * const
                                                        __PbRow_callback_f1755815cfab1b60_callback_return_w1)
                                                        .add(index))).v1
                                                    ) as *const __PbOwned_native_packet)
                                                        .cast::<c_collection_flat::NativePacket>(),
                                                ),
                                            );
                                        }
                                        Ok(())
                                    }) {
                                        ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                    }
                                }
                            }
                            {
                                let ordinal = __pb_return_ordinal;
                                __pb_return_ordinal += 1;
                                if ordinal >= __prebindgen_input_consumed.get() {
                                    if let Err(error) = ::prebindgen_runtime::boundary(|| {
                                        unsafe {
                                            drop(
                                                Box::from_raw(
                                                    callback_return_w10 as *mut c_collection_flat::Resource,
                                                ),
                                            );
                                        }
                                        Ok(())
                                    }) {
                                        ::prebindgen_runtime::report_callback_cleanup_error(&error);
                                    }
                                }
                            }
                        }
                        Ok(())
                    });
                    {
                        let callback_outcome: ::prebindgen_runtime::Result<_> = {
                            let mut callback_return_w0: Vec<
                                c_collection_flat::NativeMixed,
                            > = Vec::new();
                            if true {
                                let callback_return_w2 = callback_return_w1.len;
                                let callback_return_w1_data = callback_return_w1.data
                                    as *const __PbRow_callback_f1755815cfab1b60_callback_return_w1;
                                callback_return_w0.reserve_exact(callback_return_w2);
                                for callback_return_w3 in 0..callback_return_w2 {
                                    let callback_return_w6 = unsafe {
                                        (*callback_return_w1_data.add(callback_return_w3)).v0
                                    };
                                    let callback_return_w8 = unsafe {
                                        (*callback_return_w1_data.add(callback_return_w3)).v1
                                    };
                                    let callback_return_w10 = unsafe {
                                        (*callback_return_w1_data.add(callback_return_w3)).v2
                                    };
                                    #[allow(clippy::redundant_closure_call)]
                                    let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                        c_collection_flat::NativeMixed,
                                        String,
                                    > {
                                        {
                                            let callback_return_w6_pointer = (callback_return_w6)
                                                as *mut c_collection_flat::Resource;
                                            let callback_return_w5 = unsafe {
                                                Box::from_raw(callback_return_w6_pointer)
                                            };
                                            {
                                                __pb_callback_consumed
                                                    .set(__pb_callback_consumed.get() + (1usize));
                                            }
                                            let callback_return_w5: c_collection_flat::Resource = *callback_return_w5;
                                            let callback_return_w4_argument_0 = callback_return_w5;
                                            #[allow(unused_parens)]
                                            let callback_return_w8_storage = (callback_return_w8);
                                            let callback_return_w7 = if true {
                                                let callback_return_w8_pointer = ::core::ptr::addr_of!(
                                                    callback_return_w8_storage
                                                )
                                                    .cast::<c_collection_flat::NativePacket>();
                                                let value = unsafe {
                                                    ::core::ptr::read(callback_return_w8_pointer)
                                                };
                                                {
                                                    __pb_callback_consumed
                                                        .set(__pb_callback_consumed.get() + (1usize));
                                                }
                                                Some(value)
                                            } else {
                                                None
                                            };
                                            let callback_return_w7: c_collection_flat::NativePacket = callback_return_w7
                                                .expect("active inline owner");
                                            let callback_return_w4_argument_1 = callback_return_w7;
                                            let callback_return_w10_pointer = (callback_return_w10)
                                                as *mut c_collection_flat::Resource;
                                            let callback_return_w9 = unsafe {
                                                Box::from_raw(callback_return_w10_pointer)
                                            };
                                            {
                                                __pb_callback_consumed
                                                    .set(__pb_callback_consumed.get() + (1usize));
                                            }
                                            let callback_return_w9: c_collection_flat::Resource = *callback_return_w9;
                                            let callback_return_w4_argument_2 = callback_return_w9;
                                            let callback_return_w4: c_collection_flat::NativeMixed = c_collection_flat::native_mixed_new(
                                                callback_return_w4_argument_0,
                                                callback_return_w4_argument_1,
                                                callback_return_w4_argument_2,
                                            );
                                            let callback_return_w1_row_value = callback_return_w4;
                                            Ok(callback_return_w1_row_value)
                                        }
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
                            let callback_return_value = callback_return_w0;
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
                            "bench_callback_f1755815cfab1b60",
                            "()->Vec < c_collection_flat :: NativeMixed >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = c_collection_flat::callback_native_mixed(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = bench_callback_native_mixed_result {
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
pub struct bench_control_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_control(p0_w1: usize) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_control_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: usize = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
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
pub struct bench_fixed_borrow_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_fixed_borrow(
    p0_w1: *const c_collection_flat::Resource,
    p1_w1: *mut c_collection_flat::Resource,
) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_fixed_borrow_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut c_collection_flat::Resource;
            let p0_w0: &c_collection_flat::Resource = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let p1_w1_pointer = p1_w1 as *mut c_collection_flat::Resource;
            let p1_w0: &mut c_collection_flat::Resource = unsafe {
                &mut (*p1_w1_pointer)
            };
            let __prebindgen_argument_1 = p1_w0;
            let source_value = c_collection_flat::fixed_borrow(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = source_value;
            let result = bench_fixed_borrow_result {
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
pub struct bench_fixed_move_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_fixed_move(
    p0_w1: *mut c_collection_flat::Resource,
    p1_w1: *mut c_collection_flat::Resource,
) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_fixed_move_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut c_collection_flat::Resource;
            let p0_w0 = unsafe { Box::from_raw(p0_w1_pointer) };
            {}
            let p0_w0: c_collection_flat::Resource = *p0_w0;
            let __prebindgen_argument_0 = p0_w0;
            let p1_w1_pointer = p1_w1 as *mut c_collection_flat::Resource;
            let p1_w0 = unsafe { Box::from_raw(p1_w1_pointer) };
            {}
            let p1_w0: c_collection_flat::Resource = *p1_w0;
            let __prebindgen_argument_1 = p1_w0;
            let source_value = c_collection_flat::fixed_move(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = source_value;
            let result = bench_fixed_move_result {
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
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_make_handles(p0_w1: usize) -> __PbRows {
    let mut storage = std::mem::MaybeUninit::<bench_make_handles_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: usize = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
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
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_make_many(p0_w1: usize) -> __PbRows {
    let mut storage = std::mem::MaybeUninit::<bench_make_many_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: usize = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
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
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_make_nested(p0_w1: usize) -> __PbRows {
    let mut storage = std::mem::MaybeUninit::<bench_make_nested_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: usize = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
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
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_make_numbers(p0_w1: usize) -> __PbRows {
    let mut storage = std::mem::MaybeUninit::<bench_make_numbers_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: usize = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
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
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_make_records(p0_w1: usize) -> __PbRows {
    let mut storage = std::mem::MaybeUninit::<bench_make_records_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: usize = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
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
pub struct bench_native_packet_new_result {
    pub v0: __PbOwned_native_packet,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_native_packet_new(p0_w1: u64) -> __PbOwned_native_packet {
    let mut storage = std::mem::MaybeUninit::<bench_native_packet_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: u64 = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
            let source_value = c_collection_flat::native_packet_new(
                __prebindgen_argument_0,
            );
            let ret_w0 = __PbOwned_native_packet::guard(source_value);
            let result = bench_native_packet_new_result {
                v0: ret_w0.0,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w0);
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
    clippy::let_and_return,
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
    let mut storage = std::mem::MaybeUninit::<bench_resource_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: u64 = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
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
    clippy::let_and_return,
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
    let mut storage = std::mem::MaybeUninit::<bench_resource_value_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut c_collection_flat::Resource;
            let p0_w0: &c_collection_flat::Resource = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
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
pub struct bench_shared_input_array_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_shared_input_array(p0_w1: __PbSliceView_i64) -> i64 {
    let mut storage = std::mem::MaybeUninit::<bench_shared_input_array_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                let p0_w0_pointer = if p0_w1.len == 0 && p0_w1.data.is_null() {
                    ::core::ptr::NonNull::<::core::primitive::i64>::dangling().as_ptr()
                        as *const ::core::primitive::i64
                } else {
                    p0_w1.data
                };
                let p0_w0_slice = unsafe {
                    ::core::slice::from_raw_parts(p0_w0_pointer, p0_w1.len)
                };
                Some(unsafe {
                    &*(p0_w0_slice.as_ptr() as *const [::core::primitive::i64; 4])
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0.expect("active slice view");
            let source_value = c_collection_flat::shared_input_array(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = bench_shared_input_array_result {
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
pub struct bench_shared_input_inspect_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_shared_input_inspect(p0_w1: __PbSliceView_i64) -> usize {
    let mut storage = std::mem::MaybeUninit::<
        bench_shared_input_inspect_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                let p0_w0_pointer = if p0_w1.len == 0 && p0_w1.data.is_null() {
                    ::core::ptr::NonNull::<::core::primitive::i64>::dangling().as_ptr()
                        as *const ::core::primitive::i64
                } else {
                    p0_w1.data
                };
                let p0_w0_slice = unsafe {
                    ::core::slice::from_raw_parts(p0_w0_pointer, p0_w1.len)
                };
                Some(p0_w0_slice)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0.expect("active slice view");
            let source_value = c_collection_flat::shared_input_inspect(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = bench_shared_input_inspect_result {
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
pub struct bench_shared_input_owned_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_shared_input_owned(p0_w1: __PbBuffer) -> usize {
    let mut storage = std::mem::MaybeUninit::<bench_shared_input_owned_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: String = ::core::convert::Into::into(unsafe {
                std::str::from_utf8_unchecked(__pb_read(p0_w1))
            });
            let __prebindgen_argument_0 = p0_w0;
            let source_value = c_collection_flat::shared_input_owned(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = bench_shared_input_owned_result {
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
pub struct bench_shared_input_sum_result {
    pub v0: i64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_shared_input_sum(p0_w1: __PbSliceView_i64) -> i64 {
    let mut storage = std::mem::MaybeUninit::<bench_shared_input_sum_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if true {
                let p0_w0_pointer = if p0_w1.len == 0 && p0_w1.data.is_null() {
                    ::core::ptr::NonNull::<::core::primitive::i64>::dangling().as_ptr()
                        as *const ::core::primitive::i64
                } else {
                    p0_w1.data
                };
                let p0_w0_slice = unsafe {
                    ::core::slice::from_raw_parts(p0_w0_pointer, p0_w1.len)
                };
                Some(p0_w0_slice)
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0.expect("active slice view");
            let source_value = c_collection_flat::shared_input_sum(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = bench_shared_input_sum_result {
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
pub struct bench_shared_input_text_result {
    pub v0: usize,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_shared_input_text(p0_w1: __PbBuffer) -> usize {
    let mut storage = std::mem::MaybeUninit::<bench_shared_input_text_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            #[allow(unused_mut, unused_assignments)]
            let mut p0_w0 = None;
            p0_w0 = Some(unsafe { std::str::from_utf8_unchecked(__pb_read(p0_w1)) });
            let __prebindgen_argument_0 = ::core::convert::AsRef::<
                str,
            >::as_ref(p0_w0.as_ref().expect("active value"));
            let source_value = c_collection_flat::shared_input_text(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = bench_shared_input_text_result {
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
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_buffers(p0_w1: __PbRows) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_take_buffers_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<c_collection_flat::BuffersInput> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_take_buffers_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    let p0_w8 = unsafe { (*p0_w1_data.add(p0_w3)).v1 };
                    let p0_w10 = unsafe { (*p0_w1_data.add(p0_w3)).v2 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::BuffersInput,
                        String,
                    > {
                        {
                            let p0_w6_pointer = (p0_w6)
                                as *mut c_collection_flat::Resource;
                            let p0_w5 = unsafe { Box::from_raw(p0_w6_pointer) };
                            {}
                            let p0_w5: c_collection_flat::Resource = *p0_w5;
                            let p0_w4_argument_0 = p0_w5;
                            let p0_w7: String = ::core::convert::Into::into(unsafe {
                                std::str::from_utf8_unchecked(__pb_read(p0_w8))
                            });
                            let p0_w4_argument_1 = p0_w7;
                            let p0_w9: Vec<u8> = ::core::convert::Into::into(unsafe {
                                __pb_read(p0_w10)
                            });
                            let p0_w4_argument_2 = p0_w9;
                            let p0_w4: c_collection_flat::BuffersInput = c_collection_flat::buffers_new(
                                p0_w4_argument_0,
                                p0_w4_argument_1,
                                p0_w4_argument_2,
                            );
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
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
            let __prebindgen_argument_0 = p0_w0;
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
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_bytes(p0_w1: __PbRows) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_take_bytes_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<c_collection_flat::BytesInput> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_take_bytes_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    let p0_w8 = unsafe { (*p0_w1_data.add(p0_w3)).v1 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::BytesInput,
                        String,
                    > {
                        {
                            let p0_w6_pointer = (p0_w6)
                                as *mut c_collection_flat::Resource;
                            let p0_w5 = unsafe { Box::from_raw(p0_w6_pointer) };
                            {}
                            let p0_w5: c_collection_flat::Resource = *p0_w5;
                            let p0_w4_argument_0 = p0_w5;
                            let p0_w7: Vec<u8> = ::core::convert::Into::into(unsafe {
                                __pb_read(p0_w8)
                            });
                            let p0_w4_argument_1 = p0_w7;
                            let p0_w4: c_collection_flat::BytesInput = c_collection_flat::bytes_new(
                                p0_w4_argument_0,
                                p0_w4_argument_1,
                            );
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
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
            let __prebindgen_argument_0 = p0_w0;
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
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_handles(p0_w1: __PbRows) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_take_handles_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<c_collection_flat::Resource> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_take_handles_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::Resource,
                        String,
                    > {
                        {
                            let p0_w5_pointer = (p0_w5)
                                as *mut c_collection_flat::Resource;
                            let p0_w4 = unsafe { Box::from_raw(p0_w5_pointer) };
                            {}
                            let p0_w4: c_collection_flat::Resource = *p0_w4;
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
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
            let __prebindgen_argument_0 = p0_w0;
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
pub struct __PbRow_take_native_p0_w1 {
    pub v0: *mut __PbOwned_native_packet,
}
#[repr(C)]
pub struct bench_take_native_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_native(p0_w1: __PbRows) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_take_native_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<c_collection_flat::NativePacket> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_take_native_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::NativePacket,
                        String,
                    > {
                        {
                            let p0_w5_pointer = (p0_w5)
                                as *mut c_collection_flat::NativePacket;
                            let p0_w5_empty = if true {
                                Some(
                                    <c_collection_flat::NativePacket as ::core::default::Default>::default(),
                                )
                            } else {
                                None
                            };
                            let p0_w4 = if true {
                                {
                                    let value = unsafe {
                                        ::core::ptr::replace(
                                            p0_w5_pointer,
                                            p0_w5_empty.expect("prepared empty owner"),
                                        )
                                    };
                                    {}
                                    Some(value)
                                }
                            } else {
                                None
                            };
                            let p0_w4: c_collection_flat::NativePacket = p0_w4
                                .expect("active inline owner");
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
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
            let __prebindgen_argument_0 = p0_w0;
            let source_value = c_collection_flat::take_native(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = bench_take_native_result {
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
pub struct __PbRow_take_native_mixed_p0_w1 {
    pub v0: *mut c_collection_flat::Resource,
    pub v1: *mut __PbOwned_native_packet,
    pub v2: *mut c_collection_flat::Resource,
}
#[repr(C)]
pub struct bench_take_native_mixed_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_native_mixed(p0_w1: __PbRows) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_take_native_mixed_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<c_collection_flat::NativeMixed> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_take_native_mixed_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    let p0_w8 = unsafe { (*p0_w1_data.add(p0_w3)).v1 };
                    let p0_w10 = unsafe { (*p0_w1_data.add(p0_w3)).v2 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::NativeMixed,
                        String,
                    > {
                        {
                            let p0_w6_pointer = (p0_w6)
                                as *mut c_collection_flat::Resource;
                            let p0_w5 = unsafe { Box::from_raw(p0_w6_pointer) };
                            {}
                            let p0_w5: c_collection_flat::Resource = *p0_w5;
                            let p0_w4_argument_0 = p0_w5;
                            let p0_w8_pointer = (p0_w8)
                                as *mut c_collection_flat::NativePacket;
                            let p0_w8_empty = if true {
                                Some(
                                    <c_collection_flat::NativePacket as ::core::default::Default>::default(),
                                )
                            } else {
                                None
                            };
                            let p0_w7 = if true {
                                {
                                    let value = unsafe {
                                        ::core::ptr::replace(
                                            p0_w8_pointer,
                                            p0_w8_empty.expect("prepared empty owner"),
                                        )
                                    };
                                    {}
                                    Some(value)
                                }
                            } else {
                                None
                            };
                            let p0_w7: c_collection_flat::NativePacket = p0_w7
                                .expect("active inline owner");
                            let p0_w4_argument_1 = p0_w7;
                            let p0_w10_pointer = (p0_w10)
                                as *mut c_collection_flat::Resource;
                            let p0_w9 = unsafe { Box::from_raw(p0_w10_pointer) };
                            {}
                            let p0_w9: c_collection_flat::Resource = *p0_w9;
                            let p0_w4_argument_2 = p0_w9;
                            let p0_w4: c_collection_flat::NativeMixed = c_collection_flat::native_mixed_new(
                                p0_w4_argument_0,
                                p0_w4_argument_1,
                                p0_w4_argument_2,
                            );
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
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
            let __prebindgen_argument_0 = p0_w0;
            let source_value = c_collection_flat::take_native_mixed(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = bench_take_native_mixed_result {
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
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_numbers(p0_w1: __PbRows) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_take_numbers_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<u64> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_take_numbers_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<u64, String> {
                        {
                            let p0_w4: u64 = ::core::convert::Into::into(p0_w5);
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
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
            let __prebindgen_argument_0 = p0_w0;
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
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_records(p0_w1: __PbRows) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_take_records_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<c_collection_flat::Record> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_take_records_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    let p0_w8 = unsafe { (*p0_w1_data.add(p0_w3)).v1 };
                    let p0_w10 = unsafe { (*p0_w1_data.add(p0_w3)).v2 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::Record,
                        String,
                    > {
                        {
                            let p0_w6_pointer = (p0_w6)
                                as *mut c_collection_flat::Resource;
                            let p0_w5 = unsafe { Box::from_raw(p0_w6_pointer) };
                            {}
                            let p0_w5: c_collection_flat::Resource = *p0_w5;
                            let p0_w4_argument_0 = p0_w5;
                            let p0_w7: u64 = ::core::convert::Into::into(p0_w8);
                            let p0_w4_argument_1 = p0_w7;
                            let p0_w10_pointer = (p0_w10)
                                as *mut c_collection_flat::Resource;
                            let p0_w9 = unsafe { Box::from_raw(p0_w10_pointer) };
                            {}
                            let p0_w9: c_collection_flat::Resource = *p0_w9;
                            let p0_w4_argument_2 = p0_w9;
                            let p0_w4: c_collection_flat::Record = c_collection_flat::record_new(
                                p0_w4_argument_0,
                                p0_w4_argument_1,
                                p0_w4_argument_2,
                            );
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
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
            let __prebindgen_argument_0 = p0_w0;
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
pub struct __PbRow_take_tagged_p0_w1 {
    pub v0: *mut c_collection_flat::Resource,
    pub v1: i32,
}
#[repr(C)]
pub struct bench_take_tagged_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_tagged(p0_w1: __PbRows) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_take_tagged_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<c_collection_flat::TaggedInput> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_take_tagged_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    let p0_w8 = unsafe { (*p0_w1_data.add(p0_w3)).v1 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::TaggedInput,
                        String,
                    > {
                        {
                            let p0_w6_pointer = (p0_w6)
                                as *mut c_collection_flat::Resource;
                            let p0_w5 = unsafe { Box::from_raw(p0_w6_pointer) };
                            {}
                            let p0_w5: c_collection_flat::Resource = *p0_w5;
                            let p0_w4_argument_0 = p0_w5;
                            let p0_w7 = match p0_w8 {
                                -3i32 => c_collection_flat::InputTag::Low,
                                0i32 => c_collection_flat::InputTag::Normal,
                                17i32 => c_collection_flat::InputTag::High,
                                _ => unsafe { ::core::hint::unreachable_unchecked() }
                            };
                            let p0_w4_argument_1 = p0_w7;
                            let p0_w4: c_collection_flat::TaggedInput = c_collection_flat::tagged_new(
                                p0_w4_argument_0,
                                p0_w4_argument_1,
                            );
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
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
            let __prebindgen_argument_0 = p0_w0;
            let source_value = c_collection_flat::take_tagged(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = bench_take_tagged_result {
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
pub struct __PbRow_take_tagged_buffers_p0_w1 {
    pub v0: *mut c_collection_flat::Resource,
    pub v1: i32,
    pub v2: __PbBuffer,
    pub v3: __PbBuffer,
}
#[repr(C)]
pub struct bench_take_tagged_buffers_result {
    pub v0: u64,
}
/**# Safety
Additional output pointers must be writable and disjoint from live inputs. Input buffers must be readable for their lengths. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations. Opaque pointers must be live allocations of the declared type. Owned inputs transfer to Rust; borrowed inputs obey Rust aliasing rules. Binding errors and Rust panics abort this non-Result call.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_tagged_buffers(p0_w1: __PbRows) -> u64 {
    let mut storage = std::mem::MaybeUninit::<
        bench_take_tagged_buffers_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<c_collection_flat::TaggedBuffersInput> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_take_tagged_buffers_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    let p0_w8 = unsafe { (*p0_w1_data.add(p0_w3)).v1 };
                    let p0_w10 = unsafe { (*p0_w1_data.add(p0_w3)).v2 };
                    let p0_w12 = unsafe { (*p0_w1_data.add(p0_w3)).v3 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::TaggedBuffersInput,
                        String,
                    > {
                        {
                            let p0_w6_pointer = (p0_w6)
                                as *mut c_collection_flat::Resource;
                            let p0_w5 = unsafe { Box::from_raw(p0_w6_pointer) };
                            {}
                            let p0_w5: c_collection_flat::Resource = *p0_w5;
                            let p0_w4_argument_0 = p0_w5;
                            let p0_w7 = match p0_w8 {
                                -3i32 => c_collection_flat::InputTag::Low,
                                0i32 => c_collection_flat::InputTag::Normal,
                                17i32 => c_collection_flat::InputTag::High,
                                _ => unsafe { ::core::hint::unreachable_unchecked() }
                            };
                            let p0_w4_argument_1 = p0_w7;
                            let p0_w9: String = ::core::convert::Into::into(unsafe {
                                std::str::from_utf8_unchecked(__pb_read(p0_w10))
                            });
                            let p0_w4_argument_2 = p0_w9;
                            let p0_w11: Vec<u8> = ::core::convert::Into::into(unsafe {
                                __pb_read(p0_w12)
                            });
                            let p0_w4_argument_3 = p0_w11;
                            let p0_w4: c_collection_flat::TaggedBuffersInput = c_collection_flat::tagged_buffers_new(
                                p0_w4_argument_0,
                                p0_w4_argument_1,
                                p0_w4_argument_2,
                                p0_w4_argument_3,
                            );
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
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
            let __prebindgen_argument_0 = p0_w0;
            let source_value = c_collection_flat::take_tagged_buffers(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = bench_take_tagged_buffers_result {
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
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_text(p0_w1: __PbRows) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_take_text_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<c_collection_flat::TextInput> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_take_text_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    let p0_w8 = unsafe { (*p0_w1_data.add(p0_w3)).v1 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::TextInput,
                        String,
                    > {
                        {
                            let p0_w6_pointer = (p0_w6)
                                as *mut c_collection_flat::Resource;
                            let p0_w5 = unsafe { Box::from_raw(p0_w6_pointer) };
                            {}
                            let p0_w5: c_collection_flat::Resource = *p0_w5;
                            let p0_w4_argument_0 = p0_w5;
                            let p0_w7: String = ::core::convert::Into::into(unsafe {
                                std::str::from_utf8_unchecked(__pb_read(p0_w8))
                            });
                            let p0_w4_argument_1 = p0_w7;
                            let p0_w4: c_collection_flat::TextInput = c_collection_flat::text_new(
                                p0_w4_argument_0,
                                p0_w4_argument_1,
                            );
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
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
            let __prebindgen_argument_0 = p0_w0;
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
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn bench_take_wide(p0_w1: __PbRows) -> u64 {
    let mut storage = std::mem::MaybeUninit::<bench_take_wide_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<c_collection_flat::WideInput> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_take_wide_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    let p0_w8 = unsafe { (*p0_w1_data.add(p0_w3)).v1 };
                    let p0_w10 = unsafe { (*p0_w1_data.add(p0_w3)).v2 };
                    let p0_w12 = unsafe { (*p0_w1_data.add(p0_w3)).v3 };
                    let p0_w14 = unsafe { (*p0_w1_data.add(p0_w3)).v4 };
                    let p0_w16 = unsafe { (*p0_w1_data.add(p0_w3)).v5 };
                    let p0_w18 = unsafe { (*p0_w1_data.add(p0_w3)).v6 };
                    let p0_w20 = unsafe { (*p0_w1_data.add(p0_w3)).v7 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        c_collection_flat::WideInput,
                        String,
                    > {
                        {
                            let p0_w6_pointer = (p0_w6)
                                as *mut c_collection_flat::Resource;
                            let p0_w5 = unsafe { Box::from_raw(p0_w6_pointer) };
                            {}
                            let p0_w5: c_collection_flat::Resource = *p0_w5;
                            let p0_w4_argument_0 = p0_w5;
                            let p0_w7: u64 = ::core::convert::Into::into(p0_w8);
                            let p0_w4_argument_1 = p0_w7;
                            let p0_w9: u64 = ::core::convert::Into::into(p0_w10);
                            let p0_w4_argument_2 = p0_w9;
                            let p0_w11: u64 = ::core::convert::Into::into(p0_w12);
                            let p0_w4_argument_3 = p0_w11;
                            let p0_w13: u64 = ::core::convert::Into::into(p0_w14);
                            let p0_w4_argument_4 = p0_w13;
                            let p0_w15: u64 = ::core::convert::Into::into(p0_w16);
                            let p0_w4_argument_5 = p0_w15;
                            let p0_w17: u64 = ::core::convert::Into::into(p0_w18);
                            let p0_w4_argument_6 = p0_w17;
                            let p0_w19: u64 = ::core::convert::Into::into(p0_w20);
                            let p0_w4_argument_7 = p0_w19;
                            let p0_w4: c_collection_flat::WideInput = c_collection_flat::wide_new(
                                p0_w4_argument_0,
                                p0_w4_argument_1,
                                p0_w4_argument_2,
                                p0_w4_argument_3,
                                p0_w4_argument_4,
                                p0_w4_argument_5,
                                p0_w4_argument_6,
                                p0_w4_argument_7,
                            );
                            let p0_w1_row_value = p0_w4;
                            Ok(p0_w1_row_value)
                        }
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
            let __prebindgen_argument_0 = p0_w0;
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
    ::core::mem::size_of:: < c_collection_flat::BorrowedNumbers > () != 0,
    "flat object `BorrowedNumbers` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::BuffersInput > () != 0,
    "flat object `BuffersInput` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::BytesInput > () != 0,
    "flat object `BytesInput` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::InputTag > () != 0,
    "flat object `InputTag` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::NativeMixed > () != 0,
    "flat object `NativeMixed` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::NativePacket > () != 0,
    "flat object `NativePacket` must have nonzero size"
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
    ::core::mem::size_of:: < c_collection_flat::TaggedBuffersInput > () != 0,
    "flat object `TaggedBuffersInput` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::TaggedInput > () != 0,
    "flat object `TaggedInput` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::TextInput > () != 0,
    "flat object `TextInput` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < c_collection_flat::WideInput > () != 0,
    "flat object `WideInput` must have nonzero size"
);
const _: () = assert!(
    c_collection_flat::InputTag::Low as i128 == - 3i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    c_collection_flat::InputTag::Normal as i128 == 0i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    c_collection_flat::InputTag::High as i128 == 17i32 as i128,
    "enum discriminant differs from captured source"
);
