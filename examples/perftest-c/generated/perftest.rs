extern "C" {
    fn malloc(size: usize) -> *mut ::core::ffi::c_void;
    fn free(ptr: *mut ::core::ffi::c_void);
}
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
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
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
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
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn perftest_free(p: *mut ::core::ffi::c_void) {
    free(p);
}
#[repr(C)]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub struct string_t {
    _private: [u8; 0],
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn string_drop(this_: *mut string_t) {
    if !this_.is_null() {
        drop(::std::boxed::Box::from_raw(this_ as *mut ::std::string::String));
    }
}
#[repr(C)]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub struct storage_t {
    _private: [u8; 0],
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn storage_drop(this_: *mut storage_t) {
    if !this_.is_null() {
        drop(::std::boxed::Box::from_raw(this_ as *mut perftest_flat::Storage));
    }
}
#[repr(C)]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub struct payload_handler_t {
    _private: [u8; 0],
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn payload_handler_drop(this_: *mut payload_handler_t) {
    if !this_.is_null() {
        drop(::std::boxed::Box::from_raw(this_ as *mut perftest_flat::PayloadHandler));
    }
}
#[repr(C)]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub struct payload_vec_handler_t {
    _private: [u8; 0],
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn payload_vec_handler_drop(this_: *mut payload_vec_handler_t) {
    if !this_.is_null() {
        drop(
            ::std::boxed::Box::from_raw(this_ as *mut perftest_flat::PayloadVecHandler),
        );
    }
}
#[repr(C)]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
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
        payload_t > (), "repr_c_struct: Rust type and C mirror differ in size"
    );
    assert!(
        ::core::mem::align_of:: < perftest_flat::Payload > () == ::core::mem::align_of::
        < payload_t > (), "repr_c_struct: Rust type and C mirror differ in alignment"
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
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn payload_drop(this_: *mut payload_t) {
    if !this_.is_null() {
        ::core::ptr::drop_in_place(
            <payload_t as ::prebindgen_c_runtime::Transmute>::as_rust_mut(&mut *this_),
        );
    }
}
#[repr(C)]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub struct closure_payload_t {
    pub context: *mut ::core::ffi::c_void,
    pub call: ::core::option::Option<
        unsafe extern "C" fn(*const payload_t, *mut ::core::ffi::c_void),
    >,
    pub drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub struct closure_payload_vec_t {
    pub context: *mut ::core::ffi::c_void,
    pub call: ::core::option::Option<
        unsafe extern "C" fn(*const payload_t, usize, *mut ::core::ffi::c_void),
    >,
    pub drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn storage_new() -> *mut storage_t {
    let __result = perftest_flat::storage_new();
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(__result)) as *mut storage_t
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn storage_get(s: *const storage_t, out: *mut payload_t) -> bool {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if s.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Storage pointer"),
                );
            }
            &*(s as *const perftest_flat::Storage)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = perftest_flat::storage_get(s);
    match __result {
        ::core::option::Option::Some(__v) => {
            if !out.is_null() {
                ::core::ptr::write(
                    out,
                    <payload_t as ::prebindgen_c_runtime::Transmute>::from_rust(__v),
                );
            }
            true
        }
        ::core::option::Option::None => false,
    }
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn storage_put_by_take(
    s: *mut storage_t,
    payload: *mut payload_t,
) {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if s.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Storage pointer"),
                );
            }
            &mut *(s as *mut perftest_flat::Storage)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let payload = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if payload.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Payload value passed by value"),
                );
            }
            let __live = <payload_t as ::prebindgen_c_runtime::Transmute>::into_rust(
                ::core::ptr::read(payload),
            );
            (*payload).label = ::core::ptr::null_mut();
            __live
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = perftest_flat::storage_put_by_take(s, payload);
    let _ = __result;
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn storage_put_by_read(
    s: *mut storage_t,
    payload: *const payload_t,
) {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if s.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Storage pointer"),
                );
            }
            &mut *(s as *mut perftest_flat::Storage)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let payload = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if payload.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Payload pointer"),
                );
            }
            &*(payload as *const perftest_flat::Payload)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = perftest_flat::storage_put_by_read(s, payload);
    let _ = __result;
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn storage_put_by_read_and_update(
    s: *mut storage_t,
    payload: *mut payload_t,
) {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if s.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Storage pointer"),
                );
            }
            &mut *(s as *mut perftest_flat::Storage)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let payload = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if payload.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Payload pointer"),
                );
            }
            &mut *(payload as *mut perftest_flat::Payload)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = perftest_flat::storage_put_by_read_and_update(s, payload);
    let _ = __result;
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn storage_get_into_init(
    s: *const storage_t,
    payload: *mut payload_t,
) -> bool {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if s.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Storage pointer"),
                );
            }
            &*(s as *const perftest_flat::Storage)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let payload = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if payload.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Payload pointer"),
                );
            }
            &mut *(payload as *mut perftest_flat::Payload)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = perftest_flat::storage_get_into_init(s, payload);
    __result
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn storage_get_into_uninit(
    s: *const storage_t,
    payload: *mut payload_t,
) -> bool {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if s.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Storage pointer"),
                );
            }
            &*(s as *const perftest_flat::Storage)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let payload = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if payload.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Payload pointer"),
                );
            }
            &mut *(payload as *mut ::core::mem::MaybeUninit<perftest_flat::Payload>)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = perftest_flat::storage_get_into_uninit(s, payload);
    __result
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn payload_handler_new(
    f: closure_payload_t,
) -> *mut payload_handler_t {
    let f = {
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
        let __call = f.call;
        let __ctx = ::std::sync::Arc::new(__Ctx {
            context: f.context,
            drop: f.drop,
        });
        move |__a0: &perftest_flat::Payload| {
            let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                let __w0 = __a0 as *const perftest_flat::Payload as *const payload_t;
                if let ::core::option::Option::Some(__f) = __call {
                    unsafe { __f(__w0, __ctx.context) }
                }
                ::core::result::Result::Ok(())
            })();
            if let ::core::result::Result::Err(__err) = __res {}
        }
    };
    let __result = perftest_flat::payload_handler_new(f);
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(__result))
        as *mut payload_handler_t
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn storage_callback(
    s: *const storage_t,
    handler: *const payload_handler_t,
) {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if s.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Storage pointer"),
                );
            }
            &*(s as *const perftest_flat::Storage)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let handler = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if handler.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null PayloadHandler pointer"),
                );
            }
            &*(handler as *const perftest_flat::PayloadHandler)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = perftest_flat::storage_callback(s, handler);
    let _ = __result;
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn string_new(s: *const ::core::ffi::c_char) -> *mut string_t {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if s.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null pointer passed for str argument"),
                );
            }
            match ::std::ffi::CStr::from_ptr(s).to_str() {
                ::core::result::Result::Ok(s) => s,
                ::core::result::Result::Err(_) => {
                    return ::core::result::Result::Err(
                        ::std::string::String::from("invalid UTF-8 in str argument"),
                    );
                }
            }
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = perftest_flat::string_new(s);
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(__result)) as *mut string_t
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn string_len(s: *const string_t) -> usize {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if s.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null String pointer"),
                );
            }
            &*(s as *const ::std::string::String)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = perftest_flat::string_len(s);
    __result
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn storage_put_slice(
    s: *mut storage_t,
    payloads: *const payload_t,
    payloads_len: usize,
) {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if s.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Storage pointer"),
                );
            }
            &mut *(s as *mut perftest_flat::Storage)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let payloads = if payloads.is_null() || payloads_len == 0 {
        &[][..]
    } else {
        ::core::slice::from_raw_parts(
            payloads as *const perftest_flat::Payload,
            payloads_len,
        )
    };
    let __result = perftest_flat::storage_put_slice(s, payloads);
    let _ = __result;
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn storage_get_vec(
    s: *const storage_t,
    out: *mut *mut payload_t,
    out_len: *mut usize,
) -> bool {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if s.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Storage pointer"),
                );
            }
            &*(s as *const perftest_flat::Storage)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = perftest_flat::storage_get_vec(s);
    match __result {
        ::core::option::Option::Some(__v) => {
            let __arr: ::std::vec::Vec<payload_t> = __v
                .into_iter()
                .map(|__e| <payload_t as ::prebindgen_c_runtime::Transmute>::from_rust(
                    __e,
                ))
                .collect();
            let (__p, __n) = __cbg_alloc_array(__arr);
            if !out.is_null() {
                *out = __p;
            }
            if !out_len.is_null() {
                *out_len = __n;
            }
            true
        }
        ::core::option::Option::None => false,
    }
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn payload_vec_handler_new(
    f: closure_payload_vec_t,
) -> *mut payload_vec_handler_t {
    let f = {
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
        let __call = f.call;
        let __ctx = ::std::sync::Arc::new(__Ctx {
            context: f.context,
            drop: f.drop,
        });
        move |__a0: &[perftest_flat::Payload]| {
            let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                let (__w0, __w0_len) = (__a0.as_ptr() as *const payload_t, __a0.len());
                if let ::core::option::Option::Some(__f) = __call {
                    unsafe { __f(__w0, __w0_len, __ctx.context) }
                }
                ::core::result::Result::Ok(())
            })();
            if let ::core::result::Result::Err(__err) = __res {}
        }
    };
    let __result = perftest_flat::payload_vec_handler_new(f);
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(__result))
        as *mut payload_vec_handler_t
}
#[no_mangle]
#[allow(
    non_snake_case,
    non_camel_case_types,
    unused_variables,
    unused_mut,
    unused_unsafe,
    unused_parens,
    unused_braces,
    dead_code,
    clippy::all
)]
pub unsafe extern "C" fn storage_callback_vec(
    s: *const storage_t,
    handler: *const payload_vec_handler_t,
) {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if s.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Storage pointer"),
                );
            }
            &*(s as *const perftest_flat::Storage)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let handler = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if handler.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null PayloadVecHandler pointer"),
                );
            }
            &*(handler as *const perftest_flat::PayloadVecHandler)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = perftest_flat::storage_callback_vec(s, handler);
    let _ = __result;
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
