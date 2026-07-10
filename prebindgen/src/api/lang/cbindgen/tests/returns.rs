use super::*;

/// A `Result<(), E>` function lowers to `bool f(<inputs>, E *e)` — no
/// out-param, just `true` on `Ok`.
#[test]
fn result_unit_omits_out_param() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_unit_op(s: String) -> Result<(), Error> {
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
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_unit_op));

    let src = write(&cbindgen, &mut registry, "resultunit");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("extern\"C\"fnz_unit_op"), "{src}");
    assert!(compact.contains("->bool"), "{src}");
    // Out-param dropped; error param kept.
    assert!(!compact.contains("out:*mut"), "{src}");
    assert!(compact.contains("e:*mutz_error"), "{src}");
    // Ok arm yields `true`, with no write through `out`.
    assert!(compact.contains("Result::Ok(__v)=>true"), "{src}");
    assert!(!compact.contains("*out="), "{src}");
}

/// `Result<String, E>` returns a bare `char*` (a `malloc`'d raw block, freed
/// by `z_free`), NULL on error — no `cbg_string_t` wrapper.
#[test]
fn result_string_uses_owned_string_wire() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_config_get_json(key: String) -> Result<String, Error> {
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
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_config_get_json));

    let src = write(&cbindgen, &mut registry, "result_string");
    let compact: String = src.split_whitespace().collect();

    assert!(!compact.contains("cbg_string_t"), "{src}");
    assert!(compact.contains("extern\"C\"fnz_config_get_json"), "{src}");
    // Returns char*, no out-param; string built via the raw malloc'd block.
    assert!(compact.contains("->*mut::core::ffi::c_char"), "{src}");
    assert!(!compact.contains("out:*mut"), "{src}");
    assert!(compact.contains("__cbg_alloc_cstr(v)"), "{src}");
    assert!(
        compact.contains("fnz_free(p:*mut::core::ffi::c_void)"),
        "{src}"
    );
    // Ok arm encodes the pointer into the return slot; error → NULL.
    assert!(compact.contains("__ret=__cbg_out_String(__v);"), "{src}");
    assert!(
        compact.contains("=>{if!e.is_null(){*e=__cbg_out_Error(__err);}::core::ptr::null_mut()}"),
        "{src}"
    );
}

/// `z_encoding_schema(e: &ZEncoding) -> Option<String>` lowers to a bare
/// `char*` return where NULL encodes `None` (a value, not an error). The
/// fallible borrow input forces `.panic()`; there is no `out`/`e` param.
#[test]
fn option_string_returns_pointer_null_for_none() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_encoding_schema(e: &ZEncoding) -> Option<String> {
            unimplemented!()
        }
    );
    let mut registry =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZEncoding))
        .base_name("z_encoding")
        .function(syn::parse_quote!(z_encoding_schema))
        .panic();

    let src = write(&cbindgen, &mut registry, "option_string");
    let compact: String = src.split_whitespace().collect();

    // Plain-Option wrapper: `char*` return, no out-param, no error param.
    assert!(compact.contains("extern\"C\"fnz_encoding_schema"), "{src}");
    assert!(compact.contains("->*mut::core::ffi::c_char"), "{src}");
    assert!(!compact.contains("out:*mut"), "{src}");
    assert!(!compact.contains("e:*mut"), "{src}");
    // Inline Option encoding into the return slot: Some → inner wire, None → NULL.
    assert!(
        compact.contains("::core::option::Option::Some(__x)=>{__ret=__cbg_out_String(__x);}"),
        "{src}"
    );
    assert!(
        compact.contains("::core::option::Option::None=>{__ret=::core::ptr::null_mut();}"),
        "{src}"
    );
    // Fallible borrow decode aborts (no Result channel).
    assert!(compact.contains("panic!("), "{src}");
}

/// `Result<Option<T>, E>` cannot use NULL for both `None` and error, so it
/// takes the value-wire shape: `bool f(T **out, …, E *e)`. `None` writes a
/// NULL into `*out` and still returns `true`.
#[test]
fn result_option_uses_out_param() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_get_opt(key: String) -> Result<Option<ZThing>, Error> {
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
        .opaque_ptr(syn::parse_quote!(ZThing))
        .base_name("z_thing")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_get_opt));

    let src = write(&cbindgen, &mut registry, "result_option");
    let compact: String = src.split_whitespace().collect();

    // Value-wire shape: bool return, pointer-to-pointer out-param, error param.
    assert!(compact.contains("extern\"C\"fnz_get_opt"), "{src}");
    assert!(compact.contains("->bool"), "{src}");
    assert!(compact.contains("out:*mut*mutz_thing"), "{src}");
    assert!(compact.contains("e:*mutz_error"), "{src}");
    // Ok arm writes the Option (pointer-or-NULL) through `out`, returns true.
    assert!(compact.contains("*out=__cbg_out_ZThing(__x);"), "{src}");
    assert!(
        compact.contains("::core::option::Option::None=>{*out=::core::ptr::null_mut();}"),
        "{src}"
    );
    assert!(
        compact.contains("=>{") && compact.contains("true}"),
        "{src}"
    );
}

/// `Vec<String>` lowers to `char** f(<inputs>, size_t* len)`: the malloc'd
/// array pointer is returned, the element count goes to `*len`. Each element
/// is encoded via the inner `String` converter.
#[test]
fn vec_string_returns_ptr_and_len() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_hello_locators(h: &ZHello) -> Vec<String> {
            unimplemented!()
        }
    );
    let mut registry =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZHello))
        .base_name("z_hello")
        .function(syn::parse_quote!(z_hello_locators))
        .panic();

    let src = write(&cbindgen, &mut registry, "vec_string");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("extern\"C\"fnz_hello_locators"), "{src}");
    // Returns `char**`, with a trailing `len` out-param; no `out`/`e`.
    assert!(compact.contains("->*mut*mut::core::ffi::c_char"), "{src}");
    assert!(compact.contains("len:*mutusize"), "{src}");
    assert!(!compact.contains("e:*mut"), "{src}");
    // Built from the element converter via the malloc'd array helper.
    assert!(
        compact.contains(".map(__cbg_out_String).collect()"),
        "{src}"
    );
    assert!(
        compact.contains("let(__p,__n)=__cbg_alloc_array(__arr);"),
        "{src}"
    );
    assert!(
        compact.contains("__ret=__p;") && compact.contains("*len=__n;"),
        "{src}"
    );
    // The array builder prelude is emitted.
    assert!(compact.contains("fn__cbg_alloc_array<W>"), "{src}");
    // Fallible borrow decode aborts (no Result channel).
    assert!(compact.contains("panic!("), "{src}");
}

/// `Vec<u8>` lowers to a scalar array `uint8_t* f(<inputs>, size_t* len)` —
/// elements pass through (no per-element pointer).
#[test]
fn vec_u8_returns_scalar_array() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_zbytes_to_bytes(z: &ZZBytes) -> Vec<u8> {
            unimplemented!()
        }
    );
    let mut registry =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZZBytes))
        .base_name("z_zbytes")
        .function(syn::parse_quote!(z_zbytes_to_bytes))
        .panic();

    let src = write(&cbindgen, &mut registry, "vec_u8");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("->*mutu8"), "{src}");
    assert!(compact.contains("len:*mutusize"), "{src}");
    assert!(compact.contains("__cbg_alloc_array(__arr)"), "{src}");
}

/// `Cow<'_, [u8]>` lowers to the same owned scalar array ABI as `Vec<u8>`.
#[test]
fn cow_u8_returns_scalar_array() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_zbytes_as_bytes(z: &ZZBytes) -> ::std::borrow::Cow<'_, [u8]> {
            unimplemented!()
        }
    );
    let mut registry =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZZBytes))
        .base_name("z_zbytes")
        .function(syn::parse_quote!(z_zbytes_as_bytes))
        .panic();

    let src = write(&cbindgen, &mut registry, "cow_u8");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("->*mutu8"), "{src}");
    assert!(compact.contains("len:*mutusize"), "{src}");
    assert!(
        compact.contains(".iter().copied().map(__cbg_out_u8).collect()"),
        "{src}"
    );
    assert!(compact.contains("__cbg_alloc_array(__arr)"), "{src}");
}

/// `Result<Vec<T>, E>` has no free niche (the array NULL means *empty*), so
/// it takes `bool f(T** out, size_t* out_len, <inputs>, E* e)`.
#[test]
fn result_vec_uses_out_params() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_things(key: String) -> Result<Vec<ZThing>, Error> {
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
        .opaque_ptr(syn::parse_quote!(ZThing))
        .base_name("z_thing")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_things));

    let src = write(&cbindgen, &mut registry, "result_vec");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("->bool"), "{src}");
    assert!(compact.contains("out:*mut*mut*mutz_thing"), "{src}");
    assert!(compact.contains("out_len:*mutusize"), "{src}");
    assert!(compact.contains("e:*mutz_error"), "{src}");
    // Ok writes both out-params; Err writes `*e` and returns false.
    assert!(
        compact.contains("*out=__p;") && compact.contains("*out_len=__n;"),
        "{src}"
    );
}

/// `Option<Vec<T>>` (no `Result`): the inner `Vec` has no niche, so an
/// explicit `present` flag rides the `bool` return while the array goes to
/// `out`/`out_len`.
#[test]
fn option_vec_uses_present_and_out() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_maybe_things(h: &ZHello) -> Option<Vec<ZThing>> {
            unimplemented!()
        }
    );
    let mut registry =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZHello))
        .base_name("z_hello")
        .opaque_ptr(syn::parse_quote!(ZThing))
        .base_name("z_thing")
        .function(syn::parse_quote!(z_maybe_things))
        .panic();

    let src = write(&cbindgen, &mut registry, "option_vec");
    let compact: String = src.split_whitespace().collect();

    // `bool` return is the `present` flag; the array rides `out`/`out_len`.
    assert!(compact.contains("->bool"), "{src}");
    assert!(compact.contains("out:*mut*mut*mutz_thing"), "{src}");
    assert!(compact.contains("out_len:*mutusize"), "{src}");
    assert!(!compact.contains("e:*mut"), "{src}");
    assert!(
        compact.contains("__ret=true;") && compact.contains("__ret=false;"),
        "{src}"
    );
}

/// `Result<Option<Vec<T>>, E>`: full stack — `Result` finds no niche (Option
/// consumed it), so `bool` status; the `present` flag and the array all ride
/// out-params: `bool f(bool* out_present, T** out, size_t* out_len, …, E* e)`.
#[test]
fn result_option_vec_full() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_full(key: String) -> Result<Option<Vec<ZThing>>, Error> {
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
        .opaque_ptr(syn::parse_quote!(ZThing))
        .base_name("z_thing")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_full));

    let src = write(&cbindgen, &mut registry, "result_option_vec");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("->bool"), "{src}");
    assert!(compact.contains("out_present:*mutbool"), "{src}");
    assert!(compact.contains("out:*mut*mut*mutz_thing"), "{src}");
    assert!(compact.contains("out_len:*mutusize"), "{src}");
    assert!(compact.contains("e:*mutz_error"), "{src}");
    // present flag set inside the Ok arm; array filled when Some.
    assert!(
        compact.contains("*out_present=true;") && compact.contains("*out_present=false;"),
        "{src}"
    );
}

/// A `Result<ptr, E>` wrapper returns the pointer and signals errors with
/// NULL — both the `Err(E)` arm and an input-decode failure return null.
#[test]
fn result_pointer_returns_null_on_error() {
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

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZKeyExpr))
        .base_name("z_keyexpr")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_keyexpr_try_from));

    let src = write(&cbindgen, &mut registry, "ptr_null");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("->*mutz_keyexpr"), "{src}");
    // Err(E) arm: write *e then return null.
    assert!(compact.contains("null_mut()"), "{src}");
    // Decode failure also returns null (not `false`).
    assert!(compact.contains("return::core::ptr::null_mut()"), "{src}");
    assert!(!compact.contains("returnfalse"), "{src}");
}

/// A borrowed (non-`'static`) `&T` return of an opaque handle lowers to a
/// const, **non-owning** `*const z_X_t` (no `Box::into_raw`) — a loaned
/// accessor. The converter reinterprets the borrow.
#[test]
fn borrowed_ref_output_is_const_non_owning() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_sample_payload(s: &ZSample) -> &ZBytes {
            unimplemented!()
        }
    );
    let mut registry =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(ZSample))
        .base_name("z_sample_t")
        .opaque_ptr(syn::parse_quote!(ZBytes))
        .base_name("z_zbytes_t")
        .function(syn::parse_quote!(z_sample_payload))
        .panic();

    let src = write(&cbindgen, &mut registry, "borrow_ret");
    let compact: String = src.split_whitespace().collect();

    // Const, non-owning return; the return path goes through the reinterpret
    // (`&` → `*const`) converter, not an owning `Box::into_raw`.
    assert!(compact.contains("->*constz_zbytes_t"), "{src}");
    assert!(
        compact.contains("vas*constzenoh_flat::ZBytesas*constz_zbytes_t"),
        "{src}"
    );
    assert!(
        compact.contains("__ret=__cbg_out_ref_ZBytes(__v);"),
        "{src}"
    );
}

/// `Option<&T>` borrowed return composes: a nullable const loaned pointer
/// (NULL = `None`), via the Option null-niche path over the borrow wire.
#[test]
fn borrowed_option_ref_output_nullable() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_sample_timestamp(s: &ZSample) -> Option<&ZTimestamp> {
            unimplemented!()
        }
    );
    let mut registry =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(ZSample))
        .base_name("z_sample_t")
        .opaque_ptr(syn::parse_quote!(ZTimestamp))
        .base_name("z_timestamp_t")
        .function(syn::parse_quote!(z_sample_timestamp))
        .panic();

    let src = write(&cbindgen, &mut registry, "borrow_opt_ret");
    let compact: String = src.split_whitespace().collect();

    // Nullable const loaned pointer rides the return (no out-param needed:
    // the pointer's NULL niche encodes `None`).
    assert!(compact.contains("->*constz_timestamp_t"), "{src}");
    assert!(compact.contains("__cbg_out_ref_ZTimestamp"), "{src}");
    assert!(!compact.contains("out:*mut*constz_timestamp_t"), "{src}");
}
