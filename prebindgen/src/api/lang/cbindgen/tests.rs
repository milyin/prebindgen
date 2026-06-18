use super::*;
use crate::SourceLocation;

fn write(cbindgen: &Cbindgen, registry: &mut Registry<()>, tag: &str) -> String {
    let dir = std::env::temp_dir().join(format!("cbindgen_{}_{}", tag, std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let out = dir.join(format!("{tag}.rs"));
    let path = registry.write_rust(cbindgen, &out).expect("write_rust");
    std::fs::read_to_string(&path).unwrap()
}

fn error_struct() -> syn::ItemStruct {
    syn::parse_quote!(
        pub struct Error {
            pub message: String,
        }
    )
}

/// An adapter with no declarations writes an empty (whitespace-only) file.
#[test]
fn empty_adapter_writes_empty_file() {
    let cbindgen = Cbindgen::new();
    let mut registry: Registry<()> = Registry::default();
    let src = write(&cbindgen, &mut registry, "empty");
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

    let src = write(&cbindgen, &mut registry, "keyexpr");
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

    let mut registry =
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

    let src = write(&cbindgen, &mut registry, "opaque_error");
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

/// An `opaque_owned_struct` type is passed BY VALUE via transmute to/from an opaque
/// counterpart (no `Box`): output `ptr::read`s the bytes, consume reads them
/// out by `*mut` and writes a gravestone back, `_drop` runs the live value's
/// destructor, and size+align equality is asserted (fail-closed).
#[test]
fn opaque_owned_transmute_by_value() {
    let loc = SourceLocation::default();
    let st: syn::ItemStruct = syn::parse_quote!(
        pub struct Payload {
            pub inner: Vec<u8>,
        }
    );
    let out_fn: syn::ItemFn = syn::parse_quote!(
        pub fn z_payload_make() -> Payload {
            unimplemented!()
        }
    );
    let in_fn: syn::ItemFn = syn::parse_quote!(
        pub fn z_payload_take(p: Payload) {
            unimplemented!()
        }
    );
    let mut registry = Registry::<()>::from_items([
        (syn::Item::Struct(st), loc.clone()),
        (syn::Item::Fn(out_fn), loc.clone()),
        (syn::Item::Fn(in_fn), loc.clone()),
    ])
    .expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_owned_struct(syn::parse_quote!(Payload), syn::parse_quote!(OpaquePayload))
        .base_name("z_payload_t")
        .function(syn::parse_quote!(z_payload_make))
        .function(syn::parse_quote!(z_payload_take))
        .panic();

    let src = write(&cbindgen, &mut registry, "value_opaque");
    let compact: String = src.split_whitespace().collect();

    // Fail-closed size + align asserts against the opaque counterpart.
    assert!(
        compact
            .contains("size_of::<zenoh_flat::Payload>()==::core::mem::size_of::<OpaquePayload>()"),
        "{src}"
    );
    assert!(
        compact.contains(
            "align_of::<zenoh_flat::Payload>()==::core::mem::align_of::<OpaquePayload>()"
        ),
        "{src}"
    );
    // Autogenerated transmute glue (the single owner of the unsafe).
    assert!(
        compact.contains("impl::prebindgen::TransmuteforOpaquePayload"),
        "{src}"
    );
    assert!(compact.contains("typeRust=zenoh_flat::Payload;"), "{src}");
    // Output: by value via Transmute::from_rust (NO Box).
    assert!(!compact.contains("Box::into_raw"), "{src}");
    assert!(
        compact.contains("<OpaquePayloadas::prebindgen::Transmute>::from_rust(v)"),
        "{src}"
    );
    // Consume: `*mut OpaquePayload`, move out via Transmute::into_rust +
    // gravestone write-back (NO is_gravestone reject — empty/gravestone-
    // coinciding values stay valid).
    assert!(compact.contains("v:*mutOpaquePayload"), "{src}");
    assert!(
        compact
            .contains("<OpaquePayloadas::prebindgen::Transmute>::into_rust(::core::ptr::read(v),)"),
        "{src}"
    );
    assert!(
        compact.contains("ptr::write(v,<OpaquePayloadas::prebindgen::Gravestone>::gravestone())"),
        "{src}"
    );
    // Typed drop runs the value's destructor (via as_rust_mut) unconditionally
    // — no `is_gravestone` check (gravestone is itself safe to drop).
    assert!(
        compact.contains("fnz_payload_t_drop(this_:*mutOpaquePayload)"),
        "{src}"
    );
    assert!(
        !compact.contains("is_gravestone"),
        "drop must be unconditional: {src}"
    );
    assert!(
        compact.contains("Transmute>::as_rust_mut(&mut*this_)"),
        "{src}"
    );
}

/// An `opaque_data_struct` (plain-data) type is passed BY VALUE like `opaque_owned_struct`,
/// but holds no external resource: consume just moves it out with **no
/// gravestone write-back**, and the generated code references no `Gravestone`
/// (only the autogenerated `Transmute`). The fail-closed size+align asserts and
/// the typed `_drop` are still emitted.
#[test]
fn opaque_data_no_gravestone_writeback() {
    let loc = SourceLocation::default();
    let st: syn::ItemStruct = syn::parse_quote!(
        pub struct Stamp {
            pub ntp64: u64,
            pub id: u64,
        }
    );
    let out_fn: syn::ItemFn = syn::parse_quote!(
        pub fn z_stamp_make() -> Stamp {
            unimplemented!()
        }
    );
    let in_fn: syn::ItemFn = syn::parse_quote!(
        pub fn z_stamp_take(s: Stamp) {
            unimplemented!()
        }
    );
    let mut registry = Registry::<()>::from_items([
        (syn::Item::Struct(st), loc.clone()),
        (syn::Item::Fn(out_fn), loc.clone()),
        (syn::Item::Fn(in_fn), loc.clone()),
    ])
    .expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_data_struct(syn::parse_quote!(Stamp), syn::parse_quote!(z_stamp_t))
        .base_name("z_stamp_t")
        .function(syn::parse_quote!(z_stamp_make))
        .function(syn::parse_quote!(z_stamp_take))
        .panic();

    let src = write(&cbindgen, &mut registry, "opaque_data_struct");
    let compact: String = src.split_whitespace().collect();

    // Same transmute glue + asserts as an owned type.
    assert!(
        compact.contains("impl::prebindgen::Transmuteforz_stamp_t"),
        "{src}"
    );
    assert!(
        compact.contains("align_of::<zenoh_flat::Stamp>()==::core::mem::align_of::<z_stamp_t>()"),
        "{src}"
    );
    // Consume moves the value out by transmute…
    assert!(
        compact.contains("<z_stamp_tas::prebindgen::Transmute>::into_rust(::core::ptr::read(v)"),
        "{src}"
    );
    // …but writes NO gravestone back, and references no `Gravestone` at all.
    assert!(
        !compact.contains("Gravestone"),
        "opaque_data_struct must not reference Gravestone: {src}"
    );
    // Typed drop is still emitted.
    assert!(
        compact.contains("fnz_stamp_t_drop(this_:*mutz_stamp_t)"),
        "{src}"
    );
}

/// A `.takeable_param(idx)` callback arg is delivered as `*mut z_x_t`: the
/// closure `call` takes a pointer, the trampoline drops it after the call, and
/// a public `z_x_take(dst, src)` move function is emitted.
#[test]
fn takeable_callback_param() {
    let loc = SourceLocation::default();
    let st: syn::ItemStruct = syn::parse_quote!(
        pub struct Sample {
            pub _0: u64,
        }
    );
    // A function declaring a subscriber-like callback by value.
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_declare_sub(cb: impl Fn(Sample) + Send + Sync + 'static) {
            unimplemented!()
        }
    );
    let mut registry = Registry::<()>::from_items([
        (syn::Item::Struct(st), loc.clone()),
        (syn::Item::Fn(func), loc.clone()),
    ])
    .expect("index items");

    let cbindgen = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .opaque_owned_struct(syn::parse_quote!(Sample), syn::parse_quote!(z_sample_t))
        .callback(syn::parse_quote!(impl Fn(Sample) + Send + Sync + 'static))
        .base_name("z_closure_sample_t")
        .takeable_param(0)
        .function(syn::parse_quote!(z_declare_sub));

    let src = write(&cbindgen, &mut registry, "takeable");
    let compact: String = src.split_whitespace().collect();

    // Closure `call` receives the sample as an owned pointer.
    assert!(
        compact.contains("call:::core::option::Option<unsafeextern\"C\"fn(*mutz_sample_t,*mut"),
        "{src}"
    );
    // Trampoline passes `&mut __w0` and drops it after the call.
    assert!(compact.contains("&mut__w0as*mutz_sample_t"), "{src}");
    assert!(
        compact.contains("<z_sample_tas::prebindgen::Transmute>::into_rust(__w0)"),
        "{src}"
    );
    // Public take (move) function emitted (no name mangler in this test ⇒
    // `sample_take`; a real adapter mangles to `z_sample_take`).
    assert!(
        compact
            .contains("pubunsafeextern\"C\"fnsample_take(dst:*mutz_sample_t,src:*mutz_sample_t)"),
        "{src}"
    );
    assert!(
        compact.contains("ptr::write(src,<z_sample_tas::prebindgen::Gravestone>::gravestone())"),
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
    let mut reg =
        Registry::<()>::from_items([(syn::Item::Fn(func), loc.clone())]).expect("index items");
    let cb = Cbindgen::new()
        .source_module(syn::parse_quote!(zenoh_flat))
        .function(syn::parse_quote!(rust_init))
        .base_name("z_init");
    let src = write(&cb, &mut reg, "fnname");
    let compact: String = src.split_whitespace().collect();
    assert!(compact.contains("extern\"C\"fnz_init("), "{src}");
    assert!(compact.contains("zenoh_flat::rust_init("), "{src}");
}

// ── Strict modifier rules (misapplied modifiers are build errors) ──────

fn catch<F: FnOnce()>(f: F) -> bool {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)).is_err()
}

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
    let mut registry = Registry::<()>::from_items([
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
        let _ = registry.write_rust(&cbindgen, std::env::temp_dir().join("nofree.rs"));
    }));
    assert!(
        result.is_err(),
        "expected a build error when string memory is produced without a free fn"
    );
}

/// A subscriber-shaped fn with an `impl Fn(ZSample)` callback and a zero-arg
/// `impl Fn()` on-close: each declared callback emits a by-value `#[repr(C)]`
/// closure struct (`context`/`call`/`drop`), `call` taking the arg's **owned**
/// output wire (`z_sample_t *`) plus the `void *context`. The trampoline
/// rebuilds a Rust closure that encodes args via their output converters and
/// invokes the C `call` through an `Arc<Ctx>` that runs `drop(context)` on
/// release.
#[test]
fn callback_subscriber_emits_closure_structs() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_sub(
            session: &ZSession,
            callback: impl Fn(ZSample) + Send + Sync + 'static,
            on_close: impl Fn() + Send + Sync + 'static,
        ) -> Result<ZSubscriber, Error> {
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
        .opaque_ptr(syn::parse_quote!(ZSession))
        .base_name("z_session_t")
        .opaque_ptr(syn::parse_quote!(ZSample))
        .base_name("z_sample_t")
        .opaque_ptr(syn::parse_quote!(ZSubscriber))
        .base_name("z_subscriber_t")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .callback(syn::parse_quote!(impl Fn(ZSample) + Send + Sync + 'static))
        .base_name("z_closure_sample_t")
        .callback(syn::parse_quote!(impl Fn() + Send + Sync + 'static))
        .base_name("z_closure_drop_t")
        .function(syn::parse_quote!(z_sub));

    let src = write(&cbindgen, &mut registry, "cb_sub");
    let compact: String = src.split_whitespace().collect();

    // Closure structs: sample carries the owned handle wire; drop is zero-arg.
    assert!(compact.contains("structz_closure_sample_t"), "{src}");
    assert!(
            compact.contains(
                "pubcall:::core::option::Option<unsafeextern\"C\"fn(*mutz_sample_t,*mut::core::ffi::c_void),>"
            ),
            "{src}"
        );
    assert!(compact.contains("structz_closure_drop_t"), "{src}");

    // Trampoline: by-value struct in, `impl Fn(<src arg>)` out; Arc-held ctx.
    assert!(
            compact.contains(
                "fn__cbg_in_z_closure_sample_t(c:z_closure_sample_t,)->implFn(zenoh_flat::ZSample)+Send+Sync+'static"
            ),
            "{src}"
        );
    assert!(
        compact.contains("Arc::new(__Ctx{context:c.context,drop:c.drop"),
        "{src}"
    );
    // Arg encoded via its OUTPUT converter, then passed (owned) with context.
    assert!(
        compact.contains("let__w0=__cbg_out_ZSample(__a0);"),
        "{src}"
    );
    assert!(compact.contains("__f(__w0,__ctx.context)"), "{src}");
    assert!(compact.contains("move|__a0:zenoh_flat::ZSample|"), "{src}");
    // Zero-arg trampoline.
    assert!(
        compact.contains(
            "fn__cbg_in_z_closure_drop_t(c:z_closure_drop_t,)->implFn()+Send+Sync+'static"
        ),
        "{src}"
    );
    assert!(compact.contains("move||{"), "{src}");
    assert!(compact.contains("__f(__ctx.context)"), "{src}");
    // Drop runs the C `drop(context)` on release.
    assert!(compact.contains("Some(__d)=self.drop"), "{src}");
    assert!(compact.contains("__d(self.context)"), "{src}");

    // Wrapper takes both closures by value and decodes them.
    assert!(compact.contains("callback:z_closure_sample_t"), "{src}");
    assert!(compact.contains("on_close:z_closure_drop_t"), "{src}");
    assert!(
        compact.contains("letcallback=__cbg_in_z_closure_sample_t(callback);"),
        "{src}"
    );
    assert!(
        compact.contains("leton_close=__cbg_in_z_closure_drop_t(on_close);"),
        "{src}"
    );
    // Result of an opaque handle rides the return (NULL = Err); `e` out-param.
    assert!(compact.contains("->*mutz_subscriber_t"), "{src}");
    assert!(compact.contains("e:*mutz_error"), "{src}");
}

/// A callback with a built-in scalar argument (`impl Fn(f64)`) must NOT have its
/// argument module-qualified — `f64` lives in no source module, so emitting
/// `zenoh_flat::f64` would be invalid Rust. Regression for the primitive
/// callback-arg qualification bug.
#[test]
fn callback_scalar_arg_not_module_qualified() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_on_value(
            callback: impl Fn(f64) + Send + Sync + 'static,
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
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        .callback(syn::parse_quote!(impl Fn(f64) + Send + Sync + 'static))
        .base_name("z_closure_value_t")
        .function(syn::parse_quote!(z_on_value));

    let src = write(&cbindgen, &mut registry, "cb_scalar");
    let compact: String = src.split_whitespace().collect();

    // The bug was `f64` qualified to `zenoh_flat::f64`.
    assert!(!compact.contains("zenoh_flat::f64"), "{src}");
    // Closure param + `impl Fn` return keep `f64` bare.
    assert!(compact.contains("move|__a0:f64|"), "{src}");
    assert!(
        compact.contains(
            "fn__cbg_in_z_closure_value_t(c:z_closure_value_t,)->implFn(f64)+Send+Sync+'static"
        ),
        "{src}"
    );
}

/// Without a `.name(...)` override the closure-struct C name is composed
/// generically from the args' configured C type names (`closure_<argCname>`)
/// — `lang::Cbindgen` invents no target-language convention of its own.
#[test]
fn callback_struct_name_defaults_generically() {
    let loc = SourceLocation::default();
    let func: syn::ItemFn = syn::parse_quote!(
        pub fn z_sub2(
            session: &ZSession,
            callback: impl Fn(ZSample) + Send + Sync + 'static,
        ) -> Result<ZSubscriber, Error> {
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
        .opaque_ptr(syn::parse_quote!(ZSession))
        .base_name("z_session_t")
        .opaque_ptr(syn::parse_quote!(ZSample))
        .base_name("z_sample_t")
        .opaque_ptr(syn::parse_quote!(ZSubscriber))
        .base_name("z_subscriber_t")
        .data_struct(syn::parse_quote!(Error))
        .base_name("z_error")
        .error()
        // No `.name(...)` on the callback ⇒ generic default.
        .callback(syn::parse_quote!(impl Fn(ZSample) + Send + Sync + 'static))
        .function(syn::parse_quote!(z_sub2));

    let src = write(&cbindgen, &mut registry, "cb_default");
    let compact: String = src.split_whitespace().collect();

    // Composed from the arg's configured C name `z_sample_t`.
    assert!(compact.contains("structclosure_z_sample_t"), "{src}");
    assert!(compact.contains("callback:closure_z_sample_t"), "{src}");
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
    let mut registry = Registry::<()>::from_items([
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

    let src = write(&cbindgen, &mut registry, "manglers");
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
