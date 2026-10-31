#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_text_len(value: &String) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_text_view(value: &String) -> &[u8] {
    value.as_bytes()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_numbers_len(value: &Vec<i64>) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_perftest_flat_inlinemixed_178d49ef699c1dd0_len(
    value: &Vec<perftest_flat::InlineMixed>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_perftest_flat_inlinepacket_d1b07be462875ea1_len(
    value: &Vec<perftest_flat::InlinePacket>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_perftest_flat_inlinereturn_08baa299e0e43a9f_len(
    value: &Vec<perftest_flat::InlineReturn>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_bytes_len(value: &Vec<u8>) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_bytes_view(value: &Vec<u8>) -> &[u8] {
    value.as_slice()
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
pub unsafe extern "C" fn owner_free_error(error: *mut std::ffi::c_char) {
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
pub unsafe extern "C" fn owner_free_buffer(b: __PbBuffer) {
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
pub unsafe extern "C" fn owner_drop_child(ptr: *mut perftest_flat::InlineChild) {
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
pub unsafe extern "C" fn owner_drop_boxed_numbers(ptr: *mut Vec<i32>) {
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
pub struct __PbOwned_packet {
    storage: ::core::mem::MaybeUninit<[u8; 64usize]>,
}
const _: () = {
    assert!(
        ::core::mem::size_of:: < __PbOwned_packet > () == ::core::mem::size_of:: <
        perftest_flat::InlinePacket > (), "opaque owner size differs from its Rust type"
    );
    assert!(
        ::core::mem::align_of:: < __PbOwned_packet > () == ::core::mem::align_of:: <
        perftest_flat::InlinePacket > (),
        "opaque owner alignment differs from its Rust type"
    );
};
#[allow(non_camel_case_types)]
struct __PbOwnedGuard_packet(__PbOwned_packet);
impl Drop for __PbOwnedGuard_packet {
    fn drop(&mut self) {
        unsafe {
            ::core::ptr::drop_in_place(
                &mut self.0 as *mut __PbOwned_packet as *mut perftest_flat::InlinePacket,
            );
        }
    }
}
#[allow(dead_code)]
impl __PbOwned_packet {
    fn from_rust(value: perftest_flat::InlinePacket) -> Self {
        let mut storage = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write(
                storage.as_mut_ptr().cast::<perftest_flat::InlinePacket>(),
                value,
            );
            storage.assume_init()
        }
    }
    fn guard(value: perftest_flat::InlinePacket) -> __PbOwnedGuard_packet {
        __PbOwnedGuard_packet(Self::from_rust(value))
    }
    unsafe fn as_rust(&self) -> &perftest_flat::InlinePacket {
        &*(self as *const Self as *const perftest_flat::InlinePacket)
    }
    unsafe fn as_rust_mut(&mut self) -> &mut perftest_flat::InlinePacket {
        &mut *(self as *mut Self as *mut perftest_flat::InlinePacket)
    }
    unsafe fn into_rust(self) -> perftest_flat::InlinePacket {
        ::core::ptr::read(&self as *const Self as *const perftest_flat::InlinePacket)
    }
    fn empty() -> Self {
        Self::from_rust(perftest_flat::inline_packet_empty())
    }
}
/**# Safety
The pointer must be null or point to one live owner of this exact type. The owner is replaced by its empty value; previous views become invalid.*/
#[no_mangle]
pub unsafe extern "C" fn owner_drop_packet(value: *mut __PbOwned_packet) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !value.is_null() {
            drop(
                ::core::ptr::replace(
                    value as *mut perftest_flat::InlinePacket,
                    perftest_flat::inline_packet_empty(),
                ),
            );
        }
        Ok(())
    }) {
        panic!("C destructor failed: {}", error);
    }
}
#[allow(non_camel_case_types)]
#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct __PbOwned_text {
    storage: ::core::mem::MaybeUninit<[u8; 24usize]>,
}
const _: () = {
    assert!(
        ::core::mem::size_of:: < __PbOwned_text > () == ::core::mem::size_of:: < String >
        (), "opaque owner size differs from its Rust type"
    );
    assert!(
        ::core::mem::align_of:: < __PbOwned_text > () == ::core::mem::align_of:: < String
        > (), "opaque owner alignment differs from its Rust type"
    );
};
#[allow(non_camel_case_types)]
struct __PbOwnedGuard_text(__PbOwned_text);
impl Drop for __PbOwnedGuard_text {
    fn drop(&mut self) {
        unsafe {
            ::core::ptr::drop_in_place(
                &mut self.0 as *mut __PbOwned_text as *mut String,
            );
        }
    }
}
#[allow(dead_code)]
impl __PbOwned_text {
    fn from_rust(value: String) -> Self {
        let mut storage = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write(storage.as_mut_ptr().cast::<String>(), value);
            storage.assume_init()
        }
    }
    fn guard(value: String) -> __PbOwnedGuard_text {
        __PbOwnedGuard_text(Self::from_rust(value))
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
pub unsafe extern "C" fn owner_drop_text(value: *mut __PbOwned_text) {
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
pub unsafe extern "C" fn owner_text_len(value: *const __PbOwned_text) -> usize {
    (*value).as_rust().len()
}
/**# Safety
The owner must remain live and unmodified while its returned view is used.*/
#[no_mangle]
pub unsafe extern "C" fn owner_text_view(value: *const __PbOwned_text) -> __PbBuffer {
    let bytes = __pb_owned_text_view((*value).as_rust());
    __PbBuffer {
        data: bytes.as_ptr(),
        len: bytes.len(),
    }
}
#[allow(non_camel_case_types)]
#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct __PbOwned_numbers {
    storage: ::core::mem::MaybeUninit<[u8; 24usize]>,
}
const _: () = {
    assert!(
        ::core::mem::size_of:: < __PbOwned_numbers > () == ::core::mem::size_of:: < Vec <
        i64 > > (), "opaque owner size differs from its Rust type"
    );
    assert!(
        ::core::mem::align_of:: < __PbOwned_numbers > () == ::core::mem::align_of:: < Vec
        < i64 > > (), "opaque owner alignment differs from its Rust type"
    );
};
#[allow(non_camel_case_types)]
struct __PbOwnedGuard_numbers(__PbOwned_numbers);
impl Drop for __PbOwnedGuard_numbers {
    fn drop(&mut self) {
        unsafe {
            ::core::ptr::drop_in_place(
                &mut self.0 as *mut __PbOwned_numbers as *mut Vec<i64>,
            );
        }
    }
}
#[allow(dead_code)]
impl __PbOwned_numbers {
    fn from_rust(value: Vec<i64>) -> Self {
        let mut storage = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write(storage.as_mut_ptr().cast::<Vec<i64>>(), value);
            storage.assume_init()
        }
    }
    fn guard(value: Vec<i64>) -> __PbOwnedGuard_numbers {
        __PbOwnedGuard_numbers(Self::from_rust(value))
    }
    unsafe fn as_rust(&self) -> &Vec<i64> {
        &*(self as *const Self as *const Vec<i64>)
    }
    unsafe fn as_rust_mut(&mut self) -> &mut Vec<i64> {
        &mut *(self as *mut Self as *mut Vec<i64>)
    }
    unsafe fn into_rust(self) -> Vec<i64> {
        ::core::ptr::read(&self as *const Self as *const Vec<i64>)
    }
    fn empty() -> Self {
        Self::from_rust(<Vec<i64> as ::core::default::Default>::default())
    }
}
/**# Safety
The pointer must be null or point to one live owner of this exact type. The owner is replaced by its empty value; previous views become invalid.*/
#[no_mangle]
pub unsafe extern "C" fn owner_drop_numbers(value: *mut __PbOwned_numbers) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !value.is_null() {
            drop(
                ::core::ptr::replace(
                    value as *mut Vec<i64>,
                    <Vec<i64> as ::core::default::Default>::default(),
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
pub unsafe extern "C" fn owner_numbers_len(value: *const __PbOwned_numbers) -> usize {
    (*value).as_rust().len()
}
/**# Safety
The owner must remain live and unmodified while an element pointer is used.*/
#[no_mangle]
pub unsafe extern "C" fn owner_numbers_get(
    value: *const __PbOwned_numbers,
    index: usize,
) -> *const i64 {
    (*value)
        .as_rust()
        .get(index)
        .map_or(::core::ptr::null(), |element| element as *const i64)
}
#[allow(non_camel_case_types)]
#[repr(C, align(8))]
#[derive(Clone, Copy)]
pub struct __PbOwned_bytes {
    storage: ::core::mem::MaybeUninit<[u8; 24usize]>,
}
const _: () = {
    assert!(
        ::core::mem::size_of:: < __PbOwned_bytes > () == ::core::mem::size_of:: < Vec <
        u8 > > (), "opaque owner size differs from its Rust type"
    );
    assert!(
        ::core::mem::align_of:: < __PbOwned_bytes > () == ::core::mem::align_of:: < Vec <
        u8 > > (), "opaque owner alignment differs from its Rust type"
    );
};
#[allow(non_camel_case_types)]
struct __PbOwnedGuard_bytes(__PbOwned_bytes);
impl Drop for __PbOwnedGuard_bytes {
    fn drop(&mut self) {
        unsafe {
            ::core::ptr::drop_in_place(
                &mut self.0 as *mut __PbOwned_bytes as *mut Vec<u8>,
            );
        }
    }
}
#[allow(dead_code)]
impl __PbOwned_bytes {
    fn from_rust(value: Vec<u8>) -> Self {
        let mut storage = ::core::mem::MaybeUninit::<Self>::uninit();
        unsafe {
            ::core::ptr::write(storage.as_mut_ptr().cast::<Vec<u8>>(), value);
            storage.assume_init()
        }
    }
    fn guard(value: Vec<u8>) -> __PbOwnedGuard_bytes {
        __PbOwnedGuard_bytes(Self::from_rust(value))
    }
    unsafe fn as_rust(&self) -> &Vec<u8> {
        &*(self as *const Self as *const Vec<u8>)
    }
    unsafe fn as_rust_mut(&mut self) -> &mut Vec<u8> {
        &mut *(self as *mut Self as *mut Vec<u8>)
    }
    unsafe fn into_rust(self) -> Vec<u8> {
        ::core::ptr::read(&self as *const Self as *const Vec<u8>)
    }
    fn empty() -> Self {
        Self::from_rust(<Vec<u8> as ::core::default::Default>::default())
    }
}
/**# Safety
The pointer must be null or point to one live owner of this exact type. The owner is replaced by its empty value; previous views become invalid.*/
#[no_mangle]
pub unsafe extern "C" fn owner_drop_bytes(value: *mut __PbOwned_bytes) {
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if !value.is_null() {
            drop(
                ::core::ptr::replace(
                    value as *mut Vec<u8>,
                    <Vec<u8> as ::core::default::Default>::default(),
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
pub unsafe extern "C" fn owner_bytes_len(value: *const __PbOwned_bytes) -> usize {
    (*value).as_rust().len()
}
/**# Safety
The owner must remain live and unmodified while its returned view is used.*/
#[no_mangle]
pub unsafe extern "C" fn owner_bytes_view(value: *const __PbOwned_bytes) -> __PbBuffer {
    let bytes = __pb_owned_bytes_view((*value).as_rust());
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
pub struct __PbRow_callback_1782e472ad837f0b_callback_return_w1 {
    pub v0: __PbOwned_packet,
    pub v1: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_1782e472ad837f0b {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> __PbRows>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbRows),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_7a0d2e4eb3c7fbb1 {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            __PbOwned_packet,
        ) -> __PbOwned_packet,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_d1e1f55a20f3880c_callback_return_w1 {
    pub v0: __PbOwned_packet,
    pub v1: *mut perftest_flat::InlineChild,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_d1e1f55a20f3880c {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> __PbRows>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbRows),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_df547e71faee3c03 {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            __PbOwned_bytes,
        ) -> __PbOwned_bytes,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_e857880e5782d2cb_result {
    pub v0: __PbOwned_packet,
    pub v1: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_e857880e5782d2cb {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
        ) -> owner_callback_e857880e5782d2cb_result,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
pub struct owner_inline_boxed_numbers_len_result {
    pub v0: usize,
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
pub unsafe extern "C" fn owner_inline_boxed_numbers_len(
    p0_w1: *const Vec<i32>,
) -> usize {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_boxed_numbers_len_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let p0_w1_pointer = if true {
                p0_w1 as *mut Vec<i32>
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
                    let p0_w0: &Vec<i32> = unsafe { &(*p0_w1_pointer) };
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::inline_boxed_numbers_len(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_boxed_numbers_len_result {
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
pub struct owner_inline_boxed_numbers_new_result {
    pub v0: *mut Vec<i32>,
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
pub unsafe extern "C" fn owner_inline_boxed_numbers_new() -> *mut Vec<i32> {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_boxed_numbers_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::inline_boxed_numbers_new();
            let ret_w0: *mut Vec<i32> = Box::into_raw(Box::new(source_value));
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = owner_inline_boxed_numbers_new_result {
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
pub struct owner_inline_boxed_numbers_take_result {
    pub v0: i32,
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
pub unsafe extern "C" fn owner_inline_boxed_numbers_take(p0_w1: *mut Vec<i32>) -> i32 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_boxed_numbers_take_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let _: () = {};
            let p0_w1_pointer = if true {
                p0_w1 as *mut Vec<i32>
            } else {
                ::core::ptr::null_mut()
            };
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                let mut p0_w0_built_transferred_0 = 0usize;
                if true {
                    for address in ::core::iter::once((p0_w1_pointer.addr()) as usize) {
                        __prebindgen_input_aliases.access(address, true)?;
                        p0_w0_built_transferred_0 += 1;
                    }
                }
                let p0_w0 = if true {
                    Some(unsafe { Box::from_raw(p0_w1_pointer) })
                } else {
                    None
                };
                __prebindgen_input_consumed
                    .set(__prebindgen_input_consumed.get() + p0_w0_built_transferred_0);
                Some({
                    let p0_w0: Vec<i32> = *p0_w0.expect("active allocation");
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::inline_boxed_numbers_take(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_boxed_numbers_take_result {
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
pub struct owner_inline_bytes_result {
    pub v0: __PbOwned_bytes,
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
pub unsafe extern "C" fn owner_inline_bytes() -> __PbOwned_bytes {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_bytes_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::inline_bytes();
            let ret_w0 = __PbOwned_bytes::guard(source_value);
            let result = owner_inline_bytes_result {
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
pub struct owner_inline_bytes_alias_result {
    pub v0: usize,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_inline_bytes_alias(
    p0_w1: *mut __PbOwned_bytes,
    p1_w1: *const __PbOwned_bytes,
    out0: *mut usize,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<owner_inline_bytes_alias_result>::uninit();
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
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::FixedInputs::<
                2usize,
            >::default();
            let _: () = {};
            let p0_w1_pointer = p0_w1 as *mut Vec<u8>;
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p0_w1_empty = if true {
                Some(<Vec<u8> as ::core::default::Default>::default())
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                let mut p0_w0_built_transferred_0 = 0usize;
                if true {
                    for address in ::core::iter::once((p0_w1_pointer.addr()) as usize) {
                        __prebindgen_input_aliases.access(address, true)?;
                        p0_w0_built_transferred_0 += 1;
                    }
                }
                let p0_w0 = if true {
                    Some(unsafe {
                        ::core::ptr::replace(
                            p0_w1_pointer,
                            p0_w1_empty.expect("prepared empty owner"),
                        )
                    })
                } else {
                    None
                };
                __prebindgen_input_consumed
                    .set(__prebindgen_input_consumed.get() + p0_w0_built_transferred_0);
                Some({
                    let p0_w0: Vec<u8> = p0_w0.expect("active inline owner");
                    p0_w0
                })
            } else {
                None
            };
            let p1_w1_pointer = p1_w1 as *mut Vec<u8>;
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
                    let p1_w0: &Vec<u8> = unsafe { &*(p1_w1_pointer) };
                    p1_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let __prebindgen_argument_1 = p1_w0_built.expect("active input");
            let source_value = perftest_flat::inline_bytes_alias(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::inline_packet_error(&ret_w1),
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
            let result = owner_inline_bytes_alias_result {
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
pub struct owner_inline_bytes_callback_result {
    pub v0: usize,
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
pub unsafe extern "C" fn owner_inline_bytes_callback(
    p0_callback: owner_callback_df547e71faee3c03,
) -> usize {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_bytes_callback_result,
    >::uninit();
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
                    let p0_callback_value = move |callback_source_0: Vec<u8>| {
                        let result = ::prebindgen_runtime::boundary(|| {
                            let callback_arg_0_w0 = __PbOwned_bytes::guard(
                                callback_source_0,
                            );
                            let __pb_callback_consumed = ::core::cell::Cell::new(0usize);
                            let __prebindgen_input_consumed = &__pb_callback_consumed;
                            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
                            let callback_argument_0 = callback_arg_0_w0.0;
                            {
                                ::core::mem::forget(callback_arg_0_w0);
                            }
                            let raw_result = unsafe {
                                p0_callback_run(
                                    p0_callback_owner.context(),
                                    callback_argument_0,
                                )
                            };
                            let callback_return_w1 = raw_result;
                            let _callback_return_owner = ::prebindgen_runtime::TransferredOwner::new(
                                __PbOwnedGuard_bytes(callback_return_w1),
                                __prebindgen_input_consumed,
                                0,
                            );
                            #[allow(unused_parens)]
                            let callback_return_w1_storage = callback_return_w1;
                            #[allow(clippy::let_and_return, clippy::unit_arg)]
                            let callback_return_w0_built = if true {
                                let callback_return_w0 = if true {
                                    let callback_return_w1_pointer = ::core::ptr::addr_of!(
                                        callback_return_w1_storage
                                    )
                                        .cast::<Vec<u8>>();
                                    let value = unsafe {
                                        ::core::ptr::read(callback_return_w1_pointer)
                                    };
                                    __prebindgen_input_consumed
                                        .set(__prebindgen_input_consumed.get() + 1);
                                    Some(value)
                                } else {
                                    None
                                };
                                Some({
                                    let callback_return_w0: Vec<u8> = callback_return_w0
                                        .expect("active inline owner");
                                    callback_return_w0
                                })
                            } else {
                                None
                            };
                            let callback_return_value = callback_return_w0_built
                                .expect("active input");
                            Ok(callback_return_value)
                        });
                        result
                            .unwrap_or_else(|error| {
                                ::prebindgen_runtime::report_callback_error(
                                    "owner_callback_df547e71faee3c03",
                                    "(Vec < u8 >)->Vec < u8 >",
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
            let source_value = perftest_flat::inline_bytes_callback(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_bytes_callback_result {
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
pub struct owner_inline_bytes_capacity_result {
    pub v0: usize,
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
pub unsafe extern "C" fn owner_inline_bytes_capacity(
    p0_w1: *const __PbOwned_bytes,
) -> usize {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_bytes_capacity_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let p0_w1_pointer = p0_w1 as *mut Vec<u8>;
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
                    let p0_w0: &Vec<u8> = unsafe { &*(p0_w1_pointer) };
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::inline_bytes_capacity(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_bytes_capacity_result {
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
pub struct owner_inline_bytes_mutable_result {
    pub v0: __PbOwned_bytes,
    pub v1: usize,
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
pub unsafe extern "C" fn owner_inline_bytes_mutable(
    out1: *mut usize,
) -> __PbOwned_bytes {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_bytes_mutable_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let source_value = perftest_flat::inline_bytes_mutable();
            let mut ret_w0 = __PbOwned_bytes::guard(source_value);
            let ret_w1 = perftest_flat::inline_bytes_clear(unsafe {
                &mut *(::core::ptr::addr_of_mut!(ret_w0.0).cast::<Vec<u8>>())
            });
            let ret_w2 = ret_w1;
            let result = owner_inline_bytes_mutable_result {
                v0: ret_w0.0,
                v1: ret_w2,
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
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct owner_inline_bytes_new_result {
    pub v0: __PbOwned_bytes,
    pub v1: __PbBuffer,
    pub v2: usize,
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
pub unsafe extern "C" fn owner_inline_bytes_new(
    out1: *mut __PbBuffer,
    out2: *mut usize,
) -> __PbOwned_bytes {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_bytes_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        if out2.is_null() {
            return Err("null success output".into());
        }
        {
            let source_value = perftest_flat::inline_bytes_new();
            let mut ret_w0 = __PbOwned_bytes::guard(source_value);
            let ret_w1 = __pb_owned_bytes_view(unsafe {
                &*(::core::ptr::addr_of_mut!(ret_w0.0).cast::<Vec<u8>>())
            });
            let ret_w2 = ret_w1;
            let ret_w2 = __PbBuffer {
                data: ret_w2.as_ptr().cast::<u8>(),
                len: ret_w2.len(),
            };
            let ret_w3 = __pb_owned_bytes_len(unsafe {
                &*(::core::ptr::addr_of_mut!(ret_w0.0).cast::<Vec<u8>>())
            });
            let ret_w4 = ret_w3;
            let result = owner_inline_bytes_new_result {
                v0: ret_w0.0,
                v1: ret_w2,
                v2: ret_w4,
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
    std::ptr::write(out1, value.v1);
    std::ptr::write(out2, value.v2);
    value.v0
}
#[repr(C)]
pub struct owner_inline_bytes_take_result {
    pub v0: usize,
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
pub unsafe extern "C" fn owner_inline_bytes_take(p0_w1: *mut __PbOwned_bytes) -> usize {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_bytes_take_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let _: () = {};
            let p0_w1_pointer = p0_w1 as *mut Vec<u8>;
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p0_w1_empty = if true {
                Some(<Vec<u8> as ::core::default::Default>::default())
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                let mut p0_w0_built_transferred_0 = 0usize;
                if true {
                    for address in ::core::iter::once((p0_w1_pointer.addr()) as usize) {
                        __prebindgen_input_aliases.access(address, true)?;
                        p0_w0_built_transferred_0 += 1;
                    }
                }
                let p0_w0 = if true {
                    Some(unsafe {
                        ::core::ptr::replace(
                            p0_w1_pointer,
                            p0_w1_empty.expect("prepared empty owner"),
                        )
                    })
                } else {
                    None
                };
                __prebindgen_input_consumed
                    .set(__prebindgen_input_consumed.get() + p0_w0_built_transferred_0);
                Some({
                    let p0_w0: Vec<u8> = p0_w0.expect("active inline owner");
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::inline_bytes_take(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = owner_inline_bytes_take_result {
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
pub struct owner_inline_child_id_result {
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
pub unsafe extern "C" fn owner_inline_child_id(
    p0_w1: *const perftest_flat::InlineChild,
) -> u64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_child_id_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let p0_w1_pointer = if true {
                p0_w1 as *mut perftest_flat::InlineChild
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
                    let p0_w0: &perftest_flat::InlineChild = unsafe {
                        &(*p0_w1_pointer)
                    };
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::inline_child_id(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = owner_inline_child_id_result {
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
pub struct owner_inline_mixed_rows_sum_result {
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
pub unsafe extern "C" fn owner_inline_mixed_rows_sum(
    p0_callback: owner_callback_d1e1f55a20f3880c,
) -> u64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_mixed_rows_sum_result,
    >::uninit();
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
                let p0_callback_release_result = p0_callback.release_result;
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
                            let mut __prebindgen_input_aliases = ::prebindgen_runtime::DynamicInputs::default();
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
                            let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                __prebindgen_input_consumed,
                            );
                            ::prebindgen_runtime::validate_slice(
                                callback_return_w1.data
                                    as *const __PbRow_callback_d1e1f55a20f3880c_callback_return_w1,
                                callback_return_w1.len,
                            )?;
                            for index in 0..callback_return_w1.len {
                                let callback_return_w6 = unsafe {
                                    (*((callback_return_w1.data
                                        as *const __PbRow_callback_d1e1f55a20f3880c_callback_return_w1)
                                        .add(index)))
                                        .v0
                                };
                                let callback_return_w8 = unsafe {
                                    (*((callback_return_w1.data
                                        as *const __PbRow_callback_d1e1f55a20f3880c_callback_return_w1)
                                        .add(index)))
                                        .v1
                                };
                                unsafe {
                                    callback_return_owners
                                        .receive_inline(
                                            (::core::ptr::addr_of!(
                                                (* ((callback_return_w1.data as * const
                                                __PbRow_callback_d1e1f55a20f3880c_callback_return_w1)
                                                .add(index))).v0
                                            ) as *const __PbOwned_packet)
                                                .cast::<perftest_flat::InlinePacket>(),
                                        );
                                }
                                unsafe {
                                    callback_return_owners
                                        .receive(
                                            callback_return_w8 as *mut perftest_flat::InlineChild,
                                        );
                                }
                            }
                            callback_return_owners.check()?;
                            {
                                let mut callback_return_w0: Vec<
                                    perftest_flat::InlineMixed,
                                > = Vec::new();
                                if true {
                                    let callback_return_w2 = callback_return_w1.len;
                                    ::prebindgen_runtime::validate_slice(
                                        callback_return_w1.data
                                            as *const __PbRow_callback_d1e1f55a20f3880c_callback_return_w1,
                                        callback_return_w2,
                                    )?;
                                    let mut callback_return_w1_accesses = __prebindgen_input_aliases
                                        .begin_group(callback_return_w2.saturating_mul(2usize));
                                    let mut callback_return_w1_prepared = Vec::with_capacity(
                                        callback_return_w2,
                                    );
                                    for callback_return_w3 in 0..callback_return_w2 {
                                        let callback_return_w6 = unsafe {
                                            (*((callback_return_w1.data
                                                as *const __PbRow_callback_d1e1f55a20f3880c_callback_return_w1)
                                                .add(callback_return_w3)))
                                                .v0
                                        };
                                        let callback_return_w8 = unsafe {
                                            (*((callback_return_w1.data
                                                as *const __PbRow_callback_d1e1f55a20f3880c_callback_return_w1)
                                                .add(callback_return_w3)))
                                                .v1
                                        };
                                        #[allow(clippy::redundant_closure_call)]
                                        let prepared = (|| {
                                            #[allow(unused_parens)]
                                            let callback_return_w6_storage = (callback_return_w6);
                                            let callback_return_w8_pointer = if true {
                                                (callback_return_w8) as *mut perftest_flat::InlineChild
                                            } else {
                                                ::core::ptr::null_mut()
                                            };
                                            if true {
                                                ::prebindgen_runtime::validate_pointer(
                                                    callback_return_w8_pointer,
                                                )?;
                                            }
                                            if true {
                                                for address in ::core::iter::once(
                                                    (callback_return_w8_pointer.addr()) as usize,
                                                ) {
                                                    callback_return_w1_accesses.push((address, true));
                                                }
                                            }
                                            Ok::<
                                                _,
                                                String,
                                            >(move || -> ::core::result::Result<
                                                perftest_flat::InlineMixed,
                                                String,
                                            > {
                                                if true {
                                                    __prebindgen_input_consumed
                                                        .set(
                                                            __prebindgen_input_consumed.get()
                                                                + (::core::iter::once(
                                                                    (callback_return_w8_pointer.addr()) as usize,
                                                                ))
                                                                    .count(),
                                                        );
                                                }
                                                let callback_return_w5 = if true {
                                                    let callback_return_w6_pointer = ::core::ptr::addr_of!(
                                                        callback_return_w6_storage
                                                    )
                                                        .cast::<perftest_flat::InlinePacket>();
                                                    let value = unsafe {
                                                        ::core::ptr::read(callback_return_w6_pointer)
                                                    };
                                                    __prebindgen_input_consumed
                                                        .set(__prebindgen_input_consumed.get() + 1);
                                                    Some(value)
                                                } else {
                                                    None
                                                };
                                                let callback_return_w7 = if true {
                                                    Some(unsafe { Box::from_raw(callback_return_w8_pointer) })
                                                } else {
                                                    None
                                                };
                                                let callback_return_w5: perftest_flat::InlinePacket = callback_return_w5
                                                    .expect("active inline owner");
                                                let callback_return_w7: perftest_flat::InlineChild = *callback_return_w7
                                                    .expect("active allocation");
                                                let callback_return_w4: perftest_flat::InlineMixed = perftest_flat::inline_mixed_new(
                                                    callback_return_w5,
                                                    callback_return_w7,
                                                );
                                                let callback_return_w0 = callback_return_w4;
                                                Ok(callback_return_w0)
                                            })
                                        })();
                                        callback_return_w1_prepared.push(prepared?);
                                    }
                                    __prebindgen_input_aliases
                                        .accept_group(callback_return_w1_accesses)?;
                                    callback_return_w0.reserve_exact(callback_return_w2);
                                    for build in callback_return_w1_prepared {
                                        let element = build()?;
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
                            }
                        });
                        result
                            .unwrap_or_else(|error| {
                                ::prebindgen_runtime::report_callback_error(
                                    "owner_callback_d1e1f55a20f3880c",
                                    "()->Vec < perftest_flat :: InlineMixed >",
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
            let source_value = perftest_flat::inline_mixed_rows_sum(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_mixed_rows_sum_result {
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
pub struct owner_inline_numbers_result {
    pub v0: __PbOwned_numbers,
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
pub unsafe extern "C" fn owner_inline_numbers() -> __PbOwned_numbers {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_numbers_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::inline_numbers();
            let ret_w0 = __PbOwned_numbers::guard(source_value);
            let result = owner_inline_numbers_result {
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
pub struct owner_inline_numbers_capacity_result {
    pub v0: usize,
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
pub unsafe extern "C" fn owner_inline_numbers_capacity(
    p0_w1: *const __PbOwned_numbers,
) -> usize {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_numbers_capacity_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let p0_w1_pointer = p0_w1 as *mut Vec<i64>;
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
                    let p0_w0: &Vec<i64> = unsafe { &*(p0_w1_pointer) };
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::inline_numbers_capacity(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_numbers_capacity_result {
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
pub struct owner_inline_numbers_new_result {
    pub v0: __PbOwned_numbers,
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
pub unsafe extern "C" fn owner_inline_numbers_new() -> __PbOwned_numbers {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_numbers_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::inline_numbers_new();
            let ret_w0 = __PbOwned_numbers::guard(source_value);
            let result = owner_inline_numbers_new_result {
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
pub struct owner_inline_numbers_take_result {
    pub v0: i64,
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
pub unsafe extern "C" fn owner_inline_numbers_take(
    p0_w1: *mut __PbOwned_numbers,
) -> i64 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_numbers_take_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let _: () = {};
            let p0_w1_pointer = p0_w1 as *mut Vec<i64>;
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p0_w1_empty = if true {
                Some(<Vec<i64> as ::core::default::Default>::default())
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                let mut p0_w0_built_transferred_0 = 0usize;
                if true {
                    for address in ::core::iter::once((p0_w1_pointer.addr()) as usize) {
                        __prebindgen_input_aliases.access(address, true)?;
                        p0_w0_built_transferred_0 += 1;
                    }
                }
                let p0_w0 = if true {
                    Some(unsafe {
                        ::core::ptr::replace(
                            p0_w1_pointer,
                            p0_w1_empty.expect("prepared empty owner"),
                        )
                    })
                } else {
                    None
                };
                __prebindgen_input_consumed
                    .set(__prebindgen_input_consumed.get() + p0_w0_built_transferred_0);
                Some({
                    let p0_w0: Vec<i64> = p0_w0.expect("active inline owner");
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::inline_numbers_take(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_numbers_take_result {
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
pub struct owner_inline_packet_callback_result {
    pub v0: usize,
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
pub unsafe extern "C" fn owner_inline_packet_callback(
    p0_callback: owner_callback_7a0d2e4eb3c7fbb1,
) -> usize {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packet_callback_result,
    >::uninit();
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
                    let p0_callback_value = move |
                        callback_source_0: perftest_flat::InlinePacket|
                    {
                        let result = ::prebindgen_runtime::boundary(|| {
                            let callback_arg_0_w0 = __PbOwned_packet::guard(
                                callback_source_0,
                            );
                            let __pb_callback_consumed = ::core::cell::Cell::new(0usize);
                            let __prebindgen_input_consumed = &__pb_callback_consumed;
                            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
                            let callback_argument_0 = callback_arg_0_w0.0;
                            {
                                ::core::mem::forget(callback_arg_0_w0);
                            }
                            let raw_result = unsafe {
                                p0_callback_run(
                                    p0_callback_owner.context(),
                                    callback_argument_0,
                                )
                            };
                            let callback_return_w1 = raw_result;
                            let _callback_return_owner = ::prebindgen_runtime::TransferredOwner::new(
                                __PbOwnedGuard_packet(callback_return_w1),
                                __prebindgen_input_consumed,
                                0,
                            );
                            #[allow(unused_parens)]
                            let callback_return_w1_storage = callback_return_w1;
                            #[allow(clippy::let_and_return, clippy::unit_arg)]
                            let callback_return_w0_built = if true {
                                let callback_return_w0 = if true {
                                    let callback_return_w1_pointer = ::core::ptr::addr_of!(
                                        callback_return_w1_storage
                                    )
                                        .cast::<perftest_flat::InlinePacket>();
                                    let value = unsafe {
                                        ::core::ptr::read(callback_return_w1_pointer)
                                    };
                                    __prebindgen_input_consumed
                                        .set(__prebindgen_input_consumed.get() + 1);
                                    Some(value)
                                } else {
                                    None
                                };
                                Some({
                                    let callback_return_w0: perftest_flat::InlinePacket = callback_return_w0
                                        .expect("active inline owner");
                                    callback_return_w0
                                })
                            } else {
                                None
                            };
                            let callback_return_value = callback_return_w0_built
                                .expect("active input");
                            Ok(callback_return_value)
                        });
                        result
                            .unwrap_or_else(|error| {
                                ::prebindgen_runtime::report_callback_error(
                                    "owner_callback_7a0d2e4eb3c7fbb1",
                                    "(perftest_flat :: InlinePacket)->perftest_flat :: InlinePacket",
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
            let source_value = perftest_flat::inline_packet_callback(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_packet_callback_result {
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
pub struct owner_inline_packet_drops_result {
    pub v0: usize,
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
pub unsafe extern "C" fn owner_inline_packet_drops() -> usize {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packet_drops_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::inline_packet_drops();
            let ret_w0 = source_value;
            let result = owner_inline_packet_drops_result {
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
pub struct owner_inline_packet_new_result {
    pub v0: __PbOwned_packet,
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
pub unsafe extern "C" fn owner_inline_packet_new() -> __PbOwned_packet {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_packet_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::inline_packet_new();
            let ret_w0 = __PbOwned_packet::guard(source_value);
            let result = owner_inline_packet_new_result {
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
pub struct owner_inline_packet_optional_result {
    pub v0: i32,
    pub v1: __PbOwned_packet,
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
pub unsafe extern "C" fn owner_inline_packet_optional(
    p0_w1: u8,
    out1: *mut __PbOwned_packet,
) -> i32 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packet_optional_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        if out1.is_null() {
            return Err("null success output".into());
        }
        {
            let p0_w0 = if true { Some(p0_w1 != 0) } else { None };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                Some({
                    let p0_w0: bool = ::core::convert::Into::into(
                        p0_w0.expect("active value"),
                    );
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::inline_packet_optional(
                __prebindgen_argument_0,
            );
            let (ret_w0, (ret_w0_value0,), ret_w0_commit) = if let Some(ret_w1) = source_value {
                let ret_w2 = __PbOwned_packet::guard(ret_w1);
                let ret_w0_values = (ret_w2.0,);
                (
                    1i32,
                    ret_w0_values,
                    Some(move || {
                        ::core::mem::forget(ret_w2);
                    }),
                )
            } else {
                (0i32, (__PbOwned_packet::empty(),), None)
            };
            let result = owner_inline_packet_optional_result {
                v0: ret_w0,
                v1: ret_w0_value0,
            };
            std::ptr::write(output, result);
            let _: () = {
                if let Some(ret_w0_action) = ret_w0_commit {
                    ret_w0_action();
                }
            };
            Ok(())
        }
    }) {
        panic!("C binding failed: {}", error);
    }
    let value = storage.assume_init();
    std::ptr::write(out1, value.v1);
    value.v0
}
#[repr(C)]
pub struct owner_inline_packet_ref_result {
    pub v0: *const perftest_flat::InlineChild,
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
pub unsafe extern "C" fn owner_inline_packet_ref(
    p0_w1: *const __PbOwned_packet,
) -> *const perftest_flat::InlineChild {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_packet_ref_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let p0_w1_pointer = p0_w1 as *mut perftest_flat::InlinePacket;
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
                    let p0_w0: &perftest_flat::InlinePacket = unsafe {
                        &*(p0_w1_pointer)
                    };
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::inline_packet_ref(__prebindgen_argument_0);
            let ret_w0: *const perftest_flat::InlineChild = source_value
                as *const perftest_flat::InlineChild;
            let result = owner_inline_packet_ref_result {
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
pub struct owner_inline_packet_result_result {
    pub v0: __PbOwned_packet,
    pub v1: usize,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_inline_packet_result(
    out0: *mut __PbOwned_packet,
    out1: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packet_result_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        if out1.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let source_value = perftest_flat::inline_packet_result();
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::inline_packet_error(&ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w3 = perftest_flat::inline_packet_panic(&ret_w0);
            let ret_w4 = ret_w3;
            let ret_w2 = __PbOwned_packet::guard(ret_w0);
            let result = owner_inline_packet_result_result {
                v0: ret_w2.0,
                v1: ret_w4,
            };
            std::ptr::write(output, result);
            let _: () = {
                ::core::mem::forget(ret_w2);
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            std::ptr::write(out1, value.v1);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                let _ = ::prebindgen_runtime::boundary(|| {
                    drop((value.v0).into_rust());
                    Ok(())
                });
            }
            if !error_reported && !error.is_null() {
                unsafe { __pb_report_message(message, error) };
            }
            false
        }
    }
}
#[repr(C)]
pub struct owner_inline_packet_take_result {
    pub v0: usize,
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
pub unsafe extern "C" fn owner_inline_packet_take(
    p0_w1: *mut __PbOwned_packet,
) -> usize {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_packet_take_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let _: () = {};
            let p0_w1_pointer = p0_w1 as *mut perftest_flat::InlinePacket;
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p0_w1_empty = if true {
                Some(perftest_flat::inline_packet_empty())
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                let mut p0_w0_built_transferred_0 = 0usize;
                if true {
                    for address in ::core::iter::once((p0_w1_pointer.addr()) as usize) {
                        __prebindgen_input_aliases.access(address, true)?;
                        p0_w0_built_transferred_0 += 1;
                    }
                }
                let p0_w0 = if true {
                    Some(unsafe {
                        ::core::ptr::replace(
                            p0_w1_pointer,
                            p0_w1_empty.expect("prepared empty owner"),
                        )
                    })
                } else {
                    None
                };
                __prebindgen_input_consumed
                    .set(__prebindgen_input_consumed.get() + p0_w0_built_transferred_0);
                Some({
                    let p0_w0: perftest_flat::InlinePacket = p0_w0
                        .expect("active inline owner");
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::inline_packet_take(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_packet_take_result {
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
pub struct __PbRow_inline_packets_rows_ret_w3 {
    pub v0: __PbOwned_packet,
}
/**# Safety
The rows must be an unchanged allocation returned with this exact element layout. This frees only row storage, not element-owned fields.*/
#[no_mangle]
pub unsafe extern "C" fn owner_inline_packets_rows_ret_w3_rows_free(rows: __PbRows) {
    if rows.len != 0 {
        drop(
            Box::from_raw(
                std::ptr::slice_from_raw_parts_mut(
                    rows.data as *mut __PbRow_inline_packets_rows_ret_w3,
                    rows.len,
                ),
            ),
        );
    }
}
#[repr(C)]
pub struct owner_inline_packets_rows_result {
    pub v0: __PbRows,
    pub v1: usize,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract.*/
#[allow(
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_inline_packets_rows(
    out0: *mut __PbRows,
    out1: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packets_rows_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        if out0.is_null() {
            return Err("null success-output pointer".into());
        }
        if out1.is_null() {
            return Err("null success-output pointer".into());
        }
        {
            let source_value = perftest_flat::inline_packets_rows();
            let ret_w0 = match source_value {
                ::core::result::Result::Ok(value) => value,
                ::core::result::Result::Err(ret_w1) => {
                    return Err({
                        if !error.is_null() {
                            unsafe {
                                __pb_report_message(
                                    perftest_flat::inline_packet_error(&ret_w1),
                                    error,
                                )
                            };
                        }
                        error_reported = true;
                        String::new()
                    });
                }
            };
            let ret_w2 = perftest_flat::inline_packets_getter(&ret_w0);
            let ret_w3_source = ret_w2;
            let ret_w5 = ret_w3_source.len();
            let mut ret_w3_commits = Vec::with_capacity(ret_w5);
            let mut ret_w3: Vec<__PbRow_inline_packets_rows_ret_w3> = Vec::with_capacity(
                ret_w5,
            );
            for (ret_w6, ret_w4) in ret_w3_source.into_iter().enumerate() {
                let ret_w7 = __PbOwned_packet::guard(ret_w4);
                ret_w3
                    .push(__PbRow_inline_packets_rows_ret_w3 {
                        v0: ret_w7.0,
                    });
                ret_w3_commits
                    .push(move || {
                        ::core::mem::forget(ret_w7);
                    });
            }
            let ret_w3 = ret_w3.into_boxed_slice();
            let ret_w8 = perftest_flat::inline_packets_late(&ret_w0);
            let ret_w9 = ret_w8;
            let result = owner_inline_packets_rows_result {
                v0: __PbRows {
                    data: if ret_w3.is_empty() {
                        ::core::ptr::null()
                    } else {
                        ret_w3.as_ptr().cast()
                    },
                    len: ret_w3.len(),
                },
                v1: ret_w9,
            };
            std::ptr::write(output, result);
            let _: () = {
                let _ = Box::into_raw(ret_w3);
                for commit in ret_w3_commits {
                    commit();
                }
                success_committed = true;
            };
            Ok(())
        }
    });
    match outcome {
        Ok(()) => {
            let value = storage.assume_init();
            std::ptr::write(out0, value.v0);
            std::ptr::write(out1, value.v1);
            true
        }
        Err(message) => {
            if success_committed {
                let value = storage.assume_init();
                if (value.v0).len != 0 {
                    let rows = Box::from_raw(
                        std::ptr::slice_from_raw_parts_mut(
                            (value.v0).data as *mut __PbRow_inline_packets_rows_ret_w3,
                            (value.v0).len,
                        ),
                    );
                    for row in rows.iter() {
                        let _ = ::prebindgen_runtime::boundary(|| {
                            drop((row.v0).into_rust());
                            Ok(())
                        });
                    }
                }
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
pub struct __PbRow_inline_packets_take_p0_w1 {
    pub v0: *mut __PbOwned_packet,
}
#[repr(C)]
pub struct owner_inline_packets_take_result {
    pub v0: usize,
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
pub unsafe extern "C" fn owner_inline_packets_take(p0_w1: __PbRows) -> usize {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packets_take_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::DynamicInputs::default();
            let _: () = {};
            let mut p0_w0: Vec<perftest_flat::InlinePacket> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                ::prebindgen_runtime::validate_slice(
                    p0_w1.data as *const __PbRow_inline_packets_take_p0_w1,
                    p0_w2,
                )?;
                let mut p0_w1_accesses = __prebindgen_input_aliases
                    .begin_group(p0_w2.saturating_mul(1usize));
                let mut p0_w1_prepared = Vec::with_capacity(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe {
                        (*((p0_w1.data as *const __PbRow_inline_packets_take_p0_w1)
                            .add(p0_w3)))
                            .v0
                    };
                    #[allow(clippy::redundant_closure_call)]
                    let prepared = (|| {
                        let p0_w5_pointer = (p0_w5) as *mut perftest_flat::InlinePacket;
                        if true {
                            ::prebindgen_runtime::validate_pointer(p0_w5_pointer)?;
                        }
                        let p0_w5_empty = if true {
                            Some(perftest_flat::inline_packet_empty())
                        } else {
                            None
                        };
                        if true {
                            for address in ::core::iter::once(
                                (p0_w5_pointer.addr()) as usize,
                            ) {
                                p0_w1_accesses.push((address, true));
                            }
                        }
                        Ok::<
                            _,
                            String,
                        >(move || -> ::core::result::Result<
                            perftest_flat::InlinePacket,
                            String,
                        > {
                            if true {
                                __prebindgen_input_consumed
                                    .set(
                                        __prebindgen_input_consumed.get()
                                            + (::core::iter::once((p0_w5_pointer.addr()) as usize))
                                                .count(),
                                    );
                            }
                            let p0_w4 = if true {
                                Some(unsafe {
                                    ::core::ptr::replace(
                                        p0_w5_pointer,
                                        p0_w5_empty.expect("prepared empty owner"),
                                    )
                                })
                            } else {
                                None
                            };
                            let p0_w4: perftest_flat::InlinePacket = p0_w4
                                .expect("active inline owner");
                            let p0_w0 = p0_w4;
                            Ok(p0_w0)
                        })
                    })();
                    p0_w1_prepared.push(prepared?);
                }
                __prebindgen_input_aliases.accept_group(p0_w1_accesses)?;
                p0_w0.reserve_exact(p0_w2);
                for build in p0_w1_prepared {
                    let element = build()?;
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
            let source_value = perftest_flat::inline_packets_take(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_packets_take_result {
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
pub struct owner_inline_return_catch_result {
    pub v0: u8,
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
pub unsafe extern "C" fn owner_inline_return_catch(
    p0_callback: owner_callback_e857880e5782d2cb,
) -> u8 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_return_catch_result,
    >::uninit();
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
                            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
                            let raw_result = unsafe {
                                p0_callback_run(p0_callback_owner.context())
                            };
                            let callback_return_w2 = raw_result.v0;
                            let callback_return_w4 = raw_result.v1;
                            let mut __pb_guard_index = 0usize;
                            let callback_return_w2_return_guard = if true {
                                let ordinal = __pb_guard_index;
                                __pb_guard_index += 1;
                                Some(
                                    ::prebindgen_runtime::TransferredOwner::new(
                                        __PbOwnedGuard_packet(callback_return_w2),
                                        __prebindgen_input_consumed,
                                        ordinal,
                                    ),
                                )
                            } else {
                                None
                            };
                            {
                                #[allow(unused_parens)]
                                let callback_return_w2_storage = callback_return_w2;
                                #[allow(clippy::let_and_return, clippy::unit_arg)]
                                let callback_return_w1_built = if true {
                                    let callback_return_w1 = if true {
                                        let callback_return_w2_pointer = ::core::ptr::addr_of!(
                                            callback_return_w2_storage
                                        )
                                            .cast::<perftest_flat::InlinePacket>();
                                        let value = unsafe {
                                            ::core::ptr::read(callback_return_w2_pointer)
                                        };
                                        __prebindgen_input_consumed
                                            .set(__prebindgen_input_consumed.get() + 1);
                                        Some(value)
                                    } else {
                                        None
                                    };
                                    Some({
                                        let callback_return_w1: perftest_flat::InlinePacket = callback_return_w1
                                            .expect("active inline owner");
                                        callback_return_w1
                                    })
                                } else {
                                    None
                                };
                                let callback_return_w3 = if true {
                                    Some(
                                        char::from_u32(callback_return_w4)
                                            .ok_or("invalid Unicode scalar")?,
                                    )
                                } else {
                                    None
                                };
                                #[allow(clippy::let_and_return, clippy::unit_arg)]
                                let callback_return_w3_built = if true {
                                    Some({
                                        let callback_return_w3: char = ::core::convert::Into::into(
                                            callback_return_w3.expect("active value"),
                                        );
                                        callback_return_w3
                                    })
                                } else {
                                    None
                                };
                                #[allow(clippy::let_and_return, clippy::unit_arg)]
                                let callback_return_w0_built = if true {
                                    Some({
                                        let callback_return_w0: perftest_flat::InlineReturn = perftest_flat::inline_return_new(
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
                                    "owner_callback_e857880e5782d2cb",
                                    "()->perftest_flat :: InlineReturn",
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
            let source_value = perftest_flat::inline_return_catch(
                __prebindgen_argument_0,
            );
            let ret_w0 = u8::from(source_value);
            let result = owner_inline_return_catch_result {
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
pub struct owner_inline_rows_catch_result {
    pub v0: u8,
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
pub unsafe extern "C" fn owner_inline_rows_catch(
    p0_callback: owner_callback_1782e472ad837f0b,
) -> u8 {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_rows_catch_result>::uninit();
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
                let p0_callback_release_result = p0_callback.release_result;
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
                            let mut __prebindgen_input_aliases = ::prebindgen_runtime::DynamicInputs::default();
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
                            let mut callback_return_owners = ::prebindgen_runtime::CallbackReturnAllocations::tracking(
                                __prebindgen_input_consumed,
                            );
                            ::prebindgen_runtime::validate_slice(
                                callback_return_w1.data
                                    as *const __PbRow_callback_1782e472ad837f0b_callback_return_w1,
                                callback_return_w1.len,
                            )?;
                            for index in 0..callback_return_w1.len {
                                let callback_return_w6 = unsafe {
                                    (*((callback_return_w1.data
                                        as *const __PbRow_callback_1782e472ad837f0b_callback_return_w1)
                                        .add(index)))
                                        .v0
                                };
                                let callback_return_w8 = unsafe {
                                    (*((callback_return_w1.data
                                        as *const __PbRow_callback_1782e472ad837f0b_callback_return_w1)
                                        .add(index)))
                                        .v1
                                };
                                unsafe {
                                    callback_return_owners
                                        .receive_inline(
                                            (::core::ptr::addr_of!(
                                                (* ((callback_return_w1.data as * const
                                                __PbRow_callback_1782e472ad837f0b_callback_return_w1)
                                                .add(index))).v0
                                            ) as *const __PbOwned_packet)
                                                .cast::<perftest_flat::InlinePacket>(),
                                        );
                                }
                            }
                            callback_return_owners.check()?;
                            {
                                let mut callback_return_w0: Vec<
                                    perftest_flat::InlineReturn,
                                > = Vec::new();
                                if true {
                                    let callback_return_w2 = callback_return_w1.len;
                                    ::prebindgen_runtime::validate_slice(
                                        callback_return_w1.data
                                            as *const __PbRow_callback_1782e472ad837f0b_callback_return_w1,
                                        callback_return_w2,
                                    )?;
                                    callback_return_w0.reserve_exact(callback_return_w2);
                                    for callback_return_w3 in 0..callback_return_w2 {
                                        let callback_return_w6 = unsafe {
                                            (*((callback_return_w1.data
                                                as *const __PbRow_callback_1782e472ad837f0b_callback_return_w1)
                                                .add(callback_return_w3)))
                                                .v0
                                        };
                                        let callback_return_w8 = unsafe {
                                            (*((callback_return_w1.data
                                                as *const __PbRow_callback_1782e472ad837f0b_callback_return_w1)
                                                .add(callback_return_w3)))
                                                .v1
                                        };
                                        #[allow(clippy::redundant_closure_call)]
                                        let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                            perftest_flat::InlineReturn,
                                            String,
                                        > {
                                            #[allow(unused_parens)]
                                            let callback_return_w6_storage = (callback_return_w6);
                                            let callback_return_w7 = if true {
                                                Some(
                                                    char::from_u32(callback_return_w8)
                                                        .ok_or("invalid Unicode scalar")?,
                                                )
                                            } else {
                                                None
                                            };
                                            let callback_return_w5 = if true {
                                                let callback_return_w6_pointer = ::core::ptr::addr_of!(
                                                    callback_return_w6_storage
                                                )
                                                    .cast::<perftest_flat::InlinePacket>();
                                                let value = unsafe {
                                                    ::core::ptr::read(callback_return_w6_pointer)
                                                };
                                                __prebindgen_input_consumed
                                                    .set(__prebindgen_input_consumed.get() + 1);
                                                Some(value)
                                            } else {
                                                None
                                            };
                                            let callback_return_w5: perftest_flat::InlinePacket = callback_return_w5
                                                .expect("active inline owner");
                                            let callback_return_w7: char = ::core::convert::Into::into(
                                                callback_return_w7.expect("active value"),
                                            );
                                            let callback_return_w4: perftest_flat::InlineReturn = perftest_flat::inline_return_new(
                                                callback_return_w5,
                                                callback_return_w7,
                                            );
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
                            }
                        });
                        result
                            .unwrap_or_else(|error| {
                                ::prebindgen_runtime::report_callback_error(
                                    "owner_callback_1782e472ad837f0b",
                                    "()->Vec < perftest_flat :: InlineReturn >",
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
            let source_value = perftest_flat::inline_rows_catch(__prebindgen_argument_0);
            let ret_w0 = u8::from(source_value);
            let result = owner_inline_rows_catch_result {
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
pub struct owner_inline_text_result {
    pub v0: __PbOwned_text,
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
pub unsafe extern "C" fn owner_inline_text() -> __PbOwned_text {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_text_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::inline_text();
            let ret_w0 = __PbOwned_text::guard(source_value);
            let result = owner_inline_text_result {
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
pub struct owner_inline_text_capacity_result {
    pub v0: usize,
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
pub unsafe extern "C" fn owner_inline_text_capacity(
    p0_w1: *const __PbOwned_text,
) -> usize {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_text_capacity_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let p0_w1_pointer = p0_w1 as *mut String;
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
                    let p0_w0: &String = unsafe { &*(p0_w1_pointer) };
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::inline_text_capacity(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_text_capacity_result {
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
pub struct owner_inline_text_new_result {
    pub v0: __PbOwned_text,
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
pub unsafe extern "C" fn owner_inline_text_new() -> __PbOwned_text {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_text_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::inline_text_new();
            let ret_w0 = __PbOwned_text::guard(source_value);
            let result = owner_inline_text_new_result {
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
pub struct owner_inline_text_take_result {
    pub v0: usize,
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
pub unsafe extern "C" fn owner_inline_text_take(p0_w1: *mut __PbOwned_text) -> usize {
    #[allow(unused_mut)]
    let _ownership_accepted = ::core::cell::Cell::new(0usize);
    let mut storage = std::mem::MaybeUninit::<owner_inline_text_take_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let __prebindgen_input_consumed = &_ownership_accepted;
            let mut __prebindgen_input_aliases = ::prebindgen_runtime::SharedInputs;
            let _: () = {};
            let p0_w1_pointer = p0_w1 as *mut String;
            if true {
                ::prebindgen_runtime::validate_pointer(p0_w1_pointer)?;
            }
            let p0_w1_empty = if true {
                Some(<String as ::core::default::Default>::default())
            } else {
                None
            };
            #[allow(clippy::let_and_return, clippy::unit_arg)]
            let p0_w0_built = if true {
                let mut p0_w0_built_transferred_0 = 0usize;
                if true {
                    for address in ::core::iter::once((p0_w1_pointer.addr()) as usize) {
                        __prebindgen_input_aliases.access(address, true)?;
                        p0_w0_built_transferred_0 += 1;
                    }
                }
                let p0_w0 = if true {
                    Some(unsafe {
                        ::core::ptr::replace(
                            p0_w1_pointer,
                            p0_w1_empty.expect("prepared empty owner"),
                        )
                    })
                } else {
                    None
                };
                __prebindgen_input_consumed
                    .set(__prebindgen_input_consumed.get() + p0_w0_built_transferred_0);
                Some({
                    let p0_w0: String = p0_w0.expect("active inline owner");
                    p0_w0
                })
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0_built.expect("active input");
            let source_value = perftest_flat::inline_text_take(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = owner_inline_text_take_result {
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
    ::core::mem::size_of:: < perftest_flat::InlineChild > () != 0,
    "flat object `InlineChild` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::InlineChildBox > () != 0,
    "flat object `InlineChildBox` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::InlineMixed > () != 0,
    "flat object `InlineMixed` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::InlinePacket > () != 0,
    "flat object `InlinePacket` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::InlineReturn > () != 0,
    "flat object `InlineReturn` must have nonzero size"
);
