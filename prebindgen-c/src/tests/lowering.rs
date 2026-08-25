use super::*;

#[test]
fn bounded_duration_option_is_one_scalar_with_named_niche() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "#[prebindgen] pub type Duration = std::time::Duration;",
        "pub fn duration_from_millis(v: u64) -> Duration { unimplemented!() }",
        "pub fn duration_to_millis(v: &Duration) -> u64 { unimplemented!() }",
        "pub fn duration_echo(v: Option<Duration>) -> Option<Duration> { unimplemented!() }",
        "pub fn duration_nested_echo(v: Option<Option<Duration>>) -> Option<Option<Duration>> { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| {
        // `syn::Item`, not `ItemFn`: a fixture declares the types it names.
        let item: syn::Item = syn::parse_str(source).unwrap();
        (item, loc.clone())
    })
    .collect();
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).unwrap();
    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(myflat))
        .convert(
            prebindgen_registry::convert!(Duration)
                .input(prebindgen_registry::fun!(duration_from_millis))
                .output(prebindgen_registry::fun!(duration_to_millis))
                .valid_range(0u64..=1_000_000u64),
        )
        .base_name("z_duration")
        .function(syn::parse_quote!(duration_echo))
        .panic()
        .function(syn::parse_quote!(duration_nested_echo))
        .panic();

    let src = write(cbindgen, registry, "bounded_duration");
    let compact: String = src.split_whitespace().collect();

    assert!(
        compact.contains("pubconstZ_DURATION_NICHE_0:u64=18446744073709551615"),
        "{src}"
    );
    assert!(
        compact.contains("pubconstZ_DURATION_NICHE_1:u64=18446744073709551614"),
        "{src}"
    );
    assert!(
        compact.contains("pubconstZ_DURATION_NONE:u64=18446744073709551615"),
        "{src}"
    );
    assert!(
        compact.contains("extern\"C\"fnduration_echo(v:u64)->u64"),
        "{src}"
    );
    assert!(
        compact.contains("extern\"C\"fnduration_nested_echo(v:u64)->u64"),
        "{src}"
    );
    assert!(!compact.contains("v:*constu64"), "{src}");
    assert!(!compact.contains("v_present"), "{src}");
    assert!(compact.contains("ifv==18446744073709551615"), "{src}");
}

#[test]
fn bounded_float_option_uses_a_finite_bit_exact_niche() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "pub fn ratio_from_f64(v: f64) -> Ratio { unimplemented!() }",
        "pub fn ratio_to_f64(v: Ratio) -> f64 { unimplemented!() }",
        "pub fn ratio_echo(v: Option<Ratio>) -> Option<Ratio> { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| {
        // `syn::Item`, not `ItemFn`: a fixture declares the types it names.
        let item: syn::Item = syn::parse_str(source).unwrap();
        (item, loc.clone())
    })
    .collect();
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).unwrap();
    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(myflat))
        .convert(
            prebindgen_registry::convert!(Ratio)
                .input(prebindgen_registry::fun!(ratio_from_f64))
                .output(prebindgen_registry::fun!(ratio_to_f64))
                .valid_range(0.0f64..=1.0f64),
        )
        .base_name("z_ratio")
        .function(syn::parse_quote!(ratio_echo))
        .panic();

    let src = write(cbindgen, registry, "bounded_float");
    let compact: String = src.split_whitespace().collect();

    assert!(
        compact.contains("pubconstZ_RATIO_NONE:f64=1.7976931348623157e308f64"),
        "{src}"
    );
    assert!(
        compact.contains("extern\"C\"fnratio_echo(v:f64)->f64"),
        "{src}"
    );
    assert!(
        compact.contains("v.to_bits()==9218868437227405311"),
        "{src}"
    );
    assert!(compact.contains("myflat::ratio_to_f64(v)"), "{src}");
}

#[test]
fn custom_conversion_without_domain_stays_infallible() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "pub fn ratio_from_f64(v: f64) -> Ratio { unimplemented!() }",
        "pub fn ratio_to_f64(v: Ratio) -> f64 { unimplemented!() }",
        "pub fn ratio_echo(v: Ratio) -> Ratio { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| {
        // `syn::Item`, not `ItemFn`: a fixture declares the types it names.
        let item: syn::Item = syn::parse_str(source).unwrap();
        (item, loc.clone())
    })
    .collect();
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).unwrap();
    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(myflat))
        .convert(
            prebindgen_registry::convert!(Ratio)
                .input(prebindgen_registry::fun!(ratio_from_f64))
                .output(prebindgen_registry::fun!(ratio_to_f64)),
        )
        .function(syn::parse_quote!(ratio_echo));

    let src = write(cbindgen, registry, "unbounded_conversion");
    let compact: String = src.split_whitespace().collect();

    assert!(
        compact.contains("fn__cbg_in_Ratio(v:f64)->myflat::Ratio"),
        "{src}"
    );
    assert!(
        compact.contains("fn__cbg_out_Ratio(v:myflat::Ratio)->f64"),
        "{src}"
    );
    assert!(
        compact.contains("extern\"C\"fnratio_echo(v:f64)->f64"),
        "{src}"
    );
}

#[test]
fn custom_conversion_stays_unrendered_until_final_write() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "pub fn ratio_from_f64(v: f64) -> Ratio { unimplemented!() }",
        "pub fn ratio_to_f64(v: Ratio) -> f64 { unimplemented!() }",
        "pub fn ratio_echo(v: Ratio) -> Ratio { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| (syn::parse_str(source).unwrap(), loc.clone()))
    .collect();
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).unwrap();
    let generated = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(myflat))
        .convert(
            prebindgen_registry::convert!(Ratio)
                .input(prebindgen_registry::fun!(ratio_from_f64))
                .output(prebindgen_registry::fun!(ratio_to_f64)),
        )
        .function(syn::parse_quote!(ratio_echo))
        .build_with(registry)
        .expect("resolve");

    assert_eq!(
        generated
            .gen
            .compiled_fns
            .iter()
            .filter(|function| function.is_custom())
            .count(),
        2,
        "both custom directions must retain semantic plans before final writing"
    );
}

#[test]
fn trait_backed_custom_conversion_renders_from_the_late_plan() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> =
        ["pub fn ratio_echo(v: Ratio) -> Ratio { unimplemented!() }"]
            .into_iter()
            .map(|source| (syn::parse_str(source).unwrap(), loc.clone()))
            .collect();
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).unwrap();
    let cbindgen = CbindgenBuilder::new()
        .source_module(syn::parse_quote!(myflat))
        .convert(
            prebindgen_registry::convert!(Ratio)
                .input(prebindgen_registry::from!(f64))
                .output(prebindgen_registry::into!(f64)),
        )
        .function(syn::parse_quote!(ratio_echo));

    let src = write(cbindgen, registry, "trait_custom_conversion");
    let compact: String = src.split_whitespace().collect();
    assert!(
        compact.contains("<f64as::core::convert::Into<myflat::Ratio>>::into(v)"),
        "{src}"
    );
    assert!(
        compact.contains("<myflat::Ratioas::core::convert::Into<f64>>::into(v)"),
        "{src}"
    );
}

#[test]
fn output_terminals_stay_unrendered_until_final_write() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "pub struct Handle;",
        "pub struct Error;",
        "pub struct Payload;",
        "pub enum Mode { First, Second }",
        "pub fn output_unit() {}",
        "pub fn output_string() -> String { unimplemented!() }",
        "pub fn output_scalar() -> u64 { unimplemented!() }",
        "pub fn output_handle() -> Handle { unimplemented!() }",
        "pub fn output_error() -> Error { unimplemented!() }",
        "pub fn output_value_opaque() -> Payload { unimplemented!() }",
        "pub fn output_enum() -> Mode { unimplemented!() }",
    ]
    .into_iter()
    .map(|source| (syn::parse_str(source).unwrap(), loc.clone()))
    .collect();
    let registry = crate::test_util::reg_from_items(items).unwrap();
    let generated = CbindgenBuilder::new()
        .free_memory_function("binding_free")
        .opaque_ptr(syn::parse_quote!(Handle))
        .opaque_error(syn::parse_quote!(Error), syn::parse_quote!(error_message))
        .opaque_owned_struct(syn::parse_quote!(Payload), syn::parse_quote!(OpaquePayload))
        .enum_type(syn::parse_quote!(Mode))
        .function(syn::parse_quote!(output_unit))
        .function(syn::parse_quote!(output_string))
        .function(syn::parse_quote!(output_scalar))
        .function(syn::parse_quote!(output_handle))
        .function(syn::parse_quote!(output_error))
        .function(syn::parse_quote!(output_value_opaque))
        .function(syn::parse_quote!(output_enum))
        .build_with(registry)
        .expect("resolve");

    assert_eq!(
        generated
            .gen
            .compiled_fns
            .iter()
            .filter(|function| function.is_output_terminal())
            .count(),
        7,
        "every whole-value output operation must retain a semantic plan before final writing"
    );
}

#[test]
fn input_terminals_stay_unrendered_until_final_write() {
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = [
        "pub struct Handle;",
        "pub struct Payload;",
        "pub enum Mode { First, Second }",
        "pub fn input_handle(v: Handle) {}",
        "pub fn input_value_opaque(v: Payload) {}",
        "pub fn input_enum(v: Mode) {}",
        "pub fn input_string(v: String) {}",
        "pub fn input_str(v: &str) {}",
        "pub fn input_bool(v: bool) {}",
        "pub fn input_scalar(v: u64) {}",
    ]
    .into_iter()
    .map(|source| (syn::parse_str(source).unwrap(), loc.clone()))
    .collect();
    let registry = crate::test_util::reg_from_items(items).unwrap();
    let generated = CbindgenBuilder::new()
        .opaque_ptr(syn::parse_quote!(Handle))
        .opaque_owned_struct(syn::parse_quote!(Payload), syn::parse_quote!(OpaquePayload))
        .enum_type(syn::parse_quote!(Mode))
        .function(syn::parse_quote!(input_handle))
        .panic()
        .function(syn::parse_quote!(input_value_opaque))
        .panic()
        .function(syn::parse_quote!(input_enum))
        .panic()
        .function(syn::parse_quote!(input_string))
        .panic()
        .function(syn::parse_quote!(input_str))
        .panic()
        .function(syn::parse_quote!(input_bool))
        .function(syn::parse_quote!(input_scalar))
        .build_with(registry)
        .expect("resolve");

    assert_eq!(
        generated
            .gen
            .compiled_fns
            .iter()
            .filter(|function| function.is_input_terminal())
            .count(),
        7,
        "every whole-value input operation must retain a semantic plan before final writing"
    );
}

/// An adapter with no declarations writes an empty (whitespace-only) file.
#[test]
fn empty_adapter_writes_empty_file() {
    let cbindgen = CbindgenBuilder::new();
    let registry: RegistryBuilder = crate::test_util::reg_from_items(Vec::new()).expect("empty");
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
        crate::test_util::reg_from_items(declare_referenced([(syn::Item::Fn(func), loc.clone())]))
            .expect("index items");

    let cbindgen = CbindgenBuilder::new()
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

/// A declared conversion's own fallibility is the one that counts, at whatever
/// depth it sits.
///
/// Three walks read one value — the shape, the encode, and whether the encode
/// can fail — and the third used to peel `Option`/`Vec`/`Cow` before consulting
/// the converter table. Once the first two stop at a node with a wire of its
/// own, the third disagreeing is not cosmetic: it decides whether a non-`Result`
/// binding must opt into `.panic()`. Reading past a fallible custom converter
/// says "infallible", and the emitted wrapper aborts where nobody opted in
/// (#428 review).
///
/// Here `Option<Duration>` has a **fallible** declared output converter over an
/// **infallible** `Duration` one, so the two answers differ and only the outer
/// one is right.
#[test]
fn a_declared_conversion_owns_its_own_fallibility() {
    let items = || {
        let loc = SourceLocation::default();
        [
            "#[prebindgen] pub type Duration = std::time::Duration;",
            "pub fn duration_from_millis(v: u64) -> Duration { unimplemented!() }",
            "pub fn duration_to_millis(v: &Duration) -> u64 { unimplemented!() }",
            "pub fn maybe_from_millis(v: i64) -> Option<Duration> { unimplemented!() }",
            "pub fn maybe_to_millis(v: &Option<Duration>) -> Result<i64, String> { unimplemented!() }",
            "pub fn maybe_get() -> Option<Duration> { unimplemented!() }",
        ]
        .into_iter()
        .map(|source| {
            let item: syn::Item = syn::parse_str(source).unwrap();
            (item, loc.clone())
        })
        .collect::<Vec<_>>()
    };
    let declare = || {
        CbindgenBuilder::new()
            .source_module(syn::parse_quote!(myflat))
            .convert(
                prebindgen_registry::convert!(Duration)
                    .input(prebindgen_registry::fun!(duration_from_millis))
                    .output(prebindgen_registry::fun!(duration_to_millis)),
            )
            .convert(
                prebindgen_registry::convert!(Option<Duration>)
                    .input(prebindgen_registry::fun!(maybe_from_millis))
                    .output(prebindgen_registry::fun!(maybe_to_millis)),
            )
    };

    // No `.panic()`: the outer converter can fail and the function returns no
    // `Result`, so there is nowhere for the error to go and the declaration is
    // refused.
    let registry = crate::test_util::reg_from_items(declare_referenced(items())).unwrap();
    let message = catch_msg(|| {
        let _ = write(
            declare().function(syn::parse_quote!(maybe_get)),
            registry,
            "fallible_custom_option",
        );
    });
    assert!(
        message.contains("fallible binding conversion") && message.contains(".panic()"),
        "the refusal names the missing opt-in: {message}"
    );

    // …and with the opt-in, the same declaration generates.
    let registry = crate::test_util::reg_from_items(declare_referenced(items())).unwrap();
    let src = write(
        declare().function(syn::parse_quote!(maybe_get)).panic(),
        registry,
        "fallible_custom_option_panic",
    );
    assert!(
        src.contains("__cbg_out_Option___Duration__"),
        "the declared converter is what the wrapper calls:\n{src}"
    );
}
