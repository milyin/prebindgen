use super::*;

/// Build the representative config: an opaque handle (`ZThing`) with a
/// free-function constructor returning `Result<ZThing, Error>` (exception
/// routing) and a free-function accessor, a C-like enum (`Color`, mixed
/// discriminants), and a throwable data class (`Error`).
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

    let dir = unique_test_dir("jnigen_snap");
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

    let dir = unique_test_dir("jnigen_boxstr");
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

    let dir = unique_test_dir("jnigen_slice_vec_handle");
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

    let dir = unique_test_dir("jnigen_native_init");
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
