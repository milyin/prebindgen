use super::*;

/// An adapter with no declarations writes an empty (whitespace-only) file.
#[test]
fn empty_adapter_writes_empty_file() {
    let cbindgen = Cbindgen::new();
    let registry: Registry<()> = Registry::default();
    let src = write(cbindgen, registry, "empty");
    assert!(src.trim().is_empty(), "expected empty output, got:\n{src}");
}

/// `z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error>` lowers to a
/// **pointer-returning** wrapper (opaque handle, NULL on error); decode
/// failures route through `From<String>` into the declared error type.
#[test]
fn keyexpr_try_from_lowering() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> {
            unimplemented!()
        }
    );

    let registry = Registry::<()>::from_items([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ])
    .expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZKeyExpr))
        .base_name("z_keyexpr")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_keyexpr_try_from));

    let src = write(cbindgen, registry, "keyexpr");
    // Whitespace-insensitive haystack (the file is prettyplease-formatted).
    let compact: String = src.split_whitespace().collect();

    // Pointer-return wrapper: returns the opaque handle, no `out` param.
    assert!(compact.contains("extern\"C\"fnz_keyexpr_try_from"), "{src}");
    assert!(compact.contains("->*mutz_keyexpr"), "{src}");
    assert!(!compact.contains("out:*mut"), "{src}");
    assert!(compact.contains("e:*mutz_error"), "{src}");
    // Opaque handle marker struct + typed destructor (`<name>_drop`) on the
    // bare ptr.
    assert!(compact.contains("structz_keyexpr{_private"), "{src}");
    assert!(compact.contains("structz_error"), "{src}");
    assert!(
        compact.contains("fnz_keyexpr_drop(this_:*mutz_keyexpr"),
        "{src}"
    );
    assert!(
        compact.contains("Box::from_raw(this_as*mutzenoh_flat::ZKeyExpr)"),
        "{src}"
    );
    // String memory ⇒ malloc/free decls + a single `z_free`; no per-type
    // string/error destructors.
    assert!(compact.contains("fnmalloc(size:usize)"), "{src}");
    assert!(
        compact.contains("fnz_free(p:*mut::core::ffi::c_void)"),
        "{src}"
    );
    assert!(!compact.contains("z_error_drop"), "{src}");
    assert!(!compact.contains("cbg_string_t"), "{src}");
    // Source call fully qualified.
    assert!(compact.contains("zenoh_flat::z_keyexpr_try_from"), "{src}");
    // Error model: decode failure routes via From<String> through the declared
    // error's output converter, and the failing return is NULL.
    assert!(!compact.contains("__CErr"), "{src}");
    assert!(
        compact.contains("as::core::convert::From<::std::string::String"),
        "{src}"
    );
    assert!(compact.contains("__cbg_out_Error"), "{src}");
    assert!(compact.contains("return::core::ptr::null_mut()"), "{src}");
}

/// An **opaque error** (`ZError`, *not* a by-value data struct) used as the `E`
/// of a `Result<_, E>` is marshalled to C as a `char*` message obtained from the
/// recorded accessor (`z_error_message`); the wrapper's error out-param is thus
/// `char **e`, and no error struct is generated.
#[test]
fn opaque_error_lowering() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, ZError> {
            unimplemented!()
        }
    );

    let registry =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZKeyExpr))
        .base_name("z_keyexpr")
        .opaque_error(
            syn::parse_quote!(ZError),
            syn::parse_quote!(z_error_message),
        )
        .function(syn::parse_quote!(z_keyexpr_try_from));

    let src = write(cbindgen, registry, "opaque_error");
    let compact: String = src.split_whitespace().collect();

    // Pointer-return wrapper; the error out-param is a bare `char **e`.
    assert!(compact.contains("extern\"C\"fnz_keyexpr_try_from"), "{src}");
    assert!(compact.contains("->*mutz_keyexpr"), "{src}");
    assert!(compact.contains("e:*mut*mut::core::ffi::c_char"), "{src}");
    // The error converter marshals the opaque error via the recorded accessor.
    assert!(compact.contains("zenoh_flat::z_error_message(&v)"), "{src}");
    assert!(compact.contains("__cbg_alloc_cstr"), "{src}");
    // No by-value error struct is generated for an opaque error.
    assert!(!compact.contains("structz_error"), "{src}");
    // Fallible-input messages still lift into the error via `From<String>`.
    assert!(
        compact.contains("as::core::convert::From<::std::string::String"),
        "{src}"
    );
}
