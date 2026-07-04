use super::*;

/// Returning `Result<_, E>` where `E` is not declared via `.error()` is a
/// build error.
#[test]
fn result_error_not_declared_is_build_error() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> {
            unimplemented!()
        }
    );
    let mut registry = Registry::<()>::from_items([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ])
    .expect("index items");

    // Error declared as data_struct but NOT marked `.error()`.
    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZKeyExpr))
        .base_name("z_keyexpr")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .function(syn::parse_quote!(z_keyexpr_try_from));

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = registry.write_rust(&cbindgen, std::env::temp_dir().join("nope.rs"));
    }));
    assert!(
        result.is_err(),
        "expected a build error for undeclared error type"
    );
}

/// A non-`Result` fn with a fallible (`String`) input needs `.panic()`;
/// without it that's a build error, with it the wrapper `panic!`s.
#[test]
fn fallible_input_without_result_needs_panic() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_log(s: String) {
            unimplemented!()
        }
    );

    // No `.panic()` → build error.
    let mut reg1 = Registry::<()>::from_items([(syn::Item::Fn(func.clone()), loc.clone())])
        .expect("index items");
    let cb1 = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .function(syn::parse_quote!(z_log));
    let err = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = reg1.write_rust(&cb1, std::env::temp_dir().join("nope2.rs"));
    }));
    assert!(err.is_err(), "expected a build error without .panic()");

    // With `.panic()` → wrapper aborts on decode failure.
    let mut reg2 =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");
    let cb2 = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .function(syn::parse_quote!(z_log))
        .panic();
    let src = write(&cb2, &mut reg2, "panicfn");
    let compact: String = src.split_whitespace().collect();
    assert!(compact.contains("extern\"C\"fnz_log"), "{src}");
    assert!(compact.contains("panic!("), "{src}");
}

/// Contract: the error out-parameter `e` may be NULL. EVERY `*e =` write in a
/// generated wrapper is guarded by `if !e.is_null()` — both on the input-decode
/// failure path and on the `Result::Err` return path, and for both error-routing
/// modes (pointer-return in-band niche, and `bool`-status). Consumers (e.g. the
/// zenoh-c compat layer) rely on passing NULL and reading the return value.
#[test]
fn error_out_param_is_null_guarded() {
    let loc = SourceLocation::default();
    // (a) pointer-returning Result<Handle, E> + a fallible `String` input
    //     (exercises both the input-decode and the Result::Err error paths).
    let ptr_fn: syn::ItemFn = syn::parse_quote!(
        pub fn z_keyexpr_try_from(s: String) -> Result<ZKeyExpr, Error> {
            unimplemented!()
        }
    );
    // (b) Result<(), E> → `bool` status return.
    let unit_fn: syn::ItemFn = syn::parse_quote!(
        pub fn z_unit_op(s: String) -> Result<(), Error> {
            unimplemented!()
        }
    );
    let mut registry = Registry::<()>::from_items([
        (syn::Item::Fn(ptr_fn), loc.clone()),
        (syn::Item::Fn(unit_fn), loc.clone()),
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
        .function(syn::parse_quote!(z_keyexpr_try_from))
        .function(syn::parse_quote!(z_unit_op));

    let src = write(&cbindgen, &mut registry, "err_null_guard");
    let compact: String = src.split_whitespace().collect();

    // Pointer-return Err arm: guarded write, then NULL.
    assert!(
        compact.contains("=>{if!e.is_null(){*e=__cbg_out_Error(__err);}::core::ptr::null_mut()}"),
        "{src}"
    );
    // `Result<(),E>` Err arm: guarded write, then `false`.
    assert!(
        compact.contains("=>{if!e.is_null(){*e=__cbg_out_Error(__err);}false}"),
        "{src}"
    );
    // The input-decode failure path also guards the write (it routes the
    // message through `From<String>`). Both functions have a fallible `String`
    // input plus a `Result::Err` arm, so the guarded write appears ≥4 times.
    assert!(
        compact
            .matches("if!e.is_null(){*e=__cbg_out_Error(")
            .count()
            >= 4,
        "expected ≥4 guarded `*e =` writes (2 input-decode + 2 Err arms):\n{src}"
    );

    // Strongest guarantee: NO unguarded `*e =`. Every occurrence of `*e=` in
    // the compacted source is immediately preceded by `if!e.is_null(){`.
    let mut search = compact.as_str();
    while let Some(pos) = search.find("*e=") {
        let before = &search[..pos];
        assert!(
            before.ends_with("if!e.is_null(){"),
            "unguarded `*e =` found before offset {pos}:\n{src}"
        );
        search = &search[pos + 3..];
    }
}
