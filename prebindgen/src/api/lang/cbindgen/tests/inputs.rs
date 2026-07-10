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
    let mut registry =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(ZZBytes))
        .base_name("z_zbytes")
        .function(syn::parse_quote!(z_zbytes_from_bytes));

    let src = write(&cbindgen, &mut registry, "slice_u8");
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
    let mut registry = Registry::<()>::from_items([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ])
    .expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZZBytes))
        .base_name("z_zbytes")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_op));

    let src = write(&cbindgen, &mut registry, "option_in_opaque");
    let compact: String = src.split_whitespace().collect();

    // Param reuses the bare handle pointer; NULL ⇒ None.
    assert!(compact.contains("attachment:*mutz_zbytes"), "{src}");
    assert!(
        compact.contains(
            "ifv.is_null(){return::core::result::Result::Ok(::core::option::Option::None);}"
        ),
        "{src}"
    );
    // Non-null path consumes through the inner handle converter.
    assert!(compact.contains("match__cbg_in_ZZBytes(v)"), "{src}");
    // Fallible inner decode routes its error through the Result channel (`*e`).
    assert!(compact.contains("e:*mutz_error"), "{src}");
    assert!(compact.contains("__cbg_out_Error"), "{src}");
}

/// `Option<i64>` input (scalar inner, no niche) is boxed behind a `*const`
/// pointer: NULL ⇒ `None`, else `*v`. Infallible.
#[test]
fn option_scalar_input_boxed_pointer() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_op(timestamp_ntp64: Option<i64>) {
            unimplemented!()
        }
    );
    let mut registry =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .function(syn::parse_quote!(z_op));

    let src = write(&cbindgen, &mut registry, "option_in_scalar");
    let compact: String = src.split_whitespace().collect();

    // Boxed behind a const pointer; NULL ⇒ None, else `Some(*v)`.
    assert!(compact.contains("timestamp_ntp64:*consti64"), "{src}");
    assert!(
        compact.contains("ifv.is_null(){::core::option::Option::None}"),
        "{src}"
    );
    assert!(compact.contains("::core::option::Option::Some"), "{src}");
    // Infallible ⇒ no error param.
    assert!(!compact.contains("e:*mut"), "{src}");
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
    let mut registry =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .function(syn::parse_quote!(z_init_logs))
        .panic();

    let src = write(&cbindgen, &mut registry, "str_borrow");
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

    let mut registry = Registry::<()>::from_items([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Enum(enum_item), loc.clone()),
    ])
    .expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(ZKeyExpr))
        .base_name("z_keyexpr")
        .enum_type(syn::parse_quote!(SetIntersectionLevel))
        .base_name("z_intersection")
        .function(syn::parse_quote!(z_keyexpr_relation_to))
        .panic();

    let src = write(&cbindgen, &mut registry, "relation_to");
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
    let mut registry = Registry::<()>::from_items([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ])
    .expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZConfig))
        .base_name("z_config")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_config_insert_json5));

    let src = write(&cbindgen, &mut registry, "mut_opaque_borrow");
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
