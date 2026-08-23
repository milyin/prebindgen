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
pub struct caption_t {
    pub id: u64,
    pub text: *mut ::core::ffi::c_char,
    pub emphatic: ::core::mem::MaybeUninit<bool>,
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct drawing_t {
    pub id: u64,
    pub shape: ::core::mem::MaybeUninit<shape_t>,
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct foo_t {
    pub id: u64,
    pub x86_64_field: u64,
    pub unstable_field: u64,
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum grade_t {
    Low = 1,
    High = 2,
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[allow(non_camel_case_types)]
pub enum inside_foo_t {
    DouddleDee = 42,
    DouddleDum = 24,
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
pub enum note_t {
    Silent,
    Titled(caption_t),
    After(u64),
    Flagged(::core::mem::MaybeUninit<bool>),
    Sketched(drawing_t),
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn note_drop(this_: *mut ::core::mem::MaybeUninit<note_t>) {
    if this_.is_null() {
        return;
    }
    const _: () = {
        assert!(
            ::core::mem::size_of:: < note_t > () >= ::core::mem::size_of:: <
            ::core::ffi::c_int > (),
            "`note_t`: a #[repr(C)] enum with payload variants must be at least as large as its C `int` discriminant"
        );
    };
    let __tag: ::core::ffi::c_int = ::core::ptr::read(
        (*this_).as_ptr() as *const ::core::ffi::c_int,
    );
    if !((__tag as i64) >= 0 && (__tag as i64) < 5i64) {
        return;
    }
    match (*this_).assume_init_mut() {
        note_t::Titled(__f0) => {
            free((*__f0).text as *mut ::core::ffi::c_void);
            (*__f0).text = ::core::ptr::null_mut();
        }
        note_t::Sketched(__f0) => {
            shape_drop(&mut (*__f0).shape);
        }
        _ => {}
    }
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub enum shape_t {
    Empty,
    Circle(f64),
    Rect { width: f64, height: f64 },
    Labeled(*mut ::core::ffi::c_char, ::core::mem::MaybeUninit<operation_t>),
}
#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub unsafe extern "C" fn shape_drop(this_: *mut ::core::mem::MaybeUninit<shape_t>) {
    if this_.is_null() {
        return;
    }
    const _: () = {
        assert!(
            ::core::mem::size_of:: < shape_t > () >= ::core::mem::size_of:: <
            ::core::ffi::c_int > (),
            "`shape_t`: a #[repr(C)] enum with payload variants must be at least as large as its C `int` discriminant"
        );
    };
    let __tag: ::core::ffi::c_int = ::core::ptr::read(
        (*this_).as_ptr() as *const ::core::ffi::c_int,
    );
    if !((__tag as i64) >= 0 && (__tag as i64) < 4i64) {
        return;
    }
    match (*this_).assume_init_mut() {
        shape_t::Labeled(__f0, __f1) => {
            free(*__f0 as *mut ::core::ffi::c_void);
            *__f0 = ::core::ptr::null_mut();
        }
        _ => {}
    }
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct closure_history_batch_t {
    pub context: *mut ::core::ffi::c_void,
    pub call: ::core::option::Option<
        unsafe extern "C" fn(
            ::core::mem::MaybeUninit<*mut f64>,
            ::core::mem::MaybeUninit<usize>,
            *mut ::core::ffi::c_void,
        ),
    >,
    pub drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct closure_maybe_grade_t {
    pub context: *mut ::core::ffi::c_void,
    pub call: ::core::option::Option<
        unsafe extern "C" fn(
            ::core::mem::MaybeUninit<bool>,
            ::core::mem::MaybeUninit<grade_t>,
            *mut ::core::ffi::c_void,
        ),
    >,
    pub drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
}
#[repr(C)]
#[allow(non_camel_case_types)]
pub struct closure_maybe_value_t {
    pub context: *mut ::core::ffi::c_void,
    pub call: ::core::option::Option<
        unsafe extern "C" fn(
            ::core::mem::MaybeUninit<bool>,
            ::core::mem::MaybeUninit<f64>,
            *mut ::core::ffi::c_void,
        ),
    >,
    pub drop: ::core::option::Option<unsafe extern "C" fn(*mut ::core::ffi::c_void)>,
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
pub(crate) unsafe fn __cbg_in_Caption(v: caption_t) -> example_flat::Caption {
    example_flat::Caption {
        id: __cbg_in_u64(v.id),
        text: __cbg_in_String_field(v.text),
        emphatic: __cbg_in_bool(v.emphatic),
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_Drawing(
    v: drawing_t,
) -> ::core::result::Result<example_flat::Drawing, ::std::string::String> {
    ::core::result::Result::Ok(example_flat::Drawing {
        id: __cbg_in_u64(v.id),
        shape: __cbg_in_Shape(v.shape)?,
    })
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_Foo(v: foo_t) -> example_flat::Foo {
    example_flat::Foo {
        id: __cbg_in_u64(v.id),
        x86_64_field: __cbg_in_u64(v.x86_64_field),
        unstable_field: __cbg_in_u64(v.unstable_field),
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_Grade(
    v: ::core::mem::MaybeUninit<grade_t>,
) -> ::core::result::Result<example_flat::Grade, ::std::string::String> {
    const _: () = {
        assert!(
            ::core::mem::size_of:: < grade_t > () == ::core::mem::size_of:: <
            ::core::ffi::c_int > (),
            "`grade_t`: a #[repr(C)] enum must have the size of a C `int`"
        );
        assert!(
            ::core::mem::align_of:: < grade_t > () == ::core::mem::align_of:: <
            ::core::ffi::c_int > (),
            "`grade_t`: a #[repr(C)] enum must have the alignment of a C `int`"
        );
    };
    let __raw: ::core::ffi::c_int = ::core::ptr::read(
        v.as_ptr() as *const ::core::ffi::c_int,
    );
    if __raw == grade_t::Low as ::core::ffi::c_int {
        return ::core::result::Result::Ok(example_flat::Grade::Low);
    }
    if __raw == grade_t::High as ::core::ffi::c_int {
        return ::core::result::Result::Ok(example_flat::Grade::High);
    }
    ::core::result::Result::Err(
        ::std::format!("invalid discriminant {} for `grade_t`", __raw),
    )
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_InsideFoo(
    v: ::core::mem::MaybeUninit<inside_foo_t>,
) -> ::core::result::Result<example_flat::InsideFoo, ::std::string::String> {
    const _: () = {
        assert!(
            ::core::mem::size_of:: < inside_foo_t > () == ::core::mem::size_of:: <
            ::core::ffi::c_int > (),
            "`inside_foo_t`: a #[repr(C)] enum must have the size of a C `int`"
        );
        assert!(
            ::core::mem::align_of:: < inside_foo_t > () == ::core::mem::align_of:: <
            ::core::ffi::c_int > (),
            "`inside_foo_t`: a #[repr(C)] enum must have the alignment of a C `int`"
        );
    };
    let __raw: ::core::ffi::c_int = ::core::ptr::read(
        v.as_ptr() as *const ::core::ffi::c_int,
    );
    if __raw == inside_foo_t::DouddleDee as ::core::ffi::c_int {
        return ::core::result::Result::Ok(example_flat::InsideFoo::DouddleDee);
    }
    if __raw == inside_foo_t::DouddleDum as ::core::ffi::c_int {
        return ::core::result::Result::Ok(example_flat::InsideFoo::DouddleDum);
    }
    ::core::result::Result::Err(
        ::std::format!("invalid discriminant {} for `inside_foo_t`", __raw),
    )
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_in_Millis(v: u64) -> example_flat::Millis {
    example_flat::millis_from_raw(v)
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_Note(
    v: ::core::mem::MaybeUninit<note_t>,
) -> ::core::result::Result<example_flat::Note, ::std::string::String> {
    ::core::result::Result::Ok({
        let __tag = {
            const _: () = {
                assert!(
                    ::core::mem::size_of:: < note_t > () >= ::core::mem::size_of:: <
                    ::core::ffi::c_int > (),
                    "`note_t`: a #[repr(C)] enum with payload variants must be at least as large as its C `int` discriminant"
                );
            };
            unsafe { ::core::ptr::read((v).as_ptr() as *const ::core::ffi::c_int) }
        };
        match __tag {
            0 => example_flat::Note::Silent,
            1 => {
                let __choice = unsafe { (v).assume_init() };
                let __arm = {
                    match __choice {
                        note_t::Titled(__wire_part0) => (__wire_part0,),
                        _ => {
                            unreachable!("validated Choice tag selected a different arm")
                        }
                    }
                };
                example_flat::Note::Titled(__cbg_in_Caption((__arm).0))
            }
            2 => {
                let __choice = unsafe { (v).assume_init() };
                let __arm = {
                    match __choice {
                        note_t::After(__wire_part0) => (__wire_part0,),
                        _ => {
                            unreachable!("validated Choice tag selected a different arm")
                        }
                    }
                };
                example_flat::Note::After(__cbg_in_Millis((__arm).0))
            }
            3 => {
                let __choice = unsafe { (v).assume_init() };
                let __arm = {
                    match __choice {
                        note_t::Flagged(__wire_part0) => (__wire_part0,),
                        _ => {
                            unreachable!("validated Choice tag selected a different arm")
                        }
                    }
                };
                example_flat::Note::Flagged(__cbg_in_bool((__arm).0))
            }
            4 => {
                let __choice = unsafe { (v).assume_init() };
                let __arm = {
                    match __choice {
                        note_t::Sketched(__wire_part0) => (__wire_part0,),
                        _ => {
                            unreachable!("validated Choice tag selected a different arm")
                        }
                    }
                };
                example_flat::Note::Sketched(__cbg_in_Drawing((__arm).0)?)
            }
            _ => {
                return ::core::result::Result::Err(
                    ::std::format!(
                        "invalid tag {} for `{}` (expected 0..{})", __tag,
                        stringify!(note_t), 5usize,
                    ),
                );
            }
        }
    })
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_Operation(
    v: ::core::mem::MaybeUninit<operation_t>,
) -> ::core::result::Result<example_flat::Operation, ::std::string::String> {
    const _: () = {
        assert!(
            ::core::mem::size_of:: < operation_t > () == ::core::mem::size_of:: <
            ::core::ffi::c_int > (),
            "`operation_t`: a #[repr(C)] enum must have the size of a C `int`"
        );
        assert!(
            ::core::mem::align_of:: < operation_t > () == ::core::mem::align_of:: <
            ::core::ffi::c_int > (),
            "`operation_t`: a #[repr(C)] enum must have the alignment of a C `int`"
        );
    };
    let __raw: ::core::ffi::c_int = ::core::ptr::read(
        v.as_ptr() as *const ::core::ffi::c_int,
    );
    if __raw == operation_t::Add as ::core::ffi::c_int {
        return ::core::result::Result::Ok(example_flat::Operation::Add);
    }
    if __raw == operation_t::Sub as ::core::ffi::c_int {
        return ::core::result::Result::Ok(example_flat::Operation::Sub);
    }
    if __raw == operation_t::Mul as ::core::ffi::c_int {
        return ::core::result::Result::Ok(example_flat::Operation::Mul);
    }
    if __raw == operation_t::Div as ::core::ffi::c_int {
        return ::core::result::Result::Ok(example_flat::Operation::Div);
    }
    ::core::result::Result::Err(
        ::std::format!("invalid discriminant {} for `operation_t`", __raw),
    )
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_Shape(
    v: ::core::mem::MaybeUninit<shape_t>,
) -> ::core::result::Result<example_flat::Shape, ::std::string::String> {
    ::core::result::Result::Ok({
        let __tag = {
            const _: () = {
                assert!(
                    ::core::mem::size_of:: < shape_t > () >= ::core::mem::size_of:: <
                    ::core::ffi::c_int > (),
                    "`shape_t`: a #[repr(C)] enum with payload variants must be at least as large as its C `int` discriminant"
                );
            };
            unsafe { ::core::ptr::read((v).as_ptr() as *const ::core::ffi::c_int) }
        };
        match __tag {
            0 => example_flat::Shape::Empty,
            1 => {
                let __choice = unsafe { (v).assume_init() };
                let __arm = {
                    match __choice {
                        shape_t::Circle(__wire_part0) => (__wire_part0,),
                        _ => {
                            unreachable!("validated Choice tag selected a different arm")
                        }
                    }
                };
                example_flat::Shape::Circle(__cbg_in_f64((__arm).0))
            }
            2 => {
                let __choice = unsafe { (v).assume_init() };
                let __arm = {
                    match __choice {
                        shape_t::Rect { width: __wire_part0, height: __wire_part1 } => {
                            (__wire_part0, __wire_part1)
                        }
                        _ => {
                            unreachable!("validated Choice tag selected a different arm")
                        }
                    }
                };
                example_flat::Shape::Rect {
                    width: __cbg_in_f64((__arm).0),
                    height: __cbg_in_f64((__arm).1),
                }
            }
            3 => {
                let __choice = unsafe { (v).assume_init() };
                let __arm = {
                    match __choice {
                        shape_t::Labeled(__wire_part0, __wire_part1) => {
                            (__wire_part0, __wire_part1)
                        }
                        _ => {
                            unreachable!("validated Choice tag selected a different arm")
                        }
                    }
                };
                example_flat::Shape::Labeled(
                    __cbg_in_String_field((__arm).0),
                    __cbg_in_Operation((__arm).1)?,
                )
            }
            _ => {
                return ::core::result::Result::Err(
                    ::std::format!(
                        "invalid tag {} for `{}` (expected 0..{})", __tag,
                        stringify!(shape_t), 4usize,
                    ),
                );
            }
        }
    })
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_String(
    v: *const ::core::ffi::c_char,
) -> ::core::result::Result<::std::string::String, ::std::string::String> {
    if v.is_null() {
        return ::core::result::Result::Err(
            ::std::string::String::from("null pointer passed for String argument"),
        );
    }
    match ::std::ffi::CStr::from_ptr(v).to_str() {
        ::core::result::Result::Ok(s) => ::core::result::Result::Ok(s.to_owned()),
        ::core::result::Result::Err(_) => {
            ::core::result::Result::Err(
                ::std::string::String::from("invalid UTF-8 in String argument"),
            )
        }
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_String_field(
    v: *const ::core::ffi::c_char,
) -> ::std::string::String {
    if v.is_null() {
        ::std::string::String::new()
    } else {
        ::std::ffi::CStr::from_ptr(v).to_string_lossy().into_owned()
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
pub(crate) unsafe fn __cbg_in_bool(v: ::core::mem::MaybeUninit<bool>) -> bool {
    ::core::ptr::read(v.as_ptr() as *const u8) != 0
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_closure_history_batch_t(
    c: closure_history_batch_t,
) -> impl Fn(::std::vec::Vec<f64>) + Send + Sync + 'static {
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
    move |__a0: ::std::vec::Vec<f64>| {
        if let ::core::option::Option::Some(__f) = __call {
            let mut __w0_0 = ::core::mem::MaybeUninit::<*mut f64>::zeroed();
            let mut __w0_1 = ::core::mem::MaybeUninit::<usize>::zeroed();
            let __arr: ::std::vec::Vec<f64> = __cbg_out_chain_vec_f64(__a0);
            let (__p, __n) = __cbg_alloc_array(__arr);
            *__w0_0.as_mut_ptr() = __p;
            *__w0_1.as_mut_ptr() = __n;
            unsafe { __f(__w0_0, __w0_1, __ctx.context) }
        }
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_closure_maybe_grade_t(
    c: closure_maybe_grade_t,
) -> impl Fn(::core::option::Option<example_flat::Grade>) + Send + Sync + 'static {
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
    move |__a0: ::core::option::Option<example_flat::Grade>| {
        if let ::core::option::Option::Some(__f) = __call {
            let mut __w0_0 = ::core::mem::MaybeUninit::<bool>::zeroed();
            let mut __w0_1 = ::core::mem::MaybeUninit::<grade_t>::zeroed();
            match __a0 {
                ::core::option::Option::Some(__x) => {
                    *__w0_0.as_mut_ptr() = true;
                    *__w0_1.as_mut_ptr() = __cbg_out_Grade(__x);
                }
                ::core::option::Option::None => {
                    *__w0_0.as_mut_ptr() = false;
                }
            }
            unsafe { __f(__w0_0, __w0_1, __ctx.context) }
        }
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __cbg_in_closure_maybe_value_t(
    c: closure_maybe_value_t,
) -> impl Fn(::core::option::Option<f64>) + Send + Sync + 'static {
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
    move |__a0: ::core::option::Option<f64>| {
        if let ::core::option::Option::Some(__f) = __call {
            let mut __w0_0 = ::core::mem::MaybeUninit::<bool>::zeroed();
            let mut __w0_1 = ::core::mem::MaybeUninit::<f64>::zeroed();
            match __a0 {
                ::core::option::Option::Some(__x) => {
                    *__w0_0.as_mut_ptr() = true;
                    *__w0_1.as_mut_ptr() = __cbg_out_f64(__x);
                }
                ::core::option::Option::None => {
                    *__w0_0.as_mut_ptr() = false;
                }
            }
            unsafe { __f(__w0_0, __w0_1, __ctx.context) }
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
        if let ::core::option::Option::Some(__f) = __call {
            let __w0 = __cbg_out_f64(__a0);
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
pub(crate) fn __cbg_out_Caption(v: example_flat::Caption) -> caption_t {
    caption_t {
        id: __cbg_out_u64(v.id),
        text: __cbg_out_String(v.text),
        emphatic: __cbg_out_bool_field(v.emphatic),
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_Drawing(v: example_flat::Drawing) -> drawing_t {
    drawing_t {
        id: __cbg_out_u64(v.id),
        shape: __cbg_out_Shape(v.shape),
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_Error(v: example_flat::Error) -> *mut ::core::ffi::c_char {
    __cbg_alloc_cstr(example_flat::error_get_message(&v))
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_Foo(v: example_flat::Foo) -> foo_t {
    foo_t {
        id: __cbg_out_u64(v.id),
        x86_64_field: __cbg_out_u64(v.x86_64_field),
        unstable_field: __cbg_out_u64(v.unstable_field),
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_Grade(v: example_flat::Grade) -> grade_t {
    match v {
        example_flat::Grade::Low => grade_t::Low,
        example_flat::Grade::High => grade_t::High,
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
pub(crate) fn __cbg_out_Millis(v: example_flat::Millis) -> u64 {
    example_flat::millis_to_raw(&v)
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __cbg_out_Note(v: example_flat::Note) -> ::core::mem::MaybeUninit<note_t> {
    ::core::mem::MaybeUninit::new({
        match v {
            example_flat::Note::Silent => note_t::Silent,
            example_flat::Note::Titled(__part0) => {
                let __built_arm = (__cbg_out_Caption(__part0),);
                note_t::Titled(__built_arm.0)
            }
            example_flat::Note::After(__part0) => {
                let __built_arm = (__cbg_out_Millis(__part0),);
                note_t::After(__built_arm.0)
            }
            example_flat::Note::Flagged(__part0) => {
                let __built_arm = (__cbg_out_bool_field(__part0),);
                note_t::Flagged(__built_arm.0)
            }
            example_flat::Note::Sketched(__part0) => {
                let __built_arm = (__cbg_out_Drawing(__part0),);
                note_t::Sketched(__built_arm.0)
            }
        }
    })
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
pub(crate) fn __cbg_out_Shape(
    v: example_flat::Shape,
) -> ::core::mem::MaybeUninit<shape_t> {
    ::core::mem::MaybeUninit::new({
        match v {
            example_flat::Shape::Empty => shape_t::Empty,
            example_flat::Shape::Circle(__part0) => {
                let __built_arm = (__cbg_out_f64(__part0),);
                shape_t::Circle(__built_arm.0)
            }
            example_flat::Shape::Rect { width: __part0, height: __part1 } => {
                let __built_arm = (__cbg_out_f64(__part0), __cbg_out_f64(__part1));
                shape_t::Rect {
                    width: __built_arm.0,
                    height: __built_arm.1,
                }
            }
            example_flat::Shape::Labeled(__part0, __part1) => {
                let __built_arm = (
                    __cbg_out_String(__part0),
                    ::core::mem::MaybeUninit::new(__cbg_out_Operation(__part1)),
                );
                shape_t::Labeled(__built_arm.0, __built_arm.1)
            }
        }
    })
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
pub(crate) fn __cbg_out_bool_field(v: bool) -> ::core::mem::MaybeUninit<bool> {
    ::core::mem::MaybeUninit::new(v)
}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline(always)]
pub(crate) fn __cbg_out_chain_vec_f64(v: ::std::vec::Vec<f64>) -> ::std::vec::Vec<f64> {
    {
        let __sequence_source = v;
        let mut __sequence_output: ::std::vec::Vec<f64> = ::std::vec::Vec::with_capacity(
            (__sequence_source).len(),
        );
        for __sequence_element in __sequence_source.into_iter() {
            let __sequence_part = __cbg_out_f64(__sequence_element);
            __sequence_output.push(__sequence_part);
        }
        __sequence_output
    }
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
pub(crate) fn __cbg_outmark_option_Grade() {}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __cbg_outmark_option_f64() {}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __cbg_result_Result___Calculator___Error__() {}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __cbg_result_Result___f64___Error__() {}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_absorb(
    a: *mut calculator_t,
    b: *const calculator_t,
    out: *mut f64,
    e: *mut *mut ::core::ffi::c_char,
) -> bool {
    if !(a as *const ()).is_null() && (a as *const ()) == (b as *const ()) {
        let __msg = ::std::string::String::from(
            "aliasing arguments: `a` (consumed) and `b` (borrowed) are the same `Calculator` — a consumed or exclusively-borrowed resource may not be named twice in one call",
        );
        if !e.is_null() {
            *e = __cbg_out_Error(
                <example_flat::Error as ::core::convert::From<
                    ::std::string::String,
                >>::from(__msg),
            );
        }
        return false;
    }
    let a = match __cbg_in_Calculator(a) {
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
    let b = match __cbg_in___Calculator(b) {
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
    match example_flat::calculator_absorb(a, b) {
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
pub unsafe extern "C" fn calculator_apply(
    c: *mut calculator_t,
    op: ::core::mem::MaybeUninit<operation_t>,
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
    let op = match __cbg_in_Operation(op) {
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
    let __arr: ::std::vec::Vec<f64> = __cbg_out_chain_vec_f64(__v);
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
pub unsafe extern "C" fn calculator_grade_or_none(
    c: *const calculator_t,
    f: closure_maybe_grade_t,
) {
    let c = match __cbg_in___Calculator(c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let f = __cbg_in_closure_maybe_grade_t(f);
    example_flat::calculator_grade_or_none(c, f);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_history_batch(
    c: *const calculator_t,
    f: closure_history_batch_t,
) {
    let c = match __cbg_in___Calculator(c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let f = __cbg_in_closure_history_batch_t(f);
    example_flat::calculator_history_batch(c, f);
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
pub unsafe extern "C" fn calculator_last_or_none(
    c: *const calculator_t,
    f: closure_maybe_value_t,
) {
    let c = match __cbg_in___Calculator(c) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let f = __cbg_in_closure_maybe_value_t(f);
    example_flat::calculator_last_or_none(c, f);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_merge(
    a: *mut calculator_t,
    b: *mut calculator_t,
    e: *mut *mut ::core::ffi::c_char,
) -> *mut calculator_t {
    if !(a as *const ()).is_null() && (a as *const ()) == (b as *const ()) {
        let __msg = ::std::string::String::from(
            "aliasing arguments: `a` (consumed) and `b` (consumed) are the same `Calculator` — a consumed or exclusively-borrowed resource may not be named twice in one call",
        );
        if !e.is_null() {
            *e = __cbg_out_Error(
                <example_flat::Error as ::core::convert::From<
                    ::std::string::String,
                >>::from(__msg),
            );
        }
        return ::core::ptr::null_mut();
    }
    let a = match __cbg_in_Calculator(a) {
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
    let b = match __cbg_in_Calculator(b) {
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
    match example_flat::calculator_merge(a, b) {
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
pub unsafe extern "C" fn caption_new(
    id: u64,
    text: *const ::core::ffi::c_char,
    emphatic: ::core::mem::MaybeUninit<bool>,
) -> caption_t {
    let id = __cbg_in_u64(id);
    let text = match __cbg_in___str(text) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let emphatic = __cbg_in_bool(emphatic);
    let __v = example_flat::caption_new(id, text, emphatic);
    let __ret: caption_t;
    __ret = __cbg_out_Caption(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn drawing_get_shape(
    d: drawing_t,
) -> ::core::mem::MaybeUninit<shape_t> {
    let d = match __cbg_in_Drawing(d) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::drawing_get_shape(d);
    let __ret: ::core::mem::MaybeUninit<shape_t>;
    __ret = __cbg_out_Shape(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn drawing_new(
    id: u64,
    shape: ::core::mem::MaybeUninit<shape_t>,
) -> drawing_t {
    let id = __cbg_in_u64(id);
    let shape = match __cbg_in_Shape(shape) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::drawing_new(id, shape);
    let __ret: drawing_t;
    __ret = __cbg_out_Drawing(__v);
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
pub unsafe extern "C" fn inside_foo_value(
    x: ::core::mem::MaybeUninit<inside_foo_t>,
) -> i32 {
    let x = match __cbg_in_InsideFoo(x) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::inside_foo_value(x);
    let __ret: i32;
    __ret = __cbg_out_i32(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_emphatic(n: ::core::mem::MaybeUninit<note_t>) -> bool {
    let n = match __cbg_in_Note(n) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::note_emphatic(n);
    let __ret: bool;
    __ret = __cbg_out_bool(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_new_after(
    millis: u64,
) -> ::core::mem::MaybeUninit<note_t> {
    let millis = __cbg_in_u64(millis);
    let __v = example_flat::note_new_after(millis);
    let __ret: ::core::mem::MaybeUninit<note_t>;
    __ret = __cbg_out_Note(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_new_flagged(
    flag: ::core::mem::MaybeUninit<bool>,
) -> ::core::mem::MaybeUninit<note_t> {
    let flag = __cbg_in_bool(flag);
    let __v = example_flat::note_new_flagged(flag);
    let __ret: ::core::mem::MaybeUninit<note_t>;
    __ret = __cbg_out_Note(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_new_silent() -> ::core::mem::MaybeUninit<note_t> {
    let __v = example_flat::note_new_silent();
    let __ret: ::core::mem::MaybeUninit<note_t>;
    __ret = __cbg_out_Note(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_new_sketched(
    id: u64,
    label: *const ::core::ffi::c_char,
) -> ::core::mem::MaybeUninit<note_t> {
    let id = __cbg_in_u64(id);
    let label = match __cbg_in___str(label) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::note_new_sketched(id, label);
    let __ret: ::core::mem::MaybeUninit<note_t>;
    __ret = __cbg_out_Note(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_new_titled(
    id: u64,
    text: *const ::core::ffi::c_char,
    emphatic: ::core::mem::MaybeUninit<bool>,
) -> ::core::mem::MaybeUninit<note_t> {
    let id = __cbg_in_u64(id);
    let text = match __cbg_in___str(text) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let emphatic = __cbg_in_bool(emphatic);
    let __v = example_flat::note_new_titled(id, text, emphatic);
    let __ret: ::core::mem::MaybeUninit<note_t>;
    __ret = __cbg_out_Note(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_value(n: ::core::mem::MaybeUninit<note_t>) -> u64 {
    let n = match __cbg_in_Note(n) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::note_value(n);
    let __ret: u64;
    __ret = __cbg_out_u64(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_area(s: ::core::mem::MaybeUninit<shape_t>) -> f64 {
    let s = match __cbg_in_Shape(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::shape_area(s);
    let __ret: f64;
    __ret = __cbg_out_f64(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_get_label(
    s: ::core::mem::MaybeUninit<shape_t>,
) -> *mut ::core::ffi::c_char {
    let s = match __cbg_in_Shape(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::shape_get_label(s);
    let __ret: *mut ::core::ffi::c_char;
    __ret = __cbg_out_String(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_new_circle(
    radius: f64,
) -> ::core::mem::MaybeUninit<shape_t> {
    let radius = __cbg_in_f64(radius);
    let __v = example_flat::shape_new_circle(radius);
    let __ret: ::core::mem::MaybeUninit<shape_t>;
    __ret = __cbg_out_Shape(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_new_empty() -> ::core::mem::MaybeUninit<shape_t> {
    let __v = example_flat::shape_new_empty();
    let __ret: ::core::mem::MaybeUninit<shape_t>;
    __ret = __cbg_out_Shape(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_new_labeled(
    label: *const ::core::ffi::c_char,
    op: ::core::mem::MaybeUninit<operation_t>,
) -> ::core::mem::MaybeUninit<shape_t> {
    let label = match __cbg_in___str(label) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let op = match __cbg_in_Operation(op) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::shape_new_labeled(label, op);
    let __ret: ::core::mem::MaybeUninit<shape_t>;
    __ret = __cbg_out_Shape(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_new_rect(
    width: f64,
    height: f64,
) -> ::core::mem::MaybeUninit<shape_t> {
    let width = __cbg_in_f64(width);
    let height = __cbg_in_f64(height);
    let __v = example_flat::shape_new_rect(width, height);
    let __ret: ::core::mem::MaybeUninit<shape_t>;
    __ret = __cbg_out_Shape(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_try_area(
    s: ::core::mem::MaybeUninit<shape_t>,
    out: *mut f64,
    e: *mut *mut ::core::ffi::c_char,
) -> bool {
    let s = match __cbg_in_Shape(s) {
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
    match example_flat::shape_try_area(s) {
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
const _: () = {
    konst::assertc_eq!(
        example_flat::FEATURES, "example-flat/internal example-flat/unstable",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
