use super::*;

/// A scalar slice input `&[u8]` lowers to two wire params (`*const u8`,
/// `usize`) decoded zero-copy; a NULL pointer is an empty slice.
#[test]
fn slice_u8_input_two_params() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_zbytes_from_bytes(bytes: &[u8]) -> ZZBytes {
            unimplemented!()
        }
    );
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(ZZBytes))
        .base_name("z_zbytes")
        .function(syn::parse_quote!(z_zbytes_from_bytes));

    let src = write(cbindgen, registry, "slice_u8");
    let compact: String = src.split_whitespace().collect();

    // Two params: pointer + length.
    assert!(compact.contains("bytes:*constu8"), "{src}");
    assert!(compact.contains("bytes_len:usize"), "{src}");
    // Zero-copy decode, NULL ⇒ empty slice.
    assert!(
        compact.contains("::core::slice::from_raw_parts(bytes,bytes_len)"),
        "{src}"
    );
    // Returns the opaque handle (Box::into_raw).
    assert!(compact.contains("->*mutz_zbytes"), "{src}");
}

/// `Option<ZZBytes>` input (opaque, pointer-wire inner) reuses the handle
/// wire `z_zbytes_t*`: NULL ⇒ `None`, non-NULL is consumed via the inner
/// converter. The inner is fallible, so the decode routes through the
/// `Result<(), Error>` error channel.
#[test]
fn option_opaque_input_reuses_pointer() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_op(attachment: Option<ZZBytes>) -> Result<(), Error> {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZZBytes))
        .base_name("z_zbytes")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_op));

    let src = write(cbindgen, registry, "option_in_opaque");
    let compact: String = src.split_whitespace().collect();

    // Param reuses the bare handle pointer; NULL ⇒ None.
    assert!(compact.contains("attachment:*mutz_zbytes"), "{src}");
    assert!(
        compact.contains("if(v).is_null(){::core::option::Option::None}"),
        "{src}"
    );
    // Non-null path consumes through the inner handle converter.
    assert!(
        operation_call(&compact, "__c_in_convert_wire_to_ZZBytes_", "__present"),
        "{src}"
    );
    // Fallible inner decode routes its error through the Result channel (`*e`).
    assert!(compact.contains("e:*mutz_error"), "{src}");
    assert!(compact.contains("__c_out_convert_Error_"), "{src}");
}

/// `Option<i64>` input (scalar inner, no niche) is boxed behind a `*const`
/// pointer: NULL ⇒ `None`, else the pointee, read. Infallible.
#[test]
fn option_scalar_input_boxed_pointer() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_op(timestamp_ntp64: Option<i64>) {
            unimplemented!()
        }
    );
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .function(syn::parse_quote!(z_op));

    let src = write(cbindgen, registry, "option_in_scalar");
    let compact: String = src.split_whitespace().collect();

    // Boxed behind a const pointer; NULL ⇒ None, else `Some` of the pointee, read.
    assert!(compact.contains("timestamp_ntp64:*consti64"), "{src}");
    assert!(
        compact.contains("if(v).is_null(){::core::option::Option::None}"),
        "{src}"
    );
    assert!(compact.contains("::core::option::Option::Some"), "{src}");
    // Infallible ⇒ no error param.
    assert!(!compact.contains("e:*mut"), "{src}");
}

/// `Option<Rec>` over a `data_struct` inner takes the same `*const` wire as the
/// scalar above, and reaches the inner converter by `ptr::read`. A mirror
/// struct is not `Copy`, so dereferencing the pointer instead would be a move
/// out of it and the binding would not build (#412).
#[test]
fn option_data_struct_input_reads_the_pointee() {
    let loc = SourceLocation::default();
    let rec: syn::ItemStruct = syn::parse_quote!(
        pub struct Rec {
            pub id: u64,
            pub tag: u32,
        }
    );
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_op(v: Option<Rec>) {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Struct(rec), loc.clone()),
        (syn::Item::Fn(func), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .data_struct(syn::parse_quote!(Rec))
        .function(syn::parse_quote!(z_op));

    let src = write(cbindgen, registry, "option_in_data_struct");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("v:*constrec"), "{src}");
    assert!(
        compact.contains("let__present=::core::ptr::read(v);"),
        "{src}"
    );
    assert!(
        operation_call(&compact, "__c_in_convert_wire_to_Rec_", "__present"),
        "{src}"
    );
}

/// `&str` inputs decode directly from `const char *` and can be used by
/// non-`Result` wrappers when `.panic()` is enabled.
#[test]
fn str_borrow_input_lowering() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_init_logs(filter: &str) {
            unimplemented!()
        }
    );
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .function(syn::parse_quote!(z_init_logs))
        .panic();

    let src = write(cbindgen, registry, "str_borrow");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("extern\"C\"fnz_init_logs"), "{src}");
    assert!(
        compact.contains("filter:*const::core::ffi::c_char"),
        "{src}"
    );
    assert!(compact.contains("CStr::from_ptr(v).to_str()"), "{src}");
    assert!(compact.contains("panic!("), "{src}");
}

/// `z_keyexpr_relation_to(a: &ZKeyExpr, b: &ZKeyExpr) -> SetIntersectionLevel`
/// lowers to a borrow-input + enum-return wrapper; `.panic()` lets the
/// fallible borrow decode abort.
#[test]
fn relation_to_lowering() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_keyexpr_relation_to(a: &ZKeyExpr, b: &ZKeyExpr) -> SetIntersectionLevel {
            unimplemented!()
        }
    );
    let enum_item: syn::ItemEnum = syn::parse_quote!(
        pub enum SetIntersectionLevel {
            Disjoint = 0,
            Intersects = 1,
            Includes = 2,
            Equals = 3,
        }
    );

    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Enum(enum_item), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(ZKeyExpr))
        .base_name("z_keyexpr")
        .enum_type(syn::parse_quote!(SetIntersectionLevel))
        .base_name("z_intersection")
        .function(syn::parse_quote!(z_keyexpr_relation_to))
        .panic();

    let src = write(cbindgen, registry, "relation_to");
    let compact: String = src.split_whitespace().collect();

    // repr(C) enum mirror with discriminants — renamed via `.base_name()`.
    assert!(compact.contains("#[repr(C)]"), "{src}");
    assert!(compact.contains("pubenumz_intersection"), "{src}");
    assert!(compact.contains("Disjoint=0"), "{src}");
    // Wrapper: borrow params (renamed type) + enum return.
    assert!(
        compact.contains("extern\"C\"fnz_keyexpr_relation_to"),
        "{src}"
    );
    assert!(compact.contains("a:*constz_keyexpr"), "{src}");
    assert!(compact.contains("b:*constz_keyexpr"), "{src}");
    assert!(compact.contains("->z_intersection"), "{src}");
    // Fallible borrow decode aborts (no Result channel).
    assert!(compact.contains("panic!("), "{src}");
    // Enum output converter matches by variant name (src enum → C enum).
    assert!(
        compact.contains("zenoh_flat::SetIntersectionLevel::Disjoint=>z_intersection::Disjoint"),
        "{src}"
    );
}

/// A declared `enum_type` crossing **C → Rust** never materialises the
/// caller-supplied discriminant (#158): the wire is `MaybeUninit<mirror>`,
/// which has the mirror's ABI (and the mirror's C spelling — cbindgen renders
/// `MaybeUninit<T>` as `T`) but may legally hold any bit pattern. The raw
/// `c_int` is compared against the mirror's own variants, so an unmatched value
/// reaches the error channel instead of becoming an invalid Rust enum.
#[test]
fn enum_input_validates_the_discriminant() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_keyexpr_require(level: SetIntersectionLevel) -> Result<(), Error> {
            unimplemented!()
        }
    );
    let enum_item: syn::ItemEnum = syn::parse_quote!(
        pub enum SetIntersectionLevel {
            Disjoint = 0,
            Equals = LEVELS - 1,
        }
    );

    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Enum(enum_item), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .enum_type(syn::parse_quote!(SetIntersectionLevel))
        .base_name("z_intersection")
        .function(syn::parse_quote!(z_keyexpr_require));

    let src = write(cbindgen, registry, "enum_input_validated");
    let compact: String = src.split_whitespace().collect();

    // The wire is the bit-pattern-agnostic wrapper, in the extern and in the
    // converter — never the mirror enum itself.
    assert!(
        compact.contains("level:::core::mem::MaybeUninit<z_intersection>"),
        "{src}"
    );
    assert!(!compact.contains("(v:z_intersection)"), "{src}");
    // The discriminant is read as a plain integer and compared against the
    // mirror's variants — a `const`-driven one needs no evaluation here.
    assert!(
        compact.contains(
            "let__raw:::core::ffi::c_int=::core::ptr::read(v.as_ptr()as*const::core::ffi::c_int"
        ),
        "{src}"
    );
    assert!(
        compact.contains("if__raw==z_intersection::Equalsas::core::ffi::c_int"),
        "{src}"
    );
    assert!(compact.contains("Equals=LEVELS-1"), "{src}");
    // Unmatched ⇒ a binding error, routed to the `char **e` channel.
    assert!(
        compact.contains("invaliddiscriminant{}for`z_intersection`"),
        "{src}"
    );
    assert!(compact.contains("if!e.is_null()"), "{src}");
}

/// The same enum input in a function with **no** `Result` channel is a fallible
/// decode with nowhere to report, so it needs `.panic()` — the rule null borrows
/// already follow.
#[test]
fn enum_input_without_error_channel_requires_panic() {
    let enum_item: syn::ItemEnum = syn::parse_quote!(
        pub enum SetIntersectionLevel {
            Disjoint = 0,
            Equals = 1,
        }
    );
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_intersection_value(level: SetIntersectionLevel) -> i32 {
            unimplemented!()
        }
    );
    let build = |allow_panic: bool| {
        let loc = SourceLocation::default();
        let registry = crate::test_util::reg_from_items(declare_referenced([
            (syn::Item::Fn(func.clone()), loc.clone()),
            (syn::Item::Enum(enum_item.clone()), loc.clone()),
        ]))
        .expect("index items");
        let cbindgen = CbindgenBuilder::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .enum_type(syn::parse_quote!(SetIntersectionLevel))
            .base_name("z_intersection")
            .function(syn::parse_quote!(z_intersection_value));
        let cbindgen = if allow_panic {
            cbindgen.panic()
        } else {
            cbindgen
        };
        write(cbindgen, registry, "enum_input_panic")
    };

    assert!(catch(|| {
        build(false);
    }));

    let src = build(true);
    let compact: String = src.split_whitespace().collect();
    assert!(compact.contains("panic!("), "{src}");
}

/// A mutable borrow of an opaque handle lowers to `*mut <handle>` and
/// decodes back to `&mut T`.
#[test]
fn mutable_opaque_borrow_input_lowering() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_config_insert_json5(
            c: &mut ZConfig,
            key: String,
            value: String,
        ) -> Result<(), Error> {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZConfig))
        .base_name("z_config")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_config_insert_json5));

    let src = write(cbindgen, registry, "mut_opaque_borrow");
    let compact: String = src.split_whitespace().collect();

    assert!(
        compact.contains("extern\"C\"fnz_config_insert_json5"),
        "{src}"
    );
    assert!(compact.contains("c:*mutz_config"), "{src}");
    // The handle pointer IS the box — decode directly, no `_0` indirection.
    assert!(!compact.contains("__h._0"), "{src}");
    assert!(
        compact.contains("Result::Ok(&mut*(vas*mutzenoh_flat::ZConfig))"),
        "{src}"
    );
}
