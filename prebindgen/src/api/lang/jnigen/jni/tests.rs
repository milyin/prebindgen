use super::*;
use crate::api::core::niches::{NicheSlot, Niches};
use crate::api::core::registry::{Registry, TypeEntry, TypeKey};
use quote::ToTokens;

/// A process-unique temp directory for a snapshot pipeline run. Keyed by pid +
/// a monotonic counter so the snapshot tests (which share a helper and run on
/// separate threads) never clobber each other's output dir.
fn unique_snapshot_dir(prefix: &str) -> std::path::PathBuf {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static SEQ: AtomicUsize = AtomicUsize::new(0);
    let seq = SEQ.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!("{prefix}_{}_{}", std::process::id(), seq))
}

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
        .fun(syn::parse_quote!(z_thing_new))
        .fun(syn::parse_quote!(z_thing_name));

    let dir = unique_snapshot_dir("jnigen_snap");
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
    // Errors funnel to the single `signal_error` channel fn (no JVM throw); it
    // now invokes the error callback with `(je, ze…)`.
    assert!(rc.contains("fnsignal_error"), "{rust}");
    assert!(
        rc.contains("signal_error(&mutenv,&__error_sink,::core::option::Option::Some(&__e.to_string()),__ze_defaults"),
        "{rust}"
    );
    // The extern takes the trailing error-callback param; no throw fn exists.
    assert!(rc.contains("__error_sink:jni::objects::JObject"), "{rust}");
    assert!(!rc.contains("throw_Error"), "{rust}");
    // JNI extern wrappers.
    assert!(rc.contains("externfn") || rc.contains("extern\"C\""), "{rust}");
}

#[test]
fn snapshot_kotlin_side() {
    let (_, kotlin) = snapshot_pipeline();
    let names: Vec<&String> = kotlin.keys().collect();
    // Output is now one merged `.kt` file per package, so look declarations up
    // by content marker rather than by per-class file name.
    let find = |needle: &str| -> String {
        kotlin
            .values()
            .find(|v| v.contains(needle))
            .cloned()
            .unwrap_or_else(|| panic!("no generated file contains `{needle}`; files: {names:?}"))
    };

    // Shared base + centralized native holder are always emitted (merged into
    // their package's single file).
    let nh = find("abstract class NativeHandle");
    let native = find("object JNINative");

    // No framework `ErrorSink` interface — the error channel is a plain function
    // type passed per call. `ZException` remains as the `onError` default throw.
    let nhc: String = nh.split_whitespace().collect();
    assert!(!nhc.contains("funinterfaceErrorSink"), "{nh}");
    assert!(nhc.contains("classZException"), "{nh}");

    let nativec: String = native.split_whitespace().collect();
    assert!(nativec.contains("externalfun"), "{native}");
    // Each extern declares the trailing `errorSink: Any` param.
    assert!(nativec.contains("errorSink:Any"), "{native}");

    // Enum class with mixed discriminants 0 / 5 / 6 and a `fromInt` factory.
    let color = find("enum class Color");
    let cc: String = color.split_whitespace().collect();
    assert!(cc.contains("enumclassColor"), "{color}");
    assert!(cc.contains("RED(0)"), "{color}");
    assert!(cc.contains("GREEN(5)"), "{color}");
    assert!(cc.contains("BLUE(6)"), "{color}");
    assert!(cc.contains("funfromInt"), "{color}");

    // Typed handle subclass of NativeHandle.
    let thing = find("class ZThing(");
    assert!(
        thing.split_whitespace().collect::<String>().contains(":NativeHandle"),
        "{thing}"
    );

    // The free-function wrappers live in the namespace package object, take a
    // trailing `onError` callback, and call it on failure (no throw).
    let pkg = kotlin
        .values()
        .find(|v| v.contains("public fun zThingNew"))
        .cloned()
        .unwrap_or_default();
    let pc: String = pkg.split_whitespace().collect();
    // The error lambda's params are named: the fixed `je` binding message,
    // then the decomposed error leaves.
    assert!(pc.contains("onError:(je:String?"), "package wrappers: {pkg}");
    assert!(pc.contains("if(__cap_failed)returnonError("), "package wrappers: {pkg}");
    // The throw lives only in the `onError` *default* (overridable per call), not
    // in the wrapper body itself.
    assert!(pc.contains("=>throwZException") || pc.contains("->throwZException"), "package wrappers: {pkg}");
}

// ────────────────────────────────────────────────────────────────────────
// Callback pipeline snapshot: `impl Fn(...)` params unified onto the
// output-expansion machinery — a decomposed arg (ZThing has a canonical
// output) delivers its leaves through the erased lambda `invoke`; a
// plan-less arg (ZOther) falls back to whole-handle delivery with the
// post-invoke `close()`; `impl Fn()` is a zero-arg `() -> Unit`.
// ────────────────────────────────────────────────────────────────────────

#[cfg(test)]
fn callback_snapshot_pipeline() -> (String, std::collections::BTreeMap<String, String>) {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_name(this_: &ZThing) -> String {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_thing_sub(
                    cb: impl Fn(ZThing) + Send + Sync + 'static,
                    on_close: impl Fn() + Send + Sync + 'static,
                ) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_other_sub(cb: impl Fn(ZOther) + Send + Sync + 'static) {
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
        .package("thing")
        .ptr_class(syn::parse_quote!(ZThing))
        // Canonical output: handle (identity) + its string form — a callback
        // arg of ZThing decomposes into these 2 leaves.
        .ptr_class_output_direct()
        .ptr_class_output(syn::parse_quote!(z_thing_name))
        .fun_accessor(syn::parse_quote!(z_thing_name))
        // ZOther: plain ptr_class, no canonical output ⇒ whole-handle fallback.
        .ptr_class(syn::parse_quote!(ZOther))
        .fun(syn::parse_quote!(z_thing_sub))
        .fun(syn::parse_quote!(z_other_sub));

    let dir = unique_snapshot_dir("jnigen_cb_snap");
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
fn callback_snapshot_rust_side() {
    let (rust, _) = callback_snapshot_pipeline();
    let rc: String = rust.split_whitespace().collect();
    // The trampoline invokes the erased Kotlin lambda — never a typed `run`.
    assert!(rc.contains(r#""invoke""#), "{rust}");
    assert!(!rc.contains(r#""run""#), "{rust}");
    // Decomposed ZThing arg: 2 leaves ⇒ 2-object descriptor; on_close ⇒
    // zero-arg Function0 descriptor.
    assert!(
        rc.contains(r#""(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;""#),
        "{rust}"
    );
    assert!(rc.contains(r#""()Ljava/lang/Object;""#), "{rust}");
    // Fallback ZOther arg: 1-object descriptor + post-invoke `close()` of the
    // boxed handle. The decomposed path never closes (ownership transfers).
    assert!(
        rc.contains(r#""(Ljava/lang/Object;)Ljava/lang/Object;""#),
        "{rust}"
    );
    assert!(rc.contains(r#""close""#), "{rust}");
    // Daemon-thread attachment + local-frame bracketing kept from the old
    // trampoline.
    assert!(rc.contains("attach_current_thread_as_daemon"), "{rust}");
    assert!(rc.contains("push_local_frame"), "{rust}");
    assert!(rc.contains("pop_local_frame"), "{rust}");
    // Identity leaf of the decomposed arg: moved into a fresh box and wrapped
    // in the typed handle class (declared under the `thing` subpackage).
    assert!(rc.contains("io/test/jni/thing/ZThing"), "{rust}");
    // The decomposed leaf encode calls the accessor off the owned root.
    assert!(rc.contains("myflat::z_thing_name"), "{rust}");
}

#[test]
fn callback_snapshot_kotlin_side() {
    let (_, kotlin) = callback_snapshot_pipeline();
    let names: Vec<&String> = kotlin.keys().collect();

    // No fun-interface files are generated (the `callbacks/` subsystem is gone).
    assert!(
        !names.iter().any(|n| n.contains("Callback")),
        "files: {names:?}"
    );
    for v in kotlin.values() {
        assert!(!v.contains("fun interface"), "{v}");
    }

    // Extern tier: callbacks erased to `Any`, like the errorSink.
    let native: String = kotlin
        .values()
        .find(|v| v.contains("object JNINative"))
        .map(|v| v.split_whitespace().collect())
        .unwrap_or_else(|| panic!("no generated file contains `object JNINative`; files: {names:?}"));
    assert!(native.contains("cb:Any"), "{native}");
    assert!(native.contains("onClose:Any"), "{native}");

    // Wrapper tier: typed lambdas with NAMED parameters — decomposed ZThing's
    // identity leaf is `handle`, its accessor leaf strips the receiver-type
    // prefix (`z_thing_name` on `&ZThing` → `name`); on_close ⇒ () -> Unit;
    // the plan-less fallback arg is the decapped type short (`zOther`).
    let pkg = kotlin
        .values()
        .find(|v| v.contains("public fun zThingSub"))
        .cloned()
        .unwrap_or_default();
    let pc: String = pkg.split_whitespace().collect();
    assert!(pc.contains("cb:(handle:ZThing,name:String)->Unit"), "{pkg}");
    assert!(pc.contains("onClose:()->Unit"), "{pkg}");
    assert!(pc.contains("cb:(zOther:ZOther)->Unit"), "{pkg}");
}

#[test]
#[should_panic(expected = "Function22")]
fn erased_invoke_descriptor_rejects_over_22() {
    let _ = erased_invoke_descriptor(23);
}

#[test]
fn strip_receiver_prefix_cases() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let fns: &[&str] = &[
        // Regular snake: ZSample → z_sample_.
        "fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { todo!() }",
        // Irregular snake: type ZKeyExpr but prefix z_keyexpr_ — the
        // normalized (underscore-free) comparison still matches.
        "fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { todo!() }",
        // Double-letter type short: ZZBytes → z_zbytes_.
        "fn z_zbytes_to_bytes(z: &ZZBytes) -> Vec<u8> { todo!() }",
        // Receiver mismatch: falls back to stripping a bare `z_`.
        "fn z_error_code(f: &Foo) -> i32 { todo!() }",
        // No type prefix, no z_: kept whole.
        "fn get_name(f: &Foo) -> &str { todo!() }",
    ];
    let items = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect::<Vec<_>>();
    let reg = Registry::<KotlinMeta>::from_items(items).expect("index");
    let id = |s: &str| syn::Ident::new(s, proc_macro2::Span::call_site());

    assert_eq!(strip_receiver_prefix(&reg, &id("z_sample_key_expr")), "key_expr");
    assert_eq!(strip_receiver_prefix(&reg, &id("z_keyexpr_as_str")), "as_str");
    assert_eq!(strip_receiver_prefix(&reg, &id("z_zbytes_to_bytes")), "to_bytes");
    assert_eq!(strip_receiver_prefix(&reg, &id("z_error_code")), "error_code");
    assert_eq!(strip_receiver_prefix(&reg, &id("get_name")), "get_name");
}
