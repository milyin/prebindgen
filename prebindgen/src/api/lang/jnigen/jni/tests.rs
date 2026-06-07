use super::*;
use crate::api::core::niches::{NicheSlot, Niches};
use crate::api::core::registry::{Registry, TypeEntry, TypeKey};
use quote::ToTokens;

/// Build a `TypeEntry` for use in tests. The function body is not
/// inspected by `option_input` / `option_output`; only the ident,
/// destination, and niches matter, so we use a stub `ItemFn`.
fn entry(wire: syn::Type, conv_name: &str, niches: Niches) -> TypeEntry<KotlinMeta> {
    let ident = syn::Ident::new(conv_name, proc_macro2::Span::call_site());
    let func: syn::ItemFn = syn::parse_quote!(
        unsafe fn #ident<'env, 'v>(
            env: &mut jni::JNIEnv<'env>,
            v: &#wire,
        ) -> ::core::result::Result<(), __JniErr> {
            Ok(())
        }
    );
    TypeEntry {
        destination: wire,
        function: func,
        pre_stages: vec![],
        subs: vec![],
        required: false,
        niches,
        into_sources: None,
        metadata: KotlinMeta::default(),
    }
}

fn install_input(
    reg: &mut Registry<KotlinMeta>,
    ty_str: &str,
    rank: usize,
    e: TypeEntry<KotlinMeta>,
) {
    reg.input_types[rank].insert(TypeKey::parse(ty_str), Some(e));
}
fn install_output(
    reg: &mut Registry<KotlinMeta>,
    ty_str: &str,
    rank: usize,
    e: TypeEntry<KotlinMeta>,
) {
    reg.output_types[rank].insert(TypeKey::parse(ty_str), Some(e));
}

/// Single niche, single Option layer — wire stays the inner wire,
/// remainder is empty. No widening to JObject.
#[test]
fn option_carves_single_niche() {
    let mut reg = Registry::default();
    install_input(
        &mut reg,
        "TestType",
        0,
        entry(
            syn::parse_quote!(jni::sys::jlong),
            "jlong_to_TestType_aaaa",
            Niches::one(syn::parse_quote!(0i64), syn::parse_quote!(*v == 0)),
        ),
    );

    let inner_ty: syn::Type = syn::parse_quote!(TestType);
    let (wire, _body, niches) =
        option_input(&inner_ty, &reg).expect("Option<TestType> resolves");

    assert_eq!(
        wire.to_token_stream().to_string(),
        "jni :: sys :: jlong",
        "wire stays jlong (no JObject widening)"
    );
    assert!(niches.is_empty(), "single niche fully consumed");
}

/// Two niches, two cascading Option layers, both stay on the same
/// wire. The third layer hits empty niches and falls back to box.
#[test]
fn option_cascades_through_multi_niche() {
    let mut reg = Registry::default();

    // TestType: jint with two niches (MIN, MAX).
    install_input(
        &mut reg,
        "TestType",
        0,
        entry(
            syn::parse_quote!(jni::sys::jint),
            "jint_to_TestType_aaaa",
            Niches::from_slots([
                NicheSlot {
                    value: syn::parse_quote!(jni::sys::jint::MIN),
                    matches: syn::parse_quote!(*v == jni::sys::jint::MIN),
                },
                NicheSlot {
                    value: syn::parse_quote!(jni::sys::jint::MAX),
                    matches: syn::parse_quote!(*v == jni::sys::jint::MAX),
                },
            ]),
        ),
    );

    // Layer 1: Option<TestType>.
    let layer1_ty: syn::Type = syn::parse_quote!(TestType);
    let (w1, _, n1) = option_input(&layer1_ty, &reg).expect("layer 1 resolves");
    assert_eq!(w1.to_token_stream().to_string(), "jni :: sys :: jint");
    assert_eq!(n1.len(), 1, "first carve leaves one niche");

    // Install the layer-1 wrapper as a rank-1 entry so layer-2 can
    // look it up. (In the real resolver this happens automatically;
    // here we mimic it by installing the produced ConverterImpl.)
    install_input(
        &mut reg,
        "Option < TestType >",
        1,
        entry(w1.clone(), "jint_to_OptionTestType_bbbb", n1),
    );

    // Layer 2: Option<Option<TestType>>.
    let layer2_ty: syn::Type = syn::parse_quote!(Option<TestType>);
    let (w2, _, n2) = option_input(&layer2_ty, &reg).expect("layer 2 resolves");
    assert_eq!(
        w2.to_token_stream().to_string(),
        "jni :: sys :: jint",
        "wire still jint at layer 2 — no widening"
    );
    assert!(n2.is_empty(), "second carve consumes the last niche");

    // Install layer-2 wrapper for the layer-3 lookup.
    install_input(
        &mut reg,
        "Option < Option < TestType > >",
        1,
        entry(w2.clone(), "jint_to_OptionOptionTestType_cccc", n2),
    );

    // Layer 3: Option<Option<Option<TestType>>>. No niches left,
    // inner wire is jint (a JNI primitive) → boxed-Long fallback.
    let layer3_ty: syn::Type = syn::parse_quote!(Option<Option<TestType>>);
    let (w3, _, n3) =
        option_input(&layer3_ty, &reg).expect("layer 3 resolves via box fallback");
    assert_eq!(
        w3.to_token_stream().to_string(),
        "jni :: objects :: JObject",
        "layer 3 widens to JObject (box fallback)"
    );
    assert!(
        n3.is_empty(),
        "boxed wrapper exposes no further niches — every JObject carries meaning"
    );
}

/// Output side mirrors input: niche values are emitted in the
/// `None` arm of the match, and the remainder is re-exported.
#[test]
fn option_output_cascades_through_multi_niche() {
    let mut reg = Registry::default();
    install_output(
        &mut reg,
        "TestType",
        0,
        entry(
            syn::parse_quote!(jni::sys::jint),
            "TestType_to_jint_aaaa",
            Niches::from_slots([
                NicheSlot {
                    value: syn::parse_quote!(-1i32),
                    matches: syn::parse_quote!(*v == -1),
                },
                NicheSlot {
                    value: syn::parse_quote!(-2i32),
                    matches: syn::parse_quote!(*v == -2),
                },
            ]),
        ),
    );

    let inner_ty: syn::Type = syn::parse_quote!(TestType);
    let (w1, body1, n1) =
        option_output(&inner_ty, &reg).expect("Option<TestType> output resolves");
    assert_eq!(w1.to_token_stream().to_string(), "jni :: sys :: jint");
    assert_eq!(n1.len(), 1, "one slot left after carving the first");
    // The body must reference the carved value (-1) in the None arm.
    let body_str = body1.to_token_stream().to_string();
    assert!(
        body_str.contains("None => - 1i32") || body_str.contains("None => -1i32"),
        "expected `None => -1i32` in body; got:\n{}",
        body_str,
    );

    install_output(
        &mut reg,
        "Option < TestType >",
        1,
        entry(w1.clone(), "OptionTestType_to_jint_bbbb", n1),
    );

    let layer2_ty: syn::Type = syn::parse_quote!(Option<TestType>);
    let (w2, body2, n2) =
        option_output(&layer2_ty, &reg).expect("Option<Option<TestType>> output resolves");
    assert_eq!(w2.to_token_stream().to_string(), "jni :: sys :: jint");
    assert!(n2.is_empty());
    let body2_str = body2.to_token_stream().to_string();
    assert!(
        body2_str.contains("None => - 2i32") || body2_str.contains("None => -2i32"),
        "second layer must use the second niche (-2); got:\n{}",
        body2_str,
    );
}

/// JObject-shaped wires get the implicit `null` niche via
/// [`default_niches_for_wire`], so `Option<T>` over a struct
/// decoder stays on `JObject` (no boxing).
#[test]
fn option_over_jobject_uses_default_null_niche() {
    let mut reg = Registry::default();
    install_input(
        &mut reg,
        "MyStruct",
        0,
        entry(
            syn::parse_quote!(jni::objects::JObject),
            "JObject_to_MyStruct_aaaa",
            default_niches_for_wire(&syn::parse_quote!(jni::objects::JObject)),
        ),
    );

    let ty: syn::Type = syn::parse_quote!(MyStruct);
    let (wire, _, rest) = option_input(&ty, &reg).expect("Option<MyStruct> resolves");
    assert_eq!(
        wire.to_token_stream().to_string(),
        "jni :: objects :: JObject"
    );
    assert!(rest.is_empty(), "JObject's single null niche is consumed");
}

/// No niche AND non-primitive wire → wrap fails (resolver falls
/// through). Demonstrates that the boxed fallback only kicks in for
/// JNI primitives.
#[test]
fn option_fails_when_no_niche_and_non_primitive_wire() {
    let mut reg = Registry::default();
    install_input(
        &mut reg,
        "MyStruct",
        0,
        entry(
            syn::parse_quote!(jni::objects::JObject),
            "JObject_to_MyStruct_aaaa",
            Niches::empty(), // explicit empty — author opted out
        ),
    );
    let ty: syn::Type = syn::parse_quote!(MyStruct);
    assert!(option_input(&ty, &reg).is_none());
}

/// Boxed fallback widens to `JObject` and exposes no further
/// niches — protects callers from cascading when a layer has had
/// to widen.
#[test]
fn option_box_fallback_exposes_no_niches() {
    let mut reg = Registry::default();
    install_input(
        &mut reg,
        "i64",
        0,
        entry(
            syn::parse_quote!(jni::sys::jlong),
            "jlong_to_i64_aaaa",
            Niches::empty(), // primitive `i64` — no niche
        ),
    );
    let ty: syn::Type = syn::parse_quote!(i64);
    let (wire, _, rest) = option_input(&ty, &reg).expect("Option<i64> via box fallback");
    assert_eq!(
        wire.to_token_stream().to_string(),
        "jni :: objects :: JObject"
    );
    assert!(rest.is_empty());
}

// ────────────────────────────────────────────────────────────────────────
// End-to-end pipeline snapshot: drive a representative `JniGen` config
// through `write_rust` + `write_kotlin` and assert on the generated Rust and
// Kotlin. Mirrors `cbindgen`'s `tests.rs` behavioural-assertion style (the
// authoritative byte-for-byte check is the `zenoh-flat-jni` consumer diff);
// this is the in-crate regression net.
// ────────────────────────────────────────────────────────────────────────

/// Build the representative config: an opaque handle (`ZThing`) with a
/// free-function constructor returning `Result<ZThing, Error>` (exception
/// routing) and a free-function accessor, a C-like enum (`Color`, mixed
/// discriminants), and a throwable data class (`Error`).
#[cfg(test)]
fn snapshot_pipeline() -> (String, std::collections::BTreeMap<String, String>) {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Error {
                    pub message: String,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Color {
                    Red,
                    Green = 5,
                    Blue,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_new() -> Result<ZThing, Error> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_name(this_: &ZThing) -> String {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
    ];
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .source_module(syn::parse_quote!(myflat))
        .package_prefix("io.test.jni")
        .data_class(syn::parse_quote!(Error))
        .ptr_class(syn::parse_quote!(ZThing))
        .enum_class(syn::parse_quote!(Color))
        .package("thing")
        .package_fun(syn::parse_quote!(z_thing_new))
        .package_fun(syn::parse_quote!(z_thing_name));

    let dir = std::env::temp_dir().join(format!("jnigen_snap_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let mut kotlin = std::collections::BTreeMap::new();
    for p in &paths {
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        kotlin.insert(name, std::fs::read_to_string(p).unwrap());
    }
    (rust, kotlin)
}

#[test]
fn snapshot_rust_side() {
    let (rust, _) = snapshot_pipeline();
    let rc: String = rust.split_whitespace().collect();
    // Source-module-qualified calls into the flat crate.
    assert!(rc.contains("myflat::z_thing_new"), "{rust}");
    assert!(rc.contains("myflat::z_thing_name"), "{rust}");
    // Opaque handle round-trips as a boxed pointer of the source type.
    assert!(rc.contains("myflat::ZThing"), "{rust}");
    assert!(rc.contains("Box::from_raw"), "{rust}");
    // Errors funnel to the single `signal_error` sink fn (no JVM throw).
    assert!(rc.contains("fnsignal_error"), "{rust}");
    assert!(
        rc.contains("signal_error(&mutenv,&__error_sink,&__e)"),
        "{rust}"
    );
    // The extern takes the trailing error-sink param; no throw fn exists.
    assert!(rc.contains("__error_sink:jni::objects::JObject"), "{rust}");
    assert!(!rc.contains("throw_Error"), "{rust}");
    // JNI extern wrappers.
    assert!(rc.contains("externfn") || rc.contains("extern\"C\""), "{rust}");
}

#[test]
fn snapshot_kotlin_side() {
    let (_, kotlin) = snapshot_pipeline();
    let names: Vec<&String> = kotlin.keys().collect();

    // Shared base + centralized native holder always emitted.
    assert!(kotlin.contains_key("NativeHandle.kt"), "files: {names:?}");
    assert!(kotlin.contains_key("JNINative.kt"), "files: {names:?}");

    // The error-sink channel lives in NativeHandle.kt; no exception classes.
    let nh: String = kotlin["NativeHandle.kt"].split_whitespace().collect();
    assert!(nh.contains("funinterfaceErrorSink"), "{}", kotlin["NativeHandle.kt"]);
    assert!(nh.contains("classZException"), "{}", kotlin["NativeHandle.kt"]);
    assert!(!kotlin.contains_key("Error.kt") || !kotlin["Error.kt"].contains(": Exception"));

    let native: String = kotlin["JNINative.kt"].split_whitespace().collect();
    assert!(native.contains("externalfun"), "{}", kotlin["JNINative.kt"]);
    // Each extern declares the trailing `errorSink: Any` param.
    assert!(native.contains("errorSink:Any"), "{}", kotlin["JNINative.kt"]);

    // Enum class with mixed discriminants 0 / 5 / 6 and a `fromInt` factory.
    let color = kotlin.get("Color.kt").expect("Color.kt");
    let cc: String = color.split_whitespace().collect();
    assert!(cc.contains("enumclassColor"), "{color}");
    assert!(cc.contains("RED(0)"), "{color}");
    assert!(cc.contains("GREEN(5)"), "{color}");
    assert!(cc.contains("BLUE(6)"), "{color}");
    assert!(cc.contains("funfromInt"), "{color}");

    // Typed handle subclass of NativeHandle.
    let thing = kotlin.get("ZThing.kt").expect("ZThing.kt");
    assert!(
        thing.split_whitespace().collect::<String>().contains(":NativeHandle"),
        "{thing}"
    );

    // The free-function wrappers live in the namespace package object, install
    // a default sink, and rethrow.
    let pkg = kotlin
        .values()
        .find(|v| v.contains("public fun zThingNew"))
        .cloned()
        .unwrap_or_default();
    let pc: String = pkg.split_whitespace().collect();
    assert!(pc.contains("ErrorSink{"), "package wrappers: {pkg}");
    assert!(pc.contains("throwZException(it)"), "package wrappers: {pkg}");
}
