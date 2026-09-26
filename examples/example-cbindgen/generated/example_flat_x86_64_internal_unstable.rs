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
pub unsafe extern "C" fn example_free(p: *mut ::core::ffi::c_void) {
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
pub struct calculator_t {
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
pub unsafe extern "C" fn calculator_drop(this_: *mut calculator_t) {
    if !this_.is_null() {
        drop(::std::boxed::Box::from_raw(this_ as *mut example_flat::Calculator));
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
pub(crate) fn __cbg_out_Error(v: example_flat::Error) -> *mut ::core::ffi::c_char {
    __cbg_alloc_cstr(example_flat::error_get_message(&v))
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
pub enum operation_t {
    Add = 0,
    Sub = 1,
    Mul = 2,
    Div = 3,
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
pub(crate) fn __cbg_out_Operation(v: example_flat::Operation) -> operation_t {
    match v {
        example_flat::Operation::Add => operation_t::Add,
        example_flat::Operation::Sub => operation_t::Sub,
        example_flat::Operation::Mul => operation_t::Mul,
        example_flat::Operation::Div => operation_t::Div,
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
pub enum shape_t {
    Empty,
    Circle(f64),
    Rect { width: f64, height: f64 },
    Labeled(*mut ::core::ffi::c_char, ::core::mem::MaybeUninit<operation_t>),
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
pub(crate) unsafe fn __cbg_in_Shape(
    v: ::core::mem::MaybeUninit<shape_t>,
) -> ::core::result::Result<example_flat::Shape, ::std::string::String> {
    const _: () = {
        assert!(
            ::core::mem::size_of:: < shape_t > () >= ::core::mem::size_of:: <
            ::core::ffi::c_int > (),
            "`shape_t`: a #[repr(C)] enum with payload variants must be at least as large as its C `int` discriminant"
        );
    };
    let __tag: ::core::ffi::c_int = ::core::ptr::read(
        v.as_ptr() as *const ::core::ffi::c_int,
    );
    if !((__tag as i64) >= 0 && (__tag as i64) < 4i64) {
        return ::core::result::Result::Err(
            ::std::format!("invalid tag {} for `shape_t` (expected 0..4)", __tag),
        );
    }
    let v = v.assume_init();
    (|| -> ::core::result::Result<example_flat::Shape, ::std::string::String> {
        ::core::result::Result::Ok(
            match v {
                shape_t::Empty => example_flat::Shape::Empty,
                shape_t::Circle(__f0) => example_flat::Shape::Circle(__f0),
                shape_t::Rect { width: __f0, height: __f1 } => {
                    example_flat::Shape::Rect {
                        width: __f0,
                        height: __f1,
                    }
                }
                shape_t::Labeled(__f0, __f1) => {
                    example_flat::Shape::Labeled(
                        if __f0.is_null() {
                            ::std::string::String::new()
                        } else {
                            ::std::ffi::CStr::from_ptr(__f0)
                                .to_string_lossy()
                                .into_owned()
                        },
                        __cbg_in_Operation(__f1)?,
                    )
                }
            },
        )
    })()
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
pub(crate) fn __cbg_out_Shape(
    v: example_flat::Shape,
) -> ::core::mem::MaybeUninit<shape_t> {
    ::core::mem::MaybeUninit::new(
        match v {
            example_flat::Shape::Empty => shape_t::Empty,
            example_flat::Shape::Circle(__f0) => {
                let __f0 = __f0;
                shape_t::Circle(__f0)
            }
            example_flat::Shape::Rect { width: __f0, height: __f1 } => {
                let __f0 = __f0;
                let __f1 = __f1;
                shape_t::Rect {
                    width: __f0,
                    height: __f1,
                }
            }
            example_flat::Shape::Labeled(__f0, __f1) => {
                let __f0 = __cbg_alloc_cstr(__f0);
                let __f1 = ::core::mem::MaybeUninit::new(__cbg_out_Operation(__f1));
                shape_t::Labeled(__f0, __f1)
            }
        },
    )
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
            free((*__f0) as *mut ::core::ffi::c_void);
            (*__f0) = ::core::ptr::null_mut();
        }
        _ => {}
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
pub struct drawing_t {
    pub id: u64,
    pub shape: ::core::mem::MaybeUninit<shape_t>,
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
pub(crate) unsafe fn __cbg_in_Drawing(
    v: drawing_t,
) -> ::core::result::Result<example_flat::Drawing, ::std::string::String> {
    (|| -> ::core::result::Result<example_flat::Drawing, ::std::string::String> {
        ::core::result::Result::Ok({
            let drawing_t { id, shape } = v;
            example_flat::Drawing {
                id: id,
                shape: __cbg_in_Shape(shape)?,
            }
        })
    })()
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
pub(crate) fn __cbg_out_Drawing(v: example_flat::Drawing) -> drawing_t {
    {
        let example_flat::Drawing { id: __f0, shape: __f1 } = v;
        let id = __f0;
        let shape = __cbg_out_Shape(__f1);
        drawing_t { id, shape }
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
pub(crate) unsafe fn __cbg_release_Drawing(v: &mut drawing_t) {
    shape_drop(&mut v.shape);
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
pub struct caption_t {
    pub id: u64,
    pub text: *mut ::core::ffi::c_char,
    pub emphatic: ::core::mem::MaybeUninit<bool>,
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
pub(crate) unsafe fn __cbg_in_Caption(v: caption_t) -> example_flat::Caption {
    {
        let caption_t { id, text, emphatic } = v;
        example_flat::Caption {
            id: id,
            text: if text.is_null() {
                ::std::string::String::new()
            } else {
                ::std::ffi::CStr::from_ptr(text).to_string_lossy().into_owned()
            },
            emphatic: (::core::ptr::read(emphatic.as_ptr() as *const u8) != 0),
        }
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
pub(crate) fn __cbg_out_Caption(v: example_flat::Caption) -> caption_t {
    {
        let example_flat::Caption { id: __f0, text: __f1, emphatic: __f2 } = v;
        let id = __f0;
        let text = __cbg_alloc_cstr(__f1);
        let emphatic = ::core::mem::MaybeUninit::new(__f2);
        caption_t { id, text, emphatic }
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
pub(crate) unsafe fn __cbg_release_Caption(v: &mut caption_t) {
    free(v.text as *mut ::core::ffi::c_void);
    v.text = ::core::ptr::null_mut();
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
pub enum note_t {
    Silent,
    Titled(caption_t),
    After(u64),
    Flagged(::core::mem::MaybeUninit<bool>),
    Sketched(drawing_t),
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
pub(crate) unsafe fn __cbg_in_Note(
    v: ::core::mem::MaybeUninit<note_t>,
) -> ::core::result::Result<example_flat::Note, ::std::string::String> {
    const _: () = {
        assert!(
            ::core::mem::size_of:: < note_t > () >= ::core::mem::size_of:: <
            ::core::ffi::c_int > (),
            "`note_t`: a #[repr(C)] enum with payload variants must be at least as large as its C `int` discriminant"
        );
    };
    let __tag: ::core::ffi::c_int = ::core::ptr::read(
        v.as_ptr() as *const ::core::ffi::c_int,
    );
    if !((__tag as i64) >= 0 && (__tag as i64) < 5i64) {
        return ::core::result::Result::Err(
            ::std::format!("invalid tag {} for `note_t` (expected 0..5)", __tag),
        );
    }
    let v = v.assume_init();
    (|| -> ::core::result::Result<example_flat::Note, ::std::string::String> {
        ::core::result::Result::Ok(
            match v {
                note_t::Silent => example_flat::Note::Silent,
                note_t::Titled(__f0) => {
                    example_flat::Note::Titled(__cbg_in_Caption(__f0))
                }
                note_t::After(__f0) => {
                    example_flat::Note::After({
                        let __cbg_repr = __f0;
                        example_flat::millis_from_raw(__cbg_repr)
                    })
                }
                note_t::Flagged(__f0) => {
                    example_flat::Note::Flagged(
                        (::core::ptr::read(__f0.as_ptr() as *const u8) != 0),
                    )
                }
                note_t::Sketched(__f0) => {
                    example_flat::Note::Sketched(__cbg_in_Drawing(__f0)?)
                }
            },
        )
    })()
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
pub(crate) fn __cbg_out_Note(v: example_flat::Note) -> ::core::mem::MaybeUninit<note_t> {
    ::core::mem::MaybeUninit::new(
        match v {
            example_flat::Note::Silent => note_t::Silent,
            example_flat::Note::Titled(__f0) => {
                let __f0 = __cbg_out_Caption(__f0);
                note_t::Titled(__f0)
            }
            example_flat::Note::After(__f0) => {
                let __f0 = {
                    let __cbg_value = __f0;
                    example_flat::millis_to_raw(&__cbg_value)
                };
                note_t::After(__f0)
            }
            example_flat::Note::Flagged(__f0) => {
                let __f0 = ::core::mem::MaybeUninit::new(__f0);
                note_t::Flagged(__f0)
            }
            example_flat::Note::Sketched(__f0) => {
                let __f0 = __cbg_out_Drawing(__f0);
                note_t::Sketched(__f0)
            }
        },
    )
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
            __cbg_release_Caption(&mut (*__f0));
        }
        note_t::Sketched(__f0) => {
            __cbg_release_Drawing(&mut (*__f0));
        }
        _ => {}
    }
}
#[repr(C)]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
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
pub enum inside_foo_t {
    DouddleDee = 42,
    DouddleDum = 24,
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
pub(crate) fn __cbg_out_InsideFoo(v: example_flat::InsideFoo) -> inside_foo_t {
    match v {
        example_flat::InsideFoo::DouddleDee => inside_foo_t::DouddleDee,
        example_flat::InsideFoo::DouddleDum => inside_foo_t::DouddleDum,
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
pub struct foo_t {
    pub id: u64,
    pub x86_64_field: u64,
    pub unstable_field: u64,
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
pub(crate) unsafe fn __cbg_in_Foo(v: foo_t) -> example_flat::Foo {
    {
        let foo_t { id, x86_64_field, unstable_field } = v;
        example_flat::Foo {
            id: id,
            x86_64_field: x86_64_field,
            unstable_field: unstable_field,
        }
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
pub(crate) fn __cbg_out_Foo(v: example_flat::Foo) -> foo_t {
    {
        let example_flat::Foo { id: __f0, x86_64_field: __f1, unstable_field: __f2 } = v;
        let id = __f0;
        let x86_64_field = __f1;
        let unstable_field = __f2;
        foo_t {
            id,
            x86_64_field,
            unstable_field,
        }
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
pub struct closure_value_t {
    pub context: *mut ::core::ffi::c_void,
    pub call: ::core::option::Option<
        unsafe extern "C" fn(f64, *mut ::core::ffi::c_void),
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
pub unsafe extern "C" fn calculator_new() -> *mut calculator_t {
    let __result = example_flat::calculator_new();
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(__result)) as *mut calculator_t
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
pub unsafe extern "C" fn calculator_new_from_str(
    s: *const ::core::ffi::c_char,
    e: *mut *mut ::core::ffi::c_char,
) -> *mut calculator_t {
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
            if !e.is_null() {
                *e = __cbg_alloc_cstr(__err);
            }
            return ::core::ptr::null_mut();
        }
    };
    let __result = example_flat::calculator_new_from_str(s);
    match __result {
        ::core::result::Result::Ok(__v) => {
            ::std::boxed::Box::into_raw(::std::boxed::Box::new(__v)) as *mut calculator_t
        }
        ::core::result::Result::Err(__e) => {
            if !e.is_null() {
                *e = __cbg_out_Error(__e);
            }
            ::core::ptr::null_mut()
        }
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
pub unsafe extern "C" fn calculator_apply(
    c: *mut calculator_t,
    op: ::core::mem::MaybeUninit<operation_t>,
    operand: f64,
    out: *mut f64,
    e: *mut *mut ::core::ffi::c_char,
) -> bool {
    let c = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if c.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator pointer"),
                );
            }
            &mut *(c as *mut example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __cbg_alloc_cstr(__err);
            }
            return false;
        }
    };
    let op = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(__cbg_in_Operation(op)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __cbg_alloc_cstr(__err);
            }
            return false;
        }
    };
    let operand = operand;
    let __result = example_flat::calculator_apply(c, op, operand);
    match __result {
        ::core::result::Result::Ok(__v) => {
            if !out.is_null() {
                ::core::ptr::write(out, __v);
            }
            true
        }
        ::core::result::Result::Err(__e) => {
            if !e.is_null() {
                *e = __cbg_out_Error(__e);
            }
            false
        }
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
pub unsafe extern "C" fn calculator_merge(
    a: *mut calculator_t,
    b: *mut calculator_t,
    e: *mut *mut ::core::ffi::c_char,
) -> *mut calculator_t {
    if !(a as *const ()).is_null() && (a as *const ()) == (b as *const ()) {
        let __err = ::std::string::String::from(
            "aliasing arguments: `a` (consumed) and `b` (consumed) are the same `Calculator` — a consumed or exclusively-borrowed resource may not be named twice in one call",
        );
        {
            if !e.is_null() {
                *e = __cbg_alloc_cstr(__err);
            }
            return ::core::ptr::null_mut();
        }
    }
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __p = a;
            if __p.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator handle passed by value"),
                );
            }
            *::std::boxed::Box::from_raw(__p as *mut example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __cbg_alloc_cstr(__err);
            }
            return ::core::ptr::null_mut();
        }
    };
    let b = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __p = b;
            if __p.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator handle passed by value"),
                );
            }
            *::std::boxed::Box::from_raw(__p as *mut example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __cbg_alloc_cstr(__err);
            }
            return ::core::ptr::null_mut();
        }
    };
    let __result = example_flat::calculator_merge(a, b);
    match __result {
        ::core::result::Result::Ok(__v) => {
            ::std::boxed::Box::into_raw(::std::boxed::Box::new(__v)) as *mut calculator_t
        }
        ::core::result::Result::Err(__e) => {
            if !e.is_null() {
                *e = __cbg_out_Error(__e);
            }
            ::core::ptr::null_mut()
        }
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
pub unsafe extern "C" fn calculator_absorb(
    a: *mut calculator_t,
    b: *const calculator_t,
    out: *mut f64,
    e: *mut *mut ::core::ffi::c_char,
) -> bool {
    if !(a as *const ()).is_null() && (a as *const ()) == (b as *const ()) {
        let __err = ::std::string::String::from(
            "aliasing arguments: `a` (consumed) and `b` (borrowed) are the same `Calculator` — a consumed or exclusively-borrowed resource may not be named twice in one call",
        );
        {
            if !e.is_null() {
                *e = __cbg_alloc_cstr(__err);
            }
            return false;
        }
    }
    let a = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            let __p = a;
            if __p.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator handle passed by value"),
                );
            }
            *::std::boxed::Box::from_raw(__p as *mut example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __cbg_alloc_cstr(__err);
            }
            return false;
        }
    };
    let b = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if b.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator pointer"),
                );
            }
            &*(b as *const example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __cbg_alloc_cstr(__err);
            }
            return false;
        }
    };
    let __result = example_flat::calculator_absorb(a, b);
    match __result {
        ::core::result::Result::Ok(__v) => {
            if !out.is_null() {
                ::core::ptr::write(out, __v);
            }
            true
        }
        ::core::result::Result::Err(__e) => {
            if !e.is_null() {
                *e = __cbg_out_Error(__e);
            }
            false
        }
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
pub unsafe extern "C" fn foo_new(id: u64) -> foo_t {
    let id = id;
    let __result = example_flat::foo_new(id);
    __cbg_out_Foo(__result)
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
pub unsafe extern "C" fn foo_get_id(f: foo_t) -> u64 {
    let f = __cbg_in_Foo(f);
    let __result = example_flat::foo_get_id(f);
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
pub unsafe extern "C" fn inside_foo_default() -> inside_foo_t {
    let __result = example_flat::inside_foo_default();
    __cbg_out_InsideFoo(__result)
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
pub unsafe extern "C" fn shape_new_empty() -> ::core::mem::MaybeUninit<shape_t> {
    let __result = example_flat::shape_new_empty();
    __cbg_out_Shape(__result)
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
pub unsafe extern "C" fn shape_new_circle(
    radius: f64,
) -> ::core::mem::MaybeUninit<shape_t> {
    let radius = radius;
    let __result = example_flat::shape_new_circle(radius);
    __cbg_out_Shape(__result)
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
pub unsafe extern "C" fn shape_new_rect(
    width: f64,
    height: f64,
) -> ::core::mem::MaybeUninit<shape_t> {
    let width = width;
    let height = height;
    let __result = example_flat::shape_new_rect(width, height);
    __cbg_out_Shape(__result)
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
pub unsafe extern "C" fn shape_try_area(
    s: ::core::mem::MaybeUninit<shape_t>,
    out: *mut f64,
    e: *mut *mut ::core::ffi::c_char,
) -> bool {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(__cbg_in_Shape(s)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __cbg_alloc_cstr(__err);
            }
            return false;
        }
    };
    let __result = example_flat::shape_try_area(s);
    match __result {
        ::core::result::Result::Ok(__v) => {
            if !out.is_null() {
                ::core::ptr::write(out, __v);
            }
            true
        }
        ::core::result::Result::Err(__e) => {
            if !e.is_null() {
                *e = __cbg_out_Error(__e);
            }
            false
        }
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
pub unsafe extern "C" fn note_new_silent() -> ::core::mem::MaybeUninit<note_t> {
    let __result = example_flat::note_new_silent();
    __cbg_out_Note(__result)
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
pub unsafe extern "C" fn note_new_after(
    millis: u64,
) -> ::core::mem::MaybeUninit<note_t> {
    let millis = millis;
    let __result = example_flat::note_new_after(millis);
    __cbg_out_Note(__result)
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
pub unsafe extern "C" fn note_new_flagged(
    flag: ::core::mem::MaybeUninit<bool>,
) -> ::core::mem::MaybeUninit<note_t> {
    let flag = (::core::ptr::read(flag.as_ptr() as *const u8) != 0);
    let __result = example_flat::note_new_flagged(flag);
    __cbg_out_Note(__result)
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
pub unsafe extern "C" fn calculator_reset(c: *mut calculator_t) {
    let c = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if c.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator pointer"),
                );
            }
            &mut *(c as *mut example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::calculator_reset(c);
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
pub unsafe extern "C" fn inside_foo_value(
    x: ::core::mem::MaybeUninit<inside_foo_t>,
) -> i32 {
    let x = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(__cbg_in_InsideFoo(x)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::inside_foo_value(x);
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
pub unsafe extern "C" fn shape_area(s: ::core::mem::MaybeUninit<shape_t>) -> f64 {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(__cbg_in_Shape(s)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::shape_area(s);
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
pub unsafe extern "C" fn shape_get_label(
    s: ::core::mem::MaybeUninit<shape_t>,
) -> *mut ::core::ffi::c_char {
    let s = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(__cbg_in_Shape(s)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::shape_get_label(s);
    __cbg_alloc_cstr(__result)
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
pub unsafe extern "C" fn drawing_new(
    id: u64,
    shape: ::core::mem::MaybeUninit<shape_t>,
) -> drawing_t {
    let id = id;
    let shape = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(__cbg_in_Shape(shape)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::drawing_new(id, shape);
    __cbg_out_Drawing(__result)
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
pub unsafe extern "C" fn drawing_get_shape(
    d: drawing_t,
) -> ::core::mem::MaybeUninit<shape_t> {
    let d = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(__cbg_in_Drawing(d)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::drawing_get_shape(d);
    __cbg_out_Shape(__result)
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
pub unsafe extern "C" fn note_value(n: ::core::mem::MaybeUninit<note_t>) -> u64 {
    let n = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(__cbg_in_Note(n)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::note_value(n);
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
pub unsafe extern "C" fn note_emphatic(n: ::core::mem::MaybeUninit<note_t>) -> bool {
    let n = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(__cbg_in_Note(n)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::note_emphatic(n);
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
pub unsafe extern "C" fn note_new_titled(
    id: u64,
    text: *const ::core::ffi::c_char,
    emphatic: ::core::mem::MaybeUninit<bool>,
) -> ::core::mem::MaybeUninit<note_t> {
    let id = id;
    let text = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if text.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null pointer passed for str argument"),
                );
            }
            match ::std::ffi::CStr::from_ptr(text).to_str() {
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
    let emphatic = (::core::ptr::read(emphatic.as_ptr() as *const u8) != 0);
    let __result = example_flat::note_new_titled(id, text, emphatic);
    __cbg_out_Note(__result)
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
pub unsafe extern "C" fn note_new_sketched(
    id: u64,
    label: *const ::core::ffi::c_char,
) -> ::core::mem::MaybeUninit<note_t> {
    let id = id;
    let label = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if label.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null pointer passed for str argument"),
                );
            }
            match ::std::ffi::CStr::from_ptr(label).to_str() {
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
    let __result = example_flat::note_new_sketched(id, label);
    __cbg_out_Note(__result)
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
pub unsafe extern "C" fn caption_new(
    id: u64,
    text: *const ::core::ffi::c_char,
    emphatic: ::core::mem::MaybeUninit<bool>,
) -> caption_t {
    let id = id;
    let text = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if text.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null pointer passed for str argument"),
                );
            }
            match ::std::ffi::CStr::from_ptr(text).to_str() {
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
    let emphatic = (::core::ptr::read(emphatic.as_ptr() as *const u8) != 0);
    let __result = example_flat::caption_new(id, text, emphatic);
    __cbg_out_Caption(__result)
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
pub unsafe extern "C" fn calculator_new_clone(
    c: *const calculator_t,
) -> *mut calculator_t {
    let c = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if c.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator pointer"),
                );
            }
            &*(c as *const example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::calculator_new_clone(c);
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(__result)) as *mut calculator_t
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
pub unsafe extern "C" fn calculator_get_value(c: *const calculator_t) -> f64 {
    let c = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if c.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator pointer"),
                );
            }
            &*(c as *const example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::calculator_get_value(c);
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
pub unsafe extern "C" fn calculator_get_count(c: *const calculator_t) -> u64 {
    let c = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if c.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator pointer"),
                );
            }
            &*(c as *const example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::calculator_get_count(c);
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
pub unsafe extern "C" fn calculator_is(c: *const calculator_t, value: f64) -> bool {
    let c = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if c.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator pointer"),
                );
            }
            &*(c as *const example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let value = value;
    let __result = example_flat::calculator_is(c, value);
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
pub unsafe extern "C" fn calculator_to_string(
    c: *const calculator_t,
) -> *mut ::core::ffi::c_char {
    let c = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if c.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator pointer"),
                );
            }
            &*(c as *const example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::calculator_to_string(c);
    __cbg_alloc_cstr(__result)
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
pub unsafe extern "C" fn calculator_get_history(
    c: *const calculator_t,
    len: *mut usize,
) -> *mut f64 {
    let c = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if c.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator pointer"),
                );
            }
            &*(c as *const example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::calculator_get_history(c);
    let __arr: ::std::vec::Vec<f64> = __result.into_iter().map(|__e| __e).collect();
    let (__p, __n) = __cbg_alloc_array(__arr);
    if !len.is_null() {
        *len = __n;
    }
    __p
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
pub unsafe extern "C" fn calculator_for_each(
    c: *const calculator_t,
    f: closure_value_t,
) {
    let c = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if c.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null Calculator pointer"),
                );
            }
            &*(c as *const example_flat::Calculator)
        })
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
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
        move |__a0: f64| {
            let __res = (|| -> ::core::result::Result<(), ::std::string::String> {
                let __w0 = __a0;
                if let ::core::option::Option::Some(__f) = __call {
                    unsafe { __f(__w0, __ctx.context) }
                }
                ::core::result::Result::Ok(())
            })();
            if let ::core::result::Result::Err(__err) = __res {}
        }
    };
    let __result = example_flat::calculator_for_each(c, f);
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
pub unsafe extern "C" fn shape_new_labeled(
    label: *const ::core::ffi::c_char,
    op: ::core::mem::MaybeUninit<operation_t>,
) -> ::core::mem::MaybeUninit<shape_t> {
    let label = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok({
            if label.is_null() {
                return ::core::result::Result::Err(
                    ::std::string::String::from("null pointer passed for str argument"),
                );
            }
            match ::std::ffi::CStr::from_ptr(label).to_str() {
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
    let op = match (|| -> ::core::result::Result<_, ::std::string::String> {
        ::core::result::Result::Ok(__cbg_in_Operation(op)?)
    })() {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__err) => {
            panic!("{}", __err);
        }
    };
    let __result = example_flat::shape_new_labeled(label, op);
    __cbg_out_Shape(__result)
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
