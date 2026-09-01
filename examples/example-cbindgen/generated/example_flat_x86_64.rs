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
    pub stable_field: u64,
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
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_impl_Fn_Vec_f64_Send_Sync_static_c_invoke_callback_capture_3dd7f8fbc61877ce(
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
            let __arr: ::std::vec::Vec<f64> = __c_out_convert_sequence_Vec_f64_to_wire_ad99887ef4e62c28(
                __a0,
            );
            let (__p, __n) = __cbg_alloc_array(__arr);
            *__w0_0.as_mut_ptr() = __p;
            *__w0_1.as_mut_ptr() = __n;
            unsafe { __f(__w0_0, __w0_1, __ctx.context) }
        }
    }
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
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_impl_Fn_Option_Grade_Send_Sync_static_c_invoke_callback_capture_5512a1f2265e79a0(
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
                    *__w0_1.as_mut_ptr() = __c_out_convert_Grade_c_terminal_output_enum_to_wire_a59c7b101e0a9e37(
                        __x,
                    );
                }
                ::core::option::Option::None => {
                    *__w0_0.as_mut_ptr() = false;
                }
            }
            unsafe { __f(__w0_0, __w0_1, __ctx.context) }
        }
    }
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
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_impl_Fn_Option_f64_Send_Sync_static_c_invoke_callback_capture_053af7d76b4f1245(
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
                    *__w0_1.as_mut_ptr() = __c_out_convert_f64_c_terminal_output_scalar_to_wire_6aa94606ef14f673(
                        __x,
                    );
                }
                ::core::option::Option::None => {
                    *__w0_0.as_mut_ptr() = false;
                }
            }
            unsafe { __f(__w0_0, __w0_1, __ctx.context) }
        }
    }
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
pub(crate) unsafe fn __c_in_convert_wire_to_impl_Fn_f64_Send_Sync_static_c_invoke_callback_capture_88739bf29d2a9906(
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
            let __w0 = __c_out_convert_f64_c_terminal_output_scalar_to_wire_6aa94606ef14f673(
                __a0,
            );
            unsafe { __f(__w0, __ctx.context) }
        }
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_Calculator_c_borrow_shared_input_48be88e9df13aa53<
    'a,
>(
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
pub(crate) unsafe fn __c_in_convert_wire_to_Calculator_c_borrow_mutable_input_f30bfe45043bc69c<
    'a,
>(
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
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_Calculator_c_terminal_input_owned_handle_b7bb400a642eb999(
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
pub(crate) fn __c_out_convert_Calculator_c_terminal_output_owned_handle_to_wire_4d20353780559007(
    v: example_flat::Calculator,
) -> *mut calculator_t {
    ::std::boxed::Box::into_raw(::std::boxed::Box::new(v)) as *mut calculator_t
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_String_c_terminal_input_string_field_b6091e2e8553ccde(
    v: *const ::core::ffi::c_char,
) -> ::std::string::String {
    if v.is_null() {
        ::std::string::String::new()
    } else {
        ::std::ffi::CStr::from_ptr(v).to_string_lossy().into_owned()
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_bool_c_terminal_input_bool_e48e0629cd6287b3(
    v: ::core::mem::MaybeUninit<bool>,
) -> bool {
    ::core::ptr::read(v.as_ptr() as *const u8) != 0
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_in_convert_wire_to_u64_c_terminal_input_scalar_4e87470ad83635c8(
    v: u64,
) -> u64 {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_Caption_c_product_intermediate_repr_c_struct_a9076d1a0d6740b3(
    v: caption_t,
) -> example_flat::Caption {
    example_flat::Caption {
        id: __c_in_convert_wire_to_u64_c_terminal_input_scalar_4e87470ad83635c8(v.id),
        text: __c_in_convert_wire_to_String_c_terminal_input_string_field_b6091e2e8553ccde(
            v.text,
        ),
        emphatic: __c_in_convert_wire_to_bool_c_terminal_input_bool_e48e0629cd6287b3(
            v.emphatic,
        ),
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_String_c_terminal_output_string_to_wire_182528409f6ab8d3(
    v: ::std::string::String,
) -> *mut ::core::ffi::c_char {
    __cbg_alloc_cstr(v)
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_bool_c_terminal_output_bool_field_to_wire_6a810eb4cb986700(
    v: bool,
) -> ::core::mem::MaybeUninit<bool> {
    ::core::mem::MaybeUninit::new(v)
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_u64_c_terminal_output_scalar_to_wire_518245fe60cf3590(
    v: u64,
) -> u64 {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_Caption_c_product_intermediate_repr_c_struct_to_wire_3bc5d236333a6e28(
    v: example_flat::Caption,
) -> caption_t {
    caption_t {
        id: __c_out_convert_u64_c_terminal_output_scalar_to_wire_518245fe60cf3590(v.id),
        text: __c_out_convert_String_c_terminal_output_string_to_wire_182528409f6ab8d3(
            v.text,
        ),
        emphatic: __c_out_convert_bool_c_terminal_output_bool_field_to_wire_6a810eb4cb986700(
            v.emphatic,
        ),
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_Operation_c_terminal_input_enum_a23b6023635a8da5(
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
pub(crate) fn __c_in_convert_wire_to_f64_c_terminal_input_scalar_7d8a0a733495e599(
    v: f64,
) -> f64 {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_Shape_c_choice_intermediate_repr_c_tagged_union_8ddb52c185c8b923(
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
                example_flat::Shape::Circle(
                    __c_in_convert_wire_to_f64_c_terminal_input_scalar_7d8a0a733495e599(
                        (__arm).0,
                    ),
                )
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
                    width: __c_in_convert_wire_to_f64_c_terminal_input_scalar_7d8a0a733495e599(
                        (__arm).0,
                    ),
                    height: __c_in_convert_wire_to_f64_c_terminal_input_scalar_7d8a0a733495e599(
                        (__arm).1,
                    ),
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
                    __c_in_convert_wire_to_String_c_terminal_input_string_field_b6091e2e8553ccde(
                        (__arm).0,
                    ),
                    __c_in_convert_wire_to_Operation_c_terminal_input_enum_a23b6023635a8da5(
                        (__arm).1,
                    )?,
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
pub(crate) unsafe fn __c_in_convert_wire_to_Drawing_c_product_intermediate_repr_c_struct_3b927b21caf2df63(
    v: drawing_t,
) -> ::core::result::Result<example_flat::Drawing, ::std::string::String> {
    ::core::result::Result::Ok(example_flat::Drawing {
        id: __c_in_convert_wire_to_u64_c_terminal_input_scalar_4e87470ad83635c8(v.id),
        shape: __c_in_convert_wire_to_Shape_c_choice_intermediate_repr_c_tagged_union_8ddb52c185c8b923(
            v.shape,
        )?,
    })
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_Operation_c_terminal_output_enum_to_wire_457f6036394dc821(
    v: example_flat::Operation,
) -> operation_t {
    match v {
        example_flat::Operation::Add => operation_t::Add,
        example_flat::Operation::Sub => operation_t::Sub,
        example_flat::Operation::Mul => operation_t::Mul,
        example_flat::Operation::Div => operation_t::Div,
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_f64_c_terminal_output_scalar_to_wire_6aa94606ef14f673(
    v: f64,
) -> f64 {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_Shape_c_choice_intermediate_repr_c_tagged_union_to_wire_1c9175b2e9cd70a6(
    v: example_flat::Shape,
) -> ::core::mem::MaybeUninit<shape_t> {
    ::core::mem::MaybeUninit::new({
        match v {
            example_flat::Shape::Empty => shape_t::Empty,
            example_flat::Shape::Circle(__part0) => {
                let __built_arm = (
                    __c_out_convert_f64_c_terminal_output_scalar_to_wire_6aa94606ef14f673(
                        __part0,
                    ),
                );
                shape_t::Circle(__built_arm.0)
            }
            example_flat::Shape::Rect { width: __part0, height: __part1 } => {
                let __built_arm = (
                    __c_out_convert_f64_c_terminal_output_scalar_to_wire_6aa94606ef14f673(
                        __part0,
                    ),
                    __c_out_convert_f64_c_terminal_output_scalar_to_wire_6aa94606ef14f673(
                        __part1,
                    ),
                );
                shape_t::Rect {
                    width: __built_arm.0,
                    height: __built_arm.1,
                }
            }
            example_flat::Shape::Labeled(__part0, __part1) => {
                let __built_arm = (
                    __c_out_convert_String_c_terminal_output_string_to_wire_182528409f6ab8d3(
                        __part0,
                    ),
                    ::core::mem::MaybeUninit::new(
                        __c_out_convert_Operation_c_terminal_output_enum_to_wire_457f6036394dc821(
                            __part1,
                        ),
                    ),
                );
                shape_t::Labeled(__built_arm.0, __built_arm.1)
            }
        }
    })
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_Drawing_c_product_intermediate_repr_c_struct_to_wire_50eac9aa069a6838(
    v: example_flat::Drawing,
) -> drawing_t {
    drawing_t {
        id: __c_out_convert_u64_c_terminal_output_scalar_to_wire_518245fe60cf3590(v.id),
        shape: __c_out_convert_Shape_c_choice_intermediate_repr_c_tagged_union_to_wire_1c9175b2e9cd70a6(
            v.shape,
        ),
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
    v: example_flat::Error,
) -> *mut ::core::ffi::c_char {
    __cbg_alloc_cstr(example_flat::error_get_message(&v))
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_Foo_c_product_intermediate_repr_c_struct_157d1b61f2a5b9d5(
    v: foo_t,
) -> example_flat::Foo {
    example_flat::Foo {
        id: __c_in_convert_wire_to_u64_c_terminal_input_scalar_4e87470ad83635c8(v.id),
        x86_64_field: __c_in_convert_wire_to_u64_c_terminal_input_scalar_4e87470ad83635c8(
            v.x86_64_field,
        ),
        stable_field: __c_in_convert_wire_to_u64_c_terminal_input_scalar_4e87470ad83635c8(
            v.stable_field,
        ),
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_Foo_c_product_intermediate_repr_c_struct_to_wire_02ab0b068798553e(
    v: example_flat::Foo,
) -> foo_t {
    foo_t {
        id: __c_out_convert_u64_c_terminal_output_scalar_to_wire_518245fe60cf3590(v.id),
        x86_64_field: __c_out_convert_u64_c_terminal_output_scalar_to_wire_518245fe60cf3590(
            v.x86_64_field,
        ),
        stable_field: __c_out_convert_u64_c_terminal_output_scalar_to_wire_518245fe60cf3590(
            v.stable_field,
        ),
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_Grade_c_terminal_output_enum_to_wire_a59c7b101e0a9e37(
    v: example_flat::Grade,
) -> grade_t {
    match v {
        example_flat::Grade::Low => grade_t::Low,
        example_flat::Grade::High => grade_t::High,
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_InsideFoo_c_terminal_input_enum_70ed7847e05e3330(
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
pub(crate) fn __c_out_convert_InsideFoo_c_terminal_output_enum_to_wire_b103ad5e4be33376(
    v: example_flat::InsideFoo,
) -> inside_foo_t {
    match v {
        example_flat::InsideFoo::DouddleDee => inside_foo_t::DouddleDee,
        example_flat::InsideFoo::DouddleDum => inside_foo_t::DouddleDum,
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_in_convert_wire_to_Millis_c_terminal_custom_771f8034eae1c639(
    v: u64,
) -> example_flat::Millis {
    example_flat::millis_from_raw(v)
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_Millis_c_terminal_custom_to_wire_0c2f6b5f81dfcd94(
    v: example_flat::Millis,
) -> u64 {
    example_flat::millis_to_raw(&v)
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) unsafe fn __c_in_convert_wire_to_Note_c_choice_intermediate_repr_c_tagged_union_4de1f2981255d608(
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
                example_flat::Note::Titled(
                    __c_in_convert_wire_to_Caption_c_product_intermediate_repr_c_struct_a9076d1a0d6740b3(
                        (__arm).0,
                    ),
                )
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
                example_flat::Note::After(
                    __c_in_convert_wire_to_Millis_c_terminal_custom_771f8034eae1c639(
                        (__arm).0,
                    ),
                )
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
                example_flat::Note::Flagged(
                    __c_in_convert_wire_to_bool_c_terminal_input_bool_e48e0629cd6287b3(
                        (__arm).0,
                    ),
                )
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
                example_flat::Note::Sketched(
                    __c_in_convert_wire_to_Drawing_c_product_intermediate_repr_c_struct_3b927b21caf2df63(
                        (__arm).0,
                    )?,
                )
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
pub(crate) fn __c_out_convert_Note_c_choice_intermediate_repr_c_tagged_union_to_wire_c9cea85d5266d21b(
    v: example_flat::Note,
) -> ::core::mem::MaybeUninit<note_t> {
    ::core::mem::MaybeUninit::new({
        match v {
            example_flat::Note::Silent => note_t::Silent,
            example_flat::Note::Titled(__part0) => {
                let __built_arm = (
                    __c_out_convert_Caption_c_product_intermediate_repr_c_struct_to_wire_3bc5d236333a6e28(
                        __part0,
                    ),
                );
                note_t::Titled(__built_arm.0)
            }
            example_flat::Note::After(__part0) => {
                let __built_arm = (
                    __c_out_convert_Millis_c_terminal_custom_to_wire_0c2f6b5f81dfcd94(
                        __part0,
                    ),
                );
                note_t::After(__built_arm.0)
            }
            example_flat::Note::Flagged(__part0) => {
                let __built_arm = (
                    __c_out_convert_bool_c_terminal_output_bool_field_to_wire_6a810eb4cb986700(
                        __part0,
                    ),
                );
                note_t::Flagged(__built_arm.0)
            }
            example_flat::Note::Sketched(__part0) => {
                let __built_arm = (
                    __c_out_convert_Drawing_c_product_intermediate_repr_c_struct_to_wire_50eac9aa069a6838(
                        __part0,
                    ),
                );
                note_t::Sketched(__built_arm.0)
            }
        }
    })
}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __c_out_convert_Option_Grade_c_marker_optional_to_wire_e2c04af90b0abd17() {}
#[allow(non_snake_case, dead_code, unused)]
pub(crate) fn __c_out_convert_Option_f64_c_marker_optional_to_wire_c1ba5adc99623ff4() {}
#[allow(non_snake_case, unused_variables, dead_code)]
#[inline(always)]
pub(crate) fn __c_out_convert_sequence_Vec_f64_to_wire_ad99887ef4e62c28(
    v: ::std::vec::Vec<f64>,
) -> ::std::vec::Vec<f64> {
    {
        let __sequence_source = v;
        let mut __sequence_output: ::std::vec::Vec<f64> = ::std::vec::Vec::with_capacity(
            (__sequence_source).len(),
        );
        for __sequence_element in __sequence_source.into_iter() {
            let __sequence_part = __c_out_convert_f64_c_terminal_output_scalar_to_wire_6aa94606ef14f673(
                __sequence_element,
            );
            __sequence_output.push(__sequence_part);
        }
        __sequence_output
    }
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_bool_c_terminal_output_scalar_to_wire_cc0ad9760da17efd(
    v: bool,
) -> bool {
    v
}
#[allow(non_snake_case, unused_variables, dead_code)]
pub(crate) fn __c_out_convert_i32_c_terminal_output_scalar_to_wire_ae82162f636ebcd5(
    v: i32,
) -> i32 {
    v
}
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
            *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
                <example_flat::Error as ::core::convert::From<
                    ::std::string::String,
                >>::from(__msg),
            );
        }
        return false;
    }
    let a = match __c_in_convert_wire_to_Calculator_c_terminal_input_owned_handle_b7bb400a642eb999(
        a,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
                    <example_flat::Error as ::core::convert::From<
                        ::std::string::String,
                    >>::from(__msg),
                );
            }
            return false;
        }
    };
    let b = match __c_in_convert_wire_to_Calculator_c_borrow_shared_input_48be88e9df13aa53(
        b,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
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
            *out = __c_out_convert_f64_c_terminal_output_scalar_to_wire_6aa94606ef14f673(
                __v,
            );
            true
        }
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
                    __err,
                );
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
    let c = match __c_in_convert_wire_to_Calculator_c_borrow_mutable_input_f30bfe45043bc69c(
        c,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
                    <example_flat::Error as ::core::convert::From<
                        ::std::string::String,
                    >>::from(__msg),
                );
            }
            return false;
        }
    };
    let op = match __c_in_convert_wire_to_Operation_c_terminal_input_enum_a23b6023635a8da5(
        op,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
                    <example_flat::Error as ::core::convert::From<
                        ::std::string::String,
                    >>::from(__msg),
                );
            }
            return false;
        }
    };
    let operand = __c_in_convert_wire_to_f64_c_terminal_input_scalar_7d8a0a733495e599(
        operand,
    );
    match example_flat::calculator_apply(c, op, operand) {
        ::core::result::Result::Ok(__v) => {
            *out = __c_out_convert_f64_c_terminal_output_scalar_to_wire_6aa94606ef14f673(
                __v,
            );
            true
        }
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
                    __err,
                );
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
    let c = match __c_in_convert_wire_to_Calculator_c_borrow_shared_input_48be88e9df13aa53(
        c,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let f = __c_in_convert_wire_to_impl_Fn_f64_Send_Sync_static_c_invoke_callback_capture_88739bf29d2a9906(
        f,
    );
    example_flat::calculator_for_each(c, f);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_get_count(c: *const calculator_t) -> u64 {
    let c = match __c_in_convert_wire_to_Calculator_c_borrow_shared_input_48be88e9df13aa53(
        c,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::calculator_get_count(c);
    let __ret: u64;
    __ret = __c_out_convert_u64_c_terminal_output_scalar_to_wire_518245fe60cf3590(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_get_history(
    c: *const calculator_t,
    len: *mut usize,
) -> *mut f64 {
    let c = match __c_in_convert_wire_to_Calculator_c_borrow_shared_input_48be88e9df13aa53(
        c,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::calculator_get_history(c);
    let __ret: *mut f64;
    let __arr: ::std::vec::Vec<f64> = __c_out_convert_sequence_Vec_f64_to_wire_ad99887ef4e62c28(
        __v,
    );
    let (__p, __n) = __cbg_alloc_array(__arr);
    __ret = __p;
    *len = __n;
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_get_value(c: *const calculator_t) -> f64 {
    let c = match __c_in_convert_wire_to_Calculator_c_borrow_shared_input_48be88e9df13aa53(
        c,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::calculator_get_value(c);
    let __ret: f64;
    __ret = __c_out_convert_f64_c_terminal_output_scalar_to_wire_6aa94606ef14f673(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_grade_or_none(
    c: *const calculator_t,
    f: closure_maybe_grade_t,
) {
    let c = match __c_in_convert_wire_to_Calculator_c_borrow_shared_input_48be88e9df13aa53(
        c,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let f = __c_in_convert_wire_to_impl_Fn_Option_Grade_Send_Sync_static_c_invoke_callback_capture_5512a1f2265e79a0(
        f,
    );
    example_flat::calculator_grade_or_none(c, f);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_history_batch(
    c: *const calculator_t,
    f: closure_history_batch_t,
) {
    let c = match __c_in_convert_wire_to_Calculator_c_borrow_shared_input_48be88e9df13aa53(
        c,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let f = __c_in_convert_wire_to_impl_Fn_Vec_f64_Send_Sync_static_c_invoke_callback_capture_3dd7f8fbc61877ce(
        f,
    );
    example_flat::calculator_history_batch(c, f);
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_is(c: *const calculator_t, value: f64) -> bool {
    let c = match __c_in_convert_wire_to_Calculator_c_borrow_shared_input_48be88e9df13aa53(
        c,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let value = __c_in_convert_wire_to_f64_c_terminal_input_scalar_7d8a0a733495e599(
        value,
    );
    let __v = example_flat::calculator_is(c, value);
    let __ret: bool;
    __ret = __c_out_convert_bool_c_terminal_output_scalar_to_wire_cc0ad9760da17efd(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_last_or_none(
    c: *const calculator_t,
    f: closure_maybe_value_t,
) {
    let c = match __c_in_convert_wire_to_Calculator_c_borrow_shared_input_48be88e9df13aa53(
        c,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let f = __c_in_convert_wire_to_impl_Fn_Option_f64_Send_Sync_static_c_invoke_callback_capture_053af7d76b4f1245(
        f,
    );
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
            *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
                <example_flat::Error as ::core::convert::From<
                    ::std::string::String,
                >>::from(__msg),
            );
        }
        return ::core::ptr::null_mut();
    }
    let a = match __c_in_convert_wire_to_Calculator_c_terminal_input_owned_handle_b7bb400a642eb999(
        a,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
                    <example_flat::Error as ::core::convert::From<
                        ::std::string::String,
                    >>::from(__msg),
                );
            }
            return ::core::ptr::null_mut();
        }
    };
    let b = match __c_in_convert_wire_to_Calculator_c_terminal_input_owned_handle_b7bb400a642eb999(
        b,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
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
            __ret = __c_out_convert_Calculator_c_terminal_output_owned_handle_to_wire_4d20353780559007(
                __v,
            );
            __ret
        }
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
                    __err,
                );
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
    __ret = __c_out_convert_Calculator_c_terminal_output_owned_handle_to_wire_4d20353780559007(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_new_clone(
    c: *const calculator_t,
) -> *mut calculator_t {
    let c = match __c_in_convert_wire_to_Calculator_c_borrow_shared_input_48be88e9df13aa53(
        c,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::calculator_new_clone(c);
    let __ret: *mut calculator_t;
    __ret = __c_out_convert_Calculator_c_terminal_output_owned_handle_to_wire_4d20353780559007(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_new_from_str(
    s: *const ::core::ffi::c_char,
    e: *mut *mut ::core::ffi::c_char,
) -> *mut calculator_t {
    let s = match __c_in_convert_wire_to_str_c_borrow_str_input_246c2b9955bb6ef2(s) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
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
            __ret = __c_out_convert_Calculator_c_terminal_output_owned_handle_to_wire_4d20353780559007(
                __v,
            );
            __ret
        }
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
                    __err,
                );
            }
            ::core::ptr::null_mut()
        }
    }
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn calculator_to_string(
    c: *const calculator_t,
) -> *mut ::core::ffi::c_char {
    let c = match __c_in_convert_wire_to_Calculator_c_borrow_shared_input_48be88e9df13aa53(
        c,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::calculator_to_string(c);
    let __ret: *mut ::core::ffi::c_char;
    __ret = __c_out_convert_String_c_terminal_output_string_to_wire_182528409f6ab8d3(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn caption_new(
    id: u64,
    text: *const ::core::ffi::c_char,
    emphatic: ::core::mem::MaybeUninit<bool>,
) -> caption_t {
    let id = __c_in_convert_wire_to_u64_c_terminal_input_scalar_4e87470ad83635c8(id);
    let text = match __c_in_convert_wire_to_str_c_borrow_str_input_246c2b9955bb6ef2(
        text,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let emphatic = __c_in_convert_wire_to_bool_c_terminal_input_bool_e48e0629cd6287b3(
        emphatic,
    );
    let __v = example_flat::caption_new(id, text, emphatic);
    let __ret: caption_t;
    __ret = __c_out_convert_Caption_c_product_intermediate_repr_c_struct_to_wire_3bc5d236333a6e28(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn drawing_get_shape(
    d: drawing_t,
) -> ::core::mem::MaybeUninit<shape_t> {
    let d = match __c_in_convert_wire_to_Drawing_c_product_intermediate_repr_c_struct_3b927b21caf2df63(
        d,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::drawing_get_shape(d);
    let __ret: ::core::mem::MaybeUninit<shape_t>;
    __ret = __c_out_convert_Shape_c_choice_intermediate_repr_c_tagged_union_to_wire_1c9175b2e9cd70a6(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn drawing_new(
    id: u64,
    shape: ::core::mem::MaybeUninit<shape_t>,
) -> drawing_t {
    let id = __c_in_convert_wire_to_u64_c_terminal_input_scalar_4e87470ad83635c8(id);
    let shape = match __c_in_convert_wire_to_Shape_c_choice_intermediate_repr_c_tagged_union_8ddb52c185c8b923(
        shape,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::drawing_new(id, shape);
    let __ret: drawing_t;
    __ret = __c_out_convert_Drawing_c_product_intermediate_repr_c_struct_to_wire_50eac9aa069a6838(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn foo_get_id(f: foo_t) -> u64 {
    let f = __c_in_convert_wire_to_Foo_c_product_intermediate_repr_c_struct_157d1b61f2a5b9d5(
        f,
    );
    let __v = example_flat::foo_get_id(f);
    let __ret: u64;
    __ret = __c_out_convert_u64_c_terminal_output_scalar_to_wire_518245fe60cf3590(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn foo_new(id: u64) -> foo_t {
    let id = __c_in_convert_wire_to_u64_c_terminal_input_scalar_4e87470ad83635c8(id);
    let __v = example_flat::foo_new(id);
    let __ret: foo_t;
    __ret = __c_out_convert_Foo_c_product_intermediate_repr_c_struct_to_wire_02ab0b068798553e(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn inside_foo_default() -> inside_foo_t {
    let __v = example_flat::inside_foo_default();
    let __ret: inside_foo_t;
    __ret = __c_out_convert_InsideFoo_c_terminal_output_enum_to_wire_b103ad5e4be33376(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn inside_foo_value(
    x: ::core::mem::MaybeUninit<inside_foo_t>,
) -> i32 {
    let x = match __c_in_convert_wire_to_InsideFoo_c_terminal_input_enum_70ed7847e05e3330(
        x,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::inside_foo_value(x);
    let __ret: i32;
    __ret = __c_out_convert_i32_c_terminal_output_scalar_to_wire_ae82162f636ebcd5(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_emphatic(n: ::core::mem::MaybeUninit<note_t>) -> bool {
    let n = match __c_in_convert_wire_to_Note_c_choice_intermediate_repr_c_tagged_union_4de1f2981255d608(
        n,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::note_emphatic(n);
    let __ret: bool;
    __ret = __c_out_convert_bool_c_terminal_output_scalar_to_wire_cc0ad9760da17efd(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_new_after(
    millis: u64,
) -> ::core::mem::MaybeUninit<note_t> {
    let millis = __c_in_convert_wire_to_u64_c_terminal_input_scalar_4e87470ad83635c8(
        millis,
    );
    let __v = example_flat::note_new_after(millis);
    let __ret: ::core::mem::MaybeUninit<note_t>;
    __ret = __c_out_convert_Note_c_choice_intermediate_repr_c_tagged_union_to_wire_c9cea85d5266d21b(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_new_flagged(
    flag: ::core::mem::MaybeUninit<bool>,
) -> ::core::mem::MaybeUninit<note_t> {
    let flag = __c_in_convert_wire_to_bool_c_terminal_input_bool_e48e0629cd6287b3(flag);
    let __v = example_flat::note_new_flagged(flag);
    let __ret: ::core::mem::MaybeUninit<note_t>;
    __ret = __c_out_convert_Note_c_choice_intermediate_repr_c_tagged_union_to_wire_c9cea85d5266d21b(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_new_silent() -> ::core::mem::MaybeUninit<note_t> {
    let __v = example_flat::note_new_silent();
    let __ret: ::core::mem::MaybeUninit<note_t>;
    __ret = __c_out_convert_Note_c_choice_intermediate_repr_c_tagged_union_to_wire_c9cea85d5266d21b(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_new_sketched(
    id: u64,
    label: *const ::core::ffi::c_char,
) -> ::core::mem::MaybeUninit<note_t> {
    let id = __c_in_convert_wire_to_u64_c_terminal_input_scalar_4e87470ad83635c8(id);
    let label = match __c_in_convert_wire_to_str_c_borrow_str_input_246c2b9955bb6ef2(
        label,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::note_new_sketched(id, label);
    let __ret: ::core::mem::MaybeUninit<note_t>;
    __ret = __c_out_convert_Note_c_choice_intermediate_repr_c_tagged_union_to_wire_c9cea85d5266d21b(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_new_titled(
    id: u64,
    text: *const ::core::ffi::c_char,
    emphatic: ::core::mem::MaybeUninit<bool>,
) -> ::core::mem::MaybeUninit<note_t> {
    let id = __c_in_convert_wire_to_u64_c_terminal_input_scalar_4e87470ad83635c8(id);
    let text = match __c_in_convert_wire_to_str_c_borrow_str_input_246c2b9955bb6ef2(
        text,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let emphatic = __c_in_convert_wire_to_bool_c_terminal_input_bool_e48e0629cd6287b3(
        emphatic,
    );
    let __v = example_flat::note_new_titled(id, text, emphatic);
    let __ret: ::core::mem::MaybeUninit<note_t>;
    __ret = __c_out_convert_Note_c_choice_intermediate_repr_c_tagged_union_to_wire_c9cea85d5266d21b(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn note_value(n: ::core::mem::MaybeUninit<note_t>) -> u64 {
    let n = match __c_in_convert_wire_to_Note_c_choice_intermediate_repr_c_tagged_union_4de1f2981255d608(
        n,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::note_value(n);
    let __ret: u64;
    __ret = __c_out_convert_u64_c_terminal_output_scalar_to_wire_518245fe60cf3590(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_area(s: ::core::mem::MaybeUninit<shape_t>) -> f64 {
    let s = match __c_in_convert_wire_to_Shape_c_choice_intermediate_repr_c_tagged_union_8ddb52c185c8b923(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::shape_area(s);
    let __ret: f64;
    __ret = __c_out_convert_f64_c_terminal_output_scalar_to_wire_6aa94606ef14f673(__v);
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_get_label(
    s: ::core::mem::MaybeUninit<shape_t>,
) -> *mut ::core::ffi::c_char {
    let s = match __c_in_convert_wire_to_Shape_c_choice_intermediate_repr_c_tagged_union_8ddb52c185c8b923(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::shape_get_label(s);
    let __ret: *mut ::core::ffi::c_char;
    __ret = __c_out_convert_String_c_terminal_output_string_to_wire_182528409f6ab8d3(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_new_circle(
    radius: f64,
) -> ::core::mem::MaybeUninit<shape_t> {
    let radius = __c_in_convert_wire_to_f64_c_terminal_input_scalar_7d8a0a733495e599(
        radius,
    );
    let __v = example_flat::shape_new_circle(radius);
    let __ret: ::core::mem::MaybeUninit<shape_t>;
    __ret = __c_out_convert_Shape_c_choice_intermediate_repr_c_tagged_union_to_wire_1c9175b2e9cd70a6(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_new_empty() -> ::core::mem::MaybeUninit<shape_t> {
    let __v = example_flat::shape_new_empty();
    let __ret: ::core::mem::MaybeUninit<shape_t>;
    __ret = __c_out_convert_Shape_c_choice_intermediate_repr_c_tagged_union_to_wire_1c9175b2e9cd70a6(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_new_labeled(
    label: *const ::core::ffi::c_char,
    op: ::core::mem::MaybeUninit<operation_t>,
) -> ::core::mem::MaybeUninit<shape_t> {
    let label = match __c_in_convert_wire_to_str_c_borrow_str_input_246c2b9955bb6ef2(
        label,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let op = match __c_in_convert_wire_to_Operation_c_terminal_input_enum_a23b6023635a8da5(
        op,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            panic!("{}", __msg);
        }
    };
    let __v = example_flat::shape_new_labeled(label, op);
    let __ret: ::core::mem::MaybeUninit<shape_t>;
    __ret = __c_out_convert_Shape_c_choice_intermediate_repr_c_tagged_union_to_wire_1c9175b2e9cd70a6(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_new_rect(
    width: f64,
    height: f64,
) -> ::core::mem::MaybeUninit<shape_t> {
    let width = __c_in_convert_wire_to_f64_c_terminal_input_scalar_7d8a0a733495e599(
        width,
    );
    let height = __c_in_convert_wire_to_f64_c_terminal_input_scalar_7d8a0a733495e599(
        height,
    );
    let __v = example_flat::shape_new_rect(width, height);
    let __ret: ::core::mem::MaybeUninit<shape_t>;
    __ret = __c_out_convert_Shape_c_choice_intermediate_repr_c_tagged_union_to_wire_1c9175b2e9cd70a6(
        __v,
    );
    __ret
}
#[no_mangle]
#[allow(non_snake_case, unused_mut, unused_variables, unused_unsafe, dead_code)]
pub unsafe extern "C" fn shape_try_area(
    s: ::core::mem::MaybeUninit<shape_t>,
    out: *mut f64,
    e: *mut *mut ::core::ffi::c_char,
) -> bool {
    let s = match __c_in_convert_wire_to_Shape_c_choice_intermediate_repr_c_tagged_union_8ddb52c185c8b923(
        s,
    ) {
        ::core::result::Result::Ok(__v) => __v,
        ::core::result::Result::Err(__msg) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
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
            *out = __c_out_convert_f64_c_terminal_output_scalar_to_wire_6aa94606ef14f673(
                __v,
            );
            true
        }
        ::core::result::Result::Err(__err) => {
            if !e.is_null() {
                *e = __c_out_convert_Error_c_terminal_output_opaque_error_to_wire_9d010015abcb2259(
                    __err,
                );
            }
            false
        }
    }
}
const _: () = {
    konst::assertc_eq!(
        example_flat::FEATURES, "",
        "prebindgen: features mismatch between source crate and prebindgen generated file.\n\
                        This usually happens if source crate is compiled with different feature set\n\
                        for build dependencies and for library usage. You may need to explicitly set\n\
                        the necessary features."
    );
};
