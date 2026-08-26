use super::*;

/// Build the representative config: an opaque handle (`ZThing`) with a
/// free-function constructor returning `Result<ZThing, Error>` (exception
/// routing) and a free-function accessor, a C-like enum (`Color`, mixed
/// discriminants), and a throwable data class (`Error`).
fn snapshot_pipeline() -> (String, std::collections::BTreeMap<String, String>) {
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!()
                .class(crate::data_class!(Error))
                .class(crate::ptr_class!(ZThing))
                .class(crate::enum_class!(Color)),
        )
        .package(
            crate::package!("thing")
                .fun(prebindgen_registry::fun!(z_thing_new))
                .fun(prebindgen_registry::fun!(z_thing_name)),
        );

    let dir = unique_test_dir("jnigen_snap");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
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
    // Opaque handle round-trips as a boxed pointer of the source type. The
    // output half is demand-driven: `Result<ZThing, Error>` must reach its
    // retained success plan for `Box::into_raw` to be present at all.
    assert!(rc.contains("myflat::ZThing"), "{rust}");
    assert!(rc.contains("Box::from_raw"), "{rust}");
    assert!(rc.contains("Box::into_raw"), "{rust}");
    // Handle-input converters reject null AND tag-bit-set (closed) pointers
    // before any dereference — the #34 guard; bit 0 is the Kotlin closed tag.
    assert!(rc.contains("if*v==0||(*v&1)==1"), "{rust}");
    // Every opaque type carries the compile-time alignment floor that keeps
    // bit 0 free for the closed tag. Its source type is qualified by the
    // registry-owned final emitter before the guard is assembled.
    assert!(rc.contains("align_of::<myflat::ZThing>()<2"), "{rust}");
    // The freePtr destructor ignores tagged (already-closed) pointers.
    assert!(rc.contains("ifptr!=0&&(ptr&1)==0"), "{rust}");
    // Errors funnel to two channel fns (no JVM throw): `signal_binding_error`
    // (a marshalling/system failure → the base `JniErrorHandler`) and
    // `signal_domain_error` (a fallible fn's decomposed `Err(E)` → the typed
    // handler). `z_thing_new`'s `Error` has no declared decomposition, so its
    // `Err` stringifies through the BINDING channel — no `__ze_defaults`.
    assert!(rc.contains("fnsignal_binding_error"), "{rust}");
    assert!(rc.contains("fnsignal_domain_error"), "{rust}");
    assert!(
        rc.contains("signal_binding_error(&mutenv,&__error_sink,&__SINK_MID,__SINK_FQN,__SINK_DESCR,&__e.to_string()"),
        "{rust}"
    );
    assert!(!rc.contains("__ze_defaults"), "{rust}");
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

    // Tag-bit lifecycle: closed = bit 0 set (or the 0 sentinel); the lock
    // ordering key is the immutable masked address, never the live `ptr`
    // (a mutable key could invert concurrent lock order → deadlock, #35).
    assert!(nhc.contains("ptr==0L||(ptrand1L)!=0L"), "{nh}");
    assert!(nhc.contains("sortedBy{it.ptrand-2L}"), "{nh}");
    assert!(nhc.contains("(a.ptrand-2L)<=(b.ptrand-2L)"), "{nh}");
    assert!(!nhc.contains("sortedBy{it.ptr}"), "{nh}");

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
    let thing = find("class ZThing private constructor(");
    let thingc: String = thing.split_whitespace().collect();
    assert!(thingc.contains(":NativeHandle"), "{thing}");
    // close()/take() mark the handle closed by setting the tag bit — the
    // address bits (= the lock-ordering key) are never rewritten.
    assert!(thingc.contains("ptr=por1L"), "{thing}");
    assert!(!thingc.contains("ptr=0L"), "{thing}");

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
        pc.contains("if(__bcap.failed)returnonError.run("),
        "package wrappers: {pkg}"
    );
    // Pre-lock closed guards go through `isClosed()` (tag-bit aware), not a
    // raw `ptr == 0L` compare — a closed handle is a binding failure.
    assert!(pc.contains(".isClosed())returnonError.run("), "{pkg}");
    // `onError` is a **required** parameter (no default) and the wrappers
    // never throw — error surfacing is entirely the caller's business.
    assert!(
        !pkg.contains("throw") && !pkg.contains("ZException"),
        "package wrappers: {pkg}"
    );
}

/// #37: no raw native pointer crosses into a consumer's hands unguarded —
/// from Kotlin **or** from Java.
///
/// `internal` and `@RequiresOptIn` both stop at the Kotlin compiler: an
/// `internal` member is a public JVM member under a mangled name, and javac
/// does not know what an opt-in requirement is. So every raw-pointer entry
/// point carries `@JvmSynthetic` (invisible to javac, invisible to nothing
/// else — JNI resolves on the name, which it leaves alone) and, where it must
/// stay callable from generated Kotlin, the opt-in marker as well:
///
/// * handle constructors — `private`, behind an `internal` + synthetic
///   `fromRawPtr`. `@JvmSynthetic` is not applicable to a constructor, and
///   `internal` alone left `new ZThing(0xdeadbeefL)` compiling from Java;
/// * `NativeHandle.peek()` and the `fromParts` factories — reached from Rust
///   by `call_method` / `call_static_method`, so the name must survive:
///   synthetic + marked;
/// * every `external fun` on `JNINative`, and each class's static `freePtr` —
///   `internal object` is a *public* JVM class, so these were callable
///   directly, handles bypassed entirely;
/// * `ptr`, `markConsumed`, and the other internal members — a Java caller
///   could otherwise repoint a live handle before letting it close.
///
/// Generated code opts itself in per file; that blanket is what a consumer
/// does not get.
#[test]
fn raw_pointer_entry_points_are_guarded() {
    let (_, kotlin) = snapshot_pipeline();
    let all: String = kotlin.values().cloned().collect::<Vec<_>>().join("\n");
    let c: String = all.split_whitespace().collect();

    // The marker is declared once, in the base package.
    let marker_files: Vec<&String> = kotlin
        .values()
        .filter(|v| v.contains("annotation class UnsafeNativeApi"))
        .collect();
    assert_eq!(
        marker_files.len(),
        1,
        "exactly one marker declaration:\n{all}"
    );
    assert!(
        marker_files[0].contains("@RequiresOptIn")
            && marker_files[0].contains("RequiresOptIn.Level.ERROR"),
        "the marker must be an ERROR-level opt-in requirement:\n{}",
        marker_files[0]
    );

    // Every generated file opts in — including the one declaring the marker.
    for (name, src) in &kotlin {
        assert!(
            src.contains("@file:OptIn(io.test.jni.UnsafeNativeApi::class)"),
            "{name} does not opt in:\n{src}"
        );
    }

    // Constructors: the concrete handle is `private`, reachable only through
    // the synthetic factory. `internal` here would still be a public JVM
    // constructor — that was the Java hole.
    assert!(
        c.contains("classZThingprivateconstructor(initialPtr:Long)"),
        "{all}"
    );
    assert!(!c.contains("classZThing(initialPtr:Long)"), "{all}");
    assert!(
        c.contains("@JvmSyntheticinternalfunfromRawPtr(initialPtr:Long):ZThing"),
        "{all}"
    );
    // The base stays `internal` (subclasses need `super`), which is inert: no
    // generated signature accepts a foreign subclass, and nothing it could
    // reach from Java is left visible.
    assert!(
        c.contains("abstractclassNativeHandleinternalconstructor(initialPtr:Long)"),
        "{all}"
    );

    // peek() keeps its name for JNI, and carries both guards.
    assert!(
        c.contains("@JvmSynthetic@io.test.jni.UnsafeNativeApipublicfunpeek():Long"),
        "{all}"
    );
    // A `fromParts` is guarded only when it actually takes a pointer. This
    // fixture's one factory is `Error.fromParts(message: String)`, which mints
    // nothing — guarding it would delete a safe factory from Java and force
    // consumers to opt into a raw-pointer contract it does not have. The
    // positive half of the rule lives in
    // `values::data_class_properties_match_their_from_parts_params`.
    assert!(
        c.contains("@JvmStaticpublicfunfromParts(message:String):Error"),
        "a pointer-free `fromParts` carries neither guard:\n{all}"
    );

    // The extern surface: `internal object` is public on the JVM, so every
    // member needs the flag, not just the object.
    assert!(c.contains("internalobjectJNINative"), "{all}");
    for occurrence in all.match_indices("external fun") {
        let head = &all[..occurrence.0];
        assert!(
            head.rfind("@JvmSynthetic")
                .is_some_and(|m| !head[m..].contains("fun ")),
            "an `external fun` reachable from Java:\n{all}"
        );
    }

    // Internal state a Java caller could otherwise use to repoint a live
    // handle: accessors hidden, not just the Kotlin-side visibility.
    assert!(
        c.contains("@get:JvmSynthetic@set:JvmSynthetic@Volatileinternalopenvarptr:Long"),
        "{all}"
    );
    assert!(
        c.contains("@JvmSyntheticinternalopenfunmarkConsumed()"),
        "{all}"
    );
}

/// The opt-in marker lives in the base package and every generated file names
/// it fully qualified. With no base package it lands in the root package,
/// which Kotlin cannot import from a subpackage — so a subpackage file could
/// not opt in, and the raw-pointer entry points would be unguarded by default.
/// That configuration is refused rather than silently degraded — as an error
/// from `write_kotlin`, since it is the caller's configuration, not a bug.
#[test]
fn a_subpackage_without_a_base_package_is_refused() {
    let loc = myflat_loc();
    let items: Vec<(syn::Item, prebindgen::SourceLocation)> = vec![(
        syn::Item::Fn(
            syn::parse_str("pub fn z_thing_new() -> ZThing { unimplemented!() }").unwrap(),
        ),
        loc,
    )];
    let registry = crate::test_util::reg_from_items(declare_referenced(items)).expect("index");
    // No `set_package_prefix`, but the declarations name a subpackage.
    let jni = JniGenBuilder::new().package(
        crate::package!("thing")
            .class(crate::ptr_class!(ZThing))
            .fun(prebindgen_registry::fun!(z_thing_new)),
    );
    let dir = unique_test_dir("jnigen_no_base_package");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let err = gen
        .write_kotlin(&dir.join("kotlin"))
        .expect_err("a subpackage without a base package must be refused");
    let msg = err.to_string();
    assert!(msg.contains("no base package is configured"), "{msg}");
}

/// Generated onError handler interfaces carry the split-channel contract KDoc:
/// the typed domain `<Err>Handler` documents that it fires only on `Err(E)`
/// (naming the decomposed error type) and points binding failures at
/// `onBindingError`; the shared `JniErrorHandler` documents `je` as the
/// binding/system message.
#[test]
fn handler_interfaces_carry_split_contract_kdoc() {
    // The snapshot pipeline has no error plan, so it emits the SHARED handler.
    let (_, kotlin) = snapshot_pipeline();
    let shared = kotlin
        .values()
        .find(|v| v.contains("fun interface JniErrorHandler<"))
        .cloned()
        .expect("no generated file declares JniErrorHandler");
    assert!(
        shared.contains("binding/system failure channel"),
        "{shared}"
    );
    assert!(shared.contains("from `run` is safe"), "{shared}");
    assert!(shared.contains("nullable you may"), "{shared}");
    assert!(shared.contains("return `null` to decline"), "{shared}");
    assert!(
        shared.contains("handler firing, not the returned value, is the error discriminator"),
        "{shared}"
    );

    // A `Result<_, ZErr>` fn with a declared error decomposition emits the
    // TYPED domain handler; its KDoc states it fires only on a domain error,
    // names the decomposed error type, and points binding failures elsewhere.
    let loc = myflat_loc();
    let fns: &[&str] = &[
        "pub fn z_err_message(e: &ZErr) -> String { unimplemented!() }",
        "pub fn z_fallible() -> Result<i64, ZErr> { unimplemented!() }",
    ];
    let items: Vec<(syn::Item, SourceLocation)> = fns
        .iter()
        .map(|src| {
            let f: syn::ItemFn = syn::parse_str(src).expect("parse fn");
            (syn::Item::Fn(f), loc.clone())
        })
        .collect();
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");
    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(crate::package!("ops").fun(prebindgen_registry::fun!(z_fallible)))
        .expand(
            prebindgen_registry::expand_return!(ZErr)
                .field(prebindgen_registry::fun!(z_err_message).name("message")),
        );

    let dir = unique_test_dir("jnigen_handler_kdoc");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
    let typed = paths
        .iter()
        .filter_map(|p| std::fs::read_to_string(p).ok())
        .find(|v| v.contains("fun interface ZErrHandler<"))
        .expect("no generated file declares ZErrHandler");
    assert!(typed.contains("Domain-error callback"), "{typed}");
    assert!(typed.contains("decomposed `ZErr`"), "{typed}");
    assert!(typed.contains("onBindingError"), "{typed}");
    assert!(typed.contains("from `run` is safe"), "{typed}");
    assert!(typed.contains("nullable you may"), "{typed}");
    assert!(typed.contains("return `null` to decline"), "{typed}");
    assert!(
        typed.contains("handler firing, not the returned value, is the error discriminator"),
        "{typed}"
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
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("payload")
                .class(crate::data_class!(Payload))
                .fun(prebindgen_registry::fun!(payload_get))
                .fun(prebindgen_registry::fun!(payload_put)),
        );

    let dir = unique_test_dir("jnigen_boxstr");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
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
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
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
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .package(
            crate::package!("foo")
                .class(crate::data_class!(Foo))
                .fun(prebindgen_registry::fun!(put_slice))
                .fun(prebindgen_registry::fun!(put_vec)),
        );

    let dir = unique_test_dir("jnigen_slice_vec_handle");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
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

    // Rust: the three helper symbols + both frozen site-pipeline operations
    // (a non-owning carrier for borrow, `mem::take` for consume).
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
        rc.contains("OwnedObject::from_raw(v_handleas*constVec<myflat::Foo>)"),
        "{rust}"
    );
    assert!(
        rc.contains("#[inline]fnderef(&self)->&Self::Target"),
        "the non-owning carrier's dereference must be explicitly inline:\n{rust}"
    );
    assert!(
        rc.contains("mem::take(&mut*(v_handleas*mutVec<myflat::Foo>))"),
        "{rust}"
    );
}

/// #86: every native-export family — function wrappers, typed-handle
/// destructors, vec build helpers — routes through the spec-compliant JNI
/// symbol encoder, so underscores in package segments, the harness class,
/// class names, and method names escape to `_1` (the issue's acceptance
/// fixture: package `io.example.my_pkg`, class `Native_Harness`, method
/// `do_work`). Kotlin-side names stay verbatim.
#[test]
fn native_symbols_are_jni_escaped() {
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
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
            syn::Item::Struct(syn::parse_quote!(
                pub struct Foo {
                    pub id: i64,
                }
            )),
            loc.clone(),
        ),
        (
            syn::Item::Fn(syn::parse_quote!(
                pub fn do_work() -> ZThing {
                    unimplemented!()
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
    ];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.example.my_pkg")
        .set_harness_name_mangle(|_| "Native_Harness".to_string())
        .package(
            crate::package!("sub_pkg")
                .class(crate::ptr_class!(ZThing).name("Z_Thing"))
                .class(crate::data_class!(Foo))
                .fun(prebindgen_registry::fun!(do_work))
                .fun(prebindgen_registry::fun!(put_slice)),
        );

    let dir = unique_test_dir("jnigen_symbol_escaping");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    let rust_path = gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let rust = std::fs::read_to_string(&rust_path).unwrap();
    let rc: String = rust.split_whitespace().collect();

    let kdir = dir.join("kotlin");
    let paths = gen.write_kotlin(&kdir).expect("write_kotlin");
    let kotlin: String = paths
        .iter()
        .map(|p| std::fs::read_to_string(p).unwrap())
        .collect::<Vec<_>>()
        .join("\n");
    let kc: String = kotlin.split_whitespace().collect();

    // Wrapper extern: escaped package + harness + camelized method.
    assert!(
        rc.contains("fnJava_io_example_my_1pkg_Native_1Harness_doWork"),
        "{rust}"
    );
    // Handle destructor: the class's own (sub)package and underscored name.
    assert!(
        rc.contains("fnJava_io_example_my_1pkg_sub_1pkg_Z_1Thing_freePtr"),
        "{rust}"
    );
    // Vec build helper trio: shares the harness class path.
    assert!(
        rc.contains("fnJava_io_example_my_1pkg_Native_1Harness_fooVecNew"),
        "{rust}"
    );
    // No symbol may carry the raw (unescaped) package spelling.
    assert!(!rc.contains("fnJava_io_example_my_pkg_"), "{rust}");
    // Kotlin-side names stay verbatim: the harness object and the class.
    assert!(kc.contains("objectNative_Harness"), "{kotlin}");
    assert!(kc.contains("classZ_Thing"), "{kotlin}");
}

/// `.set_jni_native_init(code)` injects an `init { code }` block into the generated
/// centralized externs object (`JNINative`) — the single static-init point a
/// consumer uses to trigger native-library loading. Unset (the `snapshot_*`
/// tests) emits no init block.
#[test]
fn jni_native_init_emits_init_block() {
    use prebindgen::SourceLocation;
    let loc = myflat_loc();
    let items: Vec<(syn::Item, SourceLocation)> = vec![(
        syn::Item::Fn(syn::parse_quote!(
            pub fn z_ping() {
                unimplemented!()
            }
        )),
        loc.clone(),
    )];
    let registry =
        crate::test_util::reg_from_items(declare_referenced(items)).expect("index items");

    let jni = JniGenBuilder::new()
        .set_package_prefix("io.test.jni")
        .set_jni_native_init("io.test.jni.NativeLibrary.ensureLoaded()")
        .package(crate::package!("thing").fun(prebindgen_registry::fun!(z_ping)));

    let dir = unique_test_dir("jnigen_native_init");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let gen = jni.build_with(registry).expect("resolve");
    gen.write_rust(dir.join("gen.rs")).expect("write_rust");
    let paths = gen.write_kotlin(&dir.join("kotlin")).expect("write_kotlin");
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
