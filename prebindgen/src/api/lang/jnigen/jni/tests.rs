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
    let (wire, _body, niches) = option_input(&inner_ty, &reg).expect("Option<TestType> resolves");

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
    let (w3, _, n3) = option_input(&layer3_ty, &reg).expect("layer 3 resolves via box fallback");
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
    let (w1, body1, n1) = option_output(&inner_ty, &reg).expect("Option<TestType> output resolves");
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
        rc.contains("let__zd=__ze_defaults(&mutenv);signal_error(&mutenv,&__error_sink,&__SINK_MID,__SINK_FQN,__SINK_DESCR,::core::option::Option::Some(&__e.to_string()),&__zd"),
        "{rust}"
    );
    // The sink's typed handler `run` is resolved once per process via the
    // cached interface-method statics.
    assert!(rc.contains("CachedIfaceMethod"), "{rust}");
    // The extern takes the trailing error-callback param; no throw fn exists.
    assert!(rc.contains("__error_sink:jni::objects::JObject"), "{rust}");
    assert!(!rc.contains("throw_Error"), "{rust}");
    // JNI extern wrappers.
    assert!(
        rc.contains("externfn") || rc.contains("extern\"C\""),
        "{rust}"
    );
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
    // type passed per call. No `ZException` either: the generated code never
    // throws; the consumer's `onError` decides how a failure surfaces.
    let nhc: String = nh.split_whitespace().collect();
    assert!(!nhc.contains("funinterfaceErrorSink"), "{nh}");
    assert!(!nhc.contains("ZException"), "{nh}");

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
        thing
            .split_whitespace()
            .collect::<String>()
            .contains(":NativeHandle"),
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
    // `onError` is the typed handler fun interface, instantiated at the
    // wrapper's result type; the wrapper calls its `run` on failure.
    assert!(
        pc.contains("onError:JniErrorHandler<") || pc.contains("Handler<"),
        "package wrappers: {pkg}"
    );
    assert!(
        pc.contains("if(__cap_failed)returnonError.run("),
        "package wrappers: {pkg}"
    );
    // `onError` is a **required** parameter (no default) and the wrappers
    // never throw — error surfacing is entirely the caller's business.
    assert!(
        !pkg.contains("throw") && !pkg.contains("ZException"),
        "package wrappers: {pkg}"
    );
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
    // The trampoline invokes the typed callback interface's `run` — never the
    // erased `FunctionN.invoke`.
    assert!(rc.contains(r#""run""#), "{rust}");
    assert!(!rc.contains(r#""invoke""#), "{rust}");
    // Decomposed ZThing arg: 2 typed leaves (raw jlong handle + String),
    // void return; on_close ⇒ zero-arg `()V`.
    assert!(rc.contains(r#""(JLjava/lang/String;)V""#), "{rust}");
    assert!(rc.contains(r#""()V""#), "{rust}");
    // Fallback ZOther arg: 1 typed handle param + post-invoke `close()` of
    // the boxed handle. The decomposed path never closes (ownership
    // transfers).
    assert!(rc.contains(r#""(Lio/test/jni/thing/ZOther;)V""#), "{rust}");
    assert!(rc.contains(r#""close""#), "{rust}");
    // Daemon-thread attachment + local-frame bracketing kept from the old
    // trampoline.
    assert!(rc.contains("attach_current_thread_as_daemon"), "{rust}");
    assert!(rc.contains("push_local_frame"), "{rust}");
    assert!(rc.contains("pop_local_frame"), "{rust}");
    // Identity leaf of the decomposed arg: moved into a fresh box and crosses
    // as a RAW jlong jvalue — no native `new_object` of the typed class.
    assert!(rc.contains("jni::sys::jvalue{j:"), "{rust}");
    assert!(!rc.contains("io/test/jni/thing/ZThing"), "{rust}");
    // The decomposed leaf encode calls the accessor off the owned root.
    assert!(rc.contains("myflat::z_thing_name"), "{rust}");
}

#[test]
fn callback_snapshot_kotlin_side() {
    let (_, kotlin) = callback_snapshot_pipeline();
    let names: Vec<&String> = kotlin.keys().collect();

    // Extern tier: callbacks erased to `Any`, like the errorSink.
    let native: String = kotlin
        .values()
        .find(|v| v.contains("object JNINative"))
        .map(|v| v.split_whitespace().collect())
        .unwrap_or_else(|| {
            panic!("no generated file contains `object JNINative`; files: {names:?}")
        });
    assert!(native.contains("cb:Any"), "{native}");
    assert!(native.contains("onClose:Any"), "{native}");

    // Typed callback `fun interface`s with NAMED parameters — decomposed
    // ZThing's identity leaf is `handle`, its accessor leaf strips the
    // receiver-type prefix (`z_thing_name` on `&ZThing` → `name`); `Fn()` ⇒
    // the shared zero-arg `VoidCallback` (root package); the plan-less
    // fallback arg is the decapped type short (`zOther`).
    let all: String = kotlin
        .values()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
        .split_whitespace()
        .collect();
    assert!(
        all.contains("funinterfaceZThingCallback{publicfunrun(handle:Long,name:String)"),
        "{all}"
    );
    assert!(
        all.contains("funinterfaceZOtherCallback{publicfunrun(zOther:ZOther)"),
        "{all}"
    );
    assert!(all.contains("funinterfaceVoidCallback{publicfunrun()"), "{all}");

    // Wrapper tier: the params are the typed interfaces, forwarded bare.
    let pkg = kotlin
        .values()
        .find(|v| v.contains("public fun zThingSub"))
        .cloned()
        .unwrap_or_default();
    let pc: String = pkg.split_whitespace().collect();
    assert!(pc.contains("cb:ZThingCallback"), "{pkg}");
    assert!(pc.contains("onClose:VoidCallback"), "{pkg}");
    assert!(pc.contains("cb:ZOtherCallback"), "{pkg}");
}

/// Regression: a callback-delivered type that has BOTH a nested handle identity
/// (a child `ptr_class` reached by an accessor) AND its own root identity
/// (`.ptr_class_output_direct()`) must emit the root MOVE after every borrow of
/// the owned value — otherwise the nested child clone (which borrows the root)
/// follows `Box::into_raw(Box::new(value))` and fails to compile with "use of
/// moved value". Declaring `.ptr_class_output_direct()` LAST guarantees the
/// correct order (the emitter emits identity leaves in declaration order, after
/// all non-identity leaves). This mirrors the zenoh-flat `ZQuery` queryable
/// callback (handle + decomposed fields, nested `ZKeyExpr` identity).
#[test]
fn callback_root_identity_moved_after_nested_borrow() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_parent_child(this_: &ZParent) -> &ZChild {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_child_name(this_: &ZChild) -> String {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_parent_sub(
                    cb: impl Fn(ZParent) + Send + Sync + 'static,
                    on_close: impl Fn() + Send + Sync + 'static,
                ) {
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
        // Child handle: canonical output = identity (clone) + its name string.
        .ptr_class(syn::parse_quote!(ZChild))
        .ptr_class_output_direct()
        .ptr_class_output(syn::parse_quote!(z_child_name))
        .fun_accessor(syn::parse_quote!(z_child_name))
        // Parent: a nested child-handle record, then its OWN root identity LAST.
        .ptr_class(syn::parse_quote!(ZParent))
        .ptr_class_output(syn::parse_quote!(z_parent_child))
        .ptr_class_output_direct()
        .fun_accessor(syn::parse_quote!(z_parent_child))
        .fun(syn::parse_quote!(z_parent_sub));

    let dir = unique_snapshot_dir("jnigen_root_id_order");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // The root `ZParent` identity is a move (`Box::new(__cb_arg0)`); the nested
    // `ZChild` identity (and its `z_child_name` leaf) borrow the same owned arg
    // via `z_parent_child(&__cb_arg0)`. Every borrow must precede the move.
    let move_pos = rc
        .find("Box::new(__cb_arg0")
        .unwrap_or_else(|| panic!("root identity move not found in:\n{rust}"));
    let last_borrow = rc
        .rfind("z_parent_child(&__cb_arg0")
        .unwrap_or_else(|| panic!("nested child borrow not found in:\n{rust}"));
    assert!(
        last_borrow < move_pos,
        "root identity move must follow every borrow of the owned arg\n{rust}"
    );
}

/// ZReply-shaped product decomposition: the callback arg's plan contains leaf
/// paths with MULTIPLE `Option`-returning nesting steps (`z_reply_sample` →
/// `z_sample_timestamp`), a nested handle identity reached *through* an
/// `Option` step (`z_reply_sample` → `z_sample_key_expr`), and an Acc leaf
/// whose own return keeps its full `Option<…>` as the converter input
/// (`z_reply_zid -> Option<ZId>`, a value_blob with no canonical child).
/// Every `Option` nesting step must become its own `match` (`None` ⇒ null
/// leaf) — never a blind accessor compose through an `Option`.
#[test]
fn callback_double_option_unwrap_pipeline() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let fns: &[&str] = &[
        "pub fn z_reply_zid(r: &ZReply) -> Option<ZId> { unimplemented!() }",
        "pub fn z_reply_is_ok(r: &ZReply) -> bool { unimplemented!() }",
        "pub fn z_reply_sample(r: &ZReply) -> Option<&ZSample> { unimplemented!() }",
        "pub fn z_reply_err(r: &ZReply) -> Option<&ZErr> { unimplemented!() }",
        "pub fn z_sample_key_expr(s: &ZSample) -> &ZKeyExpr { unimplemented!() }",
        "pub fn z_sample_timestamp(s: &ZSample) -> Option<&ZTs> { unimplemented!() }",
        "pub fn z_ts_ntp64(t: &ZTs) -> i64 { unimplemented!() }",
        "pub fn z_keyexpr_as_str(ke: &ZKeyExpr) -> &str { unimplemented!() }",
        "pub fn z_err_payload(e: &ZErr) -> Vec<u8> { unimplemented!() }",
    ];
    let mut items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    items.push((
        syn::Item::Fn(syn::parse_quote!(
            pub fn z_get(cb: impl Fn(ZReply) + Send + Sync + 'static) {
                unimplemented!()
            }
        )),
        loc.clone(),
    ));
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .source_module(syn::parse_quote!(myflat))
        .package_prefix("io.test.jni")
        .package("query")
        .value_blob(syn::parse_quote!(ZId))
        .ptr_class(syn::parse_quote!(ZKeyExpr))
        .ptr_class_output_direct()
        .ptr_class_output(syn::parse_quote!(z_keyexpr_as_str))
        .fun_accessor(syn::parse_quote!(z_keyexpr_as_str))
        .ptr_class(syn::parse_quote!(ZTs))
        .ptr_class_output(syn::parse_quote!(z_ts_ntp64))
        .fun_accessor(syn::parse_quote!(z_ts_ntp64))
        .ptr_class(syn::parse_quote!(ZSample))
        .ptr_class_output(syn::parse_quote!(z_sample_key_expr))
        .ptr_class_output(syn::parse_quote!(z_sample_timestamp))
        .fun_accessor(syn::parse_quote!(z_sample_key_expr))
        .fun_accessor(syn::parse_quote!(z_sample_timestamp))
        .ptr_class(syn::parse_quote!(ZErr))
        .ptr_class_output(syn::parse_quote!(z_err_payload))
        .fun_accessor(syn::parse_quote!(z_err_payload))
        .ptr_class(syn::parse_quote!(ZReply))
        .ptr_class_output(syn::parse_quote!(z_reply_zid))
        .ptr_class_output(syn::parse_quote!(z_reply_is_ok))
        .ptr_class_output(syn::parse_quote!(z_reply_sample))
        .ptr_class_output(syn::parse_quote!(z_reply_err))
        .fun_accessor(syn::parse_quote!(z_reply_zid))
        .fun_accessor(syn::parse_quote!(z_reply_is_ok))
        .fun_accessor(syn::parse_quote!(z_reply_sample))
        .fun_accessor(syn::parse_quote!(z_reply_err))
        .fun(syn::parse_quote!(z_get));

    let dir = unique_snapshot_dir("jnigen_double_opt");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // Both Option nesting steps of the timestamp leaf get their own match;
    // the innermost accessor composes off the second unwrap binding.
    assert!(rc.contains("matchmyflat::z_reply_sample("), "{rust}");
    assert!(rc.contains("matchmyflat::z_sample_timestamp("), "{rust}");
    assert!(rc.contains("myflat::z_ts_ntp64(__n1)"), "{rust}");
    // Never a blind compose through an `Option`-returning accessor.
    assert!(
        !rc.contains("myflat::z_ts_ntp64(myflat::z_sample_timestamp("),
        "{rust}"
    );
    assert!(
        !rc.contains("myflat::z_sample_key_expr(myflat::z_reply_sample("),
        "{rust}"
    );
    // The nested keyexpr identity is reached through the `Option` unwrap and
    // has a null arm.
    assert!(rc.contains("myflat::z_sample_key_expr(__n0)"), "{rust}");
    assert!(rc.contains("jni::objects::JObject::null()"), "{rust}");
    // The `Option<ZId>` Acc leaf composes its full return directly into the
    // converter — no unwrap of the leaf's own `Option`.
    assert!(rc.contains("myflat::z_reply_zid(&__cb_arg0)"), "{rust}");
    assert!(!rc.contains("matchmyflat::z_reply_zid("), "{rust}");
    // 6 leaves ⇒ typed `run` descriptor: nullable value-blob `[B`, raw `Z`
    // for the non-null bool discriminator, typed handle class (full FQN),
    // nullable String, BOXED Long for the nullable timestamp, nullable `[B`.
    assert!(
        rc.contains("\"([BZLjava/lang/Long;Ljava/lang/String;Ljava/lang/Long;[B)V\""),
        "{rust}"
    );
    // The non-null bool crosses as a raw typed jvalue — never boxed.
    assert!(rc.contains("jni::sys::jvalue{z:"), "{rust}");

    // Kotlin tier: the generated callback `fun interface` carries the typed
    // params — ok-arm and err-arm leaves nullable (the value may be absent),
    // the discriminator non-null; the value-blob leaf surfaces as its raw
    // (nullable) ByteArray wire, NOT the value class — the SDK wraps.
    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let iface_file = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .find(|v| v.contains("fun interface ZReplyCallback"))
        .unwrap_or_default();
    // Scope to the interface block — the merged package file also holds the
    // ZId value class and other decls.
    let iface = iface_file
        .split("fun interface ZReplyCallback")
        .nth(1)
        .and_then(|s| s.split_once('}').map(|(b, _)| b.to_string()))
        .unwrap_or_default();
    let ic: String = iface.split_whitespace().collect();
    assert!(ic.contains("isOk:Boolean"), "{iface}");
    assert!(ic.contains("sampleKeyExpr:Long?"), "{iface}");
    assert!(ic.contains(":Long?"), "{iface}");
    assert!(ic.contains(":ByteArray?"), "{iface}");
    assert!(!ic.contains(":ZId"), "{iface}");
    // The wrapper takes the typed interface and forwards it bare (no
    // value-blob rebuilding adapter exists anymore).
    let pkg = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .find(|v| v.contains("public fun zGet"))
        .unwrap_or_default();
    let pc: String = pkg.split_whitespace().collect();
    assert!(pc.contains("cb:ZReplyCallback"), "{pkg}");
    // The callback is forwarded bare — no value-blob rebuilding adapter
    // lambda wraps it at the call site (the interface itself delivers the
    // raw ByteArray wire).
    assert!(pc.contains("JNINative.zGet(cb,"), "{pkg}");
}

#[test]
fn strip_accessor_prefix_cases() {
    use crate::api::core::unfold::strip_accessor_prefix;
    // Regular snake: ZSample → z_sample_.
    assert_eq!(
        strip_accessor_prefix("z_sample_key_expr", "ZSample"),
        "key_expr"
    );
    // Irregular snake: type ZKeyExpr but prefix z_keyexpr_ — the
    // normalized (underscore-free) comparison still matches.
    assert_eq!(strip_accessor_prefix("z_keyexpr_as_str", "ZKeyExpr"), "as_str");
    // Double-letter type short: ZZBytes → z_zbytes_.
    assert_eq!(
        strip_accessor_prefix("z_zbytes_to_bytes", "ZZBytes"),
        "to_bytes"
    );
    // Receiver mismatch: falls back to stripping a bare `z_`.
    assert_eq!(strip_accessor_prefix("z_error_code", "Foo"), "error_code");
    // No type prefix, no z_: kept whole.
    assert_eq!(strip_accessor_prefix("get_name", "Foo"), "get_name");
}

// ────────────────────────────────────────────────────────────────────────
// Declaration-keyed interfaces: a type may have several decompositions —
// the canonical (unnamed) deconstructor, named alternatives
// (`.ptr_class_deconstructor(name)` + `.fun_*_named`), and per-fn inline
// records (`.fun_output`). Interface identity follows the DECLARATION, so
// differently-decomposed functions get distinct interfaces instead of
// colliding on one type-keyed name.
// ────────────────────────────────────────────────────────────────────────

/// Two fns failing with the same error type under different error
/// decompositions: the canonical (message only) and a named "full"
/// alternative (message + code). Each fn's extern must carry ITS handler's
/// FQN + descriptor, and both interfaces must be emitted.
#[test]
fn named_error_deconstructor_gets_own_handler() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let fns: &[&str] = &[
        "pub fn z_err_message(e: &ZErr) -> String { unimplemented!() }",
        "pub fn z_err_code(e: &ZErr) -> i32 { unimplemented!() }",
        "pub fn z_simple() -> Result<i64, ZErr> { unimplemented!() }",
        "pub fn z_detailed() -> Result<i64, ZErr> { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .source_module(syn::parse_quote!(myflat))
        .package_prefix("io.test.jni")
        .package("errors")
        .ptr_class(syn::parse_quote!(ZErr))
        // Canonical error decomposition: message only.
        .ptr_class_output(syn::parse_quote!(z_err_message))
        // Named alternative: message + code.
        .ptr_class_deconstructor("full")
        .ptr_class_output(syn::parse_quote!(z_err_message))
        .ptr_class_output(syn::parse_quote!(z_err_code))
        .fun_accessor(syn::parse_quote!(z_err_message))
        .fun_accessor(syn::parse_quote!(z_err_code))
        .fun(syn::parse_quote!(z_simple))
        .fun(syn::parse_quote!(z_detailed))
        .fun_error_named("full");

    let dir = unique_snapshot_dir("jnigen_named_err");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // Each extern's sink statics name its OWN handler interface + descriptor.
    assert!(rc.contains("io/test/jni/errors/ZErrHandler"), "{rust}");
    assert!(rc.contains("io/test/jni/errors/ZErrFullHandler"), "{rust}");
    // Builder-typed ze: canonical = (je, message: String); full adds the raw
    // `I` code (non-null Int crosses as a raw primitive, like a builder leaf).
    assert!(
        rc.contains("\"(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/Object;\""),
        "{rust}"
    );
    assert!(
        rc.contains("\"(Ljava/lang/String;Ljava/lang/String;I)Ljava/lang/Object;\""),
        "{rust}"
    );

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
        .split_whitespace()
        .collect();
    // Both interfaces emitted; the full one carries the extra nullable code.
    assert!(
        all.contains("funinterfaceZErrHandler<outR>{publicfunrun(je:String?,message:String):R"),
        "{all}"
    );
    assert!(
        all.contains(
            "funinterfaceZErrFullHandler<outR>{publicfunrun(je:String?,message:String,code:Int):R"
        ),
        "{all}"
    );
    // Wrappers take their own handler types.
    assert!(all.contains("funzSimple(onError:ZErrHandler<Long>)"), "{all}");
    assert!(
        all.contains("funzDetailed(onError:ZErrFullHandler<Long>)"),
        "{all}"
    );
}

/// Two fns returning the same type under different output decompositions:
/// the canonical one and a per-fn `.fun_output(...)` inline record list.
/// Each gets its own builder interface.
#[test]
fn inline_fun_output_gets_own_builder() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let fns: &[&str] = &[
        "pub fn z_thing_name(t: &ZThing) -> String { unimplemented!() }",
        "pub fn z_thing_size(t: &ZThing) -> i64 { unimplemented!() }",
        "pub fn z_make_a() -> ZThing { unimplemented!() }",
        "pub fn z_make_b() -> ZThing { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .source_module(syn::parse_quote!(myflat))
        .package_prefix("io.test.jni")
        .package("thing")
        .ptr_class(syn::parse_quote!(ZThing))
        // Canonical output: name + size (2 leaves ⇒ builder callback).
        .ptr_class_output(syn::parse_quote!(z_thing_name))
        .ptr_class_output(syn::parse_quote!(z_thing_size))
        .fun_accessor(syn::parse_quote!(z_thing_name))
        .fun_accessor(syn::parse_quote!(z_thing_size))
        .fun(syn::parse_quote!(z_make_a))
        // Per-fn inline records: identity handle + name (different shape).
        .fun(syn::parse_quote!(z_make_b))
        .fun_output([
            syn::parse_quote!(z_thing_name),
            syn::parse_quote!(z_thing_size),
            syn::parse_quote!(z_thing_name),
        ]);

    let dir = unique_snapshot_dir("jnigen_inline_out");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // Each extern names its own builder interface: the canonical
    // `ZThingBuilder` for z_make_a, the per-fn `ZThingZMakeBBuilder`.
    assert!(rc.contains("io/test/jni/thing/ZThingBuilder"), "{rust}");
    assert!(rc.contains("io/test/jni/thing/ZThingZMakeBBuilder"), "{rust}");

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
        .split_whitespace()
        .collect();
    // Canonical builder: (name, size); inline builder: (name, size, name2).
    assert!(
        all.contains("funinterfaceZThingBuilder<outR>{publicfunrun(name:String,size:Long):R"),
        "{all}"
    );
    assert!(
        all.contains(
            "funinterfaceZThingZMakeBBuilder<outR>{publicfunrun(name:String,size:Long,name2:String):R"
        ),
        "{all}"
    );
    // Wrappers take their own builder types.
    assert!(all.contains("build:ZThingBuilder<R>"), "{all}");
    assert!(all.contains("build:ZThingZMakeBBuilder<R>"), "{all}");
}

/// Error decomposition is the OUTPUT decomposition with a fixed leading `je`:
/// the same record kinds work — an identity record (the error itself as an
/// owned handle), plain accessors, and accessors nested through `Option`
/// (spliced child decomposition, nullable leaves). The ze params are typed
/// exactly like a builder's; on a binding error the native side fills typed
/// defaults (closed handle, "", null for nullable).
#[test]
fn error_unwrap_universal_records() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let fns: &[&str] = &[
        "pub fn z_err_message(e: &ZErr) -> String { unimplemented!() }",
        "pub fn z_err_detail(e: &ZErr) -> Option<&ZDetail> { unimplemented!() }",
        "pub fn z_detail_code(d: &ZDetail) -> i32 { unimplemented!() }",
        "pub fn z_fallible() -> Result<i64, ZErr> { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .source_module(syn::parse_quote!(myflat))
        .package_prefix("io.test.jni")
        .package("errors")
        .ptr_class(syn::parse_quote!(ZDetail))
        .ptr_class_output(syn::parse_quote!(z_detail_code))
        .fun_accessor(syn::parse_quote!(z_detail_code))
        .ptr_class(syn::parse_quote!(ZErr))
        // Canonical error decomposition: the owned error handle itself, its
        // message, and the Option-nested detail spliced to its code leaf.
        .ptr_class_output_direct()
        .ptr_class_output(syn::parse_quote!(z_err_message))
        .ptr_class_output(syn::parse_quote!(z_err_detail))
        .fun_accessor(syn::parse_quote!(z_err_message))
        .fun_accessor(syn::parse_quote!(z_err_detail))
        .fun(syn::parse_quote!(z_fallible));

    let dir = unique_snapshot_dir("jnigen_err_universal");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    // Handler descriptor: typed handle class, non-null String, BOXED nullable
    // Integer for the Option-nested code — exactly the builder typing.
    assert!(
        rc.contains(
            "\"(Ljava/lang/String;JLjava/lang/String;Ljava/lang/Integer;)Ljava/lang/Object;\""
        ),
        "{rust}"
    );
    // Domain-error arm: the SAME shared leaf encoder — owned identity moves
    // the error into a boxed handle, the nested Option accessor unwraps via
    // a match.
    assert!(
        rc.contains("std::boxed::Box::new(__de)"),
        "{rust}"
    );
    assert!(rc.contains("matchmyflat::z_err_detail(&__de)"), "{rust}");
    // Binding-error defaults: zeroed jlong for the handle (no native
    // construction), empty string, null for the nullable leaf — built lazily
    // in the __ze_defaults closure.
    assert!(!rc.contains("env.new_object(\"io/test/jni/errors/ZErr\""), "{rust}");
    assert!(rc.contains("env.new_string(\"\")"), "{rust}");

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n")
        .split_whitespace()
        .collect();
    // Builder-typed handler interface.
    assert!(
        all.contains(
            "funinterfaceZErrHandler<outR>{publicfunrun(je:String?,handle:Long,message:String,detailCode:Int?):R"
        ),
        "{all}"
    );
    // Wrapper: nullable capture slots, `!!` redispatch for the non-null ze,
    // pass-through for the nullable one — NO `?:` default coalescing.
    assert!(
        all.contains("returnonError.run(__cap_je,__cap_ze0!!,__cap_ze1!!,__cap_ze2)"),
        "{all}"
    );
    assert!(!all.contains("?:\"\""), "{all}");
}
