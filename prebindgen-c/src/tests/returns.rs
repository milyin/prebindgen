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
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_unit_op));

    let src = write(cbindgen, registry, "resultunit");
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
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_config_get_json));

    let src = write(cbindgen, registry, "result_string");
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
    assert!(
        operation_call(&compact, "__c_out_convert_String_", "__v"),
        "{src}"
    );
    assert!(
        operation_call(&compact, "__c_out_convert_Error_", "__err"),
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZEncoding))
        .base_name("z_encoding")
        .function(syn::parse_quote!(z_encoding_schema))
        .panic();

    let src = write(cbindgen, registry, "option_string");
    let compact: String = src.split_whitespace().collect();

    // Plain-Option wrapper: `char*` return, no out-param, no error param.
    assert!(compact.contains("extern\"C\"fnz_encoding_schema"), "{src}");
    assert!(compact.contains("->*mut::core::ffi::c_char"), "{src}");
    assert!(!compact.contains("out:*mut"), "{src}");
    assert!(!compact.contains("e:*mut"), "{src}");
    // Inline Option encoding into the return slot: Some → inner wire, None → NULL.
    assert!(
        operation_call(&compact, "__c_out_convert_String_", "__x"),
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
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZThing))
        .base_name("z_thing")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_get_opt));

    let src = write(cbindgen, registry, "result_option");
    let compact: String = src.split_whitespace().collect();

    // Value-wire shape: bool return, pointer-to-pointer out-param, error param.
    assert!(compact.contains("extern\"C\"fnz_get_opt"), "{src}");
    assert!(compact.contains("->bool"), "{src}");
    assert!(compact.contains("out:*mut*mutz_thing"), "{src}");
    assert!(compact.contains("e:*mutz_error"), "{src}");
    // Ok arm writes the Option (pointer-or-NULL) through `out`, returns true.
    assert!(
        operation_call(&compact, "__c_out_convert_ZThing_", "__x"),
        "{src}"
    );
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZHello))
        .base_name("z_hello")
        .function(syn::parse_quote!(z_hello_locators))
        .panic();

    let src = write(cbindgen, registry, "vec_string");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("extern\"C\"fnz_hello_locators"), "{src}");
    // Returns `char**`, with a trailing `len` out-param; no `out`/`e`.
    assert!(compact.contains("->*mut*mut::core::ffi::c_char"), "{src}");
    assert!(compact.contains("len:*mutusize"), "{src}");
    assert!(!compact.contains("e:*mut"), "{src}");
    // The registry-owned Sequence loop invokes the element converter; the ABI
    // wrapper only transfers its one Vec intermediate to the array helper.
    assert!(
        compact.contains("fn__c_out_convert_sequence_Vec_String_to_wire_"),
        "{src}"
    );
    assert!(
        operation_call(&compact, "__c_out_convert_String_", "__sequence_element",),
        "{src}"
    );
    assert!(
        operation_call(
            &compact,
            "__c_out_convert_sequence_Vec_String_to_wire_",
            "__v",
        ),
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZZBytes))
        .base_name("z_zbytes")
        .function(syn::parse_quote!(z_zbytes_to_bytes))
        .panic();

    let src = write(cbindgen, registry, "vec_u8");
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZZBytes))
        .base_name("z_zbytes")
        .function(syn::parse_quote!(z_zbytes_as_bytes))
        .panic();

    let src = write(cbindgen, registry, "cow_u8");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("->*mutu8"), "{src}");
    assert!(compact.contains("len:*mutusize"), "{src}");
    assert!(
        compact.contains(".iter().copied().map(__c_out_convert_u8_")
            && compact.contains(").collect()"),
        "{src}"
    );
    assert!(compact.contains("__cbg_alloc_array(__arr)"), "{src}");
}

/// `&[u64]` returns the same owned scalar array as `Cow<'_, [u8]>` above.
///
/// A shared slice already had an output entry, for the callback-argument
/// lowering, whose destination is a marker with no wire. As a RETURN that made
/// the wrapper give C nothing at all and call the marker with an argument it
/// does not take (#413), so the exported signature is what this pins.
#[test]
fn slice_returns_scalar_array() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_ids(z: &ZZBytes) -> &'static [u64] {
            unimplemented!()
        }
    );
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZZBytes))
        .function(syn::parse_quote!(z_ids))
        .panic();

    let src = write(cbindgen, registry, "slice_ret");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("->*mutu64"), "{src}");
    assert!(compact.contains("len:*mutusize"), "{src}");
    assert!(
        compact.contains(".iter().copied().map(__c_out_convert_u64_")
            && compact.contains(").collect()"),
        "{src}"
    );
    assert!(compact.contains("__cbg_alloc_array(__arr)"), "{src}");
}

/// `Vec<&T>` maps its elements through the borrow converter, which is an
/// `unsafe fn` — and an `unsafe fn` item implements no `Fn` trait, so passing
/// it to `map` by name did not type-check (#413). It is called in a closure
/// instead. A safe element converter is still passed by name.
#[test]
fn vec_of_borrows_calls_the_unsafe_element_converter() {
    let loc = SourceLocation::default();
    let handle: syn::ItemStruct = syn::parse_quote!(
        pub struct ZHandle {
            id: u64,
        }
    );
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_handles() -> Vec<&'static ZHandle> {
            unimplemented!()
        }
    );
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Struct(handle), loc.clone()),
        (syn::Item::Fn(func), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZHandle))
        .function(syn::parse_quote!(z_handles))
        .panic();

    let src = write(cbindgen, registry, "vec_of_borrows");
    let compact: String = src.split_whitespace().collect();

    assert!(compact.contains("->*mut*constz_handle"), "{src}");
    assert!(
        compact.contains("unsafefn__c_out_convert_sequence_Vec_static_ZHandle_to_wire_"),
        "{src}"
    );
    assert!(
        operation_call(
            &compact,
            "__c_out_convert_ZHandle_c_borrow_shared_output_to_wire_",
            "__sequence_element",
        ),
        "{src}"
    );
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
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZThing))
        .base_name("z_thing")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_things));

    let src = write(cbindgen, registry, "result_vec");
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZHello))
        .base_name("z_hello")
        .opaque_ptr(syn::parse_quote!(ZThing))
        .base_name("z_thing")
        .function(syn::parse_quote!(z_maybe_things))
        .panic();

    let src = write(cbindgen, registry, "option_vec");
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
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZThing))
        .base_name("z_thing")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_full));

    let src = write(cbindgen, registry, "result_option_vec");
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
    let registry = crate::test_util::reg_from_items(declare_referenced([
        (syn::Item::Fn(func), loc.clone()),
        (syn::Item::Struct(error_struct()), loc.clone()),
    ]))
    .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .opaque_ptr(syn::parse_quote!(ZKeyExpr))
        .base_name("z_keyexpr")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .function(syn::parse_quote!(z_keyexpr_try_from));

    let src = write(cbindgen, registry, "ptr_null");
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(ZSample))
        .base_name("z_sample_t")
        .opaque_ptr(syn::parse_quote!(ZBytes))
        .base_name("z_zbytes_t")
        .function(syn::parse_quote!(z_sample_payload))
        .panic();

    let src = write(cbindgen, registry, "borrow_ret");
    let compact: String = src.split_whitespace().collect();

    // Const, non-owning return; the return path goes through the reinterpret
    // (`&` → `*const`) converter, not an owning `Box::into_raw`.
    assert!(compact.contains("->*constz_zbytes_t"), "{src}");
    assert!(
        compact.contains("vas*constzenoh_flat::ZBytesas*constz_zbytes_t"),
        "{src}"
    );
    assert!(
        operation_call(
            &compact,
            "__c_out_convert_ZBytes_c_borrow_shared_output_to_wire_",
            "__v",
        ),
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_ptr(syn::parse_quote!(ZSample))
        .base_name("z_sample_t")
        .opaque_ptr(syn::parse_quote!(ZTimestamp))
        .base_name("z_timestamp_t")
        .function(syn::parse_quote!(z_sample_timestamp))
        .panic();

    let src = write(cbindgen, registry, "borrow_opt_ret");
    let compact: String = src.split_whitespace().collect();

    // Nullable const loaned pointer rides the return (no out-param needed:
    // the pointer's NULL niche encodes `None`).
    assert!(compact.contains("->*constz_timestamp_t"), "{src}");
    assert!(
        compact.contains("__c_out_convert_ZTimestamp_c_borrow_shared_output_to_wire_"),
        "{src}"
    );
    assert!(!compact.contains("out:*mut*constz_timestamp_t"), "{src}");
}

/// A unit **parameter** is reported by the resolver, which is the diagnostic a
/// binding author can act on.
///
/// `CCompile::plans_site` declines a unit crossing because C has nothing to hand
/// back at a `()` return, and that answer is scoped to the return: the renderer
/// asks for every parameter through `CWrapper::site(Role::Param { .. })`, so a
/// declined parameter would leave it asking for a site nobody planned (#687
/// review).
///
/// **This test does not discriminate that scoping.** It passes either way,
/// because the crossing walk needs a construct converter for `()` and fails
/// first. It is kept for what it does pin — which of the two failures a unit
/// parameter produces — and the scoping is a correctness fix without a
/// reachable case that I could construct.
#[test]
fn a_unit_parameter_is_reported_by_the_resolver() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_take_unit(marker: (), n: u32) -> u32 {
            unimplemented!()
        }
    );
    let registry =
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc)]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .free_memory_function("z_free")
        .function(syn::parse_quote!(z_take_unit));

    // C has no construct converter for `()`, so this binding cannot be built —
    // and the point is *which* way it fails. The site is planned, so the
    // failure is the resolver naming the type it could not convert. While the
    // unit decline covered every role, the position was skipped instead, and
    // the renderer panicked asking for a site nobody had planned.
    let message = catch_msg(|| {
        let _ = write(cbindgen, registry, "unitparam");
    });
    assert!(
        message.contains("unresolved prebindgen input type") && message.contains("()"),
        "a unit parameter is reported by the resolver, not by a missing site: {message}"
    );
}
