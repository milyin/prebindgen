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
pub struct payload_vec_handler_t {
    _private: [u8; 0],
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn payload_vec_handler_drop(this_: *mut payload_vec_handler_t) {
    if !this_.is_null() {
        drop(
            ::std::boxed::Box::from_raw(this_ as *mut perftest_flat::PayloadVecHandler),
        );
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
impl ::prebindgen_c_runtime::Transmute for payload_t {
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
            <payload_t as ::prebindgen_c_runtime::Transmute>::as_rust_mut(&mut *this_),
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
pub(crate) unsafe fn __c_in_convert_wire_to_impl_Fn_Payload_Send_Sync_static_c_invoke_callback_capture_1f115f7d391197ee(
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
        if let ::core::option::Option::Some(__f) = __call {
            let __w0 = __c_out_convert_Payload_c_borrow_shared_output_to_wire_5d954ba915ba18c7(
                __a0,
            );
            unsafe { __f(__w0, __ctx.context) }
        }
    }
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct closure_payload_vec_t {
    pub context: *mut ::core::ffi::c_void,
    pub call: ::core::option::Option<
        unsafe extern "C" fn(*const payload_t, usize, *mut ::core::ffi::c_void),
    >,
    pub drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_impl_Fn_Payload_Send_Sync_static_c_invoke_callback_capture_6d4224db8a8b8070(
    c: closure_payload_vec_t,
) -> impl Fn(&[perftest_flat::Payload]) + Send + Sync + 'static {
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
    move |__a0: &[perftest_flat::Payload]| {
        if let ::core::option::Option::Some(__f) = __call {
            unsafe { __f(__a0.as_ptr() as *const payload_t, __a0.len(), __ctx.context) }
        }
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_Payload_c_borrow_shared_input_9c9dcfae3e193513<
    'a,
>(
    v: *const payload_t,
) -> ::core::result::Result<&'a perftest_flat::Payload, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null Payload pointer"),
        );
    }
    ::core::result::Result::Ok(&*(v as *const perftest_flat::Payload))
}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) unsafe fn __c_out_convert_Payload_c_borrow_shared_output_to_wire_5d954ba915ba18c7(
    v: &perftest_flat::Payload,
) -> *const payload_t {
    v as *const perftest_flat::Payload as *const payload_t
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_PayloadHandler_c_borrow_shared_input_9caa5450154416a9<
    'a,
>(
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
pub(crate) unsafe fn __c_in_convert_wire_to_PayloadVecHandler_c_borrow_shared_input_9362e4165ce71691<
    'a,
>(
    v: *const payload_vec_handler_t,
) -> ::core::result::Result<
    &'a perftest_flat::PayloadVecHandler,
    ::std::string::String,
> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null PayloadVecHandler pointer"),
        );
    }
    ::core::result::Result::Ok(&*(v as *const perftest_flat::PayloadVecHandler))
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_Storage_c_borrow_shared_input_5ce0e04c530e69b0<
    'a,
>(
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
pub(crate) unsafe fn __c_in_convert_wire_to_String_c_borrow_shared_input_1342bc21c35103c2<
    'a,
>(
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
pub(crate) unsafe fn __c_in_convert_wire_to_Payload_c_terminal_input_value_opaque_1025354dd257d200(
    v: *mut payload_t,
) -> ::core::result::Result<perftest_flat::Payload, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null Payload value passed by value"),
        );
    }
    let __live = <payload_t as ::prebindgen_c_runtime::Transmute>::into_rust(
        ::core::ptr::read(v),
    );
    (*v).label = ::core::ptr::null_mut();
    ::core::result::Result::Ok(__live)
}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __c_in_convert_wire_to_Payload_c_slice_input_reinterpret_45029be4ad5be227() {}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_Payload_c_terminal_output_value_opaque_to_wire_ce0f5eae80482d02(
    v: perftest_flat::Payload,
) -> payload_t {
    <payload_t as ::prebindgen_c_runtime::Transmute>::from_rust(v)
}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __c_out_convert_Payload_c_marker_sequence_to_wire_2e9a65c76a50a9dc() {}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_Payload_c_borrow_mutable_uninit_input_ad1f621c5ea6e082<
    'a,
>(
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
pub(crate) unsafe fn __c_in_convert_wire_to_Payload_c_borrow_mutable_input_580822629a1eb85c<
    'a,
>(
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
pub(crate) unsafe fn __c_in_convert_wire_to_Storage_c_borrow_mutable_input_8ce0ec1505140f75<
    'a,
>(
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
pub(crate) unsafe fn __c_in_convert_wire_to_str_c_borrow_str_input_246c2b9955bb6ef2<'a>(
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
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __c_out_convert_Option_Payload_c_marker_optional_to_wire_0d65bf71671af35a() {}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline(always)]
pub(crate) fn __c_out_convert_sequence_Vec_Payload_to_wire_dc890ff48c52049e(
    v: ::std::vec::Vec<perftest_flat::Payload>,
) -> ::std::vec::Vec<payload_t> {
    {
        let __sequence_source = v;
        let mut __sequence_output: ::std::vec::Vec<payload_t> = ::std::vec::Vec::with_capacity(
            (__sequence_source).len(),
        );
        for __sequence_element in __sequence_source.into_iter() {
            let __sequence_part = __c_out_convert_Payload_c_terminal_output_value_opaque_to_wire_ce0f5eae80482d02(
                __sequence_element,
            );
            __sequence_output.push(__sequence_part);
        }
        __sequence_output
    }
}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __c_out_convert_Option_Vec_Payload_c_marker_optional_to_wire_7a54934e42568ea6() {}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_PayloadHandler_c_terminal_input_owned_handle_bbf919ff94219da7(
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
pub(crate) fn __c_out_convert_PayloadHandler_c_terminal_output_owned_handle_to_wire_138a2e3a98f0b6c1(
    v: perftest_flat::PayloadHandler,
) -> *mut payload_handler_t {
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut payload_handler_t
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_PayloadVecHandler_c_terminal_input_owned_handle_b1c246bc53a8947f(
    v: *mut payload_vec_handler_t,
) -> ::core::result::Result<perftest_flat::PayloadVecHandler, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null PayloadVecHandler handle passed by value"),
        );
    }
    ::core::result::Result::Ok(
        *::std::boxed::Box::from_raw(v as *mut perftest_flat::PayloadVecHandler),
    )
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_PayloadVecHandler_c_terminal_output_owned_handle_to_wire_90de88c77a3ca699(
    v: perftest_flat::PayloadVecHandler,
) -> *mut payload_vec_handler_t {
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut payload_vec_handler_t
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_Storage_c_terminal_input_owned_handle_0940f8aedc25ac46(
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
pub(crate) fn __c_out_convert_Storage_c_terminal_output_owned_handle_to_wire_1aad3d8ed7ca5bfa(
    v: perftest_flat::Storage,
) -> *mut storage_t {
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut storage_t
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_String_c_terminal_input_owned_handle_23e92025a440d7c8(
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
pub(crate) fn __c_out_convert_String_c_terminal_output_owned_handle_to_wire_da496556652b0d98(
    v: ::std::string::String,
) -> *mut string_t {
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut string_t
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_bool_c_terminal_output_scalar_to_wire_cc0ad9760da17efd(
    v: bool,
) -> bool {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_usize_c_terminal_output_scalar_to_wire_4b27414858b3ddc9(
    v: usize,
) -> usize {
    v
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn payload_handler_new(
    f: closure_payload_t,
) -> *mut payload_handler_t {
    let f = __c_in_convert_wire_to_impl_Fn_Payload_Send_Sync_static_c_invoke_callback_capture_1f115f7d391197ee(
        f,
    );
    let __v = perftest_flat::payload_handler_new(f);
    let __ret: *mut payload_handler_t;
    __ret = __c_out_convert_PayloadHandler_c_terminal_output_owned_handle_to_wire_138a2e3a98f0b6c1(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn payload_vec_handler_new(
    f: closure_payload_vec_t,
) -> *mut payload_vec_handler_t {
    let f = __c_in_convert_wire_to_impl_Fn_Payload_Send_Sync_static_c_invoke_callback_capture_6d4224db8a8b8070(
        f,
    );
    let __v = perftest_flat::payload_vec_handler_new(f);
    let __ret: *mut payload_vec_handler_t;
    __ret = __c_out_convert_PayloadVecHandler_c_terminal_output_owned_handle_to_wire_90de88c77a3ca699(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_callback(
    s: *const storage_t,
    handler: *const payload_handler_t,
) {
    let s = match __c_in_convert_wire_to_Storage_c_borrow_shared_input_5ce0e04c530e69b0(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let handler = match __c_in_convert_wire_to_PayloadHandler_c_borrow_shared_input_9caa5450154416a9(
        handler,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    perftest_flat::storage_callback(s, handler);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_callback_vec(
    s: *const storage_t,
    handler: *const payload_vec_handler_t,
) {
    let s = match __c_in_convert_wire_to_Storage_c_borrow_shared_input_5ce0e04c530e69b0(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let handler = match __c_in_convert_wire_to_PayloadVecHandler_c_borrow_shared_input_9362e4165ce71691(
        handler,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    perftest_flat::storage_callback_vec(s, handler);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_get(s: *const storage_t, out: *mut payload_t) -> bool {
    let s = match __c_in_convert_wire_to_Storage_c_borrow_shared_input_5ce0e04c530e69b0(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = perftest_flat::storage_get(s);
    let __ret: bool;
    match __v {
        ::core::option::Option::Some(__x) => {
            __ret = true;
            *out = __c_out_convert_Payload_c_terminal_output_value_opaque_to_wire_ce0f5eae80482d02(
                __x,
            );
        }
        ::core::option::Option::None => {
            __ret = false;
        }
    }
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_get_into_init(
    s: *const storage_t,
    payload: *mut payload_t,
) -> bool {
    let s = match __c_in_convert_wire_to_Storage_c_borrow_shared_input_5ce0e04c530e69b0(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let payload = match __c_in_convert_wire_to_Payload_c_borrow_mutable_input_580822629a1eb85c(
        payload,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = perftest_flat::storage_get_into_init(s, payload);
    let __ret: bool;
    __ret = __c_out_convert_bool_c_terminal_output_scalar_to_wire_cc0ad9760da17efd(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_get_into_uninit(
    s: *const storage_t,
    payload: *mut payload_t,
) -> bool {
    let s = match __c_in_convert_wire_to_Storage_c_borrow_shared_input_5ce0e04c530e69b0(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let payload = match __c_in_convert_wire_to_Payload_c_borrow_mutable_uninit_input_ad1f621c5ea6e082(
        payload,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = perftest_flat::storage_get_into_uninit(s, payload);
    let __ret: bool;
    __ret = __c_out_convert_bool_c_terminal_output_scalar_to_wire_cc0ad9760da17efd(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_get_vec(
    s: *const storage_t,
    out: *mut *mut payload_t,
    out_len: *mut usize,
) -> bool {
    let s = match __c_in_convert_wire_to_Storage_c_borrow_shared_input_5ce0e04c530e69b0(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = perftest_flat::storage_get_vec(s);
    let __ret: bool;
    match __v {
        ::core::option::Option::Some(__x) => {
            __ret = true;
            let __arr: ::std::vec::Vec<payload_t> = __c_out_convert_sequence_Vec_Payload_to_wire_dc890ff48c52049e(
                __x,
            );
            let (__p, __n) = __cbg_alloc_array(__arr);
            *out = __p;
            *out_len = __n;
        }
        ::core::option::Option::None => {
            __ret = false;
        }
    }
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_new() -> *mut storage_t {
    let __v = perftest_flat::storage_new();
    let __ret: *mut storage_t;
    __ret = __c_out_convert_Storage_c_terminal_output_owned_handle_to_wire_1aad3d8ed7ca5bfa(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn storage_put_by_read(
    s: *mut storage_t,
    payload: *const payload_t,
) {
    let s = match __c_in_convert_wire_to_Storage_c_borrow_mutable_input_8ce0ec1505140f75(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let payload = match __c_in_convert_wire_to_Payload_c_borrow_shared_input_9c9dcfae3e193513(
        payload,
    ) {
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
    let s = match __c_in_convert_wire_to_Storage_c_borrow_mutable_input_8ce0ec1505140f75(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let payload = match __c_in_convert_wire_to_Payload_c_borrow_mutable_input_580822629a1eb85c(
        payload,
    ) {
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
    let s = match __c_in_convert_wire_to_Storage_c_borrow_mutable_input_8ce0ec1505140f75(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let payload = match __c_in_convert_wire_to_Payload_c_terminal_input_value_opaque_1025354dd257d200(
        payload,
    ) {
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
    let s = match __c_in_convert_wire_to_Storage_c_borrow_mutable_input_8ce0ec1505140f75(
        s,
    ) {
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
    let s = match __c_in_convert_wire_to_String_c_borrow_shared_input_1342bc21c35103c2(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = perftest_flat::string_len(s);
    let __ret: usize;
    __ret = __c_out_convert_usize_c_terminal_output_scalar_to_wire_4b27414858b3ddc9(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn string_new(s: *const ::core::ffi::c_char) -> *mut string_t {
    let s = match __c_in_convert_wire_to_str_c_borrow_str_input_246c2b9955bb6ef2(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = perftest_flat::string_new(s);
    let __ret: *mut string_t;
    __ret = __c_out_convert_String_c_terminal_output_owned_handle_to_wire_da496556652b0d98(
        __v,
    );
    __ret
}
/// The storage capacity limit advertised to bindings (a primitive const).
pub const COVER_MAGIC: i64 = perftest_flat::COVER_MAGIC;
/// The coverage surface's tag string (a string const).
pub const COVER_TAG: &str = perftest_flat::COVER_TAG;
const _: () = {
    konst::assertc_eq!(
        perftest_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
