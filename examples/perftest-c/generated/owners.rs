#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_text_len(value: &String) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_text_view(value: &String) -> &[u8] {
    value.as_bytes()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_option_i64_33d2f0297e3115f7_len(
    value: &Vec<Option<i64>>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_option_perftest_flat_inlinecallbackrow_c14842e5d7db57c5_len(
    value: &Vec<Option<perftest_flat::InlineCallbackRow>>,
) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_string_3292bf5148c953ec_len(value: &Vec<String>) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_vec_u8_3f5fd420367502d4_len(value: &Vec<Vec<u8>>) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_numbers_len(value: &Vec<i64>) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_numbers_view(value: &Vec<i64>) -> &[i64] {
    value.as_slice()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_perftest_flat_inlinecallbackrow_7629716a78339b6a_len(
    value: &Vec<perftest_flat::InlineCallbackRow>,
) -> usize {
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
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_bytes_view_mut(value: &mut Vec<u8>) -> &mut [u8] {
    value.as_mut_slice()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_usize_4efcb9bb80acdd15_len(value: &Vec<usize>) -> usize {
    value.len()
}
#[allow(dead_code, clippy::ptr_arg)]
fn __pb_owned_owned_vec_usize_4efcb9bb80acdd15_view(value: &Vec<usize>) -> &[usize] {
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
pub unsafe extern "C" fn owner_free_error(error: *mut std::ffi::c_char) {
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
pub unsafe extern "C" fn owner_free_buffer(b: __PbBuffer) {
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
/**# Safety
The pointer must be null or a live allocation of the declared type returned by this library. It must not be used again after this call.*/
#[no_mangle]
pub unsafe extern "C" fn owner_drop_resource(ptr: *mut perftest_flat::OwnedResource) {
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
The owner must remain live and unmodified while its returned view is used. Obtaining a mutable view invalidates earlier views, even if no bytes are changed.*/
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
Keep the owner alive; do not mutate it, obtain overlapping exclusive/mutable access, consume or destroy it until the returned view's last use.*/
#[no_mangle]
pub unsafe extern "C" fn owner_numbers_view(
    value: *const __PbOwned_numbers,
) -> __PbSliceView_i64 {
    let values = __pb_owned_numbers_view((*value).as_rust());
    __PbSliceView_i64 {
        data: values.as_ptr(),
        len: values.len(),
    }
}
/**# Safety
The owner must remain live and unmodified while an element pointer is used. The index must be smaller than the owner length.*/
#[no_mangle]
pub unsafe extern "C" fn owner_numbers_get(
    value: *const __PbOwned_numbers,
    index: usize,
) -> *const i64 {
    (*value).as_rust().get_unchecked(index) as *const i64
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
The owner must remain live and unmodified while its returned view is used. Obtaining a mutable view invalidates earlier views, even if no bytes are changed.*/
#[no_mangle]
pub unsafe extern "C" fn owner_bytes_view(value: *const __PbOwned_bytes) -> __PbBuffer {
    let bytes = __pb_owned_bytes_view((*value).as_rust());
    __PbBuffer {
        data: bytes.as_ptr(),
        len: bytes.len(),
    }
}
/**# Safety
The owner must remain live and stationary. Preserve exclusive access to its allocation until the returned view's last use; do not access, consume or drop the owner during that borrow.*/
#[no_mangle]
pub unsafe extern "C" fn owner_bytes_view_mut(
    value: *mut __PbOwned_bytes,
) -> __PbBufferMut {
    let bytes = __pb_owned_bytes_view_mut((*value).as_rust_mut());
    __PbBufferMut {
        data: bytes.as_mut_ptr(),
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
pub struct owner_callback_0c47d5ea09f8970c {
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
pub struct __PbRow_callback_37300b78c94d93fa_callback_return_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
    pub v1: __PbOwned_packet,
    pub v2: *mut perftest_flat::OwnedResource,
    pub v3: u32,
    pub v4: usize,
    pub v5: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_37300b78c94d93fa {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> __PbRows>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbRows),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_70db60a6f9dbbb06_callback_return_w3 {
    pub v0: *mut perftest_flat::OwnedResource,
    pub v1: __PbOwned_packet,
    pub v2: *mut perftest_flat::OwnedResource,
    pub v3: u32,
    pub v4: usize,
    pub v5: usize,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_70db60a6f9dbbb06_result {
    pub v0: i32,
    pub v1: __PbRows,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_70db60a6f9dbbb06 {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
        ) -> owner_callback_70db60a6f9dbbb06_result,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
            *const owner_callback_70db60a6f9dbbb06_result,
        ),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct __PbRow_callback_805117dbed50edfb_callback_return_w1 {
    pub v0: __PbOwned_packet,
    pub v1: *mut perftest_flat::InlineChild,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_805117dbed50edfb {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> __PbRows>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbRows),
    >,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_c381621a1cf6edc5_result {
    pub v0: __PbOwned_packet,
    pub v1: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_c381621a1cf6edc5 {
    pub run: Option<
        unsafe extern "C" fn(
            *mut ::core::ffi::c_void,
        ) -> owner_callback_c381621a1cf6edc5_result,
    >,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_f5757ec9aa1c45d2 {
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
pub struct __PbRow_callback_f5d26722508e30c3_callback_return_w1 {
    pub v0: __PbOwned_packet,
    pub v1: u32,
}
#[repr(C)]
#[derive(Clone, Copy)]
pub struct owner_callback_f5d26722508e30c3 {
    pub run: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void) -> __PbRows>,
    pub context: *mut ::core::ffi::c_void,
    pub drop: Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    pub release_result: Option<
        unsafe extern "C" fn(*mut ::core::ffi::c_void, *const __PbRows),
    >,
}
#[repr(C)]
pub struct owner_collection_resource_new_result {
    pub v0: *mut perftest_flat::OwnedResource,
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
pub unsafe extern "C" fn owner_collection_resource_new(
    p0_w1: i64,
) -> *mut perftest_flat::OwnedResource {
    let mut storage = std::mem::MaybeUninit::<
        owner_collection_resource_new_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: i64 = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::collection_resource_new(
                __prebindgen_argument_0,
            );
            let ret_w0: *mut perftest_flat::OwnedResource = Box::into_raw(
                Box::new(source_value),
            );
            let ret_w0_owner = __PbOutputOwner(ret_w0);
            let result = owner_collection_resource_new_result {
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
pub struct owner_enum_inputs_constructors_result {
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
pub unsafe extern "C" fn owner_enum_inputs_constructors() -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_enum_inputs_constructors_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::enum_inputs_constructors();
            let ret_w0 = source_value;
            let result = owner_enum_inputs_constructors_result {
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
pub struct __PbRow_enum_inputs_inline_p0_w1 {
    pub v0: *mut __PbOwned_packet,
    pub v1: i32,
}
#[repr(C)]
pub struct owner_enum_inputs_inline_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_enum_inputs_inline(
    p0_w1: __PbRows,
    out0: *mut i64,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<owner_enum_inputs_inline_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<i64> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_enum_inputs_inline_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    let p0_w8 = unsafe { (*p0_w1_data.add(p0_w3)).v1 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<i64, String> {
                        {
                            let p0_w6_pointer = (p0_w6)
                                as *mut perftest_flat::InlinePacket;
                            let p0_w6_empty = if true {
                                Some(perftest_flat::inline_packet_empty())
                            } else {
                                None
                            };
                            let p0_w5 = if true {
                                {
                                    let value = unsafe {
                                        ::core::ptr::replace(
                                            p0_w6_pointer,
                                            p0_w6_empty.expect("prepared empty owner"),
                                        )
                                    };
                                    {
                                        ownership_accepted.set(ownership_accepted.get() + (1usize));
                                    }
                                    Some(value)
                                }
                            } else {
                                None
                            };
                            let p0_w5: perftest_flat::InlinePacket = p0_w5
                                .expect("active inline owner");
                            let p0_w4_argument_0 = p0_w5;
                            let p0_w7 = match p0_w8 {
                                -3i32 => perftest_flat::InputTag::Low,
                                0i32 => perftest_flat::InputTag::Normal,
                                17i32 => perftest_flat::InputTag::High,
                                _ => unsafe { ::core::hint::unreachable_unchecked() }
                            };
                            let p0_w4_argument_1 = p0_w7;
                            let p0_w4: i64 = perftest_flat::enum_input_inline(
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
            let source_value = perftest_flat::enum_inputs_inline(
                __prebindgen_argument_0,
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
            let result = owner_enum_inputs_inline_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    std::ptr::write(ownership, ownership_accepted.get());
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
pub struct owner_enum_inputs_reset_result {
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
pub unsafe extern "C" fn owner_enum_inputs_reset() {
    let mut storage = std::mem::MaybeUninit::<owner_enum_inputs_reset_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::enum_inputs_reset();
            let result = owner_enum_inputs_reset_result {
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
pub struct owner_inline_boxed_numbers_len_result {
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
pub unsafe extern "C" fn owner_inline_boxed_numbers_len(
    p0_w1: *const Vec<i32>,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_boxed_numbers_len_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut Vec<i32>;
            let p0_w0: &Vec<i32> = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
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
pub unsafe extern "C" fn owner_inline_boxed_numbers_new() -> *mut Vec<i32> {
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
pub unsafe extern "C" fn owner_inline_boxed_numbers_take(p0_w1: *mut Vec<i32>) -> i32 {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_boxed_numbers_take_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut Vec<i32>;
            let p0_w0: Vec<i32> = {
                let p0_w0 = unsafe { Box::from_raw(p0_w1_pointer) };
                {}
                *p0_w0
            };
            let __prebindgen_argument_0 = p0_w0;
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
pub unsafe extern "C" fn owner_inline_bytes() -> __PbOwned_bytes {
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
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
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
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<owner_inline_bytes_alias_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut Vec<u8>;
            let p0_w1_empty = if true {
                Some(<Vec<u8> as ::core::default::Default>::default())
            } else {
                None
            };
            let p0_w0 = if true {
                {
                    let value = unsafe {
                        ::core::ptr::replace(
                            p0_w1_pointer,
                            p0_w1_empty.expect("prepared empty owner"),
                        )
                    };
                    {
                        ownership_accepted.set(ownership_accepted.get() + (1usize));
                    }
                    Some(value)
                }
            } else {
                None
            };
            let p0_w0: Vec<u8> = p0_w0.expect("active inline owner");
            let __prebindgen_argument_0 = p0_w0;
            let p1_w1_pointer = p1_w1 as *mut Vec<u8>;
            let p1_w0: &Vec<u8> = unsafe { &*(p1_w1_pointer) };
            let __prebindgen_argument_1 = p1_w0;
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
    std::ptr::write(ownership, ownership_accepted.get());
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
pub unsafe extern "C" fn owner_inline_bytes_callback(
    p0_callback: owner_callback_f5757ec9aa1c45d2,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_bytes_callback_result,
    >::uninit();
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
            let p0_callback_value = move |callback_source_0: Vec<u8>| {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = __PbOwned_bytes::guard(callback_source_0);
                    let __pb_callback_consumed = ::core::cell::Cell::new(0usize);
                    let __prebindgen_input_consumed = &__pb_callback_consumed;
                    let callback_argument_0 = callback_arg_0_w0.0;
                    {
                        ::core::mem::forget(callback_arg_0_w0);
                    }
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
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
                                            ::core::ptr::read(
                                                (&callback_return_w1 as *const __PbOwned_bytes)
                                                    .cast::<Vec<u8>>(),
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
                    let callback_outcome: ::prebindgen_runtime::Result<_> = {
                        #[allow(unused_parens)]
                        let callback_return_w1_storage = callback_return_w1;
                        let callback_return_w0 = if true {
                            let callback_return_w1_pointer = ::core::ptr::addr_of!(
                                callback_return_w1_storage
                            )
                                .cast::<Vec<u8>>();
                            let value = unsafe {
                                ::core::ptr::read(callback_return_w1_pointer)
                            };
                            {
                                __pb_callback_consumed
                                    .set(__pb_callback_consumed.get() + (1usize));
                            }
                            Some(value)
                        } else {
                            None
                        };
                        let callback_return_w0: Vec<u8> = callback_return_w0
                            .expect("active inline owner");
                        let callback_return_value = callback_return_w0;
                        Ok(callback_return_value)
                    };
                    let callback_decoded = callback_outcome?;
                    callback_return_cleanup.commit();
                    Ok(callback_decoded)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "owner_callback_f5757ec9aa1c45d2",
                            "(Vec < u8 >)->Vec < u8 >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
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
pub unsafe extern "C" fn owner_inline_bytes_capacity(
    p0_w1: *const __PbOwned_bytes,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_bytes_capacity_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut Vec<u8>;
            let p0_w0: &Vec<u8> = unsafe { &*(p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
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
pub unsafe extern "C" fn owner_inline_bytes_mutable(
    out1: *mut usize,
) -> __PbOwned_bytes {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_bytes_mutable_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
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
pub unsafe extern "C" fn owner_inline_bytes_new(
    out1: *mut __PbBuffer,
    out2: *mut usize,
) -> __PbOwned_bytes {
    let mut storage = std::mem::MaybeUninit::<owner_inline_bytes_new_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
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
pub unsafe extern "C" fn owner_inline_bytes_take(p0_w1: *mut __PbOwned_bytes) -> usize {
    let mut storage = std::mem::MaybeUninit::<owner_inline_bytes_take_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut Vec<u8>;
            let p0_w1_empty = if true {
                Some(<Vec<u8> as ::core::default::Default>::default())
            } else {
                None
            };
            let p0_w0 = if true {
                {
                    let value = unsafe {
                        ::core::ptr::replace(
                            p0_w1_pointer,
                            p0_w1_empty.expect("prepared empty owner"),
                        )
                    };
                    {}
                    Some(value)
                }
            } else {
                None
            };
            let p0_w0: Vec<u8> = p0_w0.expect("active inline owner");
            let __prebindgen_argument_0 = p0_w0;
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
pub struct owner_inline_callback_optional_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_inline_callback_optional(
    p0_callback: owner_callback_70db60a6f9dbbb06,
    out0: *mut i64,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_callback_optional_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
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
                {
                    ownership_accepted.set(ownership_accepted.get() + (1usize));
                }
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
                    let callback_return_w1 = raw_result.v0;
                    let callback_return_w3 = raw_result.v1;
                    let mut callback_return_cleanup = ::prebindgen_runtime::CallbackReturnCleanup::new(||
                    {
                        let mut __pb_return_ordinal = 0usize;
                        if callback_return_w1 == 1 {
                            #[allow(clippy::explicit_counter_loop)]
                            for index in 0..callback_return_w3.len {
                                let callback_return_w10 = unsafe {
                                    (*((callback_return_w3.data
                                        as *const __PbRow_callback_70db60a6f9dbbb06_callback_return_w3)
                                        .add(index)))
                                        .v0
                                };
                                let callback_return_w12 = unsafe {
                                    (*((callback_return_w3.data
                                        as *const __PbRow_callback_70db60a6f9dbbb06_callback_return_w3)
                                        .add(index)))
                                        .v1
                                };
                                let callback_return_w14 = unsafe {
                                    (*((callback_return_w3.data
                                        as *const __PbRow_callback_70db60a6f9dbbb06_callback_return_w3)
                                        .add(index)))
                                        .v2
                                };
                                let callback_return_w16 = unsafe {
                                    (*((callback_return_w3.data
                                        as *const __PbRow_callback_70db60a6f9dbbb06_callback_return_w3)
                                        .add(index)))
                                        .v3
                                };
                                let callback_return_w18 = unsafe {
                                    (*((callback_return_w3.data
                                        as *const __PbRow_callback_70db60a6f9dbbb06_callback_return_w3)
                                        .add(index)))
                                        .v4
                                };
                                let callback_return_w20 = unsafe {
                                    (*((callback_return_w3.data
                                        as *const __PbRow_callback_70db60a6f9dbbb06_callback_return_w3)
                                        .add(index)))
                                        .v5
                                };
                                if (callback_return_w10 as usize) != 0 {
                                    {
                                        let ordinal = __pb_return_ordinal;
                                        __pb_return_ordinal += 1;
                                        if ordinal >= __prebindgen_input_consumed.get() {
                                            if let Err(error) = ::prebindgen_runtime::boundary(|| {
                                                unsafe {
                                                    drop(
                                                        Box::from_raw(
                                                            callback_return_w10 as *mut perftest_flat::OwnedResource,
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
                                                                (* ((callback_return_w3.data as * const
                                                                __PbRow_callback_70db60a6f9dbbb06_callback_return_w3)
                                                                .add(index))).v1
                                                            ) as *const __PbOwned_packet)
                                                                .cast::<perftest_flat::InlinePacket>(),
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
                                                            callback_return_w14 as *mut perftest_flat::OwnedResource,
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
                            }
                        }
                        Ok(())
                    });
                    {
                        let callback_outcome: ::prebindgen_runtime::Result<_> = {
                            let callback_return_w0 = if callback_return_w1 == 1i32 {
                                {
                                    let mut callback_return_w2: Vec<
                                        Option<perftest_flat::InlineCallbackRow>,
                                    > = Vec::new();
                                    if callback_return_w1 == 1i32 {
                                        let callback_return_w4 = callback_return_w3.len;
                                        let callback_return_w3_data = callback_return_w3.data
                                            as *const __PbRow_callback_70db60a6f9dbbb06_callback_return_w3;
                                        callback_return_w2.reserve_exact(callback_return_w4);
                                        for callback_return_w5 in 0..callback_return_w4 {
                                            let callback_return_w10 = unsafe {
                                                (*callback_return_w3_data.add(callback_return_w5)).v0
                                            };
                                            let callback_return_w12 = unsafe {
                                                (*callback_return_w3_data.add(callback_return_w5)).v1
                                            };
                                            let callback_return_w14 = unsafe {
                                                (*callback_return_w3_data.add(callback_return_w5)).v2
                                            };
                                            let callback_return_w16 = unsafe {
                                                (*callback_return_w3_data.add(callback_return_w5)).v3
                                            };
                                            let callback_return_w18 = unsafe {
                                                (*callback_return_w3_data.add(callback_return_w5)).v4
                                            };
                                            let callback_return_w20 = unsafe {
                                                (*callback_return_w3_data.add(callback_return_w5)).v5
                                            };
                                            #[allow(clippy::redundant_closure_call)]
                                            let callback_return_w3_element_built = (|| -> ::core::result::Result<
                                                Option<perftest_flat::InlineCallbackRow>,
                                                String,
                                            > {
                                                {
                                                    let callback_return_w7: i32 = i32::from(
                                                        ((callback_return_w10) as usize) != 0,
                                                    );
                                                    let callback_return_w6 = if callback_return_w7 == 1i32 {
                                                        {
                                                            let callback_return_w10_pointer = (callback_return_w10)
                                                                as *mut perftest_flat::OwnedResource;
                                                            let callback_return_w9: perftest_flat::OwnedResource = {
                                                                let callback_return_w9 = unsafe {
                                                                    Box::from_raw(callback_return_w10_pointer)
                                                                };
                                                                {
                                                                    __pb_callback_consumed
                                                                        .set(__pb_callback_consumed.get() + (1usize));
                                                                }
                                                                *callback_return_w9
                                                            };
                                                            let callback_return_w8_argument_0 = callback_return_w9;
                                                            #[allow(unused_parens)]
                                                            let callback_return_w12_storage = (callback_return_w12);
                                                            let callback_return_w11 = if callback_return_w7 == 1i32 {
                                                                let callback_return_w12_pointer = ::core::ptr::addr_of!(
                                                                    callback_return_w12_storage
                                                                )
                                                                    .cast::<perftest_flat::InlinePacket>();
                                                                let value = unsafe {
                                                                    ::core::ptr::read(callback_return_w12_pointer)
                                                                };
                                                                {
                                                                    __pb_callback_consumed
                                                                        .set(__pb_callback_consumed.get() + (1usize));
                                                                }
                                                                Some(value)
                                                            } else {
                                                                None
                                                            };
                                                            let callback_return_w11: perftest_flat::InlinePacket = callback_return_w11
                                                                .expect("active inline owner");
                                                            let callback_return_w8_argument_1 = callback_return_w11;
                                                            let callback_return_w14_pointer = (callback_return_w14)
                                                                as *mut perftest_flat::OwnedResource;
                                                            let callback_return_w13: perftest_flat::OwnedResource = {
                                                                let callback_return_w13 = unsafe {
                                                                    Box::from_raw(callback_return_w14_pointer)
                                                                };
                                                                {
                                                                    __pb_callback_consumed
                                                                        .set(__pb_callback_consumed.get() + (1usize));
                                                                }
                                                                *callback_return_w13
                                                            };
                                                            let callback_return_w8_argument_2 = callback_return_w13;
                                                            let callback_return_w15: char = ::core::convert::Into::into(unsafe {
                                                                char::from_u32_unchecked(callback_return_w16)
                                                            });
                                                            let callback_return_w8_argument_3 = callback_return_w15;
                                                            let callback_return_w17: usize = ::core::convert::Into::into(
                                                                callback_return_w18,
                                                            );
                                                            let callback_return_w8_argument_4 = callback_return_w17;
                                                            let callback_return_w19: usize = ::core::convert::Into::into(
                                                                callback_return_w20,
                                                            );
                                                            let callback_return_w8_argument_5 = callback_return_w19;
                                                            let callback_return_w8: perftest_flat::InlineCallbackRow = perftest_flat::inline_callback_row_new(
                                                                callback_return_w8_argument_0,
                                                                callback_return_w8_argument_1,
                                                                callback_return_w8_argument_2,
                                                                callback_return_w8_argument_3,
                                                                callback_return_w8_argument_4,
                                                                callback_return_w8_argument_5,
                                                            );
                                                            let callback_return_w6_payload = callback_return_w8;
                                                            Some(callback_return_w6_payload)
                                                        }
                                                    } else {
                                                        None
                                                    };
                                                    let callback_return_w3_row_value = callback_return_w6;
                                                    Ok(callback_return_w3_row_value)
                                                }
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
                                    let callback_return_w0_payload = callback_return_w2;
                                    Some(callback_return_w0_payload)
                                }
                            } else {
                                None
                            };
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
                            "owner_callback_70db60a6f9dbbb06",
                            "()->Option < Vec < Option < perftest_flat :: InlineCallbackRow > > >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::inline_callback_optional(
                __prebindgen_argument_0,
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
            let result = owner_inline_callback_optional_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    std::ptr::write(ownership, ownership_accepted.get());
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
pub struct owner_inline_callback_rows_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_inline_callback_rows(
    p0_callback: owner_callback_37300b78c94d93fa,
    out0: *mut i64,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_callback_rows_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
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
                {
                    ownership_accepted.set(ownership_accepted.get() + (1usize));
                }
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
                                    as *const __PbRow_callback_37300b78c94d93fa_callback_return_w1)
                                    .add(index)))
                                    .v0
                            };
                            let callback_return_w8 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_37300b78c94d93fa_callback_return_w1)
                                    .add(index)))
                                    .v1
                            };
                            let callback_return_w10 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_37300b78c94d93fa_callback_return_w1)
                                    .add(index)))
                                    .v2
                            };
                            let callback_return_w12 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_37300b78c94d93fa_callback_return_w1)
                                    .add(index)))
                                    .v3
                            };
                            let callback_return_w14 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_37300b78c94d93fa_callback_return_w1)
                                    .add(index)))
                                    .v4
                            };
                            let callback_return_w16 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_37300b78c94d93fa_callback_return_w1)
                                    .add(index)))
                                    .v5
                            };
                            {
                                let ordinal = __pb_return_ordinal;
                                __pb_return_ordinal += 1;
                                if ordinal >= __prebindgen_input_consumed.get() {
                                    if let Err(error) = ::prebindgen_runtime::boundary(|| {
                                        unsafe {
                                            drop(
                                                Box::from_raw(
                                                    callback_return_w6 as *mut perftest_flat::OwnedResource,
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
                                                        __PbRow_callback_37300b78c94d93fa_callback_return_w1)
                                                        .add(index))).v1
                                                    ) as *const __PbOwned_packet)
                                                        .cast::<perftest_flat::InlinePacket>(),
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
                                                    callback_return_w10 as *mut perftest_flat::OwnedResource,
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
                                perftest_flat::InlineCallbackRow,
                            > = Vec::new();
                            if true {
                                let callback_return_w2 = callback_return_w1.len;
                                let callback_return_w1_data = callback_return_w1.data
                                    as *const __PbRow_callback_37300b78c94d93fa_callback_return_w1;
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
                                    let callback_return_w12 = unsafe {
                                        (*callback_return_w1_data.add(callback_return_w3)).v3
                                    };
                                    let callback_return_w14 = unsafe {
                                        (*callback_return_w1_data.add(callback_return_w3)).v4
                                    };
                                    let callback_return_w16 = unsafe {
                                        (*callback_return_w1_data.add(callback_return_w3)).v5
                                    };
                                    #[allow(clippy::redundant_closure_call)]
                                    let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                        perftest_flat::InlineCallbackRow,
                                        String,
                                    > {
                                        {
                                            let callback_return_w6_pointer = (callback_return_w6)
                                                as *mut perftest_flat::OwnedResource;
                                            let callback_return_w5: perftest_flat::OwnedResource = {
                                                let callback_return_w5 = unsafe {
                                                    Box::from_raw(callback_return_w6_pointer)
                                                };
                                                {
                                                    __pb_callback_consumed
                                                        .set(__pb_callback_consumed.get() + (1usize));
                                                }
                                                *callback_return_w5
                                            };
                                            let callback_return_w4_argument_0 = callback_return_w5;
                                            #[allow(unused_parens)]
                                            let callback_return_w8_storage = (callback_return_w8);
                                            let callback_return_w7 = if true {
                                                let callback_return_w8_pointer = ::core::ptr::addr_of!(
                                                    callback_return_w8_storage
                                                )
                                                    .cast::<perftest_flat::InlinePacket>();
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
                                            let callback_return_w7: perftest_flat::InlinePacket = callback_return_w7
                                                .expect("active inline owner");
                                            let callback_return_w4_argument_1 = callback_return_w7;
                                            let callback_return_w10_pointer = (callback_return_w10)
                                                as *mut perftest_flat::OwnedResource;
                                            let callback_return_w9: perftest_flat::OwnedResource = {
                                                let callback_return_w9 = unsafe {
                                                    Box::from_raw(callback_return_w10_pointer)
                                                };
                                                {
                                                    __pb_callback_consumed
                                                        .set(__pb_callback_consumed.get() + (1usize));
                                                }
                                                *callback_return_w9
                                            };
                                            let callback_return_w4_argument_2 = callback_return_w9;
                                            let callback_return_w11: char = ::core::convert::Into::into(unsafe {
                                                char::from_u32_unchecked(callback_return_w12)
                                            });
                                            let callback_return_w4_argument_3 = callback_return_w11;
                                            let callback_return_w13: usize = ::core::convert::Into::into(
                                                callback_return_w14,
                                            );
                                            let callback_return_w4_argument_4 = callback_return_w13;
                                            let callback_return_w15: usize = ::core::convert::Into::into(
                                                callback_return_w16,
                                            );
                                            let callback_return_w4_argument_5 = callback_return_w15;
                                            let callback_return_w4: perftest_flat::InlineCallbackRow = perftest_flat::inline_callback_row_new(
                                                callback_return_w4_argument_0,
                                                callback_return_w4_argument_1,
                                                callback_return_w4_argument_2,
                                                callback_return_w4_argument_3,
                                                callback_return_w4_argument_4,
                                                callback_return_w4_argument_5,
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
                            "owner_callback_37300b78c94d93fa",
                            "()->Vec < perftest_flat :: InlineCallbackRow >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
            let source_value = perftest_flat::inline_callback_rows(
                __prebindgen_argument_0,
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
            let result = owner_inline_callback_rows_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    std::ptr::write(ownership, ownership_accepted.get());
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
pub struct owner_inline_child_id_result {
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
pub unsafe extern "C" fn owner_inline_child_id(
    p0_w1: *const perftest_flat::InlineChild,
) -> u64 {
    let mut storage = std::mem::MaybeUninit::<owner_inline_child_id_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut perftest_flat::InlineChild;
            let p0_w0: &perftest_flat::InlineChild = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
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
#[derive(Clone, Copy)]
pub struct __PbRow_inline_inputs_borrow_p1_w1 {
    pub v0: *const __PbOwned_packet,
}
#[repr(C)]
pub struct owner_inline_inputs_borrow_result {
    pub v0: usize,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_inline_inputs_borrow(
    p0_w1: *mut __PbOwned_packet,
    p1_w1: __PbRows,
    out0: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_inputs_borrow_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut perftest_flat::InlinePacket;
            let p0_w0: &mut perftest_flat::InlinePacket = unsafe {
                &mut *(p0_w1_pointer)
            };
            let __prebindgen_argument_0 = p0_w0;
            let mut p1_w0: Vec<usize> = Vec::new();
            if true {
                let p1_w2 = p1_w1.len;
                let p1_w1_data = p1_w1.data as *const __PbRow_inline_inputs_borrow_p1_w1;
                p1_w0.reserve_exact(p1_w2);
                for p1_w3 in 0..p1_w2 {
                    let p1_w6 = unsafe { (*p1_w1_data.add(p1_w3)).v0 };
                    #[allow(clippy::redundant_closure_call)]
                    let p1_w1_element_built = (|| -> ::core::result::Result<
                        usize,
                        String,
                    > {
                        {
                            let p1_w6_pointer = (p1_w6)
                                as *mut perftest_flat::InlinePacket;
                            let p1_w5: &perftest_flat::InlinePacket = unsafe {
                                &*(p1_w6_pointer)
                            };
                            let p1_w4_argument_0 = p1_w5;
                            let p1_w4: usize = perftest_flat::inline_input_borrowed(
                                p1_w4_argument_0,
                            );
                            let p1_w1_row_value = p1_w4;
                            Ok(p1_w1_row_value)
                        }
                    })();
                    let element = p1_w1_element_built?;
                    #[allow(clippy::zst_offset)]
                    unsafe {
                        let initialized = p1_w0.len();
                        p1_w0.as_mut_ptr().add(initialized).write(element);
                        p1_w0.set_len(initialized + 1);
                    }
                }
            }
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::inline_inputs_borrow(
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
            let result = owner_inline_inputs_borrow_result {
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
#[derive(Clone, Copy)]
pub struct __PbRow_inline_inputs_bytes_p0_w1 {
    pub v0: *mut __PbOwned_bytes,
}
#[repr(C)]
pub struct owner_inline_inputs_bytes_result {
    pub v0: usize,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_inline_inputs_bytes(
    p0_w1: __PbRows,
    out0: *mut usize,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_inputs_bytes_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<Vec<u8>> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_inline_inputs_bytes_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        Vec<u8>,
                        String,
                    > {
                        {
                            let p0_w5_pointer = (p0_w5) as *mut Vec<u8>;
                            let p0_w5_empty = if true {
                                Some(<Vec<u8> as ::core::default::Default>::default())
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
                                    {
                                        ownership_accepted.set(ownership_accepted.get() + (1usize));
                                    }
                                    Some(value)
                                }
                            } else {
                                None
                            };
                            let p0_w4: Vec<u8> = p0_w4.expect("active inline owner");
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
            let source_value = perftest_flat::inline_inputs_bytes(
                __prebindgen_argument_0,
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
            let result = owner_inline_inputs_bytes_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    std::ptr::write(ownership, ownership_accepted.get());
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
pub struct owner_inline_inputs_calls_result {
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
pub unsafe extern "C" fn owner_inline_inputs_calls() -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_inputs_calls_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::inline_inputs_calls();
            let ret_w0 = source_value;
            let result = owner_inline_inputs_calls_result {
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
pub struct owner_inline_inputs_constructor_calls_result {
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
pub unsafe extern "C" fn owner_inline_inputs_constructor_calls() -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_inputs_constructor_calls_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::inline_inputs_constructor_calls();
            let ret_w0 = source_value;
            let result = owner_inline_inputs_constructor_calls_result {
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
pub struct owner_inline_inputs_empty_calls_result {
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
pub unsafe extern "C" fn owner_inline_inputs_empty_calls() -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_inputs_empty_calls_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::inline_inputs_empty_calls();
            let ret_w0 = source_value;
            let result = owner_inline_inputs_empty_calls_result {
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
pub struct __PbRow_inline_inputs_mixed_p0_w1 {
    pub v0: *mut perftest_flat::OwnedResource,
    pub v1: *mut __PbOwned_packet,
    pub v2: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
pub struct owner_inline_inputs_mixed_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_inline_inputs_mixed(
    p0_w1: __PbRows,
    out0: *mut i64,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_inputs_mixed_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<i64> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_inline_inputs_mixed_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    let p0_w8 = unsafe { (*p0_w1_data.add(p0_w3)).v1 };
                    let p0_w10 = unsafe { (*p0_w1_data.add(p0_w3)).v2 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<i64, String> {
                        {
                            let p0_w6_pointer = (p0_w6)
                                as *mut perftest_flat::OwnedResource;
                            let p0_w5: perftest_flat::OwnedResource = {
                                let p0_w5 = unsafe { Box::from_raw(p0_w6_pointer) };
                                {
                                    ownership_accepted.set(ownership_accepted.get() + (1usize));
                                }
                                *p0_w5
                            };
                            let p0_w4_argument_0 = p0_w5;
                            let p0_w8_pointer = (p0_w8)
                                as *mut perftest_flat::InlinePacket;
                            let p0_w8_empty = if true {
                                Some(perftest_flat::inline_packet_empty())
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
                                    {
                                        ownership_accepted.set(ownership_accepted.get() + (1usize));
                                    }
                                    Some(value)
                                }
                            } else {
                                None
                            };
                            let p0_w7: perftest_flat::InlinePacket = p0_w7
                                .expect("active inline owner");
                            let p0_w4_argument_1 = p0_w7;
                            let p0_w10_pointer = (p0_w10)
                                as *mut perftest_flat::OwnedResource;
                            let p0_w9: perftest_flat::OwnedResource = {
                                let p0_w9 = unsafe { Box::from_raw(p0_w10_pointer) };
                                {
                                    ownership_accepted.set(ownership_accepted.get() + (1usize));
                                }
                                *p0_w9
                            };
                            let p0_w4_argument_2 = p0_w9;
                            let p0_w4: i64 = perftest_flat::inline_input_mixed(
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
            let source_value = perftest_flat::inline_inputs_mixed(
                __prebindgen_argument_0,
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
            let result = owner_inline_inputs_mixed_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    std::ptr::write(ownership, ownership_accepted.get());
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
pub struct __PbRow_inline_inputs_optional_p0_w3 {
    pub v0: *mut perftest_flat::OwnedResource,
    pub v1: *mut __PbOwned_packet,
    pub v2: *mut perftest_flat::OwnedResource,
}
#[repr(C)]
pub struct owner_inline_inputs_optional_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_inline_inputs_optional(
    p0_w1: i32,
    p0_w3: __PbRows,
    out0: *mut i64,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_inputs_optional_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0 = if p0_w1 == 1i32 {
                {
                    let mut p0_w2: Vec<Option<i64>> = Vec::new();
                    if p0_w1 == 1i32 {
                        let p0_w4 = p0_w3.len;
                        let p0_w3_data = p0_w3.data
                            as *const __PbRow_inline_inputs_optional_p0_w3;
                        p0_w2.reserve_exact(p0_w4);
                        for p0_w5 in 0..p0_w4 {
                            let p0_w10 = unsafe { (*p0_w3_data.add(p0_w5)).v0 };
                            let p0_w12 = unsafe { (*p0_w3_data.add(p0_w5)).v1 };
                            let p0_w14 = unsafe { (*p0_w3_data.add(p0_w5)).v2 };
                            #[allow(clippy::redundant_closure_call)]
                            let p0_w3_element_built = (|| -> ::core::result::Result<
                                Option<i64>,
                                String,
                            > {
                                {
                                    let p0_w7: i32 = i32::from(((p0_w10) as usize) != 0);
                                    let p0_w6 = if p0_w7 == 1i32 {
                                        {
                                            let p0_w10_pointer = (p0_w10)
                                                as *mut perftest_flat::OwnedResource;
                                            let p0_w9: perftest_flat::OwnedResource = {
                                                let p0_w9 = unsafe { Box::from_raw(p0_w10_pointer) };
                                                {
                                                    ownership_accepted.set(ownership_accepted.get() + (1usize));
                                                }
                                                *p0_w9
                                            };
                                            let p0_w8_argument_0 = p0_w9;
                                            let p0_w12_pointer = (p0_w12)
                                                as *mut perftest_flat::InlinePacket;
                                            let p0_w12_empty = if p0_w7 == 1i32 {
                                                Some(perftest_flat::inline_packet_empty())
                                            } else {
                                                None
                                            };
                                            let p0_w11 = if p0_w7 == 1i32 {
                                                {
                                                    let value = unsafe {
                                                        ::core::ptr::replace(
                                                            p0_w12_pointer,
                                                            p0_w12_empty.expect("prepared empty owner"),
                                                        )
                                                    };
                                                    {
                                                        ownership_accepted.set(ownership_accepted.get() + (1usize));
                                                    }
                                                    Some(value)
                                                }
                                            } else {
                                                None
                                            };
                                            let p0_w11: perftest_flat::InlinePacket = p0_w11
                                                .expect("active inline owner");
                                            let p0_w8_argument_1 = p0_w11;
                                            let p0_w14_pointer = (p0_w14)
                                                as *mut perftest_flat::OwnedResource;
                                            let p0_w13: perftest_flat::OwnedResource = {
                                                let p0_w13 = unsafe { Box::from_raw(p0_w14_pointer) };
                                                {
                                                    ownership_accepted.set(ownership_accepted.get() + (1usize));
                                                }
                                                *p0_w13
                                            };
                                            let p0_w8_argument_2 = p0_w13;
                                            let p0_w8: i64 = perftest_flat::inline_input_mixed(
                                                p0_w8_argument_0,
                                                p0_w8_argument_1,
                                                p0_w8_argument_2,
                                            );
                                            let p0_w6_payload = p0_w8;
                                            Some(p0_w6_payload)
                                        }
                                    } else {
                                        None
                                    };
                                    let p0_w3_row_value = p0_w6;
                                    Ok(p0_w3_row_value)
                                }
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
                    let p0_w0_payload = p0_w2;
                    Some(p0_w0_payload)
                }
            } else {
                None
            };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::inline_inputs_optional(
                __prebindgen_argument_0,
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
            let result = owner_inline_inputs_optional_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    std::ptr::write(ownership, ownership_accepted.get());
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
pub struct __PbRow_inline_inputs_packets_p0_w1 {
    pub v0: *mut __PbOwned_packet,
}
#[repr(C)]
pub struct owner_inline_inputs_packets_result {
    pub v0: usize,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_inline_inputs_packets(
    p0_w1: __PbRows,
    out0: *mut usize,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_inputs_packets_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<perftest_flat::InlinePacket> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data
                    as *const __PbRow_inline_inputs_packets_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        perftest_flat::InlinePacket,
                        String,
                    > {
                        {
                            let p0_w5_pointer = (p0_w5)
                                as *mut perftest_flat::InlinePacket;
                            let p0_w5_empty = if true {
                                Some(perftest_flat::inline_packet_empty())
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
                                    {
                                        ownership_accepted.set(ownership_accepted.get() + (1usize));
                                    }
                                    Some(value)
                                }
                            } else {
                                None
                            };
                            let p0_w4: perftest_flat::InlinePacket = p0_w4
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
            let source_value = perftest_flat::inline_inputs_packets(
                __prebindgen_argument_0,
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
            let result = owner_inline_inputs_packets_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    std::ptr::write(ownership, ownership_accepted.get());
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
pub struct __PbRow_inline_inputs_pair_p0_w1 {
    pub v0: *mut __PbOwned_packet,
    pub v1: *mut __PbOwned_packet,
}
#[repr(C)]
pub struct owner_inline_inputs_pair_result {
    pub v0: i64,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_inline_inputs_pair(
    p0_w1: __PbRows,
    out0: *mut i64,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<owner_inline_inputs_pair_result>::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<i64> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_inline_inputs_pair_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w6 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    let p0_w8 = unsafe { (*p0_w1_data.add(p0_w3)).v1 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<i64, String> {
                        {
                            let p0_w6_pointer = (p0_w6)
                                as *mut perftest_flat::InlinePacket;
                            let p0_w6_empty = if true {
                                Some(perftest_flat::inline_packet_empty())
                            } else {
                                None
                            };
                            let p0_w5 = if true {
                                {
                                    let value = unsafe {
                                        ::core::ptr::replace(
                                            p0_w6_pointer,
                                            p0_w6_empty.expect("prepared empty owner"),
                                        )
                                    };
                                    {
                                        ownership_accepted.set(ownership_accepted.get() + (1usize));
                                    }
                                    Some(value)
                                }
                            } else {
                                None
                            };
                            let p0_w5: perftest_flat::InlinePacket = p0_w5
                                .expect("active inline owner");
                            let p0_w4_argument_0 = p0_w5;
                            let p0_w8_pointer = (p0_w8)
                                as *mut perftest_flat::InlinePacket;
                            let p0_w8_empty = if true {
                                Some(perftest_flat::inline_packet_empty())
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
                                    {
                                        ownership_accepted.set(ownership_accepted.get() + (1usize));
                                    }
                                    Some(value)
                                }
                            } else {
                                None
                            };
                            let p0_w7: perftest_flat::InlinePacket = p0_w7
                                .expect("active inline owner");
                            let p0_w4_argument_1 = p0_w7;
                            let p0_w4: i64 = perftest_flat::inline_input_pair(
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
            let source_value = perftest_flat::inline_inputs_pair(
                __prebindgen_argument_0,
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
            let result = owner_inline_inputs_pair_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    std::ptr::write(ownership, ownership_accepted.get());
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
pub struct owner_inline_inputs_reset_result {
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
pub unsafe extern "C" fn owner_inline_inputs_reset(p0_w1: usize, p1_w1: usize) {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_inputs_reset_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: usize = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
            let p1_w0: usize = ::core::convert::Into::into(p1_w1);
            let __prebindgen_argument_1 = p1_w0;
            let source_value = perftest_flat::inline_inputs_reset(
                __prebindgen_argument_0,
                __prebindgen_argument_1,
            );
            let result = owner_inline_inputs_reset_result {
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
#[derive(Clone, Copy)]
pub struct __PbRow_inline_inputs_texts_p0_w1 {
    pub v0: *mut __PbOwned_text,
}
#[repr(C)]
pub struct owner_inline_inputs_texts_result {
    pub v0: usize,
}
/**# Safety
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
    clippy::needless_borrow,
    clippy::let_unit_value,
    clippy::unnecessary_cast,
    clippy::useless_conversion
)]
#[no_mangle]
pub unsafe extern "C" fn owner_inline_inputs_texts(
    p0_w1: __PbRows,
    out0: *mut usize,
    ownership: *mut usize,
    error: *mut *mut std::ffi::c_char,
) -> bool {
    let ownership_accepted = ::core::cell::Cell::new(0usize);
    #[allow(unused_mut)]
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_inputs_texts_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<String> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_inline_inputs_texts_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        String,
                        String,
                    > {
                        {
                            let p0_w5_pointer = (p0_w5) as *mut String;
                            let p0_w5_empty = if true {
                                Some(<String as ::core::default::Default>::default())
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
                                    {
                                        ownership_accepted.set(ownership_accepted.get() + (1usize));
                                    }
                                    Some(value)
                                }
                            } else {
                                None
                            };
                            let p0_w4: String = p0_w4.expect("active inline owner");
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
            let source_value = perftest_flat::inline_inputs_texts(
                __prebindgen_argument_0,
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
            let result = owner_inline_inputs_texts_result {
                v0: ret_w2,
            };
            std::ptr::write(output, result);
            let _: () = {
                success_committed = true;
            };
            Ok(())
        }
    });
    std::ptr::write(ownership, ownership_accepted.get());
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
pub struct owner_inline_mixed_rows_sum_result {
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
pub unsafe extern "C" fn owner_inline_mixed_rows_sum(
    p0_callback: owner_callback_805117dbed50edfb,
) -> u64 {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_mixed_rows_sum_result,
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
                                    as *const __PbRow_callback_805117dbed50edfb_callback_return_w1)
                                    .add(index)))
                                    .v0
                            };
                            let callback_return_w8 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_805117dbed50edfb_callback_return_w1)
                                    .add(index)))
                                    .v1
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
                                                        __PbRow_callback_805117dbed50edfb_callback_return_w1)
                                                        .add(index))).v0
                                                    ) as *const __PbOwned_packet)
                                                        .cast::<perftest_flat::InlinePacket>(),
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
                                                    callback_return_w8 as *mut perftest_flat::InlineChild,
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
                                perftest_flat::InlineMixed,
                            > = Vec::new();
                            if true {
                                let callback_return_w2 = callback_return_w1.len;
                                let callback_return_w1_data = callback_return_w1.data
                                    as *const __PbRow_callback_805117dbed50edfb_callback_return_w1;
                                callback_return_w0.reserve_exact(callback_return_w2);
                                for callback_return_w3 in 0..callback_return_w2 {
                                    let callback_return_w6 = unsafe {
                                        (*callback_return_w1_data.add(callback_return_w3)).v0
                                    };
                                    let callback_return_w8 = unsafe {
                                        (*callback_return_w1_data.add(callback_return_w3)).v1
                                    };
                                    #[allow(clippy::redundant_closure_call)]
                                    let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                        perftest_flat::InlineMixed,
                                        String,
                                    > {
                                        {
                                            #[allow(unused_parens)]
                                            let callback_return_w6_storage = (callback_return_w6);
                                            let callback_return_w5 = if true {
                                                let callback_return_w6_pointer = ::core::ptr::addr_of!(
                                                    callback_return_w6_storage
                                                )
                                                    .cast::<perftest_flat::InlinePacket>();
                                                let value = unsafe {
                                                    ::core::ptr::read(callback_return_w6_pointer)
                                                };
                                                {
                                                    __pb_callback_consumed
                                                        .set(__pb_callback_consumed.get() + (1usize));
                                                }
                                                Some(value)
                                            } else {
                                                None
                                            };
                                            let callback_return_w5: perftest_flat::InlinePacket = callback_return_w5
                                                .expect("active inline owner");
                                            let callback_return_w4_argument_0 = callback_return_w5;
                                            let callback_return_w8_pointer = (callback_return_w8)
                                                as *mut perftest_flat::InlineChild;
                                            let callback_return_w7: perftest_flat::InlineChild = {
                                                let callback_return_w7 = unsafe {
                                                    Box::from_raw(callback_return_w8_pointer)
                                                };
                                                {
                                                    __pb_callback_consumed
                                                        .set(__pb_callback_consumed.get() + (1usize));
                                                }
                                                *callback_return_w7
                                            };
                                            let callback_return_w4_argument_1 = callback_return_w7;
                                            let callback_return_w4: perftest_flat::InlineMixed = perftest_flat::inline_mixed_new(
                                                callback_return_w4_argument_0,
                                                callback_return_w4_argument_1,
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
                            "owner_callback_805117dbed50edfb",
                            "()->Vec < perftest_flat :: InlineMixed >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
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
pub unsafe extern "C" fn owner_inline_numbers() -> __PbOwned_numbers {
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
pub unsafe extern "C" fn owner_inline_numbers_capacity(
    p0_w1: *const __PbOwned_numbers,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_numbers_capacity_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut Vec<i64>;
            let p0_w0: &Vec<i64> = unsafe { &*(p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
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
pub unsafe extern "C" fn owner_inline_numbers_new() -> __PbOwned_numbers {
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
pub unsafe extern "C" fn owner_inline_numbers_take(
    p0_w1: *mut __PbOwned_numbers,
) -> i64 {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_numbers_take_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut Vec<i64>;
            let p0_w1_empty = if true {
                Some(<Vec<i64> as ::core::default::Default>::default())
            } else {
                None
            };
            let p0_w0 = if true {
                {
                    let value = unsafe {
                        ::core::ptr::replace(
                            p0_w1_pointer,
                            p0_w1_empty.expect("prepared empty owner"),
                        )
                    };
                    {}
                    Some(value)
                }
            } else {
                None
            };
            let p0_w0: Vec<i64> = p0_w0.expect("active inline owner");
            let __prebindgen_argument_0 = p0_w0;
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
pub struct owner_inline_packet_bytes_address_result {
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
pub unsafe extern "C" fn owner_inline_packet_bytes_address(
    p0_w1: *const __PbOwned_packet,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packet_bytes_address_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut perftest_flat::InlinePacket;
            let p0_w0: &perftest_flat::InlinePacket = unsafe { &*(p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::inline_packet_bytes_address(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_packet_bytes_address_result {
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
pub unsafe extern "C" fn owner_inline_packet_callback(
    p0_callback: owner_callback_0c47d5ea09f8970c,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packet_callback_result,
    >::uninit();
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
            let p0_callback_value = move |
                callback_source_0: perftest_flat::InlinePacket|
            {
                let result = ::prebindgen_runtime::boundary(|| {
                    let callback_arg_0_w0 = __PbOwned_packet::guard(callback_source_0);
                    let __pb_callback_consumed = ::core::cell::Cell::new(0usize);
                    let __prebindgen_input_consumed = &__pb_callback_consumed;
                    let callback_argument_0 = callback_arg_0_w0.0;
                    {
                        ::core::mem::forget(callback_arg_0_w0);
                    }
                    let raw_result = unsafe {
                        p0_callback_run(p0_callback_owner.context(), callback_argument_0)
                    };
                    let callback_return_w1 = raw_result;
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
                                            ::core::ptr::read(
                                                (&callback_return_w1 as *const __PbOwned_packet)
                                                    .cast::<perftest_flat::InlinePacket>(),
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
                    let callback_outcome: ::prebindgen_runtime::Result<_> = {
                        #[allow(unused_parens)]
                        let callback_return_w1_storage = callback_return_w1;
                        let callback_return_w0 = if true {
                            let callback_return_w1_pointer = ::core::ptr::addr_of!(
                                callback_return_w1_storage
                            )
                                .cast::<perftest_flat::InlinePacket>();
                            let value = unsafe {
                                ::core::ptr::read(callback_return_w1_pointer)
                            };
                            {
                                __pb_callback_consumed
                                    .set(__pb_callback_consumed.get() + (1usize));
                            }
                            Some(value)
                        } else {
                            None
                        };
                        let callback_return_w0: perftest_flat::InlinePacket = callback_return_w0
                            .expect("active inline owner");
                        let callback_return_value = callback_return_w0;
                        Ok(callback_return_value)
                    };
                    let callback_decoded = callback_outcome?;
                    callback_return_cleanup.commit();
                    Ok(callback_decoded)
                });
                result
                    .unwrap_or_else(|error| {
                        ::prebindgen_runtime::report_callback_error(
                            "owner_callback_0c47d5ea09f8970c",
                            "(perftest_flat :: InlinePacket)->perftest_flat :: InlinePacket",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
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
pub unsafe extern "C" fn owner_inline_packet_drops() -> usize {
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
pub struct owner_inline_packet_len_result {
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
pub unsafe extern "C" fn owner_inline_packet_len(
    p0_w1: *const __PbOwned_packet,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<owner_inline_packet_len_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut perftest_flat::InlinePacket;
            let p0_w0: &perftest_flat::InlinePacket = unsafe { &*(p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::inline_packet_len(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = owner_inline_packet_len_result {
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
pub unsafe extern "C" fn owner_inline_packet_new() -> __PbOwned_packet {
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
pub unsafe extern "C" fn owner_inline_packet_optional(
    p0_w1: u8,
    out1: *mut __PbOwned_packet,
) -> i32 {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packet_optional_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: bool = ::core::convert::Into::into(p0_w1 != 0);
            let __prebindgen_argument_0 = p0_w0;
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
pub unsafe extern "C" fn owner_inline_packet_ref(
    p0_w1: *const __PbOwned_packet,
) -> *const perftest_flat::InlineChild {
    let mut storage = std::mem::MaybeUninit::<owner_inline_packet_ref_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut perftest_flat::InlinePacket;
            let p0_w0: &perftest_flat::InlinePacket = unsafe { &*(p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
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
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
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
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packet_result_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
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
pub unsafe extern "C" fn owner_inline_packet_take(
    p0_w1: *mut __PbOwned_packet,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<owner_inline_packet_take_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut perftest_flat::InlinePacket;
            let p0_w1_empty = if true {
                Some(perftest_flat::inline_packet_empty())
            } else {
                None
            };
            let p0_w0 = if true {
                {
                    let value = unsafe {
                        ::core::ptr::replace(
                            p0_w1_pointer,
                            p0_w1_empty.expect("prepared empty owner"),
                        )
                    };
                    {}
                    Some(value)
                }
            } else {
                None
            };
            let p0_w0: perftest_flat::InlinePacket = p0_w0.expect("active inline owner");
            let __prebindgen_argument_0 = p0_w0;
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
pub struct owner_inline_packet_text_address_result {
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
pub unsafe extern "C" fn owner_inline_packet_text_address(
    p0_w1: *const __PbOwned_packet,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packet_text_address_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut perftest_flat::InlinePacket;
            let p0_w0: &perftest_flat::InlinePacket = unsafe { &*(p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::inline_packet_text_address(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_inline_packet_text_address_result {
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
Success, ownership and nonnull error outputs must be writable and pairwise disjoint, with no unreleased prior error. Input allocations must satisfy their declared provenance, aliasing and ownership contract. Collection rows and descriptors must remain live and unchanged for the whole call, including source constructor invocations.*/
#[allow(
    clippy::let_and_return,
    clippy::needless_range_loop,
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
    let mut error_reported = false;
    let mut success_committed = false;
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packets_rows_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    let outcome = ::prebindgen_runtime::boundary(|| {
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
            let ret_w3_source: ::std::vec::Vec<perftest_flat::InlinePacket> = ret_w2;
            let ret_w5 = ::std::vec::Vec::len(&ret_w3_source);
            unsafe fn ret_w3_release_row(row: &__PbRow_inline_packets_rows_ret_w3) {
                let _ = ::prebindgen_runtime::boundary(|| {
                    drop((row.v0).into_rust());
                    Ok(())
                });
            }
            let mut ret_w3 = unsafe {
                ::prebindgen_runtime::OutputRows::<
                    __PbRow_inline_packets_rows_ret_w3,
                >::new(ret_w5, ret_w3_release_row)
            };
            for (ret_w6, ret_w4) in ::core::iter::Iterator::enumerate(
                ::core::iter::IntoIterator::into_iter(ret_w3_source),
            ) {
                let ret_w7 = __PbOwned_packet::guard(ret_w4);
                unsafe {
                    ret_w3
                        .write_next(__PbRow_inline_packets_rows_ret_w3 {
                            v0: ret_w7.0,
                        });
                }
                #[allow(
                    clippy::redundant_closure_call,
                    clippy::let_unit_value,
                    clippy::unused_unit
                )]
                let _: () = (|| -> () {
                    ::core::mem::forget(ret_w7);
                })();
                unsafe {
                    ret_w3.commit_row();
                }
            }
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
                ret_w3.commit();
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
pub unsafe extern "C" fn owner_inline_packets_take(p0_w1: __PbRows) -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_packets_take_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let mut p0_w0: Vec<perftest_flat::InlinePacket> = Vec::new();
            if true {
                let p0_w2 = p0_w1.len;
                let p0_w1_data = p0_w1.data as *const __PbRow_inline_packets_take_p0_w1;
                p0_w0.reserve_exact(p0_w2);
                for p0_w3 in 0..p0_w2 {
                    let p0_w5 = unsafe { (*p0_w1_data.add(p0_w3)).v0 };
                    #[allow(clippy::redundant_closure_call)]
                    let p0_w1_element_built = (|| -> ::core::result::Result<
                        perftest_flat::InlinePacket,
                        String,
                    > {
                        {
                            let p0_w5_pointer = (p0_w5)
                                as *mut perftest_flat::InlinePacket;
                            let p0_w5_empty = if true {
                                Some(perftest_flat::inline_packet_empty())
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
                            let p0_w4: perftest_flat::InlinePacket = p0_w4
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
pub unsafe extern "C" fn owner_inline_return_catch(
    p0_callback: owner_callback_c381621a1cf6edc5,
) -> u8 {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_return_catch_result,
    >::uninit();
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
                                            ::core::ptr::read(
                                                (&callback_return_w2 as *const __PbOwned_packet)
                                                    .cast::<perftest_flat::InlinePacket>(),
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
                            #[allow(unused_parens)]
                            let callback_return_w2_storage = callback_return_w2;
                            let callback_return_w1 = if true {
                                let callback_return_w2_pointer = ::core::ptr::addr_of!(
                                    callback_return_w2_storage
                                )
                                    .cast::<perftest_flat::InlinePacket>();
                                let value = unsafe {
                                    ::core::ptr::read(callback_return_w2_pointer)
                                };
                                {
                                    __pb_callback_consumed
                                        .set(__pb_callback_consumed.get() + (1usize));
                                }
                                Some(value)
                            } else {
                                None
                            };
                            let callback_return_w1: perftest_flat::InlinePacket = callback_return_w1
                                .expect("active inline owner");
                            let callback_return_w0_argument_0 = callback_return_w1;
                            let callback_return_w3: char = ::core::convert::Into::into(unsafe {
                                char::from_u32_unchecked(callback_return_w4)
                            });
                            let callback_return_w0_argument_1 = callback_return_w3;
                            let callback_return_w0: perftest_flat::InlineReturn = perftest_flat::inline_return_new(
                                callback_return_w0_argument_0,
                                callback_return_w0_argument_1,
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
                            "owner_callback_c381621a1cf6edc5",
                            "()->perftest_flat :: InlineReturn",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
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
pub unsafe extern "C" fn owner_inline_rows_catch(
    p0_callback: owner_callback_f5d26722508e30c3,
) -> u8 {
    let mut storage = std::mem::MaybeUninit::<owner_inline_rows_catch_result>::uninit();
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
                                    as *const __PbRow_callback_f5d26722508e30c3_callback_return_w1)
                                    .add(index)))
                                    .v0
                            };
                            let callback_return_w8 = unsafe {
                                (*((callback_return_w1.data
                                    as *const __PbRow_callback_f5d26722508e30c3_callback_return_w1)
                                    .add(index)))
                                    .v1
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
                                                        __PbRow_callback_f5d26722508e30c3_callback_return_w1)
                                                        .add(index))).v0
                                                    ) as *const __PbOwned_packet)
                                                        .cast::<perftest_flat::InlinePacket>(),
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
                                perftest_flat::InlineReturn,
                            > = Vec::new();
                            if true {
                                let callback_return_w2 = callback_return_w1.len;
                                let callback_return_w1_data = callback_return_w1.data
                                    as *const __PbRow_callback_f5d26722508e30c3_callback_return_w1;
                                callback_return_w0.reserve_exact(callback_return_w2);
                                for callback_return_w3 in 0..callback_return_w2 {
                                    let callback_return_w6 = unsafe {
                                        (*callback_return_w1_data.add(callback_return_w3)).v0
                                    };
                                    let callback_return_w8 = unsafe {
                                        (*callback_return_w1_data.add(callback_return_w3)).v1
                                    };
                                    #[allow(clippy::redundant_closure_call)]
                                    let callback_return_w1_element_built = (|| -> ::core::result::Result<
                                        perftest_flat::InlineReturn,
                                        String,
                                    > {
                                        {
                                            #[allow(unused_parens)]
                                            let callback_return_w6_storage = (callback_return_w6);
                                            let callback_return_w5 = if true {
                                                let callback_return_w6_pointer = ::core::ptr::addr_of!(
                                                    callback_return_w6_storage
                                                )
                                                    .cast::<perftest_flat::InlinePacket>();
                                                let value = unsafe {
                                                    ::core::ptr::read(callback_return_w6_pointer)
                                                };
                                                {
                                                    __pb_callback_consumed
                                                        .set(__pb_callback_consumed.get() + (1usize));
                                                }
                                                Some(value)
                                            } else {
                                                None
                                            };
                                            let callback_return_w5: perftest_flat::InlinePacket = callback_return_w5
                                                .expect("active inline owner");
                                            let callback_return_w4_argument_0 = callback_return_w5;
                                            let callback_return_w7: char = ::core::convert::Into::into(unsafe {
                                                char::from_u32_unchecked(callback_return_w8)
                                            });
                                            let callback_return_w4_argument_1 = callback_return_w7;
                                            let callback_return_w4: perftest_flat::InlineReturn = perftest_flat::inline_return_new(
                                                callback_return_w4_argument_0,
                                                callback_return_w4_argument_1,
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
                            "owner_callback_f5d26722508e30c3",
                            "()->Vec < perftest_flat :: InlineReturn >",
                            &error,
                        );
                        panic!("callback failed: {}", error)
                    })
            };
            let __prebindgen_argument_0 = p0_callback_value;
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
pub unsafe extern "C" fn owner_inline_text() -> __PbOwned_text {
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
pub unsafe extern "C" fn owner_inline_text_capacity(
    p0_w1: *const __PbOwned_text,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_inline_text_capacity_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut String;
            let p0_w0: &String = unsafe { &*(p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
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
pub unsafe extern "C" fn owner_inline_text_new() -> __PbOwned_text {
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
pub unsafe extern "C" fn owner_inline_text_take(p0_w1: *mut __PbOwned_text) -> usize {
    let mut storage = std::mem::MaybeUninit::<owner_inline_text_take_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut String;
            let p0_w1_empty = if true {
                Some(<String as ::core::default::Default>::default())
            } else {
                None
            };
            let p0_w0 = if true {
                {
                    let value = unsafe {
                        ::core::ptr::replace(
                            p0_w1_pointer,
                            p0_w1_empty.expect("prepared empty owner"),
                        )
                    };
                    {}
                    Some(value)
                }
            } else {
                None
            };
            let p0_w0: String = p0_w0.expect("active inline owner");
            let __prebindgen_argument_0 = p0_w0;
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
#[repr(C)]
pub struct owner_mutable_bytes_vector_expanded_result {
    pub v0: __PbOwned_bytes,
    pub v1: __PbBufferMut,
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
pub unsafe extern "C" fn owner_mutable_bytes_vector_expanded(
    p0_w1: usize,
    out1: *mut __PbBufferMut,
) -> __PbOwned_bytes {
    let mut storage = std::mem::MaybeUninit::<
        owner_mutable_bytes_vector_expanded_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w0: usize = ::core::convert::Into::into(p0_w1);
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::mutable_bytes_vector_expanded(
                __prebindgen_argument_0,
            );
            let mut ret_w0 = __PbOwned_bytes::guard(source_value);
            let ret_w1 = __pb_owned_bytes_view_mut(unsafe {
                &mut *(::core::ptr::addr_of_mut!(ret_w0.0).cast::<Vec<u8>>())
            });
            let ret_w2 = ret_w1;
            let ret_w2 = __PbBufferMut {
                data: ret_w2.as_mut_ptr().cast::<u8>(),
                len: ret_w2.len(),
            };
            let result = owner_mutable_bytes_vector_expanded_result {
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
pub struct owner_owned_resource_drops_result {
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
pub unsafe extern "C" fn owner_owned_resource_drops() -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_owned_resource_drops_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let source_value = perftest_flat::owned_resource_drops();
            let ret_w0 = source_value;
            let result = owner_owned_resource_drops_result {
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
pub struct owner_owned_resource_id_result {
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
pub unsafe extern "C" fn owner_owned_resource_id(
    p0_w1: *const perftest_flat::OwnedResource,
) -> i64 {
    let mut storage = std::mem::MaybeUninit::<owner_owned_resource_id_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut perftest_flat::OwnedResource;
            let p0_w0: &perftest_flat::OwnedResource = unsafe { &(*p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::owned_resource_id(__prebindgen_argument_0);
            let ret_w0 = source_value;
            let result = owner_owned_resource_id_result {
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
pub struct owner_shared_input_bytes_result {
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
pub unsafe extern "C" fn owner_shared_input_bytes(
    p0_w1: *const __PbOwned_bytes,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<owner_shared_input_bytes_result>::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut Vec<u8>;
            let p0_w0: &Vec<u8> = unsafe { &*(p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::shared_input_bytes(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_shared_input_bytes_result {
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
pub struct owner_shared_input_string_result {
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
pub unsafe extern "C" fn owner_shared_input_string(
    p0_w1: *const __PbOwned_text,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_shared_input_string_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut String;
            let p0_w0: &String = unsafe { &*(p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::shared_input_string(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_shared_input_string_result {
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
pub struct owner_shared_input_vector_result {
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
pub unsafe extern "C" fn owner_shared_input_vector(
    p0_w1: *const __PbOwned_numbers,
) -> usize {
    let mut storage = std::mem::MaybeUninit::<
        owner_shared_input_vector_result,
    >::uninit();
    let output = storage.as_mut_ptr();
    if let Err(error) = ::prebindgen_runtime::boundary(|| {
        {
            let p0_w1_pointer = p0_w1 as *mut Vec<i64>;
            let p0_w0: &Vec<i64> = unsafe { &*(p0_w1_pointer) };
            let __prebindgen_argument_0 = p0_w0;
            let source_value = perftest_flat::shared_input_vector(
                __prebindgen_argument_0,
            );
            let ret_w0 = source_value;
            let result = owner_shared_input_vector_result {
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
    ::core::mem::size_of:: < perftest_flat::InlineCallbackRow > () != 0,
    "flat object `InlineCallbackRow` must have nonzero size"
);
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
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::InputTag > () != 0,
    "flat object `InputTag` must have nonzero size"
);
const _: () = assert!(
    ::core::mem::size_of:: < perftest_flat::OwnedResource > () != 0,
    "flat object `OwnedResource` must have nonzero size"
);
const _: () = assert!(
    perftest_flat::InputTag::Low as i128 == - 3i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::InputTag::Normal as i128 == 0i32 as i128,
    "enum discriminant differs from captured source"
);
const _: () = assert!(
    perftest_flat::InputTag::High as i128 == 17i32 as i128,
    "enum discriminant differs from captured source"
);
