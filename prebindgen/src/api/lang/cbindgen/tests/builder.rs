use super::*;

/// `.base_name()` on a function sets the base for the exported `#[no_mangle]`
/// symbol while still calling the original Rust fn.
#[test]
fn function_name_renames_symbol() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn rust_init() {
            unimplemented!()
        }
    );
    let reg =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");
    let cb = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .function(syn::parse_quote!(rust_init))
        .base_name("z_init");
    let src = write(cb, reg, "fnname");
    let compact: String = src.split_whitespace().collect();
    assert!(compact.contains("extern\"C\"fnz_init("), "{src}");
    assert!(compact.contains("zenoh_flat::rust_init("), "{src}");
}

// ── Strict modifier rules (misapplied modifiers are build errors) ──────

#[test]
fn error_after_ptr_struct_panics() {
    assert!(catch(|| {
        let _ = Cbindgen::new()
            .opaque_ptr(syn::parse_quote!(ZKeyExpr))
            .error();
    }));
}

#[test]
fn panic_after_data_struct_panics() {
    assert!(catch(|| {
        let _ = Cbindgen::new()
            .data_struct(syn::parse_quote!(Error))
            .panic();
    }));
}

#[test]
fn name_with_no_declaration_panics() {
    // `source_module` is a root modifier — it resets the current declaration,
    // so a trailing `.base_name()` has nothing to apply to.
    assert!(catch(|| {
        let _ = Cbindgen::new()
            .source_module(syn::parse_quote!(zenoh_flat))
            .base_name("x");
    }));
}

#[test]
fn function_and_ignore_function_conflict_panics() {
    assert!(catch(|| {
        let _ = Cbindgen::new()
            .function(syn::parse_quote!(z_open))
            .ignore_function(syn::parse_quote!(z_open));
    }));
}

#[test]
fn data_struct_and_ignore_type_conflict_panics() {
    assert!(catch(|| {
        let _ = Cbindgen::new()
            .data_struct(syn::parse_quote!(Error))
            .ignore_type(syn::parse_quote!(Error));
    }));
}

/// Producing `char*` string memory without declaring a
/// `free_memory_function` is a build error.
#[test]
fn free_memory_function_required() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_describe(key: String) -> Result<String, Error> {
            unimplemented!()
        }
    );
    let registry = Registry::<()>::from_items([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ])
    .expect("index items");

    // String output (and an Error with a String field) but no free fn declared.
    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_describe));

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = registry
            .resolve(cbindgen)
            .and_then(|gen| gen.write_rust(std::env::temp_dir().join("nofree.rs")));
    }));
    assert!(
        result.is_err(),
        "expected a build error when string memory is produced without a free fn"
    );
}

/// The five manglers generate every C-facing name from the Rust types — the
/// base mangler centralizes per-type spelling (here `ZKeyExpr`→`keyexpr`),
/// and the type/destructor/callback/function manglers decorate it. No
/// per-declaration `.name(...)` is used.
#[test]
fn manglers_generate_all_names() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_sub(
            key_expr: ZKeyExpr,
            callback: impl Fn(ZSample) + Send + Sync + 'static,
            on_close: impl Fn() + Send + Sync + 'static,
        ) -> Result<ZSubscriber, Error> {
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
        // One base rule fixes the `KeyExpr`→`keyexpr` irregular in a single
        // place; everything else is `snake_case` of the `Z`-stripped name.
        .mangle_rust_type(|short| {
            let s = short.strip_prefix('Z').unwrap_or(short);
            match s {
                "KeyExpr" => "keyexpr".to_string(),
                other => snake_case(other),
            }
        })
        .mangle_type_name(|base| format!("z_{base}_t"))
        .mangle_destructor(|base| format!("z_{base}_drop"))
        .mangle_callback(|bases| {
            if bases.is_empty() {
                "z_closure_drop_t".to_string()
            } else {
                format!("z_closure_{}_t", bases.join("_"))
            }
        })
        .mangle_function(|n| {
            if n.starts_with("z_") {
                n.to_string()
            } else {
                format!("z_{n}")
            }
        })
        // No `.name(...)` anywhere — names come purely from the manglers.
        .opaque_ptr(syn::parse_quote!(ZKeyExpr))
        .opaque_ptr(syn::parse_quote!(ZSample))
        .opaque_ptr(syn::parse_quote!(ZSubscriber))
        .data_struct(syn::parse_quote!(Error))
        .error()
        .callback(syn::parse_quote!(impl Fn(ZSample) + Send + Sync + 'static))
        .callback(syn::parse_quote!(impl Fn() + Send + Sync + 'static))
        .function(syn::parse_quote!(z_sub));

    let src = write(cbindgen, registry, "manglers");
    let compact: String = src.split_whitespace().collect();

    // Type-name mangler over the base (note `keyexpr`, not `key_expr`).
    assert!(compact.contains("structz_keyexpr_t"), "{src}");
    assert!(compact.contains("structz_sample_t"), "{src}");
    assert!(compact.contains("structz_subscriber_t"), "{src}");
    assert!(compact.contains("structz_error_t"), "{src}");
    // Destructor mangler.
    assert!(
        compact.contains("fnz_keyexpr_drop(this_:*mutz_keyexpr_t"),
        "{src}"
    );
    assert!(
        compact.contains("fnz_sample_drop(this_:*mutz_sample_t"),
        "{src}"
    );
    // Callback mangler (arg base + zero-arg).
    assert!(compact.contains("structz_closure_sample_t"), "{src}");
    assert!(compact.contains("structz_closure_drop_t"), "{src}");
    // Callback `call` takes the owned handle wire produced via the manglers.
    assert!(
        compact.contains("fn(*mutz_sample_t,*mut::core::ffi::c_void)"),
        "{src}"
    );
    // Function mangler leaves the already-`z_`-prefixed symbol unchanged.
    assert!(compact.contains("extern\"C\"fnz_sub("), "{src}");
    // Return handle rides the return.
    assert!(compact.contains("->*mutz_subscriber_t"), "{src}");
}

/// Issue #95: a signature spelled with the source crate's own name matches
/// the bare `opaque_ptr` declaration — ingest normalizes the spelling, and
/// emission re-qualifies through `.source_module` as before.
#[test]
fn qualified_signature_spelling_matches_bare_opaque_ptr() {
    let loc = SourceLocation {
        crate_name: Some("zenoh-flat".to_string()),
        ..Default::default()
    };
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_keyexpr_len(k: &zenoh_flat::ZKeyExpr) -> i64 {
            unimplemented!()
        }
    );
    let reg =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");
    let cb = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(ZKeyExpr))
        .function(syn::parse_quote!(z_keyexpr_len))
        .panic();
    let src = write(cb, reg, "q95");
    let compact: String = src.split_whitespace().collect();
    assert!(compact.contains("extern\"C\"fnz_keyexpr_len("), "{src}");
    assert!(compact.contains("zenoh_flat::z_keyexpr_len("), "{src}");
}
