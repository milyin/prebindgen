extern "C" {
    fn malloc(size: usize) -> *mut ::core::ffi::c_void;
    fn free(ptr: *mut ::core::ffi::c_void);
}
#[allow(non_snake_case, dead_code)]
pub(crate) fn __cbg_alloc_cstr(s: ::std::string::String) -> *mut ::core::ffi::c_char {
    let c = ::std::ffi::CString::new(s).unwrap_or_default();
    let bytes = c.as_bytes_with_nul();
    unsafe {
        let p = malloc(bytes.len()) as *mut u8;
        if p.is_null() {
            return ::core::ptr::null_mut();
        }
        ::core::ptr::copy_nonoverlapping(bytes.as_ptr(), p, bytes.len());
        p as *mut ::core::ffi::c_char
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn perftest_free(p: *mut ::core::ffi::c_void) {
    free(p);
}
#[allow(non_snake_case, dead_code)]
pub(crate) unsafe fn __cbg_alloc_array<W>(v: ::std::vec::Vec<W>) -> (*mut W, usize) {
    let n = v.len();
    if n == 0 {
        return (::core::ptr::null_mut(), 0);
    }
    let p = malloc(n.wrapping_mul(::core::mem::size_of::<W>())) as *mut W;
    if p.is_null() {
        return (::core::ptr::null_mut(), 0);
    }
    for (i, e) in v.into_iter().enumerate() {
        ::core::ptr::write(p.add(i), e);
    }
    (p, n)
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct payload_handler_t {
    _private: [u8; 0],
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn payload_handler_drop(this_: *mut payload_handler_t) {
    if !this_.is_null() {
        drop(::std::boxed::Box::from_raw(this_ as *mut perftest_flat::PayloadHandler));
    }
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct storage_t {
    _private: [u8; 0],
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn storage_drop(this_: *mut storage_t) {
    if !this_.is_null() {
        drop(::std::boxed::Box::from_raw(this_ as *mut perftest_flat::Storage));
    }
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct string_t {
    _private: [u8; 0],
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn string_drop(this_: *mut string_t) {
    if !this_.is_null() {
        drop(::std::boxed::Box::from_raw(this_ as *mut ::std::string::String));
    }
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct payload_t {
    pub id: i64,
    pub seq: i32,
    pub value: f64,
    pub flag: bool,
    pub label: *mut string_t,
}
const _: () = {
    assert!(
        ::core::mem::size_of:: < perftest_flat::Payload > () == ::core::mem::size_of:: <
        payload_t > (), "value_opaque: Rust type and opaque counterpart differ in size"
    );
    assert!(
        ::core::mem::align_of:: < perftest_flat::Payload > () == ::core::mem::align_of::
        < payload_t > (),
        "value_opaque: Rust type and opaque counterpart differ in alignment"
    );
};
impl ::prebindgen::Transmute for payload_t {
    type Rust = perftest_flat::Payload;
    #[inline]
    fn from_rust(value: Self::Rust) -> Self {
        let __v = ::core::mem::ManuallyDrop::new(value);
        unsafe { ::core::ptr::read(&*__v as *const Self::Rust as *const Self) }
    }
    #[inline]
    fn into_rust(self) -> Self::Rust {
        let __v = ::core::mem::ManuallyDrop::new(self);
        unsafe { ::core::ptr::read(&*__v as *const Self as *const Self::Rust) }
    }
    #[inline]
    fn as_rust(&self) -> &Self::Rust {
        unsafe { &*(self as *const Self as *const Self::Rust) }
    }
    #[inline]
    fn as_rust_mut(&mut self) -> &mut Self::Rust {
        unsafe { &mut *(self as *mut Self as *mut Self::Rust) }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn payload_drop(this_: *mut payload_t) {
    if !this_.is_null() {
        ::core::ptr::drop_in_place(
            <payload_t as ::prebindgen::Transmute>::as_rust_mut(&mut *this_),
        );
    }
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct closure_payload_t {
    pub context: *mut ::core::ffi::c_void,
    pub call: ::core::option::Option<
        unsafe extern "C" fn(*const payload_t, *mut ::core::ffi::c_void),
    >,
    pub drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_Payload(
    v: *mut payload_t,
) -> ::core::result::Result<perftest_flat::Payload, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null Payload value passed by value"),
        );
    }
    let __live = <payload_t as ::prebindgen::Transmute>::into_rust(::core::ptr::read(v));
    (*v).label = ::core::ptr::null_mut();
    ::core::result::Result::Ok(__live)
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_PayloadHandler(
    v: *mut payload_handler_t,
) -> ::core::result::Result<perftest_flat::PayloadHandler, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null PayloadHandler handle passed by value"),
        );
    }
    ::core::result::Result::Ok(
        *::std::boxed::Box::from_raw(v as *mut perftest_flat::PayloadHandler),
    )
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_Storage(
    v: *mut storage_t,
) -> ::core::result::Result<perftest_flat::Storage, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null Storage handle passed by value"),
        );
    }
    ::core::result::Result::Ok(
        *::std::boxed::Box::from_raw(v as *mut perftest_flat::Storage),
    )
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_String(
    v: *mut string_t,
) -> ::core::result::Result<::std::string::String, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null String handle passed by value"),
        );
    }
    ::core::result::Result::Ok(
        *::std::boxed::Box::from_raw(v as *mut ::std::string::String),
    )
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in___Payload<'a>(
    v: *const payload_t,
) -> ::core::result::Result<&'a perftest_flat::Payload, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null Payload pointer"),
        );
    }
    ::core::result::Result::Ok(&*(v as *const perftest_flat::Payload))
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in___PayloadHandler<'a>(
    v: *const payload_handler_t,
) -> ::core::result::Result<&'a perftest_flat::PayloadHandler, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null PayloadHandler pointer"),
        );
    }
    ::core::result::Result::Ok(&*(v as *const perftest_flat::PayloadHandler))
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in___Storage<'a>(
    v: *const storage_t,
) -> ::core::result::Result<&'a perftest_flat::Storage, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null Storage pointer"),
        );
    }
    ::core::result::Result::Ok(&*(v as *const perftest_flat::Storage))
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in___String<'a>(
    v: *const string_t,
) -> ::core::result::Result<&'a ::std::string::String, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null String pointer"),
        );
    }
    ::core::result::Result::Ok(&*(v as *const ::std::string::String))
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in___mut_MaybeUninit___Payload__<'a>(
    v: *mut payload_t,
) -> ::core::result::Result<
    &'a mut ::core::mem::MaybeUninit<perftest_flat::Payload>,
    ::std::string::String,
> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null Payload pointer"),
        );
    }
    ::core::result::Result::Ok(
        &mut *(v as *mut ::core::mem::MaybeUninit<perftest_flat::Payload>),
    )
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in___mut_Payload<'a>(
    v: *mut payload_t,
) -> ::core::result::Result<&'a mut perftest_flat::Payload, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null Payload pointer"),
        );
    }
    ::core::result::Result::Ok(&mut *(v as *mut perftest_flat::Payload))
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in___mut_Storage<'a>(
    v: *mut storage_t,
) -> ::core::result::Result<&'a mut perftest_flat::Storage, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null Storage pointer"),
        );
    }
    ::core::result::Result::Ok(&mut *(v as *mut perftest_flat::Storage))
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in___str<'a>(
    v: *const ::core::ffi::c_char,
) -> ::core::result::Result<&'a str, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null pointer passed for str argument"),
        );
    }
    match ::std::ffi::CStr::from_ptr(v).to_str() {
        ::core::result::Result::Ok(s) => ::core::result::Result::Ok(s),
        ::core::result::Result::Err(_) => {
            ::core::result::Result::Err(
                ::std::string::String::from("invalid UTF-8 in str argument"),
            )
        }
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_in_bool(v: bool) -> bool {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_closure_payload_t(
    c: closure_payload_t,
) -> impl Fn(&perftest_flat::Payload) + Send + Sync + 'static {
    struct __Ctx {
        context: *mut ::core::ffi::c_void,
        drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
    }
    unsafe impl ::core::marker::Send for __Ctx {}
    unsafe impl ::core::marker::Sync for __Ctx {}
    impl ::core::ops::Drop for __Ctx {
        fn drop(&mut self) {
            if let ::core::option::Option::Some(__d) = self.drop {
                unsafe { __d(self.context) }
            }
        }
    }
    let __call = c.call;
    let __ctx = ::std::sync::Arc::new(__Ctx {
        context: c.context,
        drop: c.drop,
    });
    move |__a0: &perftest_flat::Payload| {
        let __w0 = __cbg_out_ref_Payload(__a0);
        if let ::core::option::Option::Some(__f) = __call {
            unsafe { __f(__w0, __ctx.context) }
        }
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_in_f64(v: f64) -> f64 {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_in_i32(v: i32) -> i32 {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_in_i64(v: i64) -> i64 {
    v
}
#[allow(non_snake_case, dead_code, unused_variables)]
pub(crate) fn __cbg_in_str() {}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __cbg_inmark_slice_Payload() {}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_Payload(v: perftest_flat::Payload) -> payload_t {
    <payload_t as ::prebindgen::Transmute>::from_rust(v)
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_PayloadHandler(
    v: perftest_flat::PayloadHandler,
) -> *mut payload_handler_t {
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut payload_handler_t
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_Storage(v: perftest_flat::Storage) -> *mut storage_t {
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut storage_t
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_String(v: ::std::string::String) -> *mut string_t {
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut string_t
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_bool(v: bool) -> bool {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_f64(v: f64) -> f64 {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_i32(v: i32) -> i32 {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_i64(v: i64) -> i64 {
    v
}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) unsafe fn __cbg_out_ref_Payload(
    v: &perftest_flat::Payload,
) -> *const payload_t {
    v as *const perftest_flat::Payload as *const payload_t
}
#[allow(non_snake_case, dead_code, unused_variables)]
pub(crate) fn __cbg_out_unit(v: ()) {}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_usize(v: usize) -> usize {
    v
}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __cbg_outmark_vec_Payload() {}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn payload_handler_new(
    f: closure_payload_t,
) -> *mut payload_handler_t {
    let f = __cbg_in_closure_payload_t(f);
    let __v = perftest_flat::payload_handler_new(f);
    let __ret: *mut payload_handler_t;
    __ret = __cbg_out_PayloadHandler(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_callback(
    s: *const storage_t,
    handler: *const payload_handler_t,
) {
    let s = match __cbg_in___Storage(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let handler = match __cbg_in___PayloadHandler(handler) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    perftest_flat::storage_callback(s, handler);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_get(s: *const storage_t) -> payload_t {
    let s = match __cbg_in___Storage(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = perftest_flat::storage_get(s);
    let __ret: payload_t;
    __ret = __cbg_out_Payload(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_get_into_init(
    s: *const storage_t,
    payload: *mut payload_t,
) {
    let s = match __cbg_in___Storage(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let payload = match __cbg_in___mut_Payload(payload) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    perftest_flat::storage_get_into_init(s, payload);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_get_into_uninit(
    s: *const storage_t,
    payload: *mut payload_t,
) {
    let s = match __cbg_in___Storage(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let payload = match __cbg_in___mut_MaybeUninit___Payload__(payload) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    perftest_flat::storage_get_into_uninit(s, payload);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_get_vec(
    s: *const storage_t,
    len: *mut usize,
) -> *mut payload_t {
    let s = match __cbg_in___Storage(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = perftest_flat::storage_get_vec(s);
    let __ret: *mut payload_t;
    let __arr: ::std::vec::Vec<payload_t> = __v
        .into_iter()
        .map(__cbg_out_Payload)
        .collect();
    let (__p, __n) = __cbg_alloc_array(__arr);
    __ret = __p;
    *len = __n;
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_new() -> *mut storage_t {
    let __v = perftest_flat::storage_new();
    let __ret: *mut storage_t;
    __ret = __cbg_out_Storage(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_put_by_read(
    s: *mut storage_t,
    payload: *const payload_t,
) {
    let s = match __cbg_in___mut_Storage(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let payload = match __cbg_in___Payload(payload) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    perftest_flat::storage_put_by_read(s, payload);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_put_by_read_and_update(
    s: *mut storage_t,
    payload: *mut payload_t,
) {
    let s = match __cbg_in___mut_Storage(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let payload = match __cbg_in___mut_Payload(payload) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    perftest_flat::storage_put_by_read_and_update(s, payload);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_put_by_take(
    s: *mut storage_t,
    payload: *mut payload_t,
) {
    let s = match __cbg_in___mut_Storage(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let payload = match __cbg_in_Payload(payload) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    perftest_flat::storage_put_by_take(s, payload);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_put_slice(
    s: *mut storage_t,
    payloads: *const payload_t,
    payloads_len: usize,
) {
    let s = match __cbg_in___mut_Storage(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let payloads: &[perftest_flat::Payload] = if payloads.is_null() {
        &[]
    } else {
        ::core::slice::from_raw_parts(
            payloads as *const perftest_flat::Payload,
            payloads_len,
        )
    };
    perftest_flat::storage_put_slice(s, payloads);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn string_len(s: *const string_t) -> usize {
    let s = match __cbg_in___String(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = perftest_flat::string_len(s);
    let __ret: usize;
    __ret = __cbg_out_usize(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn string_new(s: *const ::core::ffi::c_char) -> *mut string_t {
    let s = match __cbg_in___str(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = perftest_flat::string_new(s);
    let __ret: *mut string_t;
    __ret = __cbg_out_String(__v);
    __ret
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
