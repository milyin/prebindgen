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
pub unsafe extern "C" fn example_free(p: *mut ::core::ffi::c_void) {
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
pub struct calculator_t {
    _private: [u8; 0],
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn calculator_drop(this_: *mut calculator_t) {
    if !this_.is_null() {
        drop(::std::boxed::Box::from_raw(this_ as *mut example_flat::Calculator));
    }
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct foo_t {
    pub id: u64,
    pub aarch64_field: u64,
    pub unstable_field: u64,
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum inside_foo_t {
    DouddleDee = 14,
    DouddleDum = 88,
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum operation_t {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct closure_value_t {
    pub context: *mut ::core::ffi::c_void,
    pub call: ::core::option::Option<
        unsafe extern "C" fn(f64, *mut ::core::ffi::c_void),
    >,
    pub drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_Calculator(
    v: *mut calculator_t,
) -> ::core::result::Result<example_flat::Calculator, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null Calculator handle passed by value"),
        );
    }
    ::core::result::Result::Ok(
        *::std::boxed::Box::from_raw(v as *mut example_flat::Calculator),
    )
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_Foo(v: foo_t) -> example_flat::Foo {
    example_flat::Foo {
        id: v.id,
        aarch64_field: v.aarch64_field,
        unstable_field: v.unstable_field,
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_in_InsideFoo(v: inside_foo_t) -> example_flat::InsideFoo {
    match v {
        inside_foo_t::DouddleDee => example_flat::InsideFoo::DouddleDee,
        inside_foo_t::DouddleDum => example_flat::InsideFoo::DouddleDum,
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_in_Operation(v: operation_t) -> example_flat::Operation {
    match v {
        operation_t::Add => example_flat::Operation::Add,
        operation_t::Sub => example_flat::Operation::Sub,
        operation_t::Mul => example_flat::Operation::Mul,
        operation_t::Div => example_flat::Operation::Div,
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in___Calculator<'a>(
    v: *const calculator_t,
) -> ::core::result::Result<&'a example_flat::Calculator, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null Calculator pointer"),
        );
    }
    ::core::result::Result::Ok(&*(v as *const example_flat::Calculator))
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in___mut_Calculator<'a>(
    v: *mut calculator_t,
) -> ::core::result::Result<&'a mut example_flat::Calculator, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null Calculator pointer"),
        );
    }
    ::core::result::Result::Ok(&mut *(v as *mut example_flat::Calculator))
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
pub(crate) unsafe fn __cbg_in_closure_value_t(
    c: closure_value_t,
) -> impl Fn(f64) + Send + Sync + 'static {
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
    move |__a0: f64| {
        let __w0 = __cbg_out_f64(__a0);
        if let ::core::option::Option::Some(__f) = __call {
            unsafe { __f(__w0, __ctx.context) }
        }
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_in_f64(v: f64) -> f64 {
    v
}
#[allow(non_snake_case, dead_code, unused_variables)]
pub(crate) fn __cbg_in_str() {}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_in_u64(v: u64) -> u64 {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_Calculator(v: example_flat::Calculator) -> *mut calculator_t {
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut calculator_t
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_Error(v: example_flat::Error) -> *mut ::core::ffi::c_char {
    __cbg_alloc_cstr(example_flat::error_get_message(&v))
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_Foo(v: example_flat::Foo) -> foo_t {
    foo_t {
        id: v.id,
        aarch64_field: v.aarch64_field,
        unstable_field: v.unstable_field,
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_InsideFoo(v: example_flat::InsideFoo) -> inside_foo_t {
    match v {
        example_flat::InsideFoo::DouddleDee => inside_foo_t::DouddleDee,
        example_flat::InsideFoo::DouddleDum => inside_foo_t::DouddleDum,
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_Operation(v: example_flat::Operation) -> operation_t {
    match v {
        example_flat::Operation::Add => operation_t::Add,
        example_flat::Operation::Sub => operation_t::Sub,
        example_flat::Operation::Mul => operation_t::Mul,
        example_flat::Operation::Div => operation_t::Div,
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_String(v: ::std::string::String) -> *mut ::core::ffi::c_char {
    __cbg_alloc_cstr(v)
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
pub(crate) fn __cbg_out_u64(v: u64) -> u64 {
    v
}
#[allow(non_snake_case, dead_code, unused_variables)]
pub(crate) fn __cbg_out_unit(v: ()) {}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __cbg_outmark_vec_f64() {}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __cbg_result_Result___Calculator___Error__() {}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __cbg_result_Result___f64___Error__() {}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_apply(
    c: *mut calculator_t,
    op: operation_t,
    operand: f64,
    out: *mut f64,
    e: *mut *mut ::core::ffi::c_char,
) -> bool {
    let c = match __cbg_in___mut_Calculator(c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            if !e.is_null() {
                *e = __cbg_out_Error(
                    <example_flat::Error as ::core::convert::From<
                        ::std::string::String,
                    >>::from(__msg),
                );
            }
            return false;
        }
    };
    let op = __cbg_in_Operation(op);
    let operand = __cbg_in_f64(operand);
    match example_flat::calculator_apply(c, op, operand) {
        ::core::result::Result::Ok(__v) => {
            *out = __cbg_out_f64(__v);
            true
        }
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __cbg_out_Error(__err);
            }
            false
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_for_each(
    c: *const calculator_t,
    f: closure_value_t,
) {
    let c = match __cbg_in___Calculator(c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let f = __cbg_in_closure_value_t(f);
    example_flat::calculator_for_each(c, f);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_get_count(c: *const calculator_t) -> u64 {
    let c = match __cbg_in___Calculator(c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::calculator_get_count(c);
    let __ret: u64;
    __ret = __cbg_out_u64(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_get_history(
    c: *const calculator_t,
    len: *mut usize,
) -> *mut f64 {
    let c = match __cbg_in___Calculator(c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::calculator_get_history(c);
    let __ret: *mut f64;
    let __arr: ::std::vec::Vec<f64> = __v.into_iter().map(__cbg_out_f64).collect();
    let (__p, __n) = __cbg_alloc_array(__arr);
    __ret = __p;
    *len = __n;
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_get_value(c: *const calculator_t) -> f64 {
    let c = match __cbg_in___Calculator(c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::calculator_get_value(c);
    let __ret: f64;
    __ret = __cbg_out_f64(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_is(c: *const calculator_t, value: f64) -> bool {
    let c = match __cbg_in___Calculator(c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let value = __cbg_in_f64(value);
    let __v = example_flat::calculator_is(c, value);
    let __ret: bool;
    __ret = __cbg_out_bool(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_new() -> *mut calculator_t {
    let __v = example_flat::calculator_new();
    let __ret: *mut calculator_t;
    __ret = __cbg_out_Calculator(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_new_clone(
    c: *const calculator_t,
) -> *mut calculator_t {
    let c = match __cbg_in___Calculator(c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::calculator_new_clone(c);
    let __ret: *mut calculator_t;
    __ret = __cbg_out_Calculator(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_new_from_str(
    s: *const ::core::ffi::c_char,
    e: *mut *mut ::core::ffi::c_char,
) -> *mut calculator_t {
    let s = match __cbg_in___str(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            if !e.is_null() {
                *e = __cbg_out_Error(
                    <example_flat::Error as ::core::convert::From<
                        ::std::string::String,
                    >>::from(__msg),
                );
            }
            return ::core::ptr::null_mut();
        }
    };
    match example_flat::calculator_new_from_str(s) {
        ::core::result::Result::Ok(__v) => {
            let __ret: *mut calculator_t;
            __ret = __cbg_out_Calculator(__v);
            __ret
        }
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __cbg_out_Error(__err);
            }
            ::core::ptr::null_mut()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_reset(c: *mut calculator_t) {
    let c = match __cbg_in___mut_Calculator(c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    example_flat::calculator_reset(c);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_to_string(
    c: *const calculator_t,
) -> *mut ::core::ffi::c_char {
    let c = match __cbg_in___Calculator(c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::calculator_to_string(c);
    let __ret: *mut ::core::ffi::c_char;
    __ret = __cbg_out_String(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn foo_get_id(f: foo_t) -> u64 {
    let f = __cbg_in_Foo(f);
    let __v = example_flat::foo_get_id(f);
    let __ret: u64;
    __ret = __cbg_out_u64(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn foo_new(id: u64) -> foo_t {
    let id = __cbg_in_u64(id);
    let __v = example_flat::foo_new(id);
    let __ret: foo_t;
    __ret = __cbg_out_Foo(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn inside_foo_default() -> inside_foo_t {
    let __v = example_flat::inside_foo_default();
    let __ret: inside_foo_t;
    __ret = __cbg_out_InsideFoo(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn inside_foo_value(x: inside_foo_t) -> i32 {
    let x = __cbg_in_InsideFoo(x);
    let __v = example_flat::inside_foo_value(x);
    let __ret: i32;
    __ret = __cbg_out_i32(__v);
    __ret
}
const _: () = {
    konst::assertc_eq!(
        example_flat::FEATURES, "example-flat/internal example-flat/unstable",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
