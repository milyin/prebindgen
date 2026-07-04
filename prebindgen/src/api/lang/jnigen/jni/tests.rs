use quote::ToTokens;

use super::*;
use crate::api::core::{
    niches::{NicheSlot, Niches},
    registry::{Registry, TypeEntry, TypeKey},
};

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
    _rank: usize,
    e: TypeEntry<KotlinMeta>,
) {
    reg.input_types.insert(TypeKey::parse(ty_str), Some(e));
}
fn install_output(
    reg: &mut Registry<KotlinMeta>,
    ty_str: &str,
    _rank: usize,
    e: TypeEntry<KotlinMeta>,
) {
    reg.output_types.insert(TypeKey::parse(ty_str), Some(e));
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
        pc.contains("if(__cap.failed)returnonError.run("),
        "package wrappers: {pkg}"
    );
    // `onError` is a **required** parameter (no default) and the wrappers
    // never throw — error surfacing is entirely the caller's business.
    assert!(
        !pkg.contains("throw") && !pkg.contains("ZException"),
        "package wrappers: {pkg}"
    );
}

/// A `data_class` struct with an opaque-pointer string field
/// (`label: Option<Box<String>>`) maps that field to a nullable Kotlin `String?`
/// (via the `Box<String>` terminal converter + the `Option<_>` wrapper), and the
/// generated Rust glue encodes/decodes it through `JString` (boxing on input,
/// `new_string` on output). This lets an FFI-safe struct carry a heap string
/// while surfacing as a plain Kotlin `String?`.
#[test]
fn box_string_field_maps_to_nullable_kotlin_string() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Payload {
                    pub id: i64,
                    pub label: Option<Box<String>>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn payload_get() -> Payload {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn payload_put(p: &Payload) {
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
        .data_class(syn::parse_quote!(Payload))
        .package("payload")
        .fun(syn::parse_quote!(payload_get))
        .fun(syn::parse_quote!(payload_put));

    let dir = unique_snapshot_dir("jnigen_boxstr");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // Kotlin `data class Payload` carries the heap-string field as `String?`.
    assert!(kc.contains("dataclassPayload"), "{kotlin}");
    assert!(kc.contains("label:String?"), "{kotlin}");

    // Rust glue: output boxes-out via `new_string`; input re-boxes via `Box::new`.
    assert!(rc.contains("new_string"), "{rust}");
    assert!(
        rc.contains("Box::new") || rc.contains("Box<::std::string::String>"),
        "{rust}"
    );
}

/// A `&[T]` / `Vec<T>` input of a flattenable `data_class` is built as a
/// Rust-side `Vec` handle: Kotlin allocates the handle, pushes each element's
/// decoupled leaves in a loop, passes the `jlong` handle, then frees it in a
/// `finally` — no `List` `JObject` crosses, so the Rust side skips per-element
/// `env.get_field(...)`. `&[T]` borrows the boxed Vec; by-value `Vec<T>`
/// `mem::take`s it (the always-emitted free then drops an empty Vec). The
/// synthetic `…VecNew/Push/Free` trio is emitted once per element type and
/// shared by both functions.
#[test]
fn slice_input_builds_vec_handle() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Foo {
                    pub id: i64,
                    pub label: Option<Box<String>>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn put_slice(v: &[Foo]) {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn put_vec(v: Vec<Foo>) {
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
        .data_class(syn::parse_quote!(Foo))
        .package("foo")
        .fun(syn::parse_quote!(put_slice))
        .fun(syn::parse_quote!(put_vec));

    let dir = unique_snapshot_dir("jnigen_slice_vec_handle");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // One synthetic extern trio (shared by both functions).
    assert!(
        kc.contains("externalfunfooVecNew(cap:Int):Long"),
        "{kotlin}"
    );
    assert!(
        kc.contains("externalfunfooVecPush(handle:Long,"),
        "{kotlin}"
    );
    assert!(
        kc.contains("externalfunfooVecFree(handle:Long)"),
        "{kotlin}"
    );

    // Public surface stays `List<Foo>`; the body builds/pushes/frees the handle.
    assert!(kc.contains("v:List<Foo>"), "{kotlin}");
    assert!(
        kc.contains("val__vec_v=JNINative.fooVecNew(v.size)"),
        "{kotlin}"
    );
    assert!(kc.contains("for(__einv){"), "{kotlin}");
    assert!(
        kc.contains("JNINative.fooVecPush(__vec_v,__e.id,__e.label)"),
        "{kotlin}"
    );
    assert!(kc.contains("}finally{"), "{kotlin}");
    assert!(kc.contains("JNINative.fooVecFree(__vec_v)"), "{kotlin}");

    // Rust: the three helper symbols + both decode shapes (borrow / take).
    assert!(
        rc.contains("fnJava_io_test_jni_JNINative_fooVecNew"),
        "{rust}"
    );
    assert!(
        rc.contains("fnJava_io_test_jni_JNINative_fooVecPush"),
        "{rust}"
    );
    assert!(
        rc.contains("fnJava_io_test_jni_JNINative_fooVecFree"),
        "{rust}"
    );
    assert!(
        rc.contains("&*(v_handleas*constVec<myflat::Foo>)"),
        "{rust}"
    );
    assert!(
        rc.contains("mem::take(&mut*(v_handleas*mutVec<myflat::Foo>))"),
        "{rust}"
    );
}

/// `.jni_native_init(code)` injects an `init { code }` block into the generated
/// centralized externs object (`JNINative`) — the single static-init point a
/// consumer uses to trigger native-library loading. Unset (the `snapshot_*`
/// tests) emits no init block.
#[test]
fn jni_native_init_emits_init_block() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn z_ping() {
                unimplemented!()
            }
        )),
        loc.clone(),
    )];
    let mut registry = Registry::<KotlinMeta>::from_items(items).expect("index items");

    let jni = JniGen::new()
        .source_module(syn::parse_quote!(myflat))
        .package_prefix("io.test.jni")
        .jni_native_init("io.test.jni.NativeLibrary.ensureLoaded()")
        .package("thing")
        .fun(syn::parse_quote!(z_ping));

    let dir = unique_snapshot_dir("jnigen_native_init");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let paths = jni
        .write_kotlin(&registry, &dir.join("kotlin"))
        .expect("write_kotlin");
    let native = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .find(|v| v.contains("object JNINative"))
        .expect("a generated file contains `object JNINative`");

    // The init block is present, references the consumer's loader, and precedes
    // the `external fun` declarations.
    let flat: String = native.split_whitespace().collect();
    assert!(
        flat.contains("init{io.test.jni.NativeLibrary.ensureLoaded()}"),
        "JNINative should carry the init block:\n{native}"
    );
    let init_pos = native.find("init {").expect("init block present");
    let extern_pos = native.find("external fun").expect("externs present");
    assert!(
        init_pos < extern_pos,
        "init must precede externs:\n{native}"
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
        .accessor(syn::parse_quote!(z_thing_name), "name")
        .flatten_output()
        .field_self()
        .field("name")
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
    // Plan-less ZOther arg (Phase 3): crosses as a raw `jlong` (`(J)V`), NOT a
    // boxed handle object — so the Rust trampoline neither `new_object`s the
    // typed class nor `close()`s it (the Kotlin `asRaw` proxy wraps + closes).
    assert!(rc.contains(r#""(J)V""#), "{rust}");
    assert!(!rc.contains(r#""close""#), "{rust}");
    assert!(!rc.contains("io/test/jni/thing/ZOther"), "{rust}");
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
    // ZThing's identity leaf is `handle`, its accessor leaf carries the literal
    // author-supplied name (`z_thing_name` declared as `"name"`); `Fn()` ⇒
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
        all.contains("funinterfaceZThingCallback{publicfunrun(handle:ZThing,name:String)"),
        "{all}"
    );
    assert!(
        all.contains("funinterfaceZOtherCallback{publicfunrun(zOther:ZOther)"),
        "{all}"
    );
    assert!(
        all.contains("funinterfaceVoidCallback{publicfunrun()"),
        "{all}"
    );
    // Raw twin + proxy adapter for the decomposed-arg callback (raw jlong
    // handle at the wire); the all-passthrough interfaces get no twin.
    assert!(
        all.contains("funinterfaceZThingCallbackRaw{publicfunrun(handle:Long,name:String)"),
        "{all}"
    );
    assert!(
        all.contains("funZThingCallback.asRaw():ZThingCallbackRaw=ZThingCallbackRaw{handle,name->run(ZThing(handle),name)}"),
        "{all}"
    );
    // Plan-less ZOther arg (Phase 3): a raw twin `run(zOther: Long)` + an `asRaw`
    // proxy that wraps the pointer into the handle class AND `close()`s it in a
    // `finally` (close-unless-taken) — the Rust side delivers only the raw jlong.
    assert!(
        all.contains("funinterfaceZOtherCallbackRaw{publicfunrun(zOther:Long)"),
        "{all}"
    );
    assert!(all.contains("val__own0=ZOther(zOther)"), "{all}");
    assert!(all.contains("finally{__own0.close()}"), "{all}");
    assert!(!all.contains("VoidCallbackRaw"), "{all}");

    // Wrapper tier: the params are the typed interfaces, forwarded bare.
    let pkg = kotlin
        .values()
        .find(|v| v.contains("public fun zThingSub"))
        .cloned()
        .unwrap_or_default();
    let pc: String = pkg.split_whitespace().collect();
    assert!(pc.contains("cb:ZThingCallback"), "{pkg}");
    assert!(pc.contains("cb.asRaw()"), "{pkg}");
    assert!(pc.contains("onClose:VoidCallback"), "{pkg}");
    assert!(pc.contains("cb:ZOtherCallback"), "{pkg}");
}

/// Regression: a callback-delivered type that has BOTH a nested handle identity
/// (a child `ptr_class` reached by an accessor) AND its own root identity
/// (`.flatten_output().field_self()`) must emit the root MOVE after every borrow of
/// the owned value — otherwise the nested child clone (which borrows the root)
/// follows `Box::into_raw(Box::new(value))` and fails to compile with "use of
/// moved value". Declaring `.flatten_output().field_self()` LAST guarantees the
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
        .accessor(syn::parse_quote!(z_child_name), "name")
        .flatten_output()
        .field_self()
        .field("name")
        // Parent: a nested child-handle record, then its OWN root identity LAST.
        .ptr_class(syn::parse_quote!(ZParent))
        .accessor(syn::parse_quote!(z_parent_child), "child")
        .flatten_output()
        .field("child")
        .field_self()
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
/// (`z_reply_zid -> Option<ZId>`, a value class with no canonical child).
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
        .value_class(syn::parse_quote!(ZId))
        .ptr_class(syn::parse_quote!(ZKeyExpr))
        .accessor(syn::parse_quote!(z_keyexpr_as_str), "asStr")
        .flatten_output()
        .field_self()
        .field("asStr")
        .ptr_class(syn::parse_quote!(ZTs))
        .accessor(syn::parse_quote!(z_ts_ntp64), "ntp64")
        .flatten_output()
        .field("ntp64")
        .ptr_class(syn::parse_quote!(ZSample))
        .accessor(syn::parse_quote!(z_sample_key_expr), "keyExpr")
        .accessor(syn::parse_quote!(z_sample_timestamp), "timestamp")
        .flatten_output()
        .field("keyExpr")
        .field("timestamp")
        .ptr_class(syn::parse_quote!(ZErr))
        .accessor(syn::parse_quote!(z_err_payload), "payload")
        .flatten_output()
        .field("payload")
        .ptr_class(syn::parse_quote!(ZReply))
        .accessor(syn::parse_quote!(z_reply_zid), "zid")
        .accessor(syn::parse_quote!(z_reply_is_ok), "isOk")
        .accessor(syn::parse_quote!(z_reply_sample), "sample")
        .accessor(syn::parse_quote!(z_reply_err), "err")
        .flatten_output()
        .field("zid")
        .field("isOk")
        .field("sample")
        .field("err")
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
    assert!(ic.contains("sample__keyExpr:ZKeyExpr?"), "{iface}");
    assert!(ic.contains(":Long?"), "{iface}");
    assert!(ic.contains(":ZId?"), "{iface}");
    // The wrapper takes the typed interface and forwards it bare (no
    // value-blob rebuilding adapter exists anymore).
    let pkg = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .find(|v| v.contains("public fun zGet"))
        .unwrap_or_default();
    let pc: String = pkg.split_whitespace().collect();
    assert!(pc.contains("cb:ZReplyCallback"), "{pkg}");
    // The call site forwards the generated raw-proxy adapter — the typed
    // interface is the user surface, the extern receives the raw twin.
    assert!(pc.contains("JNINative.zGet(cb.asRaw(),"), "{pkg}");
}

// ────────────────────────────────────────────────────────────────────────
// Declaration-keyed interfaces: a type may have several decompositions —
// the default (unnamed) deconstructor and per-fn inline records
// (`.flatten_output_with`). Interface identity follows the DECLARATION, so
// differently-decomposed functions get distinct interfaces instead of
// colliding on one type-keyed name.
// ────────────────────────────────────────────────────────────────────────

/// Two fns returning the same type under different output decompositions:
/// the default one and a per-fn `.flatten_output_with(...)` inline field list.
/// Each gets its own builder interface.
#[test]
fn inline_output_gets_own_builder() {
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
        .accessor(syn::parse_quote!(z_thing_name), "name")
        .accessor(syn::parse_quote!(z_thing_size), "size")
        // Default output: name + size (2 leaves ⇒ builder callback).
        .flatten_output()
        .field("name")
        .field("size")
        .fun(syn::parse_quote!(z_make_a))
        // Per-fn inline fields: name + size + name again (different shape). The
        // third field reuses the `z_thing_name` accessor but must carry a
        // distinct (literal) leaf name — duplicate names are a hard error.
        .fun(syn::parse_quote!(z_make_b))
        .flatten_output_with()
        .field(syn::parse_quote!(z_thing_name), "name")
        .field(syn::parse_quote!(z_thing_size), "size")
        .field(syn::parse_quote!(z_thing_name), "name2");

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
    assert!(
        rc.contains("io/test/jni/thing/ZThingZMakeBBuilder"),
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
        .accessor(syn::parse_quote!(z_detail_code), "code")
        .flatten_output()
        .field("code")
        .ptr_class(syn::parse_quote!(ZErr))
        .accessor(syn::parse_quote!(z_err_message), "message")
        .accessor(syn::parse_quote!(z_err_detail), "detail")
        // Canonical error decomposition: the owned error handle itself, its
        // message, and the Option-nested detail spliced to its code leaf.
        .flatten_output()
        .field_self()
        .field("message")
        .field("detail")
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
    assert!(rc.contains("std::boxed::Box::new(__de)"), "{rust}");
    assert!(rc.contains("matchmyflat::z_err_detail(&__de)"), "{rust}");
    // Binding-error defaults: zeroed jlong for the handle (no native
    // construction), empty string, null for the nullable leaf — built lazily
    // in the __ze_defaults closure.
    assert!(
        !rc.contains("env.new_object(\"io/test/jni/errors/ZErr\""),
        "{rust}"
    );
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
            "funinterfaceZErrHandler<outR>{publicfunrun(je:String?,handle:ZErr,message:String,detail__code:Int?):R"
        ),
        "{all}"
    );
    // Raw twin carries the jlong handle; the wrapper captures raw and wraps
    // on redispatch.
    assert!(
        all.contains(
            "funinterfaceZErrHandlerRaw<outR>{publicfunrun(je:String?,handle:Long,message:String,detail__code:Int?):R"
        ),
        "{all}"
    );
    assert!(
        all.contains("returnonError.run(__cap.je,ZErr(__cap.ze0!!),__cap.ze1!!,__cap.ze2)"),
        "{all}"
    );
    // Zero-alloc thread-local capture holder generated for the error handler
    // (no per-call SAM lambda / Ref-boxed vars); the wrapper uses acquire().
    assert!(
        all.contains("internalclassZErrHandlerRawCapture:ZErrHandlerRaw<Unit>"),
        "{all}"
    );
    assert!(
        all.contains("val__cap=ZErrHandlerRawCapture.acquire()"),
        "{all}"
    );
    assert!(all.contains("ThreadLocal.withInitial"), "{all}");
    // Wrapper: nullable capture slots, `!!` redispatch for the non-null ze,
    // pass-through for the nullable one — NO `?:` default coalescing.
    assert!(!all.contains("?:\"\""), "{all}");
}

/// `.flatten_output().field(name)` must reference an `.accessor` declared on the
/// same class; an unknown name is a loud build-script panic.
#[test]
#[should_panic(expected = "no `.accessor")]
fn flatten_output_field_unknown_accessor_panics() {
    let _ = JniGen::new()
        .source_module(syn::parse_quote!(myflat))
        .package_prefix("io.test.jni")
        .package("thing")
        .ptr_class(syn::parse_quote!(ZThing))
        .accessor(syn::parse_quote!(z_thing_name), "name")
        // References a name that was never declared via `.accessor`.
        .flatten_output()
        .field("size");
}

/// `.method(f, name)` binds the `&Class` receiver to `this` (dropped from the
/// signature, its handle locked) while keeping the non-receiver params; the
/// method delegates to the same `JNINative` extern. `.constructor(f, name)`
/// emits a companion-object factory returning the class. Per-fn
/// `.flatten_output_with().field_self()` emits the handle leaf.
#[test]
fn method_constructor_and_inline_field_self() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let fns: &[&str] = &[
        "pub fn z_thing_name(t: &ZThing) -> String { unimplemented!() }",
        "pub fn z_thing_rename(t: &ZThing, name: String) -> bool { unimplemented!() }",
        "pub fn z_thing_make(name: String) -> ZThing { unimplemented!() }",
        "pub fn z_get() -> ZThing { unimplemented!() }",
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
        .accessor(syn::parse_quote!(z_thing_name), "name")
        // A method: `&ZThing` receiver + a `name: String` param.
        .method(syn::parse_quote!(z_thing_rename), "rename")
        // A constructor: factory returning ZThing.
        .constructor(syn::parse_quote!(z_thing_make), "make")
        // A free fn whose per-fn inline output decomposes to (handle, name).
        .fun(syn::parse_quote!(z_get))
        .flatten_output_with()
        .field_self()
        .field(syn::parse_quote!(z_thing_name), "name");

    let dir = unique_snapshot_dir("jnigen_method_ctor");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let all: String = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let flat: String = all.split_whitespace().collect();

    // The method binds `this` and keeps the non-receiver `name` param (no `t`).
    assert!(flat.contains("publicfunrename(name:String"), "{all}");
    // The receiver is locked under `this`.
    assert!(all.contains("withSortedHandleLocks(this)"), "{all}");
    // The constructor is a companion-object factory returning ZThing.
    assert!(flat.contains("publiccompanionobject"), "{all}");
    assert!(flat.contains("publicfunmake(name:String"), "{all}");
    // Per-fn inline output: `z_get` decomposes to (handle, name) — a 2-leaf
    // builder (`handle: ZThing, name: String`) from the inline field list.
    assert!(
        flat.contains("publicfunrun(handle:ZThing,name:String)"),
        "{all}"
    );
}

/// Phase 4: a bare `Option<primitive>` / `Option<enum>` **input** parameter
/// crosses as a decoupled `(present: Boolean, value: <prim>)` pair instead of a
/// boxed `java.lang.*` `JObject`. The Rust side reassembles the `Option` from
/// two raw scalars (`if <p>_present != 0u8 { Some(..) } else { None }`) with no
/// reflective `intValue()`/`longValue()` unbox. The public Kotlin signature
/// keeps `T?`; the call site passes `<name> != null` and `<name> ?: <zero>`
/// (`<name>?.value ?: 0` for an enum).
#[test]
fn option_scalar_param_crosses_as_present_value_pair() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Mode {
                    A,
                    B,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn z_set_timeout(ms: Option<i64>, count: Option<i32>, mode: Option<Mode>) {
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
        .enum_class(syn::parse_quote!(Mode))
        .package("cfg")
        .fun(syn::parse_quote!(z_set_timeout));

    let dir = unique_snapshot_dir("jnigen_optscalar");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // Public wrapper signature keeps the nullable typed params.
    assert!(kc.contains("ms:Long?"), "{kotlin}");
    assert!(kc.contains("count:Int?"), "{kotlin}");
    assert!(kc.contains("mode:Mode?"), "{kotlin}");

    // Extern declares the decomposed `(present, value)` pairs, never a boxed
    // `Long?`/`Int?` value wire.
    assert!(kc.contains("msPresent:Boolean"), "{kotlin}");
    assert!(kc.contains("msValue:Long"), "{kotlin}");
    assert!(kc.contains("countPresent:Boolean"), "{kotlin}");
    assert!(kc.contains("countValue:Int"), "{kotlin}");
    assert!(kc.contains("modePresent:Boolean"), "{kotlin}");
    assert!(kc.contains("modeValue:Int"), "{kotlin}");

    // Call site splits each param into present-flag + value-or-zero (enum reads
    // `?.value`).
    assert!(kc.contains("ms!=null"), "{kotlin}");
    assert!(kc.contains("ms?:0L"), "{kotlin}");
    assert!(kc.contains("count?:0"), "{kotlin}");
    assert!(kc.contains("mode?.value?:0"), "{kotlin}");

    // Rust native wrapper takes the two raw scalars and rebuilds the `Option`
    // with no boxed-object unbox, then passes the rebuilt values to the source
    // fn. (The `Option<i64>`/`Option<i32>`/`Option<Mode>` boxed converters are
    // still emitted but are now dead `#[allow(dead_code)]` — the param path no
    // longer references them, exactly like the Phase-1 dead Vec converters.)
    assert!(rc.contains("ms_present:jni::sys::jboolean"), "{rust}");
    assert!(rc.contains("ms_value:jni::sys::jlong"), "{rust}");
    assert!(rc.contains("count_value:jni::sys::jint"), "{rust}");
    assert!(rc.contains("mode_value:jni::sys::jint"), "{rust}");
    assert!(rc.contains("ifms_present!=0u8"), "{rust}");
    // The live path feeds the three rebuilt `Option`s straight to the source
    // call — no boxed `JObject` param anywhere in the wrapper.
    assert!(
        rc.contains("myflat::z_set_timeout(ms,count,mode)"),
        "{rust}"
    );
}

/// Phase 2: a `Vec<opaque-handle>` / `Option<Vec<handle>>` **return** crosses as
/// a Kotlin-side leaf fold — each element's raw `jlong` pointer crosses and the
/// generated `<Handle>Folder` singleton wraps it into the typed handle class and
/// appends to an `ArrayList`. No Rust-side `java.util.ArrayList` of handle
/// objects is built (the `reject_vec_of_handle` guard is lifted for outputs).
#[test]
fn vec_of_handle_output_folds_kotlin_side() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct ZThing {
                    _p: u8,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn thing_list() -> Vec<ZThing> {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn thing_list_opt() -> Option<Vec<ZThing>> {
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
        .ptr_class(syn::parse_quote!(ZThing))
        .package("thing")
        .fun(syn::parse_quote!(thing_list))
        .fun(syn::parse_quote!(thing_list_opt));

    let dir = unique_snapshot_dir("jnigen_vec_handle_out");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // A `ZThingFolder<A>` interface is generated, and the wrapper returns a typed
    // list, allocating the `ArrayList<ZThing>` accumulator on the Kotlin side.
    assert!(kc.contains("interfaceZThingFolder<A>"), "{kotlin}");
    assert!(kc.contains("List<ZThing>"), "{kotlin}");
    assert!(kc.contains("ArrayList<ZThing>()"), "{kotlin}");
    // The folder singleton wraps each raw `jlong` element into the typed handle
    // class and appends it — no Rust object construction.
    assert!(
        kc.contains("ZThing(element)") || kc.contains("acc.add(ZThing("),
        "{kotlin}"
    );
    // `Option<Vec<…>>` surfaces as a nullable list.
    assert!(kc.contains("List<ZThing>?"), "{kotlin}");

    // Rust: each element's pointer is delivered as a raw `jvalue { j: … }` to the
    // folder's `run`, NOT wrapped into a Java object; no Rust-side `ArrayList` is
    // built for the handle vec.
    assert!(rc.contains("jvalue{j:__enc}"), "{rust}");
    assert!(
        !rc.contains(r#"new_object("java/util/ArrayList""#),
        "no Rust-side ArrayList for Vec<handle>: {rust}"
    );
}

/// Phase 5: a `data_class` **input** param carrying an `Option<primitive>` /
/// `Option<enum>` field — which used to decline field-flattening and box the
/// whole struct into a `JObject` (Rust `env.get_field(...)`) — now flattens, the
/// `Option` field crossing as a `(<field>Present: Boolean, <field>Value: <prim>)`
/// leaf pair the Rust side rebuilds with no reflective unbox.
#[test]
fn option_scalar_struct_field_flattens() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Opts {
                    pub id: i64,
                    pub ttl: Option<i64>,
                    pub flag: Option<bool>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn opts_put(o: &Opts) {
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
        .data_class(syn::parse_quote!(Opts))
        .package("opts")
        .fun(syn::parse_quote!(opts_put));

    let dir = unique_snapshot_dir("jnigen_optfield");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // The public wrapper keeps the typed `Opts` param; the extern crosses the
    // option fields as decomposed `(present, value)` pairs (the plain `id` field
    // stays a single leaf).
    assert!(kc.contains("oTtlPresent:Boolean"), "{kotlin}");
    assert!(kc.contains("oTtlValue:Long"), "{kotlin}");
    assert!(kc.contains("oFlagPresent:Boolean"), "{kotlin}");
    assert!(kc.contains("oFlagValue:Boolean"), "{kotlin}");
    // Call site destructures the typed object: present-flag + value-or-zero.
    assert!(kc.contains("o.ttl!=null"), "{kotlin}");
    assert!(kc.contains("o.ttl?:0L"), "{kotlin}");
    assert!(kc.contains("o.flag?:false"), "{kotlin}");

    // Rust rebuilds each field's `Option` from the raw scalars (gated on present)
    // and reconstructs the struct inline from the flat leaves, passing it to the
    // source fn. (The whole-struct `JObject_to_Opts` `get_field` converter is
    // still emitted but is now dead `#[allow(dead_code)]`, like Phase 4's boxed
    // converters — the live param path no longer references it.)
    assert!(rc.contains("o_ttl_present:jni::sys::jboolean"), "{rust}");
    assert!(rc.contains("o_ttl_value:jni::sys::jlong"), "{rust}");
    assert!(rc.contains("ifo_ttl_present!=0u8"), "{rust}");
    assert!(
        rc.contains("myflat::Opts{id:__o_id,ttl:__o_ttl,flag:__o_flag"),
        "{rust}"
    );
    assert!(rc.contains("myflat::opts_put(&o)"), "{rust}");
}

/// Order-sensitive global config (`package_prefix`, the mangle closures) is
/// baked into FQNs at declaration time — configuring it after the first
/// declaration is a hard builder error, not a silent mis-naming.
#[test]
#[should_panic(expected = "package_prefix must be configured before")]
fn late_package_prefix_panics() {
    let _ = JniGen::new()
        .ptr_class(syn::parse_quote!(ZThing))
        .package_prefix("io.test.jni");
}

/// Same guard for the per-kind name-mangle closures.
#[test]
#[should_panic(expected = "kotlin_fun_name_mangle must be configured before")]
fn late_mangle_closure_panics() {
    let _ = JniGen::new()
        .package("thing")
        .fun(syn::parse_quote!(thing_get))
        .kotlin_fun_name_mangle(|n| n.to_string());
}

/// A rank-0 wrapper on a Rust builtin generates a converter qualified with the
/// `source_module` (`myflat::usize`) — invalid Rust — so the registration is
/// rejected up front.
#[test]
#[should_panic(expected = "wrapper on builtin `usize`")]
fn builtin_wrapper_pattern_panics() {
    let _ = JniGen::new().output_wrapper(
        syn::parse_quote!(usize),
        |_r: &Registry<KotlinMeta>| -> Option<(syn::Type, Option<syn::Type>, syn::Expr)> {
            Some((
                syn::parse_quote!(jni::sys::jlong),
                None,
                syn::parse_quote!(v as jni::sys::jlong),
            ))
        },
    );
}

/// A `data_class` with a NESTED data-class field plus enum / `Option<prim>` /
/// `Option<enum>` fields — the shape that declines BOTH the fixed-builder
/// output synthesis and the input leaf-flatten, so it round-trips through the
/// whole-value `fromParts` / `get_field` converters. Pins three fixes those
/// paths needed (each surfaced at runtime by `examples/covertest-kotlin`):
///  * output `fromParts` descriptor: an `Option`-boxed primitive slot is the
///    BOX class (`Ljava/lang/Long;` / `Ljava/lang/Integer;`), not the bare
///    primitive — and the Kotlin factory takes `Int?` for `Option<enum>`,
///    rebuilding via `?.let { E.fromInt(it) }`;
///  * input `get_field` descriptors are the slots' EXACT static types (nested
///    class FQN, box class, enum class + `getValue()I` decode), not the erased
///    `Ljava/lang/Object;`;
///  * a bare `Option<enum>` RETURN wires as `Int?` (the boxed discriminant),
///    mapped back in the wrapper — previously the extern claimed the enum
///    class while the native side returned a boxed `Integer`.
#[test]
fn fromparts_fallback_boxes_option_fields() {
    use crate::SourceLocation;
    let loc = SourceLocation::default();
    let items: Vec<(syn::Item, SourceLocation)> = vec![
        (
            syn::Item::Enum(syn::parse_quote!(
                pub enum Level {
                    Low = 0,
                    High = 1,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Inner {
                    pub id: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Struct(syn::parse_quote!(
                pub struct Job {
                    pub inner: Inner,
                    pub level: Level,
                    pub ttl: Option<i64>,
                    pub mode: Option<Level>,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn job_make(tag: i64) -> Job {
                    unimplemented!()
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn job_mode(j: &Job) -> Option<Level> {
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
        .package("model")
        .enum_class(syn::parse_quote!(Level))
        .data_class(syn::parse_quote!(Inner))
        .data_class(syn::parse_quote!(Job))
        .package("job")
        .fun(syn::parse_quote!(job_make))
        .fun(syn::parse_quote!(job_mode));

    let dir = unique_snapshot_dir("jnigen_fromparts_optbox");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let rust_path = registry
        .write_rust(&jni, dir.join("gen.rs"))
        .expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = jni.write_kotlin(&registry, &kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // OUTPUT (`job_make` → `fromParts`): the nested `inner` inlines to its `J`
    // leaf, the bare enum stays a raw `I`, and the two `Option` fields occupy
    // their BOX-class slots.
    assert!(
        rc.contains(r#""(JILjava/lang/Long;Ljava/lang/Integer;)Lio/test/jni/model/Job;""#),
        "{rust}"
    );
    // Kotlin factory: `Long?` / `Int?` params, enum rebuilt nullably; nested
    // child reassembled via its own factory.
    assert!(kc.contains("ttl:Long?"), "{kotlin}");
    assert!(kc.contains("mode:Int?"), "{kotlin}");
    assert!(kc.contains("mode?.let{Level.fromInt(it)}"), "{kotlin}");
    assert!(kc.contains("Inner.fromParts(inner_id)"), "{kotlin}");

    // INPUT (`job_mode`'s whole-`Job` param): every `get_field` names the
    // slot's exact static type; enum-typed slots decode via `getValue()I`.
    assert!(
        rc.contains(r#"get_field(v,"inner","Lio/test/jni/model/Inner;")"#),
        "{rust}"
    );
    assert!(
        rc.contains(r#"get_field(v,"level","Lio/test/jni/model/Level;")"#),
        "{rust}"
    );
    assert!(
        rc.contains(r#"get_field(v,"ttl","Ljava/lang/Long;")"#),
        "{rust}"
    );
    assert!(
        rc.contains(r#"get_field(v,"mode","Lio/test/jni/model/Level;")"#),
        "{rust}"
    );
    assert!(rc.contains(r#""getValue","()I""#), "{rust}");
    assert!(!rc.contains("Ljava/lang/Object;\")"), "{rust}");

    // RETURN (`job_mode` → `Option<Level>`): the extern wires `Int?`; the
    // wrapper maps the boxed discriminant back to the nullable enum.
    assert!(
        kc.contains("funjobMode(j:Job,errorSink:Any):Int?"),
        "{kotlin}"
    );
    assert!(kc.contains("?.let{Level.fromInt(it)}"), "{kotlin}");
}
